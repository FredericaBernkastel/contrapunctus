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
  let (va, vb) = (&p.voices[a], &p.voices[b]);
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
    let crossed = pb.chroma() < pa.chroma();
    let (lo_p, hi_p) = if crossed { (pb, pa) } else { (pa, pb) };
    let (lo_m, hi_m) = if crossed { (mv_b, mv_a) } else { (mv_a, mv_b) };
    let (lo_t, hi_t) = if crossed { (!ab, !aa) } else { (!aa, !ab) };

    let sym = Sym {
      vert: Vert::of(Interval::between(lo_p, hi_p)),
      lo: lo_m,
      hi: hi_m,
      lo_tied: lo_t,
      hi_tied: hi_t,
      downbeat: p.measure > 0 && time % p.measure == 0,
      crossed,
    };
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
