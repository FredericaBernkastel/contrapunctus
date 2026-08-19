//! A Standard MIDI File writer — readme §8.6, and the only file in this
//! project that exists to be *heard* rather than measured.
//!
//! Format 1, one track per voice. The lattice of readme §2.1 is already an exact
//! integer grid — 960 ticks to the whole note, so 240 to the quarter — and SMF's
//! division field is the same kind of object, so the conversion is a scale by an
//! exact integer and never a rounding. A representation that had to *round* here
//! — a continuum of onsets, or a floating-point beat — would reintroduce at the
//! last step the defect the lattice was adopted to remove.
//!
//! The division written is **960 ticks per quarter**, not the 240 the internal
//! lattice uses, which is a `×4` and still exact. 240 is a legal division and a
//! rare one; a DAW reported these files back at exactly half their length, which
//! is the signature of a host substituting an assumed timebase for the one in
//! the header rather than of anything wrong in the file. Emitting a value hosts
//! actually see costs nothing, since the factor divides exactly.
//!
//! A **time signature** is written too. It changes no playback timing whatever,
//! and it is the difference between a listener seeing bar lines where the music
//! has them and seeing them wherever the host guesses — which matters when the
//! whole point of the file is comparing two versions of the same bars.
//!
//! # Time is exact here and spelling is not
//!
//! Everything above is about *time*, where the conversion is exact and provably
//! so. **Pitch is the opposite, and this is the one lossy step in the project.**
//! A MIDI note number is a semitone integer, which is precisely the
//! representation readme §2.1 exists to reject: `Pitch { step, alter }` carries
//! a diatonic degree *and* an alteration, and `midi()` throws the first away.
//! An augmented fourth and a diminished fifth leave here as the same byte.
//!
//! Measured rather than asserted. Read back out of FL Studio's piano roll, the
//! top voice of `fill.mid` — BWV 847, C minor, three flats — returns every pitch
//! at the right height and **13 of its 20 notes under the wrong name**: `D#5`
//! for E flat, `A#4` for B flat, `G#4` for A flat. Nothing is broken; the host
//! has no spelling to read and guesses sharps, which is the worst available
//! guess in a flat key.
//!
//! Two consequences, and the second is the one that matters.
//!
//! - The **track name** written by `step5::write_score` keeps the spelling the
//!   note data cannot: `1 top A-4..F5` is correct where the notes under it are
//!   not. That is a consolation and not a fix.
//! - **This is an output format and never an interchange one.** Nothing may be
//!   read back from a file written here into the model. A round trip through
//!   MIDI would re-spell every accidental by the reader's convention rather than
//!   the key's, so the interval qualities §2.2's automaton switches on would come
//!   back decided by a coin flip. The corpus is read from `**kern`, which spells
//!   its pitches, for exactly this reason.
//!
//! Written by hand rather than taken from a crate, because the whole encoder is
//! shorter than the dependency's documentation would be. The one dependency this
//! repository has is `clap`, and it parses the command line: nothing a reported
//! figure passes through touches a crate.

use crate::kern::{Voice, TICKS_PER_QUARTER};

/// Ticks per quarter note as written to the file.
pub const PPQ: i64 = 960;
/// Exact because `PPQ % TICKS_PER_QUARTER == 0`, which a test asserts.
const SCALE: i64 = PPQ / TICKS_PER_QUARTER;

fn vlq(mut n: u32, out: &mut Vec<u8>) {
  let mut buf = [0u8; 4];
  let mut i = 0;
  loop {
    buf[i] = (n & 0x7F) as u8;
    n >>= 7;
    i += 1;
    if n == 0 {
      break;
    }
  }
  while i > 0 {
    i -= 1;
    out.push(buf[i] | if i > 0 { 0x80 } else { 0 });
  }
}

fn be16(n: u16, out: &mut Vec<u8>) {
  out.extend_from_slice(&n.to_be_bytes());
}

fn chunk(tag: &[u8; 4], body: &[u8], out: &mut Vec<u8>) {
  out.extend_from_slice(tag);
  out.extend_from_slice(&(body.len() as u32).to_be_bytes());
  out.extend_from_slice(body);
}

/// One track: a text name, a tempo if it is the first, then the notes.
fn track(v: &Voice, name: &str, channel: u8) -> Vec<u8> {
  let mut ev: Vec<(i64, u8, u8, u8)> = vec![]; // (tick, status, data1, data2)
  for n in &v.notes {
    if !n.attack {
      continue; // tied continuation: the note is already sounding
    }
    // A tie chain sounds until the last note of the chain ends.
    let mut end = n.onset + n.dur;
    for m in &v.notes {
      if !m.attack && m.onset == end {
        end += m.dur;
      }
    }
    let key = n.pitch.midi().clamp(0, 127) as u8;
    ev.push((n.onset * SCALE, 0x90 | channel, key, 80));
    ev.push((end * SCALE, 0x80 | channel, key, 0));
  }
  // note-off before note-on at the same instant, so a repeated pitch re-articulates
  ev.sort_by_key(|&(t, s, k, _)| (t, s & 0xF0, k));

  let mut body = vec![];
  // FF 03: sequence/track name
  vlq(0, &mut body);
  body.extend_from_slice(&[0xFF, 0x03]);
  vlq(name.len() as u32, &mut body);
  body.extend_from_slice(name.as_bytes());

  let mut clock = 0i64;
  for (t, status, d1, d2) in ev {
    vlq((t - clock).max(0) as u32, &mut body);
    body.extend_from_slice(&[status, d1, d2]);
    clock = t;
  }
  vlq(0, &mut body);
  body.extend_from_slice(&[0xFF, 0x2F, 0x00]); // end of track
  body
}

/// Write `voices` as a format-1 SMF at `qpm` quarter notes per minute, in
/// `beats` per bar of a note worth `1/unit` of a whole — `(4, 4)` for common
/// time. The meter comes from the score's own time signature; guessing it would
/// put the bar lines somewhere the music does not have them.
pub fn write(
  path: &std::path::Path,
  voices: &[Voice],
  names: &[String],
  qpm: u32,
  (beats, unit): (u8, u8),
) -> std::io::Result<()> {
  let mut out = vec![];
  let mut head = vec![];
  be16(1, &mut head); // format 1
  be16(voices.len() as u16 + 1, &mut head);
  be16(PPQ as u16, &mut head);
  chunk(b"MThd", &head, &mut out);

  // conductor track: tempo and meter
  let mut t0 = vec![];
  vlq(0, &mut t0);
  let us = 60_000_000u32 / qpm.max(1);
  t0.extend_from_slice(&[0xFF, 0x51, 0x03, (us >> 16) as u8, (us >> 8) as u8, us as u8]);
  // SMF stores the denominator as a power of two, so 4 is written as 2
  let dd = (unit.max(1) as f64).log2().round() as u8;
  vlq(0, &mut t0);
  t0.extend_from_slice(&[0xFF, 0x58, 0x04, beats.max(1), dd, 24, 8]);
  vlq(0, &mut t0);
  t0.extend_from_slice(&[0xFF, 0x2F, 0x00]);
  chunk(b"MTrk", &t0, &mut out);

  for (i, v) in voices.iter().enumerate() {
    let name = names.get(i).map(|s| s.as_str()).unwrap_or("voice");
    // channel 9 is percussion, so skip it
    let ch = if i as u8 >= 9 { i as u8 + 1 } else { i as u8 } & 0x0F;
    chunk(b"MTrk", &track(v, name, ch), &mut out);
  }
  std::fs::write(path, out)
}

/// Write a texture as MIDI **in score order**, top voice first, with each track
/// named by where it sits and what it is.
///
/// Both halves of this matter, and both were wrong first time. The tracks were
/// emitted in `**kern` spine order — lowest voice first — and named `voice 0`,
/// `voice 1`, `voice 2` after that index. Two consequences, both reported from a
/// DAW rather than found here. The top voice of three arrives as *voice 2*,
/// which reads as reversed to anyone expecting the ground truth's own top-down
/// `S A T B C`; and `stretto.mid` numbered its entries downward from the top
/// while `stretto-bach.mid` numbered its voices upward from the bottom, so
/// `entry 1` and `voice 4` were the same line and nothing said so.
///
/// The order is taken from the **mean sounding pitch**, not from the spine
/// index, so the label is a description of what will be heard rather than a
/// restatement of an assumption about the file. The range in each name makes the
/// pairing checkable by eye between two files without playing either.
pub fn write_score(
  path: &std::path::Path,
  voices: &[Voice],
  roles: &[String],
  qpm: u32,
  sig: (u8, u8),
) -> std::io::Result<()> {
  let mean = |v: &Voice| -> f64 {
    if v.notes.is_empty() {
      return f64::MIN;
    }
    v.notes.iter().map(|n| n.pitch.chroma() as f64).sum::<f64>() / v.notes.len() as f64
  };
  let mut order: Vec<usize> = (0..voices.len()).collect();
  order.sort_by(|&a, &b| mean(&voices[b]).partial_cmp(&mean(&voices[a])).unwrap());

  let n = order.len();
  let (mut out, mut names) = (Vec::with_capacity(n), Vec::with_capacity(n));
  for (pos, &v) in order.iter().enumerate() {
    let where_ = match pos {
      0 => "top",
      p if p + 1 == n => "bass",
      _ => "inner",
    };
    let lo = voices[v].notes.iter().map(|x| x.pitch).min_by_key(|p| p.chroma());
    let hi = voices[v].notes.iter().map(|x| x.pitch).max_by_key(|p| p.chroma());
    let span = match (lo, hi) {
      (Some(a), Some(b)) => format!("{}..{}", a.name(), b.name()),
      _ => "silent".into(),
    };
    names.push(format!("{} {where_} {span} {}", pos + 1, roles.get(v).map(|s| s.as_str()).unwrap_or("")));
    out.push(voices[v].clone());
  }
  write(path, &out, &names, qpm, sig)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{kern::Note, pitch::Pitch};

  #[test]
  fn variable_length_quantities_round_trip() {
    for (n, want) in [
      (0u32, vec![0x00u8]),
      (127, vec![0x7F]),
      (128, vec![0x81, 0x00]),
      (8192, vec![0xC0, 0x00]),
      (1_048_575, vec![0xBF, 0xFF, 0x7F]),
    ] {
      let mut v = vec![];
      vlq(n, &mut v);
      assert_eq!(v, want, "vlq({n})");
    }
  }

  /// The lattice survives the conversion. If `PPQ` ever stopped being a whole
  /// multiple of the internal tick, every onset in every file would be rounded —
  /// which is the one thing readme §2.1 exists to prevent.
  #[test]
  fn the_file_timebase_is_an_exact_multiple_of_the_lattice() {
    assert_eq!(PPQ % TICKS_PER_QUARTER, 0);
    const { assert!(PPQ <= 0x7FFF, "division must fit the 15-bit field") };
  }

  /// A whole note must land on a bar line and a semiquaver on its sixteenth, at
  /// the timebase actually written.
  #[test]
  fn durations_convert_without_rounding() {
    for (recip, name) in [(1i64, "whole"), (2, "half"), (4, "quarter"), (16, "semiquaver"), (32, "demisemi")] {
      let internal = crate::kern::TICKS_PER_WHOLE / recip;
      assert_eq!((internal * SCALE) % (4 * PPQ / recip), 0, "{name} is not exact at {PPQ} ppq");
    }
  }

  #[test]
  fn a_tie_chain_sounds_as_one_note() {
    let p = Pitch::new(28, 0);
    let v = Voice {
      notes: vec![
        Note { onset: 0, dur: 240, pitch: p, attack: true },
        Note { onset: 240, dur: 240, pitch: p, attack: false },
      ],
    };
    let t = track(&v, "x", 0);
    // one note-on and one note-off, not two of each
    assert_eq!(t.iter().filter(|&&b| b == 0x90).count(), 1);
    assert_eq!(t.iter().filter(|&&b| b == 0x80).count(), 1);
  }
}
