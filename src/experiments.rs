//! The five experiments proposed to resolve §11.1 — that the rules Bach never
//! breaks are the rules that almost never bind.
//!
//! 1. **density** — clique *size* saturates; does graph *density* discriminate?
//! 2. **pareto** — keep the permissive hard tier, calibrate the soft criteria
//!    against Bach's own stretto, and measure capacity at that level.
//! 3. **renaissance** — Fux describes 16th-century vocal polyphony. Run the
//!    same rulebook over that repertoire, where it should hold.
//! 4. **chromatic** — is the melodic rule objecting to chromaticism, i.e. to
//!    idiom rather than to error?
//! 5. **harmony** — is a harmonic constraint the thing that both Bach satisfies
//!    *and* random placements violate?

use crate::{
  automaton::{Rule, CONFIRMED, HARD, SOFT},
  corpus, kern,
  kern::{Piece, Voice},
  stretto,
};

/// Pearson correlation, for the checks that a measure is not a proxy.
pub fn pearson(a: &[f64], b: &[f64]) -> f64 {
  let n = a.len() as f64;
  let (ma, mb) = (a.iter().sum::<f64>() / n, b.iter().sum::<f64>() / n);
  let num: f64 = a.iter().zip(b).map(|(x, y)| (x - ma) * (y - mb)).sum();
  let da: f64 = a.iter().map(|x| (x - ma).powi(2)).sum::<f64>().sqrt();
  let db: f64 = b.iter().map(|y| (y - mb).powi(2)).sum::<f64>().sqrt();
  if da * db == 0.0 { 0.0 } else { num / (da * db) }
}

// ------------------------------------------------------------ harmony ------

/// Chord templates as pitch-class masks on a root of 0.
const CHORDS: [u16; 9] = [
  0b0000_1001_0001, // major triad   0 4 7
  0b0000_1000_1001, // minor triad   0 3 7
  0b0000_0100_1001, // diminished    0 3 6
  0b0001_0001_0001, // augmented     0 4 8
  0b0100_1001_0001, // dominant 7    0 4 7 10
  0b0100_1000_1001, // minor 7       0 3 7 10
  0b0100_0100_1001, // half-dim 7    0 3 6 10
  0b0010_0100_1001, // dim 7         0 3 6 9
  0b1000_1001_0001, // major 7       0 4 7 11
];

/// Is this set of pitch classes contained in some chord at some root?
pub fn explicable(pcs: u16) -> bool {
  if pcs == 0 {
    return true;
  }
  for root in 0..12u32 {
    for t in CHORDS {
      let rotated = (((t as u32) << root) | ((t as u32) >> (12 - root))) & 0xFFF;
      if pcs as u32 & !rotated == 0 {
        return true;
      }
    }
  }
  false
}

/// The fraction of a texture's sonorities that a chord explains.
///
/// Only sonorities of three or more distinct pitch classes are counted: any two
/// notes fit some chord, so including dyads would measure the arithmetic rather
/// than the music.
pub fn harmonic_fit(voices: &[Voice]) -> (usize, usize) {
  let mut times: Vec<i64> =
    voices.iter().flat_map(|v| v.notes.iter().map(|n| n.onset)).collect();
  times.sort_unstable();
  times.dedup();
  let (mut fit, mut total) = (0, 0);
  for t in times {
    let mut pcs = 0u16;
    let mut n = 0;
    for v in voices {
      if let Some((p, _)) = kern::sounding(v, t) {
        pcs |= 1 << p.chroma().rem_euclid(12);
        n += 1;
      }
    }
    if n < 3 || pcs.count_ones() < 3 {
      continue;
    }
    total += 1;
    if explicable(pcs) {
      fit += 1;
    }
  }
  (fit, total)
}

// ------------------------------------------------------------ soft ---------

/// Soft-criterion counts for one voice pair, per slice.
pub fn soft_vector(a: &Voice, b: &Voice, measure: i64) -> Vec<f64> {
  let t = corpus::check_voices(a, b, measure);
  let den = t.slices.max(1) as f64;
  SOFT.iter().map(|r| t.by_rule.get(r.name()).copied().unwrap_or(0) as f64 / den).collect()
}

fn dominated(x: &[f64], limit: &[f64]) -> bool {
  x.iter().zip(limit).all(|(a, b)| *a <= *b + 1e-9)
}

/// Capacity under the **confirmed** hard tier, with the soft criteria acting as
/// an edge filter calibrated against Bach's own stretto: a pair may join only
/// if it is no worse than Bach's worst pair on every soft criterion at once.
///
/// No weights are asserted anywhere — domination is a partial order, which is
/// Komosinski & Szachewicz's point and readme §5's position.
pub fn capacity_pareto(
  sub: &stretto::Subject,
  key: &[i8; 7],
  measure: i64,
  limit: &[f64],
  step: i64,
  cap: usize,
) -> usize {
  let mut cands: Vec<(i64, i16)> = vec![(0, 0)];
  let mut d = 0;
  while d < sub.len {
    for n in -7i16..=7 {
      if !(d == 0 && n == 0) {
        cands.push((d, n));
      }
    }
    d += step;
  }
  let voices: Vec<Voice> =
    cands.iter().map(|&(d, n)| sub.place_diatonic(d, n, key)).collect();
  let m = cands.len();
  let mut ok = vec![vec![false; m]; m];
  for i in 0..m {
    for j in (i + 1)..m {
      let hard = stretto::compatible(&voices[i], &voices[j], measure, CONFIRMED).legal();
      let soft = dominated(&soft_vector(&voices[i], &voices[j], measure), limit);
      ok[i][j] = hard && soft;
      ok[j][i] = ok[i][j];
    }
  }
  let offs: Vec<i64> = cands.iter().map(|&(d, _)| d).collect();
  let mut best = 1usize;
  let mut cur = vec![0usize];
  fn go(ok: &[Vec<bool>], off: &[i64], m: usize, start: usize, cur: &mut Vec<usize>,
        best: &mut usize, cap: usize) {
    if cur.len() > *best {
      *best = cur.len();
    }
    if cur.len() >= cap {
      return;
    }
    for v in start..m {
      if cur.len() + (m - v) <= *best {
        return;
      }
      if cur.iter().all(|&u| ok[u][v] && off[u] != off[v]) {
        cur.push(v);
        go(ok, off, m, v + 1, cur, best, cap);
        cur.pop();
      }
    }
  }
  go(&ok, &offs, m, 1, &mut cur, &mut best, cap);
  best
}

/// Bach's own stretto's soft profile: the componentwise **worst** of its ten
/// real voice pairs. Calibration by exhibition, in §10's manner — a passage
/// rather than a number someone chose.
pub fn bach_soft_limit(p: &Piece, t0: i64, t1: i64) -> Vec<f64> {
  let mut limit = vec![0.0; SOFT.len()];
  for a in 0..p.voices.len() {
    for b in (a + 1)..p.voices.len() {
      let (va, vb) = (window(&p.voices[a], t0, t1), window(&p.voices[b], t0, t1));
      for (i, v) in soft_vector(&va, &vb, p.measure).iter().enumerate() {
        if *v > limit[i] {
          limit[i] = *v;
        }
      }
    }
  }
  limit
}

pub fn window(v: &Voice, t0: i64, t1: i64) -> Voice {
  Voice {
    notes: v.notes.iter().filter(|n| n.onset >= t0 && n.onset < t1)
      .map(|n| kern::Note { onset: n.onset - t0, ..*n }).collect(),
  }
}

// ------------------------------------------------------------ chromatic ----

/// Notes carrying an accidental foreign to the key signature, as a fraction of
/// all notes — a plain measure of how chromatic a piece is.
pub fn chromaticism(p: &Piece) -> f64 {
  let (mut foreign, mut total) = (0usize, 0usize);
  for v in &p.voices {
    for n in &v.notes {
      total += 1;
      let letter = n.pitch.step.rem_euclid(7) as usize;
      if n.pitch.alter != p.key[letter] {
        foreign += 1;
      }
    }
  }
  foreign as f64 / total.max(1) as f64
}

/// Per-rule rates for one piece, for the comparisons above.
pub fn rates(p: &Piece) -> (Vec<(&'static str, f64)>, usize) {
  let t = corpus::check_piece(p);
  let out = HARD
    .iter()
    .chain(SOFT.iter())
    .map(|r| {
      let den = if *r == Rule::ForbiddenMelodic { t.melodic_moves } else { t.slices };
      (r.name(), 1000.0 * t.by_rule.get(r.name()).copied().unwrap_or(0) as f64 / den.max(1) as f64)
    })
    .collect();
  (out, t.slices)
}
