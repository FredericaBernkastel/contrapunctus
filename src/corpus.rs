//! Run the automaton as a **checker** over Bach, and count what it flags.
//!
//! This is the cheapest informative experiment in the roadmap. A rulebook that
//! flags Bach on every page has the Schottstaedt failure of readme §5 — legal
//! output that is empty where the style lives — and it is far better to learn
//! that from a checker in a second than from a composition in a month.
//!
//! Two things must be said about what this measures, before any number is
//! reported. **A pair of voices drawn from a five-voice fugue is not a
//! two-voice exercise.** Fux's rules govern a texture complete in itself; a
//! seventh between alto and bass is ordinary when a third voice supplies the
//! chord that explains it. And **the WTC is free counterpoint, not species
//! counterpoint.** So a high flag rate is not automatically evidence that the
//! rules are wrong — it may be evidence that pairwise checking is the wrong
//! scope, which is a finding about readme §3's clique relaxation rather than
//! about Fux.

use crate::{
  automaton::{self, Move, Rule, State, Sym, Vert},
  kern::{self, Piece},
  pitch::Interval,
};
use std::collections::BTreeMap;

#[derive(Default, Clone)]
pub struct Tally {
  pub slices: usize,
  pub pairs: usize,
  /// Note-to-note moves within a single voice — the denominator for melody.
  pub melodic_moves: usize,
  pub by_rule: BTreeMap<&'static str, usize>,
  /// Flagged melodic intervals as (diatonic size, semitones), for diagnosis.
  pub melodic: BTreeMap<(i16, i16), usize>,
}

impl Tally {
  pub fn merge(&mut self, other: &Tally) {
    self.slices += other.slices;
    self.pairs += other.pairs;
    self.melodic_moves += other.melodic_moves;
    for (k, v) in &other.by_rule {
      *self.by_rule.entry(k).or_default() += v;
    }
    for (k, v) in &other.melodic {
      *self.melodic.entry(*k).or_default() += v;
    }
  }
  pub fn hard_total(&self) -> usize {
    automaton::HARD.iter().filter_map(|r| self.by_rule.get(r.name())).sum()
  }
}

/// Walk one pair of voices and tally every rule that fires.
pub fn check_pair(p: &Piece, a: usize, b: usize) -> Tally {
  check_voices(&p.voices[a], &p.voices[b], p.measure)
}

/// What one voice contributes to one slice: the pitch it sounds, whether it is
/// struck there rather than held over, and how it moved into it.
pub type Sounding = (crate::pitch::Pitch, bool, Move);

/// Assemble the symbol for one slice of one pair — the *only* place the lo/hi
/// roles are assigned.
///
/// The checker and the generator both call this. That is the point of its
/// existing: §8.6 fills free voices by refusing exactly the transitions this
/// function's symbol makes the automaton reject, so if the two ever computed
/// the symbol differently, the generator could emit counterpoint its own
/// checker then flags, and neither number would mean anything.
pub fn pair_sym(a: Sounding, b: Sounding, downbeat: bool) -> Sym {
  let (pa, struck_a, mv_a) = a;
  let (pb, struck_b, mv_b) = b;
  let crossed = pb.chroma() < pa.chroma();
  let (lo_p, hi_p) = if crossed { (pb, pa) } else { (pa, pb) };
  let (lo_m, hi_m) = if crossed { (mv_b, mv_a) } else { (mv_a, mv_b) };
  let (lo_t, hi_t) = if crossed { (!struck_b, !struck_a) } else { (!struck_a, !struck_b) };
  Sym {
    vert: Vert::of(Interval::between(lo_p, hi_p)),
    lo: lo_m,
    hi: hi_m,
    lo_tied: lo_t,
    hi_tied: hi_t,
    downbeat,
    crossed,
  }
}

/// The same, on two voices that need not come from a parsed piece — which is
/// what step 2 needs, since it checks *placements* rather than scores.
pub fn check_voices(va: &kern::Voice, vb: &kern::Voice, measure: i64) -> Tally {
  check_voices_in(va, vb, measure, Fourth::Pairwise, &|_| None)
}

/// Which fourths count as dissonances — readme §8.12.
///
/// §2.2's automaton judges a **pair**, and a pair is all a two-voice exercise
/// has. In three parts or more the perfect fourth is the one interval whose
/// quality depends on a voice neither member of the pair can see: over a
/// supporting bass it is a consonance, and only against the bass is it not.
/// §8.7 measured the fourth at 31% of Bach's flagged dissonances and 44% of the
/// Renaissance's, and Marpurg's chapter on invertible counterpoint arrives at
/// the same place from the other side — the fifth must be handled as a
/// dissonance *because inversion turns it into a fourth*.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fourth {
  /// Every fourth is a dissonance, which is what the automaton has always done.
  Pairwise,
  /// A dissonance only when nothing sounds below it.
  OverBass,
  /// None is, which is the blunt control that says how much of the effect is
  /// the *scope* and how much is simply exempting the interval.
  Consonant,
}

/// [`check_voices`] with the scope a fourth is judged in, and a way to ask what
/// sounds below the pair at a given tick.
///
/// One function rather than two, because a checker that drifts from its own
/// variant is how this repository has produced wrong numbers before.
pub fn check_voices_in(
  va: &kern::Voice,
  vb: &kern::Voice,
  measure: i64,
  fourth: Fourth,
  bass: &dyn Fn(i64) -> Option<crate::pitch::Pitch>,
) -> Tally {
  let mut t = Tally { pairs: 1, ..Default::default() };
  let mut st = State::default();
  // **Per voice**, not per role. An earlier version tracked the previous
  // *lower* and *upper* pitch, so whenever the voices crossed it computed each
  // melodic interval between two different singers — inventing leaps nobody
  // sang, and corrupting the motion type that parallel detection rests on.
  let (mut prev_a, mut prev_b): (Option<crate::pitch::Pitch>, Option<crate::pitch::Pitch>) =
    (None, None);

  for time in kern::slices(va, vb) {
    let (Some((pa, aa)), Some((pb, ab))) = (kern::sounding(va, time), kern::sounding(vb, time))
    else {
      // one voice is resting: no interval, and the thread of obligation breaks
      st = State::default();
      prev_a = None;
      prev_b = None;
      continue;
    };
    // Each voice's motion comes from its own history; roles are assigned after.
    let (mv_a, mv_b) = (Move::of(prev_a, pa), Move::of(prev_b, pb));
    let mut sym = pair_sym((pa, aa, mv_a), (pb, ab, mv_b), measure > 0 && time % measure == 0);
    if fourth != Fourth::Pairwise {
      let (lo_p, hi_p) = if pa.chroma() <= pb.chroma() { (pa, pb) } else { (pb, pa) };
      let s = Interval::between(lo_p, hi_p).simple();
      let exempt = match fourth {
        Fourth::Consonant => true,
        // strictly below, so a voice doubling the pair's own bass does not
        // support it — that is the same note, not a foundation under it
        Fourth::OverBass => bass(time).map_or(false, |b| b.chroma() < lo_p.chroma()),
        Fourth::Pairwise => false,
      };
      if s.steps == 3 && s.semis == 5 && exempt {
        sym.vert = Vert::Imperfect;
      }
    }
    let (fired, next) = automaton::step(st, sym);
    for r in fired {
      if r == Rule::ForbiddenMelodic {
        continue; // counted above, from real intervals
      }
      *t.by_rule.entry(r.name()).or_default() += 1;
    }
    st = next;
    prev_a = Some(pa);
    prev_b = Some(pb);
    t.slices += 1;
  }
  t
}

/// Melody is a property of **one voice**, so it is checked once per voice —
/// not inside the pairwise walk, where a voice of a five-part fugue would have
/// every one of its intervals counted four times over, once per pair it
/// belongs to. The first run made exactly that mistake and reported 39
/// forbidden intervals per thousand slices, which is not a believable thing to
/// say about Bach and was not in fact being said about Bach.
pub fn check_melody(v: &kern::Voice, t: &mut Tally) {
  let mut prev: Option<crate::pitch::Pitch> = None;
  for n in &v.notes {
    if !n.attack {
      continue; // tied continuation: not a melodic move
    }
    if let Some(x) = prev {
      if x != n.pitch {
        t.melodic_moves += 1;
        let iv = Interval::between(x, n.pitch);
        if iv.is_forbidden_melodic() {
          *t.by_rule.entry(Rule::ForbiddenMelodic.name()).or_default() += 1;
          *t.melodic.entry((iv.steps, iv.semis)).or_default() += 1;
        }
      }
    }
    prev = Some(n.pitch);
  }
}

pub fn check_piece(p: &Piece) -> Tally {
  let mut t = Tally::default();
  for a in 0..p.voices.len() {
    for b in (a + 1)..p.voices.len() {
      let x = check_pair(p, a, b);
      t.merge(&x);
    }
  }
  for v in &p.voices {
    check_melody(v, &mut t);
  }
  t
}
