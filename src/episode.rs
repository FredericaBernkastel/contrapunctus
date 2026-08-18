//! Are episodes sequences? — readme §8.13.
//!
//! [§2.4](../readme.md)'s grammar has one production that says something
//! specific rather than something structural:
//!
//! ```text
//! Episode → Sequence(motive, transposition pattern, n)
//! ```
//!
//! `Exposition` is backed by [§8.11](../readme.md) and `Middle+` says almost
//! nothing, but this is a **claim about the music** and step 7 would build on it.
//! It is cheap to check first, and checking it first is what §8.6 and §8.10 both
//! wish had happened to them.
//!
//! # What counts as a sequence
//!
//! A pattern of some period restated at a different pitch level. Stated exactly:
//! the whole texture in `[x, x+p)` reappears in `[x+p, x+2p)` with every voice's
//! onsets and durations identical and every pitch moved by the same number of
//! **diatonic steps**, that number not being zero. A repetition at the same
//! pitch is a repetition and not a sequence, which is why zero is excluded.
//!
//! This is strict. It requires the rhythm to survive exactly and the
//! transposition to be uniform across all voices, so a sequence Bach decorates
//! or reharmonises on its second statement will not be found. The figure it
//! produces is therefore a **floor**, and the control is what matters: the same
//! strict detector is run over the spans where a subject *is* sounding, and the
//! claim is about the difference between the two, not about either alone.

use crate::kern::{Piece, Voice};

/// One voice's notes in `[t0, t1)` as `(onset relative to t0, duration, step)`.
///
/// Notes that begin before `t0` are excluded rather than clipped: a sequence is
/// a restatement of a figure, and half of a note held over from earlier is not
/// part of the figure.
fn shape(v: &Voice, t0: i64, t1: i64) -> Vec<(i64, i64, i16)> {
  v.notes
    .iter()
    .filter(|n| n.attack && n.onset >= t0 && n.onset < t1)
    .map(|n| (n.onset - t0, n.dur, n.pitch.step))
    .collect()
}

/// Does the texture at `x` reappear at `x + per`, moved `t` diatonic steps?
///
/// `least` is the smallest number of notes that may count, so that a bar of rest
/// does not match another bar of rest and report a sequence in silence.
pub fn repeats(p: &Piece, x: i64, per: i64, t: i16, least: usize) -> bool {
  let mut notes = 0usize;
  for v in &p.voices {
    let (a, b) = (shape(v, x, x + per), shape(v, x + per, x + 2 * per));
    if a.len() != b.len() {
      return false;
    }
    notes += a.len();
    if a.iter().zip(&b).any(|(u, w)| u.0 != w.0 || u.1 != w.1 || w.2 - u.2 != t) {
      return false;
    }
  }
  notes >= least
}

/// The fraction of `[s, e)` covered by at least one sequence.
///
/// Periods are whole beats and starts fall on beats, which is where sequences
/// live and keeps the search from finding coincidences at arbitrary offsets. A
/// detected repetition covers `[x, x+2p)`, and is extended for as long as it
/// keeps going, so a four-fold sequence is counted once and covers its whole
/// length.
pub fn sequenced(p: &Piece, s: i64, e: i64, beat: i64) -> f64 {
  if beat <= 0 || e <= s {
    return 0.0;
  }
  let span = e - s;
  let mut covered = vec![false; ((span / beat).max(0) + 1) as usize];
  let max_per = (span / 2).min(beat * 16);
  let mut x = s;
  while x < e {
    let mut per = beat;
    while per <= max_per {
      if x + 2 * per <= e {
        // the transposition a sequence takes: anything but a plain repetition
        if let Some(t) = (-7..=7i16).filter(|t| *t != 0).find(|&t| repeats(p, x, per, t, 4)) {
          // extend while it keeps stepping by the same interval
          let mut k = 2i64;
          while x + (k + 1) * per <= e && repeats(p, x + (k - 1) * per, per, t, 4) {
            k += 1;
          }
          for c in ((x - s) / beat)..(((x - s) + k * per) / beat).min(covered.len() as i64) {
            covered[c as usize] = true;
          }
          break;
        }
      }
      per += beat;
    }
    x += beat;
  }
  let n = (span / beat).max(1) as usize;
  covered.iter().take(n).filter(|c| **c).count() as f64 / n as f64
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    kern::{Note, Voice, TICKS_PER_QUARTER as Q},
    pitch::Pitch,
  };

  fn piece(steps: &[i16]) -> Piece {
    Piece {
      id: "t".into(),
      voices: vec![Voice {
        notes: steps
          .iter()
          .enumerate()
          .map(|(i, &s)| Note {
            onset: i as i64 * Q,
            dur: Q,
            pitch: Pitch::new(s, 0),
            attack: true,
          })
          .collect(),
      }],
      measure: 4 * Q,
      beat: Q,
      key: [0; 7],
      tonic: None,
      polyphonic_instants: 0,
    }
  }

  /// A figure restated a step lower is a sequence, and the detector must find it
  /// at the period it actually has.
  #[test]
  fn a_transposed_restatement_is_found() {
    // C D E F | B C D E — the four-note figure down a step
    let p = piece(&[28, 29, 30, 31, 27, 28, 29, 30]);
    assert!(repeats(&p, 0, 4 * Q, -1, 4));
    assert!(!repeats(&p, 0, 4 * Q, -2, 4));
    assert!(sequenced(&p, 0, 8 * Q, Q) > 0.9);
  }

  /// A figure restated at the **same** pitch is a repetition, not a sequence,
  /// and §2.4's production says sequence.
  #[test]
  fn a_plain_repetition_is_not_a_sequence() {
    let p = piece(&[28, 29, 30, 31, 28, 29, 30, 31]);
    assert!(repeats(&p, 0, 4 * Q, 0, 4));
    assert_eq!(sequenced(&p, 0, 8 * Q, Q), 0.0);
  }

  /// Rhythm has to survive, or the detector would call any two spans with the
  /// same contour a sequence.
  #[test]
  fn the_rhythm_must_survive_the_transposition() {
    let mut p = piece(&[28, 29, 30, 31, 27, 28, 29, 30]);
    p.voices[0].notes[5].dur = Q / 2;
    p.voices[0].notes[5].onset += Q / 2;
    assert!(!repeats(&p, 0, 4 * Q, -1, 4));
  }

  /// Silence must not match silence. Without the floor on note count, a detector
  /// finds a sequence in every rest.
  #[test]
  fn silence_does_not_sequence() {
    let p = piece(&[28, 29]);
    assert!(!repeats(&p, 40 * Q, 4 * Q, -1, 4));
    assert_eq!(sequenced(&p, 40 * Q, 48 * Q, Q), 0.0);
  }

  /// A four-fold sequence is one sequence covering its whole length, not two
  /// overlapping ones — the extension has to work or the coverage figure would
  /// under-report every real case.
  #[test]
  fn a_four_fold_sequence_is_covered_whole() {
    let p = piece(&[28, 29, 27, 28, 26, 27, 25, 26]);
    assert!(sequenced(&p, 0, 8 * Q, Q) > 0.9);
  }
}
