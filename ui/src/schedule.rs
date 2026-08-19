//! Turning a fugue into something a sample clock can play — spec 6.1.
//!
//! `Outcome::voices` is already exact: 960 ticks to the whole note, and the
//! metre comes off the piece. So this is arithmetic, not interpretation, and it
//! happens **once** — the audio callback then does nothing but look up where it
//! already is.
//!
//! **Both directions live here, and that is the point.** The playhead is drawn
//! from the same conversion that placed the notes, so the line on screen cannot
//! drift from what is sounding: there is no second clock to disagree with the
//! first. Spec 6.1 says the callback's own sample count *is* the position, and
//! this is what makes that true rather than approximately true.

use contrapunctus::kern::{Voice, TICKS_PER_QUARTER};

/// One stretch of one voice, in samples.
///
/// Ties are already merged: a `Note` with `attack == false` is the same sound
/// continuing, so it extends the stretch before it rather than starting one.
/// Re-striking a tied note would turn every suspension into a repeated note,
/// which is the one articulation §2.2's rules are built to distinguish.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sounding {
  pub from: u64,
  pub to: u64,
  pub midi: i16,
  pub hz: f32,
}

/// A whole piece, ready to play.
#[derive(Clone, Debug, Default)]
pub struct Score {
  pub voices: Vec<Vec<Sounding>>,
  pub samples: u64,
  pub qpm: u32,
  pub rate: u32,
}

impl Score {
  pub fn is_empty(&self) -> bool {
    self.samples == 0
  }

  /// The two conversions, done with the score's *own* tempo and rate.
  ///
  /// Free functions would take those as arguments, and a caller that passed a
  /// tempo the score was not built at would get a playhead that drifted — slowly,
  /// plausibly, and only after the tempo had been changed once. Reading them off
  /// the score removes the chance.
  pub fn tick_of(&self, sample: u64) -> i64 {
    tick_of(sample, self.qpm, self.rate)
  }
  pub fn sample_of(&self, tick: i64) -> u64 {
    sample_of(tick, self.qpm, self.rate)
  }
}

/// Samples per tick, as a float, because a tick is not a whole number of them.
fn per_tick(qpm: u32, rate: u32) -> f64 {
  60.0 * rate as f64 / (qpm.max(1) as f64 * TICKS_PER_QUARTER as f64)
}

pub fn sample_of(tick: i64, qpm: u32, rate: u32) -> u64 {
  (tick.max(0) as f64 * per_tick(qpm, rate)).round() as u64
}

pub fn tick_of(sample: u64, qpm: u32, rate: u32) -> i64 {
  (sample as f64 / per_tick(qpm, rate)).round() as i64
}

/// Equal temperament, from the MIDI number the library already computes.
///
/// The lattice is diatonic and the tuning is not: a fugue that modulates through
/// six keys is unplayable in any historical temperament without retuning, and
/// this is a tool for hearing counterpoint rather than for hearing 1722.
pub fn hz(midi: i16) -> f32 {
  440.0 * 2f32.powf((midi as f32 - 69.0) / 12.0)
}

pub fn schedule(voices: &[Voice], qpm: u32, rate: u32) -> Score {
  let mut out: Vec<Vec<Sounding>> = vec![];
  for v in voices {
    let mut line: Vec<Sounding> = vec![];
    let mut notes: Vec<_> = v.notes.iter().collect();
    notes.sort_by_key(|n| n.onset);
    for n in notes {
      let (from, to) = (sample_of(n.onset, qpm, rate), sample_of(n.onset + n.dur, qpm, rate));
      let midi = n.pitch.midi();
      match line.last_mut() {
        // a tie continues the sound; anything else starts one
        Some(last) if !n.attack && last.to == from && last.midi == midi => last.to = to,
        _ => line.push(Sounding { from, to, midi, hz: hz(midi) }),
      }
    }
    out.push(line);
  }
  let samples = out.iter().filter_map(|l| l.last().map(|s| s.to)).max().unwrap_or(0);
  Score { voices: out, samples, qpm, rate }
}

#[cfg(test)]
mod tests {
  use super::*;
  use contrapunctus::{
    automaton::Tier,
    compose::{self, Layout},
    kern::Note,
    pitch::Pitch,
  };

  const RATE: u32 = 48_000;

  fn note(onset: i64, dur: i64, step: i16, attack: bool) -> Note {
    Note { onset, dur, pitch: Pitch::new(step, 0), attack }
  }

  /// A tie is one sound, not two.
  #[test]
  fn a_tie_extends_rather_than_restrikes() {
    let v = Voice { notes: vec![note(0, 240, 28, true), note(240, 240, 28, false), note(480, 240, 30, true)] };
    let s = schedule(std::slice::from_ref(&v), 60, RATE);
    assert_eq!(s.voices[0].len(), 2, "{:?}", s.voices[0]);
    assert_eq!(s.voices[0][0].from, 0);
    assert_eq!(s.voices[0][0].to, sample_of(480, 60, RATE));
  }

  /// A repeated note at the same pitch is two sounds even though it looks like a
  /// tie — the flag is the only thing that distinguishes them, and §2.2 exists
  /// because a struck dissonance and a suspended one are different things.
  #[test]
  fn a_repeated_note_is_not_a_tie() {
    let v = Voice { notes: vec![note(0, 240, 28, true), note(240, 240, 28, true)] };
    let s = schedule(std::slice::from_ref(&v), 60, RATE);
    assert_eq!(s.voices[0].len(), 2);
  }

  /// The two directions agree, which is what stops the playhead drifting from
  /// the sound. Checked across a range of tempi because the conversion rounds.
  #[test]
  fn the_clock_runs_both_ways() {
    for qpm in [40u32, 60, 76, 120, 208] {
      for tick in [0i64, 1, 239, 240, 960, 27 * 960, 100_000] {
        let back = tick_of(sample_of(tick, qpm, RATE), qpm, RATE);
        assert!((back - tick).abs() <= 1, "qpm {qpm}, tick {tick} came back {back}");
      }
    }
  }

  /// A440 is A440, and the octave above it is twice that.
  #[test]
  fn the_tuning_is_equal_and_a_is_440() {
    assert!((hz(69) - 440.0).abs() < 0.001);
    assert!((hz(81) - 880.0).abs() < 0.01);
    assert!((hz(60) - 261.6256).abs() < 0.01, "middle C");
    assert_eq!(Pitch::new(28, 0).midi(), 60, "step 28 is middle C");
  }

  /// The piece lasts as long as its tempo says it should.
  ///
  /// Arithmetic, and worth a test because getting it wrong is the kind of fault
  /// that is glaring to a listener and invisible to everything else here — a
  /// factor of four from confusing ticks-per-quarter with ticks-per-whole would
  /// pass every other test in this file.
  #[test]
  fn the_piece_lasts_what_the_tempo_says() {
    let cat = crate::catalog::load();
    let d = cat.subjects[1].design(3);
    let o = compose::fugue(&d, &Layout::default(), Tier::Full.rules(), 0x5EED).expect("a fugue");
    let s = schedule(&o.voices, 76, RATE);

    let ticks = compose::length(&o.blocks);
    let quarters = ticks as f64 / contrapunctus::kern::TICKS_PER_QUARTER as f64;
    let want = quarters / 76.0 * 60.0;
    let got = s.samples as f64 / RATE as f64;
    assert!((got - want).abs() < 1.0, "{got:.1} s of sound where the tempo says {want:.1}");
    assert!(got > 60.0, "27 bars at 76 to the minute is well over a minute, not {got:.1} s");
  }

  /// **The texture never falls completely silent in the middle of the piece.**
  ///
  /// This is the third listening test, made mechanical. A listener reported
  /// "0.4s long silence breaks repeating every 3-6s" in a fugue whose every
  /// number looked right, because every instrument here measures a relation
  /// between notes that *sound*, and a fault consisting of nothing sounding is
  /// invisible to all of them. A scheduler can see it: it knows where silence
  /// is, in samples.
  #[test]
  fn no_gap_swallows_every_voice_at_once() {
    let cat = crate::catalog::load();
    let d = cat.subjects[1].design(3); // BWV 847, §8.16's own
    let o = compose::fugue(&d, &Layout::default(), Tier::Full.rules(), 0x5EED).expect("a fugue");
    let s = schedule(&o.voices, 76, RATE);

    // Every boundary at which anything starts or stops, and whether the whole
    // texture is silent just after it.
    let mut marks: Vec<u64> = s.voices.iter().flatten().flat_map(|x| [x.from, x.to]).collect();
    marks.sort_unstable();
    marks.dedup();
    let first = s.voices.iter().filter_map(|l| l.first().map(|x| x.from)).min().unwrap_or(0);

    let mut worst = 0u64;
    for w in marks.windows(2) {
      let (a, b) = (w[0], w[1]);
      if a < first || b > s.samples {
        continue;
      }
      let sounding = s.voices.iter().any(|l| l.iter().any(|x| x.from <= a && a < x.to));
      if !sounding {
        worst = worst.max(b - a);
      }
    }
    let ms = 1000.0 * worst as f64 / RATE as f64;
    assert!(ms < 60.0, "the whole texture falls silent for {ms:.0} ms somewhere inside the piece");
  }
}
