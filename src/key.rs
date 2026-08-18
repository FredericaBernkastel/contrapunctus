//! Local key, by the same instrument as the chord — readme §8.14.
//!
//! [§9](../readme.md)'s second open problem: *"A real functional test needs
//! degree successions relative to a **local** key, and fugues modulate
//! constantly. Without it [§2.3](../readme.md)'s functional half cannot be built
//! or tested."* And [§8.9](../readme.md) added a second reason — a form grammar
//! is a key plan before it is anything else, and a key plan nothing can check is
//! not a claim.
//!
//! # The same shape as §8.5
//!
//! Viterbi over a vocabulary of keys, charging `mu` to change key, exactly as
//! the chord analyser charges `lambda` to change chord. Linear rather than
//! quadratic in the vocabulary for the same reason: the transition cost is zero
//! to stay and `mu` to move, so the best predecessor for any key is either
//! itself or the best of the whole previous column.
//!
//! Segments are **bars**, not onsets. A chord lasts a beat or two and a key
//! lasts phrases, and segmenting a key search at every onset would be asking a
//! smoothing parameter to undo a segmentation mistake.
//!
//! # The one thing a collection cannot do, and the fix
//!
//! A diatonic collection does not distinguish a major key from its relative
//! minor — C major and A minor are the same seven pitch classes, which is the
//! same objection [`kern::Piece::tonic`] exists to answer for the global key.
//! What separates them in practice is the **raised seventh**: A minor uses G
//! sharp and C major does not. So a minor key's collection here is the natural
//! minor **plus** its leading tone, which is the ordinary theoretical statement
//! and not a parameter.
//!
//! It has a cost worth stating: a minor key admits eight pitch classes where a
//! major key admits seven, so a fit statistic alone leans minor. The tonic-triad
//! bonus below leans back, and neither is swept — only `mu` is, on §8.5's
//! precedent.

use crate::{kern::Voice, pitch::Pitch};

/// A key: tonic pitch class, and whether the mode is minor.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Key {
  pub tonic: u8,
  pub minor: bool,
}

const NAMES: [&str; 12] = ["C", "C#", "D", "E-", "E", "F", "F#", "G", "A-", "A", "B-", "B"];

impl Key {
  /// The pitch classes the key admits, as a mask. Minor carries its leading
  /// tone, which is what separates it from its relative major.
  pub fn mask(&self) -> u16 {
    let base: u16 = if self.minor {
      0b1101_1010_1101 // 0 2 3 5 7 8 10, plus 11 for the raised seventh
    } else {
      0b1010_1011_0101 // 0 2 4 5 7 9 11
    };
    let r = self.tonic as u32;
    ((((base as u32) << r) | ((base as u32) >> (12 - r))) & 0xFFF) as u16
  }
  /// The tonic triad, a strong cue for which of two keys sharing a collection is
  /// meant.
  pub fn triad(&self) -> u16 {
    let base: u16 = if self.minor { 0b0000_1000_1001 } else { 0b0000_1001_0001 };
    let r = self.tonic as u32;
    ((((base as u32) << r) | ((base as u32) >> (12 - r))) & 0xFFF) as u16
  }
  pub fn name(&self) -> String {
    format!("{}{}", NAMES[self.tonic as usize], if self.minor { "m" } else { "" })
  }
  /// All 24.
  pub fn all() -> Vec<Key> {
    (0..12u8).flat_map(|t| [Key { tonic: t, minor: false }, Key { tonic: t, minor: true }]).collect()
  }
}

/// One bar of key analysis.
#[derive(Clone, Debug)]
pub struct Span {
  pub start: i64,
  pub end: i64,
  pub key: Key,
}

/// Sounding pitch classes in `[t0, t1)` with the time each is held, doubled on
/// the notated beat — the same weighting §8.5 uses, and for the same reason: a
/// semiquaver passing note should not outvote a minim.
fn weights(voices: &[Voice], t0: i64, t1: i64, beat: i64) -> ([f64; 12], f64) {
  let mut w = [0.0f64; 12];
  let mut total = 0.0;
  for v in voices {
    for n in &v.notes {
      let a = n.onset.max(t0);
      let b = (n.onset + n.dur).min(t1);
      if b <= a {
        continue;
      }
      let strong = beat > 0 && n.onset % beat == 0;
      let x = (b - a) as f64 * if strong { 2.0 } else { 1.0 };
      w[n.pitch.chroma().rem_euclid(12) as usize] += x;
      total += x;
    }
  }
  (w, total)
}

/// How well one key explains one bar: pitches in the collection earn their
/// weight, foreign ones lose it, and the tonic triad earns a fifth of the bar
/// besides — §8.5's bass bonus, in the place where a key search needs one.
fn observation(w: &[f64; 12], total: f64, k: Key) -> f64 {
  if total <= 0.0 {
    return 0.0;
  }
  let (m, tri) = (k.mask(), k.triad());
  let mut s = 0.0;
  let mut t = 0.0;
  for (pc, &x) in w.iter().enumerate() {
    if m & (1 << pc) != 0 {
      s += x;
    } else {
      s -= x;
    }
    if tri & (1 << pc) != 0 {
      t += x;
    }
  }
  s / total + 0.2 * (t / total)
}

/// Analyse the local key bar by bar, charging `mu` for each change.
pub fn analyse(voices: &[Voice], measure: i64, beat: i64, mu: f64) -> Vec<Span> {
  let end = voices.iter().flat_map(|v| v.notes.iter().map(|n| n.onset + n.dur)).max().unwrap_or(0);
  if measure <= 0 || end <= 0 {
    return vec![];
  }
  let keys = Key::all();
  let n = keys.len();
  let steps = ((end + measure - 1) / measure) as usize;

  let mut best = vec![f64::NEG_INFINITY; n];
  let mut back: Vec<Vec<u16>> = Vec::with_capacity(steps);
  for s in 0..steps {
    let (t0, t1) = (s as i64 * measure, (s as i64 + 1) * measure);
    let (w, total) = weights(voices, t0, t1, beat);
    let obs: Vec<f64> = keys.iter().map(|&k| observation(&w, total, k)).collect();
    let mut cur = vec![f64::NEG_INFINITY; n];
    let mut bk = vec![0u16; n];
    if s == 0 {
      cur.copy_from_slice(&obs);
    } else {
      let (mut gi, mut gv) = (0usize, f64::NEG_INFINITY);
      for (i, &v) in best.iter().enumerate() {
        if v > gv {
          gv = v;
          gi = i;
        }
      }
      for j in 0..n {
        let (stay, moved) = (best[j], gv - mu);
        if stay >= moved {
          cur[j] = stay + obs[j];
          bk[j] = j as u16;
        } else {
          cur[j] = moved + obs[j];
          bk[j] = gi as u16;
        }
      }
    }
    best = cur;
    back.push(bk);
  }

  let mut j = best.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(i, _)| i).unwrap_or(0);
  let mut path = vec![0usize; steps];
  for s in (0..steps).rev() {
    path[s] = j;
    j = back[s][j] as usize;
  }
  (0..steps)
    .map(|s| Span {
      start: s as i64 * measure,
      end: (s as i64 + 1) * measure,
      key: keys[path[s]],
    })
    .collect()
}

/// The key in force at one tick.
pub fn at(path: &[Span], t: i64) -> Option<Key> {
  path.iter().find(|s| s.start <= t && t < s.end).map(|s| s.key)
}

/// Mean bars per key — the outcome `mu` governs, reported rather than imposed,
/// exactly as §8.5 reports harmonic rhythm.
pub fn key_rhythm(path: &[Span]) -> f64 {
  if path.is_empty() {
    return 0.0;
  }
  let changes = path.windows(2).filter(|w| w[0].key != w[1].key).count() + 1;
  path.len() as f64 / changes as f64
}

/// The key a Hepokoski–Darcy label names, read against the piece's own key.
///
/// `V:PAC` in C major is G major; `vi:PAC` is A minor; `III:PAC` in C minor is E
/// flat major. Case carries the mode, and the degree is counted **within the
/// piece's key signature**, so the third degree of C minor is E flat without
/// anyone having to say so.
pub fn label_key(label: &str, tonic_letter: usize, key: &[i8; 7]) -> Option<Key> {
  let roman = label.split(':').next()?.trim();
  if roman.is_empty() {
    return None;
  }
  let (accidental, rest) = match roman.chars().next()? {
    'b' => (-1i8, &roman[1..]),
    '#' => (1, &roman[1..]),
    _ => (0, roman),
  };
  if rest.is_empty() {
    return None;
  }
  let minor = rest.chars().next()?.is_lowercase();
  let degree = match rest.to_uppercase().as_str() {
    "I" => 0,
    "II" => 1,
    "III" => 2,
    "IV" => 3,
    "V" => 4,
    "VI" => 5,
    "VII" => 6,
    _ => return None,
  };
  let l = (tonic_letter + degree) % 7;
  let pc = Pitch::new(l as i16, key[l] + accidental).chroma().rem_euclid(12) as u8;
  Some(Key { tonic: pc, minor })
}

#[cfg(test)]
mod tests {
  use super::*;

  const C: [i8; 7] = [0; 7];
  const CM: [i8; 7] = [0, 0, -1, 0, 0, -1, -1]; // C minor: E, A and B flat

  /// The collection is the whole point, and a minor key must carry its leading
  /// tone or it is indistinguishable from its relative major.
  #[test]
  fn a_minor_key_carries_its_leading_tone() {
    let a_minor = Key { tonic: 9, minor: true };
    let c_major = Key { tonic: 0, minor: false };
    assert_ne!(a_minor.mask(), c_major.mask());
    assert!(a_minor.mask() & (1 << 8) != 0, "A minor must admit G sharp");
    assert!(c_major.mask() & (1 << 8) == 0, "C major must not");
    // and the collection itself, spelled out, because a mask literal with one
    // bit wrong is invisible and this one was
    let pcs = |m: u16| -> Vec<u8> { (0..12u8).filter(|i| m & (1 << i) != 0).collect() };
    assert_eq!(pcs(c_major.mask()), vec![0, 2, 4, 5, 7, 9, 11]);
    assert_eq!(pcs(Key { tonic: 0, minor: true }.mask()), vec![0, 2, 3, 5, 7, 8, 10, 11]);
    assert_eq!(pcs(Key { tonic: 0, minor: false }.triad()), vec![0, 4, 7]);
    assert_eq!(pcs(Key { tonic: 0, minor: true }.triad()), vec![0, 3, 7]);
  }

  /// A roman numeral is read within the piece's own signature, so the third
  /// degree of C minor is E flat and not E natural.
  #[test]
  fn a_roman_numeral_is_read_within_the_pieces_key() {
    assert_eq!(label_key("V:PAC", 0, &C), Some(Key { tonic: 7, minor: false }));
    assert_eq!(label_key("vi:PAC", 0, &C), Some(Key { tonic: 9, minor: true }));
    assert_eq!(label_key("III:PAC", 0, &CM), Some(Key { tonic: 3, minor: false }));
    assert_eq!(label_key("i:HC", 0, &CM), Some(Key { tonic: 0, minor: true }));
    assert_eq!(label_key(":PAC", 0, &C), None);
  }

  /// A bar of C major must be called C major and not A minor, which is the case
  /// a collection alone gets wrong and the tonic triad settles.
  #[test]
  fn the_tonic_triad_separates_a_key_from_its_relative() {
    let bar = |steps: &[i16]| -> Voice {
      Voice {
        notes: steps
          .iter()
          .enumerate()
          .map(|(i, &s)| crate::kern::Note {
            onset: i as i64 * 240,
            dur: 240,
            pitch: Pitch::new(s, 0),
            attack: true,
          })
          .collect(),
      }
    };
    // C E G C, unambiguously C major by its triad
    let path = analyse(&[bar(&[28, 30, 32, 35])], 4 * 240, 240, 1.0);
    assert_eq!(path[0].key, Key { tonic: 0, minor: false });
    // A C E A, the same collection and the other triad
    let path = analyse(&[bar(&[26, 28, 30, 33])], 4 * 240, 240, 1.0);
    assert_eq!(path[0].key, Key { tonic: 9, minor: true });
  }

  /// A large penalty must hold one key for the whole piece and a zero penalty
  /// must be free to change every bar, or `mu` is not doing what it says.
  #[test]
  fn the_penalty_governs_how_often_the_key_changes() {
    let v = Voice {
      notes: (0..32)
        .map(|i| crate::kern::Note {
          onset: i * 240,
          dur: 240,
          pitch: Pitch::new(28 + (i % 12) as i16, 0),
          attack: true,
        })
        .collect(),
    };
    let tight = analyse(&[v.clone()], 4 * 240, 240, 100.0);
    let loose = analyse(&[v], 4 * 240, 240, 0.0);
    assert_eq!(key_rhythm(&tight), tight.len() as f64, "a large penalty must hold one key");
    assert!(key_rhythm(&loose) <= key_rhythm(&tight));
  }
}
