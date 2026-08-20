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

/// Where the tilt does nothing — near the bottom of the generator's compass, so
/// that the correction only ever **attenuates**. Boosting the bass instead would
/// be the same balance and less headroom, and headroom is what a clipping test
/// has to keep.
const TILT_PIVOT_HZ: f32 = 65.0;

/// What one voice sounds at, in the register it is in.
///
/// **Equal amplitude is not equal loudness.** The ear is far more sensitive
/// around two to five kilohertz than it is low down, so a soprano and a bass
/// written at the same amplitude are not heard at the same level — the soprano
/// dominates, which is what a listener reported. Nothing about the synth was
/// wrong except that it treated a physical quantity as a perceptual one.
///
/// The correction is a **tilt**: a fixed number of decibels of attenuation per
/// octave above the pivot. That is the simplest thing that has the right shape,
/// and it is deliberately not an inversion of an equal-loudness contour — those
/// are defined at a stated listening level and flatten out as the level rises,
/// so inverting one would over-correct for anybody playing this loudly. What is
/// here instead is a knob with a sane default, and a test that measures what it
/// achieves with A-weighting rather than asserting that it works.
pub fn register_gain(hz: f32, tilt_db_per_octave: f32) -> f32 {
  let octaves = (hz / TILT_PIVOT_HZ).max(1e-6).log2().max(0.0);
  10f32.powf(-tilt_db_per_octave.max(0.0) * octaves / 20.0)
}

/// What a listener has asked for, as against what is written.
///
/// Neither of these is music: they change what is audible and nothing about what
/// is on the page. They travel together because the audio callback wants them in
/// one read.
#[derive(Clone, Copy, Debug)]
pub struct Mix {
  /// A bit per voice, set to silence it.
  pub mute: u32,
  /// Decibels of attenuation per octave above [`TILT_PIVOT_HZ`].
  pub tilt: f32,
}

impl Default for Mix {
  /// **Four and a half decibels an octave, because that is where the spread is
  /// narrowest and not because it sounded about right.**
  ///
  /// A-weighted level across the compass the generator writes in, C3 to F6:
  ///
  /// ```text
  ///   0 dB/8ve   15.7 dB spread     the complaint
  /// 1.5          10.6
  /// 3.0           6.4               guessed first, and beaten
  /// 4.5           3.1               the minimum
  /// 6.0           5.7               overshoots; the bass becomes the loud end
  /// ```
  ///
  /// One caveat the instrument cannot cover. A-weighting describes the ear at a
  /// *quiet* level, and the equal-loudness contours flatten as the level rises —
  /// so 4.5 is the right correction for someone listening softly and somewhat
  /// too much for someone listening loudly. That is what the knob is for, and
  /// why it goes down to nothing.
  fn default() -> Mix {
    Mix { mute: 0, tilt: 4.5 }
  }
}

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
  /// A bit set in `mix.mute` silences that voice. Its cursor still advances, so
  /// unmuting mid-piece rejoins the line where it now is rather than where it
  /// was left.
  pub fn render(&mut self, score: &Score, from: u64, out: &mut [f32], channels: usize, mix: Mix) -> u64 {
    self.fit(score.voices.len());
    let frames = out.len() / channels.max(1);
    let rate = score.rate.max(1) as f32;
    let attack = (ATTACK_MS * rate / 1000.0).max(1.0);
    let release = (RELEASE_MS * rate / 1000.0).max(1.0);
    let per_voice = HEADROOM / score.voices.len().max(1) as f32;

    for i in 0..frames {
      let s = from + i as u64;
      let mut sum = 0.0f32;

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
        if mix.mute & (1 << v) != 0 {
          continue;
        }
        let mut w = 0.0;
        for (k, amp) in PARTIALS {
          w += amp * (self.phase[v] * k).sin();
        }
        sum += w * NORM * gain * per_voice * register_gain(note.hz, mix.tilt);
      }

      let frame = sum.clamp(-1.0, 1.0);
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

  /// The A-weighting curve, as the standard defines it.
  ///
  /// Used here as an **instrument and not as the correction**. A-weighting is a
  /// stated approximation of what the ear does to a quiet sound, which makes it
  /// a fair yardstick for asking whether two notes are heard at the same level
  /// and a poor thing to invert into a synth, since it flattens as the level
  /// rises. Measuring with one curve and correcting with a plainer one keeps the
  /// two honest about each other.
  fn a_weight(f: f32) -> f32 {
    let f2 = f * f;
    let num = 12194.0f32.powi(2) * f2 * f2;
    let den = (f2 + 20.6f32.powi(2))
      * ((f2 + 107.7f32.powi(2)) * (f2 + 737.9f32.powi(2))).sqrt()
      * (f2 + 12194.0f32.powi(2));
    num / den
  }

  /// How loud one note of this synth is heard to be, in decibels, summing the
  /// partials the waveform actually has.
  fn heard_db(hz: f32, tilt: f32) -> f32 {
    let g = register_gain(hz, tilt);
    let e: f32 = PARTIALS.iter().map(|(k, amp)| (amp * NORM * g * a_weight(hz * k)).powi(2)).sum();
    10.0 * e.max(1e-30).log10()
  }

  fn one_note(from: u64, to: u64, midi: i16) -> Score {
    Score {
      voices: vec![vec![Sounding { from, to, midi, hz: schedule::hz(midi) }]],
      samples: to,
      qpm: 60,
      rate: RATE,
    }
  }

  /// **The tilt narrows the spread across the register, and by how much is
  /// measured rather than claimed.**
  ///
  /// A listener reported that high notes were significantly louder than low
  /// ones, which was true and was mine: every note got the same amplitude, and
  /// the ear is far more sensitive up high than down low. This measures what
  /// that costs and what the correction buys, over the compass the generator
  /// actually writes in — `catalog::compass` at three voices, step 21 to 45,
  /// which is a bass F2 to a soprano F6.
  #[test]
  fn the_tilt_narrows_what_the_ear_hears() {
    let notes: Vec<f32> = (21..=45)
      .map(|step| schedule::hz(contrapunctus::pitch::Pitch::new(step, 0).midi()))
      .collect();
    let spread = |tilt: f32| -> f32 {
      let db: Vec<f32> = notes.iter().map(|hz| heard_db(*hz, tilt)).collect();
      db.iter().cloned().fold(f32::MIN, f32::max) - db.iter().cloned().fold(f32::MAX, f32::min)
    };

    let flat = spread(0.0);
    let tilted = spread(Mix::default().tilt);
    assert!(flat > 12.0, "the complaint should be visible: only {flat:.1} dB across the compass");
    assert!(tilted < 5.0, "the tilt left {tilted:.1} dB of the {flat:.1}");

    // **The default is the minimum, not a guess that was never checked.** The
    // first one written here was 3 dB and it was beaten by a third; this fails
    // if some later change to the waveform moves the optimum away from it.
    let best = [0.0f32, 1.5, 3.0, 4.5, 6.0]
      .into_iter()
      .min_by(|a, b| spread(*a).total_cmp(&spread(*b)))
      .unwrap();
    assert_eq!(best, Mix::default().tilt, "the default tilt is no longer the narrowest of the settings tried");
  }

  /// The tilt only ever quietens, so it cannot cost headroom. Boosting the bass
  /// would have been the same balance and a clipping test to renegotiate.
  #[test]
  fn the_tilt_never_makes_anything_louder() {
    for step in 14..=52i16 {
      let hz = schedule::hz(contrapunctus::pitch::Pitch::new(step, 0).midi());
      for tilt in [0.0, 1.5, 3.0, 6.0] {
        let g = register_gain(hz, tilt);
        assert!(g <= 1.0 + 1e-6, "{hz:.0} Hz at {tilt} dB/oct gained {g}");
        assert!(g > 0.0, "{hz:.0} Hz at {tilt} dB/oct went silent");
      }
    }
    // and at zero it is exactly off
    assert!((register_gain(880.0, 0.0) - 1.0).abs() < 1e-6);
  }

  /// A note sounds where it is and nowhere else.
  #[test]
  fn silence_before_and_after_a_note() {
    let sc = one_note(1000, 3000, 69);
    let mut sy = Synth::new();
    sy.seek(&sc, 0);
    let mut buf = vec![0.0f32; 4000];
    sy.render(&sc, 0, &mut buf, 1, Mix { mute: 0, tilt: 0.0 });
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
    sy.render(&sc, 0, &mut buf, 1, Mix { mute: 0, tilt: 0.0 });
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
      at = sy.render(&sc, at, &mut buf, 1, Mix::default());
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
      sy.render(&sc, 0, &mut buf, 1, Mix { mute, tilt: 0.0 });
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
