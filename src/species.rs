//! Fux's species as a **whitelist** — readme §9 step 6, and §7.1's C1.
//!
//! The rulebook this project transcribed is a blacklist: five things that may
//! not happen, everything else permitted. §8.6 measured what that costs —
//! `10¹⁵` legal fills of three bars — and §7.1 named the difference, since
//! WaveFunctionCollapse's constraint is the other kind: *only these
//! configurations may occur.* A whitelist drawn from a real artefact is
//! enormously tighter than a handful of prohibitions.
//!
//! Fux's own book is a whitelist and was read as a blacklist. The species
//! enumerate the permitted note-against-note figures one at a time: first
//! species is consonance throughout; second admits the passing tone on the weak
//! half; third the passing and neighbour tones; fourth the suspension, tied over
//! and resolving downward. This module transcribes that enumeration and nothing
//! else — the same book and the same unfitted position as the prohibitions were.
//!
//! **It is a checker before it is a constraint.** §8.2's method is that a rule
//! earns its place by being measured against two corpora three centuries apart,
//! and the two dissonance rules this would replace are exactly the ones that
//! failed that test. So the whitelist is run over both corpora first and asked a
//! question with a number for an answer: *what fraction of the dissonances real
//! music writes are figures Fux lists?* A whitelist that cannot account for the
//! music is not a tighter rulebook, it is a wrong one, and there is no point
//! generating against it.

use crate::{
  kern::{self, Note, Voice},
  pitch::{Interval, Pitch},
};
use std::collections::BTreeMap;

/// The figures Fux permits, and one label for everything he does not.
///
/// This list *is* the rule. Adding a variant widens the whitelist and must be
/// argued from the book; the residue is reported rather than quietly absorbed.
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Figure {
  /// First species, and the ground of every other: a consonance needs no excuse.
  Consonance,
  /// Second and third species. Stepwise in, stepwise on, same direction.
  Passing,
  /// Third species. Stepwise in, stepwise back to the note it came from.
  Neighbour,
  /// Fourth species. Tied over from the previous slice, resolving down by step.
  Suspension,
  /// A dissonance none of the above accounts for.
  Unlisted,
}

impl Figure {
  pub fn name(&self) -> &'static str {
    match self {
      Figure::Consonance => "consonance",
      Figure::Passing => "passing",
      Figure::Neighbour => "neighbour",
      Figure::Suspension => "suspension",
      Figure::Unlisted => "UNLISTED",
    }
  }
  pub fn listed(&self) -> bool {
    *self != Figure::Unlisted
  }
}

/// Which note of a voice sounds at `t`, by index.
fn index_at(v: &Voice, t: i64) -> Option<usize> {
  v.notes.iter().position(|n| n.onset <= t && t < n.onset + n.dur)
}

fn steps(a: Pitch, b: Pitch) -> i16 {
  b.step - a.step
}

/// What figure one voice is making at note `i`, taken on its own terms.
///
/// `strong` is the metric condition Fux attaches to each: a suspension falls on
/// the beat and resolves off it, a passing tone fills the space between beats.
/// It is passed in rather than assumed so the cost of enforcing it can be
/// measured separately from the cost of the figures themselves.
pub fn figure_of(v: &Voice, i: usize, strong: bool, metric: bool) -> Figure {
  let n = v.notes[i];
  let prev = i.checked_sub(1).map(|j| v.notes[j]);
  let next = v.notes.get(i + 1).copied();

  // fourth species: tied over, and it must fall
  if !n.attack {
    let falls = next.map_or(false, |m| steps(n.pitch, m.pitch) == -1);
    if falls && (!metric || strong) {
      return Figure::Suspension;
    }
    return Figure::Unlisted;
  }

  let (Some(p), Some(m)) = (prev, next) else { return Figure::Unlisted };
  let in_ = steps(p.pitch, n.pitch);
  let out = steps(n.pitch, m.pitch);
  if in_.abs() != 1 {
    return Figure::Unlisted; // Fux admits no dissonance entered by leap
  }
  if metric && strong {
    return Figure::Unlisted; // a struck dissonance on the beat is a suspension or nothing
  }
  match out {
    o if o == in_ => Figure::Passing,               // straight on
    o if o == -in_ && m.pitch == p.pitch => Figure::Neighbour, // and back
    _ => Figure::Unlisted,
  }
}

#[derive(Default, Clone)]
pub struct Tally {
  pub slices: usize,
  pub dissonant: usize,
  pub by_figure: BTreeMap<&'static str, usize>,
  /// Unlisted dissonances by `(interval steps, semitones)`, for diagnosis.
  pub unlisted: BTreeMap<(i16, i16), usize>,
}

impl Tally {
  pub fn merge(&mut self, o: &Tally) {
    self.slices += o.slices;
    self.dissonant += o.dissonant;
    for (k, v) in &o.by_figure {
      *self.by_figure.entry(k).or_default() += v;
    }
    for (k, v) in &o.unlisted {
      *self.unlisted.entry(*k).or_default() += v;
    }
  }
  /// The fraction of dissonances the whitelist accounts for.
  pub fn explained(&self) -> f64 {
    if self.dissonant == 0 {
      return 1.0;
    }
    let bad = self.by_figure.get("UNLISTED").copied().unwrap_or(0);
    (self.dissonant - bad) as f64 / self.dissonant as f64
  }
  /// Unlisted dissonances per thousand slices — comparable with §8.2's column.
  pub fn per_thousand(&self) -> f64 {
    let bad = self.by_figure.get("UNLISTED").copied().unwrap_or(0);
    1000.0 * bad as f64 / self.slices.max(1) as f64
  }
}

/// Walk one pair of voices and classify every dissonance against the whitelist.
///
/// A dissonance is accounted for if **either** voice is making a figure that
/// explains it, which is how the species work: one part holds while the other
/// passes or suspends against it.
/// `fourth` says whether a perfect fourth counts as a consonance.
///
/// It is a *dissonance* everywhere else in this project, which is the classical
/// position and the one `pitch.rs` documents adopting. That position is a
/// two-voice one. In three parts or more a fourth between upper voices over a
/// supporting bass is a consonance, and only a fourth against the bass is not —
/// so a pairwise walk through a fugue reports fourths as dissonances that no
/// figure will ever explain. Passing this as a parameter is how the cost of the
/// choice gets measured instead of assumed.
pub fn check_pair(va: &Voice, vb: &Voice, beat: i64, metric: bool, fourth: bool) -> Tally {
  let mut t = Tally::default();
  for time in kern::slices(va, vb) {
    let (Some((pa, _)), Some((pb, _))) = (kern::sounding(va, time), kern::sounding(vb, time)) else {
      continue;
    };
    t.slices += 1;
    let iv = Interval::between(if pb.chroma() < pa.chroma() { pb } else { pa }, if pb.chroma() < pa.chroma() { pa } else { pb });
    let simple = iv.simple();
    let is_fourth = simple.steps == 3 && simple.semis == 5;
    if iv.quality().is_consonant() || (fourth && is_fourth) {
      *t.by_figure.entry(Figure::Consonance.name()).or_default() += 1;
      continue;
    }
    t.dissonant += 1;
    let strong = beat > 0 && time % beat == 0;
    let best = [(va, index_at(va, time)), (vb, index_at(vb, time))]
      .iter()
      .filter_map(|(v, i)| i.map(|i| figure_of(v, i, strong, metric)))
      .filter(|f| f.listed())
      .min();
    match best {
      Some(f) => *t.by_figure.entry(f.name()).or_default() += 1,
      None => {
        *t.by_figure.entry(Figure::Unlisted.name()).or_default() += 1;
        *t.unlisted.entry((iv.simple().steps, iv.simple().semis)).or_default() += 1;
      }
    }
  }
  t
}

pub fn check_piece(p: &kern::Piece, metric: bool, fourth: bool) -> Tally {
  let mut t = Tally::default();
  for a in 0..p.voices.len() {
    for b in (a + 1)..p.voices.len() {
      let x = check_pair(&p.voices[a], &p.voices[b], p.beat, metric, fourth);
      t.merge(&x);
    }
  }
  t
}

/// Unused by the checker, and here because the generator will want it: does this
/// slice belong to the whitelist at all?
pub fn permitted(v: &Voice, i: usize, consonant: bool, strong: bool, metric: bool) -> bool {
  consonant || figure_of(v, i, strong, metric).listed()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::kern::TICKS_PER_QUARTER as Q;

  fn line(steps: &[(i16, bool)]) -> Voice {
    Voice {
      notes: steps
        .iter()
        .enumerate()
        .map(|(i, &(s, attack))| Note { onset: i as i64 * Q, dur: Q, pitch: Pitch::new(s, 0), attack })
        .collect(),
    }
  }

  #[test]
  fn the_four_figures_are_recognised() {
    // C D E — a passing tone on the middle note
    let v = line(&[(28, true), (29, true), (30, true)]);
    assert_eq!(figure_of(&v, 1, false, true), Figure::Passing);
    // C D C — a neighbour
    let v = line(&[(28, true), (29, true), (28, true)]);
    assert_eq!(figure_of(&v, 1, false, true), Figure::Neighbour);
    // tied, then falling a step — a suspension
    let v = line(&[(29, true), (29, false), (28, true)]);
    assert_eq!(figure_of(&v, 1, true, true), Figure::Suspension);
  }

  #[test]
  fn a_leap_into_a_dissonance_is_not_listed() {
    let v = line(&[(28, true), (31, true), (32, true)]);
    assert_eq!(figure_of(&v, 1, false, true), Figure::Unlisted);
  }

  #[test]
  fn a_struck_dissonance_on_the_beat_is_not_listed() {
    // stepwise in and on, but struck on a strong position: Fux wants a
    // suspension there, and the metric condition is what says so
    let v = line(&[(28, true), (29, true), (30, true)]);
    assert_eq!(figure_of(&v, 1, true, true), Figure::Unlisted);
    assert_eq!(figure_of(&v, 1, true, false), Figure::Passing);
  }

  #[test]
  fn a_suspension_that_does_not_fall_is_not_listed() {
    let v = line(&[(29, true), (29, false), (30, true)]);
    assert_eq!(figure_of(&v, 1, true, true), Figure::Unlisted);
  }
}
