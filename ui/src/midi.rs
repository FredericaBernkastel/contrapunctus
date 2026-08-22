//! System MIDI out — spec 6.3, and behind the `midi-out` feature.
//!
//! **It shares the synth's scheduler and the synth's clock**, which is the whole
//! of the design. `schedule::Score` already holds every note as a span of
//! samples and a MIDI number, and `audio::Player` already counts samples in its
//! callback; so this sends what that schedule says, at the position that clock
//! reports, and there is one scheduler with two sinks rather than two
//! schedulers that will one day disagree. The sound card keeps running with
//! every voice muted, because being the clock is what it is for here.
//!
//! # Told where the playhead is, rather than what happened
//!
//! [`Out::at`] takes a position and works out the difference from the last one,
//! instead of being handed events as they pass. That costs a scan of the score
//! per frame — some hundreds of notes, sixty times a second, which is nothing —
//! and it buys the one property a stream of events cannot have: **there is no
//! way to leave a note stuck on.** A seek, a pause, a dropped frame, a piece
//! replaced mid-play, the port being closed: each is just a position that does
//! not follow the last, and the difference is taken the same way.
//!
//! A note is identified by *where it begins* and not only by its pitch. Two
//! statements of the same pitch, one ending where the next begins, are one held
//! note under a diff on pitch alone — and telling a struck note from a held one
//! is §2.2's whole business, so it is not a distinction to lose in the last ten
//! lines of the program.
//!
//! # What it is not
//!
//! Sample-accurate. Events go out once a frame, so a note can be up to a frame
//! late — about 16 ms, against a quarter note of 790 ms at the tempo §8.16
//! writes at. That is audible on the fastest figures and it is why 6.2 makes the
//! built-in synth the default: this is the sink that sounds better and keeps
//! worse time.

use crate::schedule::Score;

/// A note that is sounding, named by which voice and where it began.
///
/// **Not by pitch**, for the reason the module note gives: a restruck note and a
/// held one would be the same key on the same channel, and the difference is the
/// one §2.2 exists to see.
type Note = (usize, u64, u8);

/// The output ports there are, or why there are none.
///
/// Both arms are wanted. 6.3 says absence is a disabled dropdown with a reason
/// and never a silent failure, and "no ports" and "no MIDI at all" are different
/// reasons — one is a machine with nothing plugged in, the other is a browser
/// that does not do Web MIDI.
pub fn ports() -> Result<Vec<String>, String> {
  let out = midir::MidiOutput::new("Contrapunctus").map_err(|e| format!("no MIDI on this system: {e}"))?;
  Ok(out.ports().iter().filter_map(|p| out.port_name(p).ok()).collect())
}

pub struct Out {
  conn: midir::MidiOutputConnection,
  name: String,
  /// What is sounding, so that the next position can say what changed.
  on: Vec<Note>,
}

impl Out {
  pub fn open(which: usize) -> Result<Out, String> {
    let out = midir::MidiOutput::new("Contrapunctus").map_err(|e| format!("no MIDI on this system: {e}"))?;
    let ports = out.ports();
    let port = ports.get(which).ok_or_else(|| format!("no port {which}; there are {}", ports.len()))?;
    let name = out.port_name(port).unwrap_or_else(|_| "the chosen port".into());
    let conn = out.connect(port, "contrapunctus").map_err(|e| format!("{name} would not open: {e}"))?;
    Ok(Out { conn, name, on: vec![] })
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  /// Put the port where `sample` says, sending whatever that changes.
  ///
  /// Offs before ons, which matters for exactly one case and it is the case
  /// above: a note ending where the next begins is an off and an on at the same
  /// instant, and in the other order the synth on the far end hears the off and
  /// stops the note that had just started.
  pub fn at(&mut self, score: &Score, sample: u64) {
    let mut want: Vec<Note> = vec![];
    for (v, notes) in score.voices.iter().enumerate() {
      for n in notes.iter().filter(|n| n.from <= sample && sample < n.to) {
        want.push((v, n.from, n.midi.clamp(0, 127) as u8));
      }
    }
    for note in self.on.clone() {
      if !want.contains(&note) {
        self.off(note);
      }
    }
    for note in &want {
      if !self.on.contains(note) {
        self.on(*note);
      }
    }
    self.on = want;
  }

  /// Stop everything, and mean it.
  ///
  /// The tracked notes first, and then `All Notes Off` on every channel — the
  /// second is not redundant, because the tracking is this program's idea of
  /// what is sounding and the port's idea is the one that matters. Anything the
  /// two disagree about is a note nobody can stop.
  pub fn silence(&mut self) {
    for note in std::mem::take(&mut self.on) {
      self.off(note);
    }
    for ch in 0..16u8 {
      let _ = self.conn.send(&[0xB0 | ch, 123, 0]);
    }
  }

  fn on(&mut self, (v, _, key): Note) {
    let _ = self.conn.send(&[0x90 | (v as u8 & 0x0F), key, 80]);
  }

  fn off(&mut self, (v, _, key): Note) {
    let _ = self.conn.send(&[0x80 | (v as u8 & 0x0F), key, 0]);
  }
}

impl Drop for Out {
  /// A port that closes with notes held leaves them sounding until something
  /// else stops them, and nothing else will.
  fn drop(&mut self) {
    self.silence();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// **The port list is a list or a reason, and never a silent nothing.**
  ///
  /// Spec 6.3's rule, and the only part of this that a machine with no MIDI
  /// hardware can still check: whichever arm comes back, something is sayable
  /// about it.
  #[test]
  fn the_ports_are_named_or_the_absence_is() {
    match ports() {
      Ok(names) => {
        for n in &names {
          assert!(!n.trim().is_empty(), "a port with no name is a dropdown entry nobody can choose");
        }
        eprintln!("{} ports: {names:?}", names.len());
      }
      Err(why) => {
        assert!(!why.trim().is_empty(), "absence with no reason is the silent failure 6.3 forbids");
        eprintln!("no MIDI: {why}");
      }
    }
  }

  /// **A position becomes note-ons and note-offs, and nothing is left sounding.**
  ///
  /// Against a real port when there is one, because the whole feature is bytes
  /// arriving somewhere and a test that stopped at the boundary would check the
  /// half that was never in doubt. Skipped, loudly, where there is no port —
  /// this is the one thing in the repository that needs hardware.
  #[test]
  fn a_playhead_becomes_notes_and_leaves_none_held() {
    use contrapunctus::kern::{Note as Kern, Voice, TICKS_PER_QUARTER as Q};
    use contrapunctus::pitch::Pitch;
    let Ok(names) = ports() else { return eprintln!("skipped: no MIDI on this system") };
    // Never a virtual cable: those go to whatever is listening, and something
    // listening is somebody else's session. Only a synth that ends at a speaker.
    let Some(which) = names.iter().position(|n| n.contains("GS Wavetable")) else {
      return eprintln!("skipped: no built-in synth port among {names:?}");
    };

    // two voices, a fourth apart, one note each
    let voice = |step: i16| Voice {
      notes: vec![Kern { onset: 0, dur: Q, pitch: Pitch::new(step, 0), attack: true }],
    };
    let score = crate::schedule::schedule(&[voice(28), voice(31)], 120, 48_000);
    assert_eq!(score.voices.len(), 2);

    let mut out = Out::open(which).expect("the built-in synth opened");
    // silence, then sounding, then silence again
    out.at(&score, 0);
    assert_eq!(out.on.len(), 2, "two voices sounding at tick zero");
    out.at(&score, score.samples + 1);
    assert!(out.on.is_empty(), "the notes are past and still held");
    out.silence();
    assert!(out.on.is_empty());
  }

  /// **A note restruck where the last one ended is struck again**, which a diff
  /// on pitch alone would miss — and telling a struck note from a held one is
  /// §2.2's whole business.
  #[test]
  fn a_repeated_pitch_is_two_notes_and_not_one() {
    use crate::schedule::{Score, Sounding};
    let score = Score {
      voices: vec![vec![
        Sounding { from: 0, to: 100, midi: 60, hz: 261.6 },
        Sounding { from: 100, to: 200, midi: 60, hz: 261.6 },
      ]],
      samples: 200,
      qpm: 120,
      rate: 48_000,
    };
    // The two are the same pitch in the same voice, back to back. What tells
    // them apart is where each begins, which is why that is in the key.
    let a = (0usize, 0u64, 60u8);
    let b = (0usize, 100u64, 60u8);
    assert_ne!(a, b, "the two statements are indistinguishable, so one would never be struck");
    let want = |sample: u64| -> Vec<Note> {
      score.voices[0]
        .iter()
        .filter(|n| n.from <= sample && sample < n.to)
        .map(|n| (0usize, n.from, n.midi as u8))
        .collect()
    };
    assert_eq!(want(50), vec![a]);
    assert_eq!(want(150), vec![b], "the second statement is a different note and gets its own attack");
  }
}
