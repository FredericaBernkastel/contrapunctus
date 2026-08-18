//! A better harmonic plan, and what a perfect one is worth.
//!
//! §9 step 6's fourth proposal. §8.6 crosses three rulebooks with three sources
//! of harmony — none, a plan analysed from the **fixed voices only** (`clean`),
//! and a plan analysed from the whole texture with the answer key in it
//! (`leaky`) — and the third is a cheat run on purpose, to price what
//! §2.4's grammar could buy by supplying harmony and nothing else.
//!
//! Two things are wrong with reading that price off the table. The rows are
//! scored over **different spans**, because a tighter plan solves spans a looser
//! one refuses and refuses spans a looser one solves, so `10.4%` against `6.9%`
//! compares two sets of notes rather than two plans. And `leaky` is not the plan
//! a grammar would supply even in principle: a grammar emits a **chord
//! schedule**, coarse and decided in advance, not a chord per onset inferred
//! from the very notes it is about to ask for.
//!
//! This module builds the plans that answer both. Each is an ordinary
//! `Vec<Segment>` the realiser already knows how to obey, so the search does not
//! change and two conditions differ in exactly one thing. Nothing here is fitted
//! to anything: the analyser is [§8.5](../readme.md)'s, at the `λ` that section
//! swept, and the transformations below are arithmetic on its output.

use crate::{
  harmony::{self, Chord, Segment},
  kern::Voice,
};

/// The plan as §8.6 builds it — Viterbi over whichever voices are handed in,
/// charging `lambda` to change chord.
///
/// Named rather than called directly so that the conditions in a comparison all
/// read the same way, and so the one that varies `lambda` is visibly the same
/// construction as the one that does not.
pub fn viterbi(voices: &[Voice], beat: i64, lambda: f64) -> Vec<Segment> {
  harmony::analyse_viterbi(voices, beat, lambda)
}

/// Drop the harmonic constraint wherever the analyser's own confidence falls
/// below `tau`, keeping the segment and clearing its chord.
///
/// **A plan is a hard constraint, and a wrong one forbids the right note.** The
/// analyser reports a `fit` per segment and §8.5 measures it at 70–80% correct
/// against the cadence annotations *on a full texture*; the `clean` plan sees
/// one or two voices out of three or four, so it is being asked for the same
/// answer from less than half the evidence. Where it is guessing, the honest
/// thing is to say nothing: `chord: None` is exactly the `Plan::None` condition
/// applied to one segment, and the realiser already handles it.
///
/// This is the cheapest possible form of the idea that a plan should admit its
/// own ambiguity. The expensive form — carry the best `k` chords and accept a
/// note that any of them contains — needs a set in `Segment` and a change to the
/// obligation automaton; it is worth building only if the cheap form moves
/// anything.
pub fn gated(plan: &[Segment], tau: f64) -> Vec<Segment> {
  plan
    .iter()
    .map(|s| Segment { chord: if s.fit >= tau { s.chord } else { None }, ..s.clone() })
    .collect()
}

/// Re-quantise a plan onto a fixed grid of `grid` ticks, aligned so that cell
/// boundaries fall on the piece's own bar lines rather than the window's start.
///
/// **This is the shape of plan a grammar can actually emit.** §2.4's production
/// rules name a key plan and a cadence schedule; they do not name a chord per
/// onset, and they could not, since the onsets belong to the notes the grammar
/// is asking for. Coarsening the oracle is therefore the honest upper bound on
/// step 7 — the same information a perfect analysis carries, delivered at the
/// resolution a form grammar could deliver it.
///
/// Each cell takes the chord that holds it longest and a duration-weighted mean
/// of the fits that voted for it. `offset` is the window's own start tick, so
/// that a bar-length grid lands on bars.
pub fn coarsen(plan: &[Segment], grid: i64, offset: i64) -> Vec<Segment> {
  if plan.is_empty() || grid <= 0 {
    return plan.to_vec();
  }
  let (lo, hi) = (plan[0].start, plan[plan.len() - 1].end);
  // first cell boundary at or below `lo` in the piece's coordinate
  let first = lo - (lo + offset).rem_euclid(grid);
  let mut out = vec![];
  let mut t = first;
  while t < hi {
    let (a, b) = (t.max(lo), (t + grid).min(hi));
    let mut held: Vec<(Chord, i64, f64)> = vec![];
    for s in plan.iter() {
      let (x, y) = (s.start.max(a), s.end.min(b));
      if y <= x {
        continue;
      }
      let Some(c) = s.chord else { continue };
      match held.iter_mut().find(|(k, _, _)| *k == c) {
        Some((_, w, f)) => {
          *f = (*f * *w as f64 + s.fit * (y - x) as f64) / (*w + y - x) as f64;
          *w += y - x;
        }
        None => held.push((c, y - x, s.fit)),
      }
    }
    let win = held.iter().max_by_key(|(_, w, _)| *w);
    out.push(Segment {
      start: a,
      end: b,
      chord: win.map(|(c, _, _)| *c),
      fit: win.map(|(_, _, f)| *f).unwrap_or(0.0),
    });
    t += grid;
  }
  out
}

/// The fraction of sounding time on which two plans name the same chord.
///
/// The diagnostic that makes a comparison of plans readable: a candidate that
/// scores worse than the oracle because it is *coarse* and one that scores worse
/// because it is *wrong* look identical in an agreement column and completely
/// different here. Segments with no chord count as disagreement unless both
/// plans are silent.
pub fn overlap(a: &[Segment], b: &[Segment]) -> f64 {
  let (mut same, mut total) = (0i64, 0i64);
  for x in a.iter() {
    for y in b.iter() {
      let (lo, hi) = (x.start.max(y.start), x.end.min(y.end));
      if hi <= lo {
        continue;
      }
      total += hi - lo;
      if x.chord.map(|c| (c.root, c.quality)) == y.chord.map(|c| (c.root, c.quality)) {
        same += hi - lo;
      }
    }
  }
  if total == 0 { 0.0 } else { same as f64 / total as f64 }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn seg(start: i64, end: i64, root: u8, fit: f64) -> Segment {
    Segment { start, end, chord: Some(Chord { root, quality: 0 }), fit }
  }

  #[test]
  fn gating_clears_the_chord_and_keeps_the_segment() {
    let p = vec![seg(0, 10, 0, 0.9), seg(10, 20, 7, 0.4)];
    let g = gated(&p, 0.6);
    assert_eq!(g.len(), 2);
    assert!(g[0].chord.is_some());
    assert!(g[1].chord.is_none());
    assert_eq!(g[1].start, 10);
  }

  #[test]
  fn coarsening_takes_the_chord_that_holds_the_cell_longest() {
    // C for 3 ticks then G for 7, coarsened to one 10-tick cell: G wins
    let p = vec![seg(0, 3, 0, 1.0), seg(3, 10, 7, 1.0)];
    let c = coarsen(&p, 10, 0);
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].chord.unwrap().root, 7);
    assert_eq!((c[0].start, c[0].end), (0, 10));
  }

  #[test]
  fn coarsening_aligns_to_the_piece_rather_than_the_window() {
    // a window starting 4 ticks into an 8-tick bar: the first cell must be
    // 4 ticks long, not 8, or every later bar line is off by four
    let p = vec![seg(0, 12, 0, 1.0)];
    let c = coarsen(&p, 8, 4);
    assert_eq!(c.iter().map(|s| (s.start, s.end)).collect::<Vec<_>>(), vec![(0, 4), (4, 12)]);
  }

  #[test]
  fn overlap_is_one_against_itself_and_zero_against_a_transposition() {
    let p = vec![seg(0, 10, 0, 1.0), seg(10, 20, 7, 1.0)];
    let q = vec![seg(0, 10, 2, 1.0), seg(10, 20, 9, 1.0)];
    assert!((overlap(&p, &p) - 1.0).abs() < 1e-9);
    assert!(overlap(&p, &q).abs() < 1e-9);
  }

  #[test]
  fn overlap_counts_silence_as_agreement_only_when_both_are_silent() {
    let p = vec![seg(0, 10, 0, 1.0)];
    let q = vec![Segment { start: 0, end: 10, chord: None, fit: 0.0 }];
    assert!(overlap(&p, &q).abs() < 1e-9);
    assert!((overlap(&q, &q) - 1.0).abs() < 1e-9);
  }
}
