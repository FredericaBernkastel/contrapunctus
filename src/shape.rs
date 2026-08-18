//! Melodic shape as a **non-local** criterion — readme §9 step 6, §2.5.
//!
//! Every criterion in the soft tier looks at one slice or two. §8.6 measured
//! what that leaves out: pitch class is recovered about twice as often as pitch,
//! so the search picks the right note of the chord and the wrong octave, and
//! **register is a property of a line over a phrase** that nothing in the tier
//! can see.
//!
//! §2.5 names the machinery — Schottstaedt's `TotalRange`, `PitchRepeats` and
//! `TooMuchOfInterval` are accumulators, and an accumulator is finite-state
//! whenever its range is bounded — and also names the cost: the state count is
//! where it stops being small. Carrying a running minimum and maximum per free
//! voice would multiply an already-exploding search by a few hundred.
//!
//! So this does not go in the dynamic programme. §8.6's sampler draws whole
//! legal fills uniformly; a criterion over a *complete line* can rank them after
//! the fact, which needs no state at all. Sample and rerank buys the long-range
//! criterion for the price of the samples, and if a shape criterion is worth
//! having this is the cheapest way to find out.
//!
//! **Each criterion is transcribed and reported alone.** Combining them needs
//! weights, no weighting is defensible (§5), and §8.6 has already measured what
//! happens when one is invented anyway. Reporting them separately says which one
//! carries the signal, if any does.

use crate::kern::Voice;

/// Notes a line actually strikes, in order. Ties are continuations, not events.
fn struck(v: &Voice) -> Vec<i16> {
  v.notes.iter().filter(|n| n.attack).map(|n| n.pitch.chroma()).collect()
}

/// **One climax.** Fux asks a melody to rise to a single high point; a line that
/// touches its ceiling four times has no shape, and one that never leaves the
/// floor has none either. Scored as the reciprocal of how often the highest
/// pitch is struck, so a unique summit scores 1.
pub fn climax(v: &Voice) -> f64 {
  let p = struck(v);
  let Some(&hi) = p.iter().max() else { return 0.0 };
  let n = p.iter().filter(|&&x| x == hi).count();
  1.0 / n as f64
}

/// **A bounded compass.** Fux keeps a line inside about a tenth — a singer's
/// comfortable reach, and 16 semitones is a major tenth — but he is describing a
/// melody that *goes* somewhere within it, not one that sits still.
///
/// The first version scored 1 for anything at or below the bound, which handed a
/// perfect score to a line that never moves at all: a monotone has a range of
/// zero and is trivially inside a tenth. That rewards precisely what §8.6
/// complains about, and a guard test asking whether each criterion can separate
/// a shaped line from a flat one is what caught it. A range is now scored
/// against the bound in both directions: full marks at a tenth, less for
/// exceeding it, and less for not using it.
pub fn compass(v: &Voice) -> f64 {
  let p = struck(v);
  let (Some(&lo), Some(&hi)) = (p.iter().min(), p.iter().max()) else { return 0.0 };
  let range = (hi - lo) as f64;
  const BOUND: f64 = 16.0;
  if range <= BOUND {
    range / BOUND
  } else {
    BOUND / range
  }
}

/// **Not the same note over and over.** Schottstaedt's `PitchRepeats`, and the
/// thing §8.6's scalarisation table shows the local tier cannot prevent: several
/// of those fills sit on one pitch for eight notes and are perfectly legal.
/// Scored as the fraction of adjacent pairs that move.
pub fn variety(v: &Voice) -> f64 {
  let p = struck(v);
  if p.len() < 2 {
    return 0.0;
  }
  let moved = p.windows(2).filter(|w| w[0] != w[1]).count();
  moved as f64 / (p.len() - 1) as f64
}

/// The three at equal weight — reported beside them, not instead of them, since
/// equal weight is a choice like any other and §5 is about exactly that.
pub fn combined(v: &Voice) -> f64 {
  (climax(v) + compass(v) + variety(v)) / 3.0
}

pub const CRITERIA: [(&str, fn(&Voice) -> f64); 4] =
  [("climax", climax), ("compass", compass), ("variety", variety), ("all three", combined)];

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{kern::Note, kern::TICKS_PER_QUARTER as Q, pitch::Pitch};

  fn line(steps: &[i16]) -> Voice {
    Voice {
      notes: steps
        .iter()
        .enumerate()
        .map(|(i, &s)| Note { onset: i as i64 * Q, dur: Q, pitch: Pitch::new(s, 0), attack: true })
        .collect(),
    }
  }

  #[test]
  fn a_single_summit_beats_a_plateau() {
    assert!(climax(&line(&[28, 30, 32, 30, 28])) > climax(&line(&[28, 32, 30, 32, 28])));
    assert_eq!(climax(&line(&[28, 30, 32, 30, 28])), 1.0);
  }

  #[test]
  fn the_compass_rewards_a_tenth_and_penalises_both_extremes() {
    // `line` builds pitches by diatonic step and the criterion measures
    // semitones, so a major tenth is nine steps up from C: C4 to E5.
    assert_eq!(Pitch::new(37, 0).chroma() - Pitch::new(28, 0).chroma(), 16);
    assert_eq!(compass(&line(&[28, 37])), 1.0);
    // beyond it, and short of it, both score less
    assert!(compass(&line(&[21, 28, 45])) < 1.0);
    assert!(compass(&line(&[28, 30, 32])) < 1.0);
    // and a line that never moves scores nothing
    assert_eq!(compass(&line(&[30, 30, 30])), 0.0);
  }

  /// The conjunction is not a fourth criterion sneaking in. Averaging three
  /// scores that each live in `[0,1]` must land between the least and greatest
  /// of them, so any gain it shows has to come from the three and not from the
  /// averaging — which is what makes §8.8's result readable as a compromise
  /// rather than as an artefact of the arithmetic.
  #[test]
  fn the_conjunction_is_bounded_by_its_parts() {
    for l in [
      line(&[28, 30, 32, 30, 28]),
      line(&[30, 30, 30, 30]),
      line(&[21, 28, 42, 28]),
      line(&[28, 29, 30, 31, 30, 29, 28]),
    ] {
      let (c, k, v) = (climax(&l), compass(&l), variety(&l));
      let all = combined(&l);
      let lo = c.min(k).min(v);
      let hi = c.max(k).max(v);
      assert!(all >= lo - 1e-9 && all <= hi + 1e-9, "combined {all} outside [{lo}, {hi}]");
    }
  }

  /// Each criterion must actually discriminate — a scorer that returns the same
  /// number for every line would rank samples arbitrarily and could still show a
  /// gain by luck.
  #[test]
  fn each_criterion_separates_some_pair_of_lines() {
    let good = line(&[28, 29, 30, 31, 30, 29, 28]);
    let bad = line(&[30, 30, 30, 30, 30, 30, 30]);
    for (name, f) in CRITERIA {
      assert!(f(&good) != f(&bad), "{name} cannot tell a shaped line from a flat one");
    }
  }

  #[test]
  fn standing_still_scores_nothing_for_variety() {
    assert_eq!(variety(&line(&[30, 30, 30, 30])), 0.0);
    assert_eq!(variety(&line(&[28, 30, 32, 30])), 1.0);
  }
}
