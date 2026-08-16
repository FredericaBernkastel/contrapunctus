//! The two-voice counterpoint automaton — readme §2.2.
//!
//! The state is **the interval, plus what you owe.** A dissonance incurs an
//! obligation to resolve; a leap incurs an obligation to recover. Obligations
//! must be discharged on the very next event, which is what "strict" means and
//! is why the set stays small enough to enumerate.
//!
//! Rules are split **hard versus soft**, which readme §5 credits to
//! Schottstaedt (1984) rather than to anything recent: he writes the hard ones
//! as an `Infinity` penalty and the rest as small integers. The hard ones are
//! transitions the automaton refuses; the soft ones are counted and left
//! uncombined, for the Pareto front of §5 to order later. Nothing here is
//! weighted, because no weight in the literature is defensible.

use crate::pitch::{Interval, Pitch, Quality};

/// The vertical interval, reduced to what the rules actually distinguish.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Vert {
  PerfectUnisonOctave,
  PerfectFifth,
  Imperfect,
  Dissonant,
}

impl Vert {
  pub fn of(iv: Interval) -> Self {
    let s = iv.simple();
    match iv.quality() {
      Quality::PerfectConsonance => {
        if s.steps == 0 { Vert::PerfectUnisonOctave } else { Vert::PerfectFifth }
      }
      Quality::ImperfectConsonance => Vert::Imperfect,
      Quality::Dissonance => Vert::Dissonant,
    }
  }
  pub fn is_perfect(&self) -> bool {
    matches!(self, Vert::PerfectUnisonOctave | Vert::PerfectFifth)
  }
  pub fn is_dissonant(&self) -> bool {
    matches!(self, Vert::Dissonant)
  }
}

/// How a voice moved into this slice.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Move {
  Hold,
  StepUp,
  StepDown,
  LeapUp,
  LeapDown,
}

impl Move {
  pub fn of(prev: Option<Pitch>, now: Pitch) -> Self {
    let Some(p) = prev else { return Move::Hold };
    let iv = Interval::between(p, now);
    match (iv.steps.signum(), iv.is_step()) {
      (0, _) => Move::Hold,
      (1, true) => Move::StepUp,
      (-1, true) => Move::StepDown,
      (1, false) => Move::LeapUp,
      (-1, false) => Move::LeapDown,
      _ => Move::Hold,
    }
  }
  pub fn dir(&self) -> i8 {
    match self {
      Move::StepUp | Move::LeapUp => 1,
      Move::StepDown | Move::LeapDown => -1,
      Move::Hold => 0,
    }
  }
  pub fn is_leap(&self) -> bool {
    matches!(self, Move::LeapUp | Move::LeapDown)
  }
  pub fn is_step(&self) -> bool {
    matches!(self, Move::StepUp | Move::StepDown)
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Motion {
  Parallel,
  Similar,
  Contrary,
  Oblique,
  None,
}

fn motion(lo: Move, hi: Move, same_interval: bool) -> Motion {
  match (lo.dir(), hi.dir()) {
    (0, 0) => Motion::None,
    (0, _) | (_, 0) => Motion::Oblique,
    (a, b) if a == b => {
      if same_interval { Motion::Parallel } else { Motion::Similar }
    }
    _ => Motion::Contrary,
  }
}

/// What the automaton reads at one slice. Finite by construction, which is what
/// makes the reachable-state count in [`reachable_states`] a real measurement.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct Sym {
  pub vert: Vert,
  pub lo: Move,
  pub hi: Move,
  /// The dissonating voice is tied over from the previous slice — a prepared
  /// suspension rather than a struck dissonance. This single bit is what lets
  /// the model tell them apart, which readme §8 names as verdict test 3.
  pub lo_tied: bool,
  pub hi_tied: bool,
  pub downbeat: bool,
  pub crossed: bool,
}

// Outstanding obligations, one bit each — "what you owe".
//
// A dissonance owes differently according to **how it was entered**, and an
// earlier version missed this, demanding a downward step in every case. That
// one error produced 79% of all violations when the checker was first run
// against Bach. A *suspension* resolves downward; a *passing* dissonance
// leaves by step in either direction — Schottstaedt's third-species comment
// says it outright, "can be if passing either way". Hence two kinds of debt.
pub const RESOLVE_LO: u8 = 1; // suspension: must step *down*
pub const RESOLVE_HI: u8 = 2;
pub const RECOVER_LO_UP: u8 = 4;
pub const RECOVER_LO_DOWN: u8 = 8;
pub const RECOVER_HI_UP: u8 = 16;
pub const RECOVER_HI_DOWN: u8 = 32;
pub const LEAVE_LO: u8 = 64; // passing/neighbour: must step, either direction
pub const LEAVE_HI: u8 = 128;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord, Default)]
pub struct State {
  pub prev: Option<Vert>,
  /// Outstanding obligations. This is the "what you owe" half of the state.
  pub owed: u8,
}

/// Rules, tagged by tier. Hard rules reject a transition; soft rules are
/// counted and never summed.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Rule {
  // hard
  ParallelPerfect,
  DirectPerfectOnDownbeat,
  UnpreparedDissonance,
  UnresolvedDissonance,
  ForbiddenMelodic,
  // soft
  DirectToPerfect,
  PerfectConsonance,
  DirectMotion,
  VoiceCrossing,
  UnrecoveredLeap,
  NoteRepetition,
}

pub const HARD: &[Rule] = &[
  Rule::ParallelPerfect,
  Rule::DirectPerfectOnDownbeat,
  Rule::UnpreparedDissonance,
  Rule::UnresolvedDissonance,
  Rule::ForbiddenMelodic,
];

/// The hard rules **Bach confirms** — §9.4 measured these at about one per
/// thousand slices across the whole book, where the other three fire two
/// orders of magnitude more often and are refuted by him. Step 2 tests the
/// stretto against both tiers, because which tier is "the rulebook" is exactly
/// what is in question.
pub const CONFIRMED: &[Rule] = &[Rule::ParallelPerfect, Rule::DirectPerfectOnDownbeat];

pub const SOFT: &[Rule] = &[
  Rule::DirectToPerfect,
  Rule::PerfectConsonance,
  Rule::DirectMotion,
  Rule::VoiceCrossing,
  Rule::UnrecoveredLeap,
  Rule::NoteRepetition,
];

impl Rule {
  pub fn is_hard(&self) -> bool {
    HARD.contains(self)
  }
  pub fn name(&self) -> &'static str {
    match self {
      Rule::ParallelPerfect => "parallel perfect",
      Rule::DirectPerfectOnDownbeat => "direct to perfect on downbeat",
      Rule::UnpreparedDissonance => "unprepared dissonance",
      Rule::UnresolvedDissonance => "unresolved dissonance",
      Rule::ForbiddenMelodic => "forbidden melodic interval",
      Rule::DirectToPerfect => "direct to perfect",
      Rule::PerfectConsonance => "perfect consonance",
      Rule::DirectMotion => "direct motion",
      Rule::VoiceCrossing => "voice crossing",
      Rule::UnrecoveredLeap => "unrecovered leap",
      Rule::NoteRepetition => "repeated note",
    }
  }
}

/// One transition: which rules fire, and the state that follows.
///
/// The next state is returned even when hard rules fire, so a checker can keep
/// walking a real piece rather than stopping at the first disagreement. A
/// *generator* would treat any hard firing as a refused transition.
pub fn step(st: State, sym: Sym) -> (Vec<Rule>, State) {
  let mut fired = Vec::new();
  let next = step_into(st, sym, &mut fired);
  (fired, next)
}

/// The same, writing into a caller-owned buffer.
///
/// This exists for one reason: §8.6's search calls it several million times per
/// layer, and allocating a fresh `Vec` for each was by a wide margin the most
/// expensive thing in the project. The buffer is cleared, not appended to.
pub fn step_into(st: State, sym: Sym, fired: &mut Vec<Rule>) -> State {
  fired.clear();
  let mot = motion(sym.lo, sym.hi, Some(sym.vert) == st.prev);
  // A debt owed by a voice that has not yet moved is not yet broken — it is
  // still owed. Judging it while the voice holds counted one dissonance once
  // per slice created by the *other* voice, which is an artefact of slicing
  // rather than anything in the music.
  let mut carry = 0u8;

  // --- discharge, carry, or fail the outstanding obligations -------------
  for (bit, mv, descend) in [
    (RESOLVE_LO, sym.lo, true),
    (RESOLVE_HI, sym.hi, true),
    (LEAVE_LO, sym.lo, false),
    (LEAVE_HI, sym.hi, false),
  ] {
    if st.owed & bit == 0 {
      continue;
    }
    if mv == Move::Hold {
      carry |= bit;
    } else if !(if descend { mv == Move::StepDown } else { mv.is_step() }) {
      fired.push(Rule::UnresolvedDissonance);
    }
  }
  for (bit, mv, want) in [
    (RECOVER_LO_UP, sym.lo, Move::StepDown),
    (RECOVER_LO_DOWN, sym.lo, Move::StepUp),
    (RECOVER_HI_UP, sym.hi, Move::StepDown),
    (RECOVER_HI_DOWN, sym.hi, Move::StepUp),
  ] {
    if st.owed & bit == 0 {
      continue;
    }
    if mv == Move::Hold {
      carry |= bit;
    } else if mv != want {
      fired.push(Rule::UnrecoveredLeap); // soft: Fux calls this a guideline
    }
  }

  // --- hard vertical rules ---------------------------------------------
  if mot == Motion::Parallel && sym.vert.is_perfect() {
    fired.push(Rule::ParallelPerfect);
  }
  if mot == Motion::Similar && sym.vert.is_perfect() {
    if sym.downbeat && sym.hi.is_leap() {
      fired.push(Rule::DirectPerfectOnDownbeat);
    } else {
      fired.push(Rule::DirectToPerfect);
    }
  }

  // A dissonance is legal only if prepared (the dissonating voice tied over,
  // i.e. a suspension) or approached by step in the voice that moved into it.
  let mut owed = 0u8;
  if sym.vert.is_dissonant() {
    let hi_moved = sym.hi != Move::Hold;
    let lo_moved = sym.lo != Move::Hold;
    let suspended = (sym.hi_tied && !hi_moved) || (sym.lo_tied && !lo_moved);
    // **Every** voice that moves into a dissonance must move by step — not
    // merely one of them. An earlier version asked only that *some* voice
    // stepped, which accepts the textbook error case outright: one voice
    // stepping while the other leaps into the clash. Caught by a verdict test
    // that could not tell a suspension from an accident because both passed.
    let leapt_in = (hi_moved && sym.hi.is_leap()) || (lo_moved && sym.lo.is_leap());
    let stationary = !hi_moved && !lo_moved;
    // Only judge the dissonance where it begins; while it persists the
    // outstanding obligation is what governs.
    let newly = st.prev != Some(Vert::Dissonant);
    let first = st.prev.is_none();
    if !first && newly && !suspended && (leapt_in || stationary) {
      fired.push(Rule::UnpreparedDissonance);
    }
    // The debt is set where the dissonance begins, and its *kind* depends on
    // how it was entered: a suspension must descend, a passing note need only
    // continue by step.
    if newly {
      if suspended {
        owed |= if sym.hi_tied && !hi_moved { RESOLVE_HI } else { RESOLVE_LO };
      } else if hi_moved {
        owed |= LEAVE_HI;
      } else if lo_moved {
        owed |= LEAVE_LO;
      }
    }
  }
  owed |= carry;

  // --- leaps incur a recovery obligation --------------------------------
  match sym.lo {
    Move::LeapUp => owed |= RECOVER_LO_UP,
    Move::LeapDown => owed |= RECOVER_LO_DOWN,
    _ => {}
  }
  match sym.hi {
    Move::LeapUp => owed |= RECOVER_HI_UP,
    Move::LeapDown => owed |= RECOVER_HI_DOWN,
    _ => {}
  }

  // --- soft criteria ----------------------------------------------------
  if sym.vert.is_perfect() {
    fired.push(Rule::PerfectConsonance);
  }
  if mot == Motion::Similar || mot == Motion::Parallel {
    fired.push(Rule::DirectMotion);
  }
  if sym.crossed {
    fired.push(Rule::VoiceCrossing);
  }
  if sym.lo == Move::Hold && sym.hi == Move::Hold {
    fired.push(Rule::NoteRepetition);
  }

  State { prev: Some(sym.vert), owed }
}

/// Every symbol the automaton can read — the alphabet, enumerated.
pub fn alphabet() -> Vec<Sym> {
  let moves = [Move::Hold, Move::StepUp, Move::StepDown, Move::LeapUp, Move::LeapDown];
  let verts = [Vert::PerfectUnisonOctave, Vert::PerfectFifth, Vert::Imperfect, Vert::Dissonant];
  let mut out = Vec::new();
  for &vert in &verts {
    for &lo in &moves {
      for &hi in &moves {
        for lo_tied in [false, true] {
          for hi_tied in [false, true] {
            for downbeat in [false, true] {
              for crossed in [false, true] {
                out.push(Sym { vert, lo, hi, lo_tied, hi_tied, downbeat, crossed });
              }
            }
          }
        }
      }
    }
  }
  out
}

/// The measurement readme §8 step 1 asks for: how many states are actually
/// reachable, against the crude product of the state's components.
///
/// Reachability is computed over the **legal** transitions only — the ones a
/// generator would take — since a state reachable solely by breaking a hard
/// rule is not a state the automaton has.
pub fn reachable_states() -> (Vec<State>, usize) {
  let alpha = alphabet();
  let start = State::default();
  let mut seen = std::collections::BTreeSet::new();
  let mut stack = vec![start];
  seen.insert(start);
  while let Some(st) = stack.pop() {
    for &sym in &alpha {
      let (fired, next) = step(st, sym);
      if fired.iter().any(|r| r.is_hard()) {
        continue;
      }
      if seen.insert(next) {
        stack.push(next);
      }
    }
  }
  // crude product: 5 values of `prev` (including none) × 256 obligation sets
  (seen.into_iter().collect(), 5 * 256)
}
