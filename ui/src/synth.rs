//! The built-in synth — spec 6.2, and it is deliberately plain.
//!
//! > The point of this tool is hearing the counterpoint **clearly**, not hearing
//! > it sound good. Those are different goals and only the first is ours.
//!
//! A triangle is three sine partials and it is enough: three lines of
//! counterpoint need a waveform with a little edge so the voices separate, and
//! nothing more. What matters far more than the timbre is the **envelope** —
//! entries have to start without a click, and notes have to articulate, or a
//! repeated note and a tied one sound alike and §2.2's whole distinction goes
//! inaudible.
//!
//! No device, no thread, no state outside itself: this renders samples into a
//! buffer and can therefore be tested, which the part that talks to the sound
//! card cannot be.

use crate::schedule::Score;

/// How long the ends of a note take, in milliseconds.
///
/// The attack is short enough to be an articulation rather than a swell; the
/// release is longer because a hard stop is the click. At 76 to the minute an
/// eighth note is about 200 ms, so together they are a tenth of the shortest
/// note the generator writes — audible as separation between notes, which is
/// what makes three simultaneous lines followable.
const ATTACK_MS: f32 = 4.0;
const RELEASE_MS: f32 = 14.0;

/// Peak amplitude of the whole texture. Three voices at full scale would clip;
/// a test asserts they do not.
const HEADROOM: f32 = 0.72;

/// A triangle wave as odd partials, with the sign alternating. `8/pi^2`
/// normalises the sum back to unit peak.
const PARTIALS: [(f32, f32); 3] = [(1.0, 1.0), (3.0, -1.0 / 9.0), (5.0, 1.0 / 25.0)];
const NORM: f32 = 0.810_569_5;

/// The rate comes off the [`Score`] rather than being held here, so a score
/// scheduled at one rate cannot be rendered at another.
pub struct Synth {
  cursor: Vec<usize>,
  phase: Vec<f32>,
}

impl Default for Synth {
  fn default() -> Synth {
    Synth::new()
  }
}

impl Synth {
  pub fn new() -> Synth {
    Synth { cursor: vec![], phase: vec![] }
  }

  fn fit(&mut self, n: usize) {
    self.cursor.resize(n, 0);
    self.phase.resize(n, 0.0);
  }

  /// Point every voice at whatever is sounding at `to`.
  ///
  /// Needed because the cursors only ever walk forwards during playback, which
  /// is what keeps the callback's cost independent of how long the piece is.
  pub fn seek(&mut self, score: &Score, to: u64) {
    self.fit(score.voices.len());
    for (v, line) in score.voices.iter().enumerate() {
      self.cursor[v] = line.partition_point(|s| s.to <= to);
      self.phase[v] = 0.0;
    }
  }

  /// Render frames starting at sample `from`, returning the position after.
  ///
  /// `out` is interleaved with `channels` channels; every channel gets the same
  /// signal, because a fugue is not a stereo image and putting the voices in
  /// different ears would make the counterpoint easier to follow than it is.
  ///
  /// A bit set in `mute` silences that voice. Its cursor still advances, so
  /// unmuting mid-piece rejoins the line where it now is rather than where it
  /// was left.
  pub fn render(&mut self, score: &Score, from: u64, out: &mut [f32], channels: usize, mute: u32) -> u64 {
    self.fit(score.voices.len());
    let frames = out.len() / channels.max(1);
    let rate = score.rate.max(1) as f32;
    let attack = (ATTACK_MS * rate / 1000.0).max(1.0);
    let release = (RELEASE_MS * rate / 1000.0).max(1.0);
    let per_voice = HEADROOM / score.voices.len().max(1) as f32;

    for i in 0..frames {
      let s = from + i as u64;
      let mut mix = 0.0f32;

      for (v, line) in score.voices.iter().enumerate() {
        // walk forward past anything that has finished
        while self.cursor[v] < line.len() && line[self.cursor[v]].to <= s {
          self.cursor[v] += 1;
          self.phase[v] = 0.0;
        }
        let Some(note) = line.get(self.cursor[v]) else { continue };
        if s < note.from {
          continue; // a rest, and rests are part of the music
        }

        let since = (s - note.from) as f32;
        let until = (note.to - s) as f32;
        let gain = (since / attack).min(1.0).min(until / release).clamp(0.0, 1.0);

        let step = std::f32::consts::TAU * note.hz / rate;
        self.phase[v] += step;
        if self.phase[v] > std::f32::consts::TAU {
          self.phase[v] -= std::f32::consts::TAU;
        }
        if mute & (1 << v) != 0 {
          continue;
        }
        let mut w = 0.0;
        for (k, amp) in PARTIALS {
          w += amp * (self.phase[v] * k).sin();
        }
        mix += w * NORM * gain * per_voice;
      }

      let frame = mix.clamp(-1.0, 1.0);
      for c in 0..channels {
        out[i * channels + c] = frame;
      }
    }
    from + frames as u64
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::schedule::{self, Sounding};

  const RATE: u32 = 48_000;

  fn one_note(from: u64, to: u64, midi: i16) -> Score {
    Score {
      voices: vec![vec![Sounding { from, to, midi, hz: schedule::hz(midi) }]],
      samples: to,
      qpm: 60,
      rate: RATE,
    }
  }

  /// A note sounds where it is and nowhere else.
  #[test]
  fn silence_before_and_after_a_note() {
    let sc = one_note(1000, 3000, 69);
    let mut sy = Synth::new();
    sy.seek(&sc, 0);
    let mut buf = vec![0.0f32; 4000];
    sy.render(&sc, 0, &mut buf, 1, 0);
    assert!(buf[..1000].iter().all(|x| *x == 0.0), "sound before the note");
    assert!(buf[3000..].iter().all(|x| *x == 0.0), "sound after the note");
    assert!(buf[1500..2500].iter().any(|x| x.abs() > 0.1), "no sound during the note");
  }

  /// **No clicks.** A note begins and ends at silence, which is the whole reason
  /// there is an envelope — an entry that starts at full amplitude is a click,
  /// and a fugue is mostly entries.
  #[test]
  fn a_note_begins_and_ends_at_silence() {
    let sc = one_note(0, RATE as u64, 60);
    let mut sy = Synth::new();
    sy.seek(&sc, 0);
    let mut buf = vec![0.0f32; RATE as usize];
    sy.render(&sc, 0, &mut buf, 1, 0);
    assert!(buf[0].abs() < 0.02, "the note starts at {}", buf[0]);
    assert!(buf[RATE as usize - 1].abs() < 0.02, "the note ends at {}", buf[RATE as usize - 1]);
  }

  /// Three voices at once stay inside the rails. Worth a test rather than a
  /// constant, because a constant is the kind of thing that is set once and then
  /// invalidated by adding a partial.
  #[test]
  fn a_full_texture_does_not_clip() {
    let cat = crate::catalog::load();
    let d = cat.subjects[1].design(3);
    let o = contrapunctus::compose::fugue(
      &d,
      &contrapunctus::compose::Layout::default(),
      contrapunctus::automaton::Tier::Full.rules(),
      0x5EED,
    )
    .expect("a fugue");
    let sc = schedule::schedule(&o.voices, 76, RATE);
    let mut sy = Synth::new();
    sy.seek(&sc, 0);

    let mut peak = 0.0f32;
    let mut at = 0u64;
    let mut buf = vec![0.0f32; 4096];
    while at < sc.samples {
      at = sy.render(&sc, at, &mut buf, 1, 0);
      peak = peak.max(buf.iter().fold(0.0f32, |m, x| m.max(x.abs())));
    }
    assert!(peak > 0.2, "the whole piece rendered at peak {peak} — something is not sounding");
    assert!(peak < 1.0, "the texture clips at peak {peak}");
  }

  /// A muted voice is silent and the others are not.
  ///
  /// Two voices sounding a fifth apart, then one of them silenced: the buffer
  /// must change and must not empty. Asserting only "quieter" would pass if the
  /// mask silenced everything.
  #[test]
  fn a_muted_voice_goes_quiet_and_the_rest_do_not() {
    let sc = Score {
      voices: vec![
        vec![Sounding { from: 0, to: 20_000, midi: 60, hz: schedule::hz(60) }],
        vec![Sounding { from: 0, to: 20_000, midi: 67, hz: schedule::hz(67) }],
      ],
      samples: 20_000,
      qpm: 60,
      rate: RATE,
    };
    let energy = |mute: u32| -> f32 {
      let mut sy = Synth::new();
      sy.seek(&sc, 0);
      let mut buf = vec![0.0f32; 16_000];
      sy.render(&sc, 0, &mut buf, 1, mute);
      buf.iter().map(|x| x * x).sum::<f32>() / buf.len() as f32
    };
    let both = energy(0);
    let one = energy(0b01);
    let none = energy(0b11);
    assert!(both > 0.0, "nothing sounded at all");
    assert!(one > 0.0, "muting one voice silenced both");
    assert!(one < both, "muting a voice changed nothing: {one} against {both}");
    assert!(none == 0.0, "muting every voice left {none}");
  }

  /// Seeking into the middle lands on the note sounding there, so the playhead
  /// and the sound agree after a click on the strip.
  #[test]
  fn seeking_lands_where_the_music_is() {
    let sc = Score {
      voices: vec![vec![
        Sounding { from: 0, to: 100, midi: 60, hz: schedule::hz(60) },
        Sounding { from: 100, to: 200, midi: 62, hz: schedule::hz(62) },
        Sounding { from: 200, to: 300, midi: 64, hz: schedule::hz(64) },
      ]],
      samples: 300,
      qpm: 60,
      rate: RATE,
    };
    let mut sy = Synth::new();
    sy.seek(&sc, 150);
    assert_eq!(sy.cursor[0], 1);
    sy.seek(&sc, 0);
    assert_eq!(sy.cursor[0], 0);
    sy.seek(&sc, 299);
    assert_eq!(sy.cursor[0], 2);
  }
}
