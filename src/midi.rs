//! A Standard MIDI File writer — readme §8.6, and the only file in this
//! project that exists to be *heard* rather than measured.
//!
//! Format 1, one track per voice, and the division is `TICKS_PER_QUARTER`
//! itself. That is not a convenience: the lattice of readme §2.1 is already an
//! exact integer grid of 960 ticks to the whole note, and SMF's division field
//! is exactly the same kind of object, so the score converts with no arithmetic
//! at all. A representation that had to *round* here — a continuum of onsets,
//! or a floating-point beat — would reintroduce at the last step the defect the
//! lattice was adopted to remove.
//!
//! Written by hand rather than taken from a crate, because the whole encoder is
//! shorter than the dependency's documentation and this project has no
//! dependencies to keep the build honest.

use crate::kern::{Voice, TICKS_PER_QUARTER};

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
    ev.push((n.onset, 0x90 | channel, key, 80));
    ev.push((end, 0x80 | channel, key, 0));
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

/// Write `voices` as a format-1 SMF at `qpm` quarter notes per minute.
pub fn write(path: &std::path::Path, voices: &[Voice], names: &[String], qpm: u32) -> std::io::Result<()> {
  let mut out = vec![];
  let mut head = vec![];
  be16(1, &mut head); // format 1
  be16(voices.len() as u16 + 1, &mut head);
  be16(TICKS_PER_QUARTER as u16, &mut head);
  chunk(b"MThd", &head, &mut out);

  // conductor track: tempo only
  let mut t0 = vec![];
  vlq(0, &mut t0);
  let us = 60_000_000u32 / qpm.max(1);
  t0.extend_from_slice(&[0xFF, 0x51, 0x03, (us >> 16) as u8, (us >> 8) as u8, us as u8]);
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
