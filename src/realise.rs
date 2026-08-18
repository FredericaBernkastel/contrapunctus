//! Step 5: realisation. Fill the free voices against fixed entries and a given
//! harmony — readme §2.5, §2.6, §8.6.
//!
//! This is the shortest path of §2.5, built as stated there and not otherwise.
//! Rhythm is data (§2.6): a free voice is handed in as a `Voice` whose *pitches
//! are discarded and whose onsets are obeyed*, so the search never chooses when
//! a note happens, only what it is. The layers of the DAG are the slices at
//! which some voice articulates; the nodes are the states of §2.2's automaton,
//! one per voice pair, carried alongside the pitches now sounding; the edges are
//! the transitions no hard rule refuses.
//!
//! Three things make this more than a textbook Viterbi.
//!
//! **The harmony is a second automaton**, exactly as §2.3 claims and in the same
//! idiom: a note foreign to the prevailing chord is legal only if prepared or
//! approached by step, and it *owes* a resolution which must be discharged on
//! the next articulation. Two obligation systems run side by side over one grid
//! — the contrapuntal one from §2.2 and this one — and neither knows about the
//! other. That they compose without special-casing is the claim being tested.
//!
//! **The same DAG is summed as well as minimised.** One pass gives the cheapest
//! fill; the identical pass with `+` for `min` gives the *exact number of legal
//! fills*, which is the quantity §5 says is the real difficulty. A generator
//! that reports only its answer cannot tell you whether it chose from two
//! candidates or from ten million, and that number is the whole question.
//!
//! **The product is taken last.** A free voice's melody, its harmony, and every
//! pair it forms with a *fixed* voice depend on that voice's own note alone, so
//! they are decided before the free voices are combined; only the free-against-
//! free pairs need the product. Written the obvious way instead — enumerate the
//! joint assignment, then test it — the corpus run did not finish, which is
//! §2.7's cost profile showing up as arithmetic rather than as an argument.
//!
//! What is *not* here: no beam, no threshold, no restarts. If the state count
//! exceeds the cap the search **fails loudly** rather than quietly becoming a
//! heuristic, because §2.7 predicts precisely where that happens and a silent
//! beam would hide the prediction coming true.

use crate::{
  automaton::{self, Move, Rule, State, SOFT},
  corpus,
  harmony::{Chord, Segment},
  kern::{Note, Voice},
  pitch::{Interval, Pitch},
};
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

/// The search inserts millions of nodes per layer and does nothing else with
/// them, so the default SipHash — which exists to resist adversarial keys — is
/// most of the running time for no benefit. This is the usual multiply-rotate
/// mixer, over a node whose bytes are already a dense key.
#[derive(Default)]
struct Fx(u64);
const FX: u64 = 0x51_7c_c1_b7_27_22_0a_95;

impl Fx {
  #[inline]
  fn add(&mut self, w: u64) {
    self.0 = (self.0.rotate_left(5) ^ w).wrapping_mul(FX);
  }
}

impl Hasher for Fx {
  #[inline]
  fn finish(&self) -> u64 {
    self.0
  }
  #[inline]
  fn write(&mut self, bytes: &[u8]) {
    let mut c = bytes.chunks_exact(8);
    for b in &mut c {
      self.add(u64::from_le_bytes(b.try_into().unwrap()));
    }
    let mut tail = 0u64;
    for (i, &b) in c.remainder().iter().enumerate() {
      tail |= (b as u64) << (8 * i);
    }
    self.add(tail);
  }
  #[inline]
  fn write_u8(&mut self, i: u8) {
    self.add(i as u64);
  }
  #[inline]
  fn write_i16(&mut self, i: i16) {
    self.add(i as u64);
  }
  #[inline]
  fn write_i8(&mut self, i: i8) {
    self.add(i as u64);
  }
  #[inline]
  fn write_usize(&mut self, i: usize) {
    self.add(i as u64);
  }
}

type Map<K, V> = HashMap<K, V, BuildHasherDefault<Fx>>;

/// Voices and pairs are bounded so a search node is `Copy` and hashes without
/// touching the heap. Five voices is the largest fugue in the book.
const MAXV: usize = 6;
const MAXFREE: usize = 4;
const MAXPAIR: usize = MAXV * (MAXV - 1) / 2;

/// What a free voice may do at one slice, read off the rhythm it was given.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
  Rest,
  /// A note is sounding but was not struck here: it must continue unchanged.
  Hold,
  Strike,
}

// Harmonic debt, one value per free voice. The same shape as §2.2's `owed`,
// deliberately: a dissonance against the prevailing chord owes a resolution,
// and how it owes depends on how it was entered.
const H_FREE: u8 = 0;
const H_DOWN: u8 = 1; // suspension: must step *down*
const H_STEP: u8 = 2; // passing or neighbour: must step, either way

/// Give up rather than beam. §2.7 puts the wall at four or more free voices;
/// this is where that prediction gets tested rather than papered over.
///
/// Two budgets, because they bound different failures. `MAX_STATES` is the
/// width of one layer. `MAX_WORK` is the total number of edges relaxed, and it
/// is the one that matters in practice: a span heading for the wall spends most
/// of its time in the layers *below* the cap, so bounding only the width lets a
/// doomed search run for ten seconds before admitting it. Bounding the work
/// aborts it in a fraction of one, and leaves every tractable span exact
/// however many layers it has.
pub const MAX_STATES: usize = 60_000;
pub const MAX_WORK: u64 = 4_000_000;

pub struct Problem<'a> {
  /// Every voice in score order. A voice marked free contributes its **rhythm
  /// only** — its pitches are never read.
  pub voices: Vec<Voice>,
  pub free: Vec<bool>,
  /// Inclusive diatonic step range for each voice; ignored for fixed voices.
  pub compass: Vec<(i16, i16)>,
  pub key: [i8; 7],
  pub measure: i64,
  /// The harmonic plan. Empty means no harmonic constraint at all, which is
  /// worth being able to ask for: it is the control that shows what the
  /// contrapuntal rules alone permit.
  pub plan: Vec<Segment>,
  pub tier: &'a [Rule],
  /// One weight per entry of `SOFT`. The scalarisation is a *choice*, and §5's
  /// position is that no choice is defensible — so it is a parameter here, and
  /// §8.6 runs several and reports that they disagree.
  pub weights: [f64; 6],
  /// How many fills to draw **uniformly at random from the legal set**, beside
  /// the cheapest one — readme §9 step 6, and WaveFunctionCollapse's Weak C2
  /// stripped of the part that needs a corpus (§7.1). Zero costs nothing; any
  /// positive number makes the search record its edge list, which the cheapest
  /// path does not need.
  pub samples: usize,
  pub seed: u64,
}

pub struct Solution {
  /// The filled voices, in the same order and length as `Problem::voices`;
  /// fixed voices are returned unchanged.
  pub voices: Vec<Voice>,
  pub cost: f64,
  /// Exact number of assignments the hard tier and the plan admit.
  pub legal_fills: u128,
  pub saturated: bool,
  pub peak_states: usize,
  pub slices: usize,
  /// Melodic intervals the free voices took that Fux forbids, in the runs where
  /// `ForbiddenMelodic` is not in the tier and they are therefore permitted.
  pub melodic_flags: usize,
  /// `Problem::samples` fills drawn uniformly from the legal set. Each is a full
  /// voice list in the same shape as `voices`.
  pub sampled: Vec<Vec<Voice>>,
}

/// The pitches a free voice may strike at one slice: the key's own scale over
/// its compass, plus any respelling the prevailing chord asks for.
///
/// The second half matters more than it looks. A plan that says `D7` in C major
/// needs an F sharp, and the *spelling* has to be F sharp rather than G flat or
/// §2.1's whole argument about diatonic pitch is given away at the last step.
/// One spelling per sounding pitch, and the key's own spelling wins.
///
/// Without this the domain offers `E##` beside `F#` wherever the chord wants
/// that sound and the key already spells it — two names for one note, doubling
/// the branching factor to no purpose and putting double sharps in the output.
/// Which is *not* a licence to collapse pitch to semitones: the alternatives are
/// merged only when they sound the same **and** one of them is the key's own,
/// so the augmented fourth and the diminished fifth of §2.1 stay distinct
/// because neither is the other's respelling within a key.
fn domain(compass: (i16, i16), key: &[i8; 7], chord: Option<Chord>) -> Vec<Pitch> {
  let mut out: Vec<Pitch> = Vec::with_capacity((compass.1 - compass.0 + 1) as usize);
  // the key's own spelling first; then the plainer accidental, so that a G
  // natural is preferred to an F double sharp when the two sound alike
  let rank = |p: Pitch| {
    ((p.alter as i16 - key[p.step.rem_euclid(7) as usize] as i16).abs(), (p.alter as i16).abs())
  };
  let add = |p: Pitch, out: &mut Vec<Pitch>| {
    match out.iter_mut().find(|q| q.chroma() == p.chroma()) {
      Some(q) if rank(p) < rank(*q) => *q = p,
      Some(_) => {}
      None => out.push(p),
    }
  };
  for step in compass.0..=compass.1 {
    let nat = Pitch::new(step, key[step.rem_euclid(7) as usize]);
    add(nat, &mut out);
    let Some(c) = chord else { continue };
    if c.contains(nat) {
      continue;
    }
    for d in [1i8, -1] {
      let alt = Pitch::new(step, nat.alter + d);
      if c.contains(alt) {
        add(alt, &mut out);
      }
    }
  }
  out
}

/// What each voice does at each slice: `Ok` for a fixed voice's sounding note,
/// `Err` for a free voice's mode.
type Cell = Result<Option<(Pitch, bool)>, Mode>;

fn plan_at(plan: &[Segment], t: i64) -> Option<Chord> {
  plan.iter().find(|s| s.start <= t && t < s.end).and_then(|s| s.chord)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Node {
  /// Pitch each *free* voice sounds now, `None` while resting.
  now: [Option<Pitch>; MAXFREE],
  /// §2.2's automaton state, one per pair involving at least one free voice.
  pair: [State; MAXPAIR],
  harm: [u8; MAXFREE],
}

struct Layer {
  nodes: Vec<Node>,
  index: Map<Node, usize>,
  cost: Vec<f64>,
  back: Vec<u32>,
  took: Vec<[Option<Pitch>; MAXFREE]>,
  count: Vec<u128>,
  flags: Vec<u32>,
}

impl Layer {
  fn new() -> Self {
    Layer {
      nodes: vec![],
      index: Map::default(),
      cost: vec![],
      back: vec![],
      took: vec![],
      count: vec![],
      flags: vec![],
    }
  }
  /// Returns the index of the node relaxed into, so a caller recording the edge
  /// list for §7.1's sampler knows where the edge landed.
  fn relax(&mut self, n: Node, c: f64, from: u32, took: [Option<Pitch>; MAXFREE], cnt: u128, fl: u32) -> usize {
    match self.index.get(&n) {
      Some(&i) => {
        self.count[i] = self.count[i].saturating_add(cnt);
        if c < self.cost[i] {
          self.cost[i] = c;
          self.back[i] = from;
          self.took[i] = took;
          self.flags[i] = fl;
        }
        i
      }
      None => {
        let i = self.nodes.len();
        self.index.insert(n, i);
        self.nodes.push(n);
        self.cost.push(c);
        self.back.push(from);
        self.took.push(took);
        self.count.push(cnt);
        self.flags.push(fl);
        i
      }
    }
  }
}

/// Which voices a pair joins, once it is known which are free.
#[derive(Clone, Copy)]
enum Kind {
  /// free voice `k` against fixed voice `v`
  Fixed(usize, usize),
  /// free voice `k1` against free voice `k2`
  Both(usize, usize),
}

/// One free voice's candidate note, already judged against everything that does
/// not involve another free voice.
#[derive(Clone, Copy)]
struct Opt {
  p: Option<Pitch>,
  harm: u8,
  cost: f64,
  flags: u32,
  /// New state for each pair of kind `Fixed(k, _)` belonging to this voice.
  st: [State; MAXPAIR],
}

/// Fill the free voices. Exact: the returned fill is the cheapest one no hard
/// rule refuses, and `legal_fills` counts every one there was to choose from.
pub fn fill(pr: &Problem) -> Result<Solution, String> {
  let nv = pr.voices.len();
  if pr.free.len() != nv || pr.compass.len() != nv {
    return Err("free/compass do not match the voice count".into());
  }
  if nv > MAXV {
    return Err(format!("{nv} voices, {MAXV} supported"));
  }
  let freeix: Vec<usize> = (0..nv).filter(|&v| pr.free[v]).collect();
  if freeix.is_empty() {
    return Err("nothing to fill".into());
  }
  if freeix.len() > MAXFREE {
    return Err(format!("{} free voices, {MAXFREE} supported", freeix.len()));
  }

  // --- the layers: every instant at which some voice articulates ------------
  let mut times: Vec<i64> = pr.voices.iter().flat_map(|v| v.notes.iter().map(|n| n.onset)).collect();
  times.sort_unstable();
  times.dedup();
  if times.is_empty() {
    return Err("no notes".into());
  }

  // pairs the fill is answerable for: those with a free voice in them
  let mut pairs: Vec<(usize, usize)> = vec![];
  let mut kinds: Vec<Kind> = vec![];
  for a in 0..nv {
    for b in (a + 1)..nv {
      let (fa, fb) = (pr.free[a], pr.free[b]);
      if !fa && !fb {
        continue;
      }
      pairs.push((a, b));
      let k = |v: usize| freeix.iter().position(|&f| f == v).unwrap();
      kinds.push(match (fa, fb) {
        (true, true) => Kind::Both(k(a), k(b)),
        (true, false) => Kind::Fixed(k(a), b),
        (false, true) => Kind::Fixed(k(b), a),
        _ => unreachable!(),
      });
    }
  }

  // --- what every voice does at every slice, decided before the search ------
  //
  // Every one of these is path-independent, including *whether a voice rests*,
  // because rhythm is data. That is what makes the layer structure static.
  let cells: Vec<Vec<Cell>> = times
    .iter()
    .map(|&t| {
      (0..nv)
        .map(|v| {
          if !pr.free[v] {
            return Ok(crate::kern::sounding(&pr.voices[v], t));
          }
          let on = pr.voices[v].notes.iter().find(|n| n.onset <= t && t < n.onset + n.dur);
          Err(match on {
            None => Mode::Rest,
            // a note already sounding at the first slice has to be chosen here,
            // there being no earlier slice to have chosen it at
            Some(n) if (n.onset == t && n.attack) || t == times[0] => Mode::Strike,
            Some(_) => Mode::Hold,
          })
        })
        .collect()
    })
    .collect();

  let sounds: Vec<Vec<bool>> = cells
    .iter()
    .map(|row| row.iter().map(|c| !matches!(c, Ok(None) | Err(Mode::Rest))).collect())
    .collect();
  let pitch_at: Vec<Vec<Option<Pitch>>> = cells
    .iter()
    .map(|row| row.iter().map(|c| match c { Ok(x) => x.map(|(p, _)| p), Err(_) => None }).collect())
    .collect();
  let doms: Vec<Vec<Vec<Pitch>>> = times
    .iter()
    .map(|&t| {
      let c = plan_at(&pr.plan, t);
      freeix.iter().map(|&v| domain(pr.compass[v], &pr.key, c)).collect()
    })
    .collect();

  // --- the search ----------------------------------------------------------
  let mut layers: Vec<Layer> = Vec::with_capacity(times.len() + 1);
  let mut prev = Layer::new();
  prev.relax(
    Node { now: [None; MAXFREE], pair: [State::default(); MAXPAIR], harm: [H_FREE; MAXFREE] },
    0.0,
    u32::MAX,
    [None; MAXFREE],
    1,
    0,
  );
  let mut peak = 1usize;
  let mut work = 0u64;
  let hard_melodic = pr.tier.contains(&Rule::ForbiddenMelodic);

  let mut opts: Vec<Vec<Opt>> = vec![vec![]; freeix.len()];
  let mut fired: Vec<Rule> = Vec::with_capacity(8);
  // Recorded only when a sample is asked for. `(from, to)` is all that is
  // needed: the pick that produced an edge is recoverable from the node it
  // lands on, since a node's key *is* the pitches the free voices took.
  let mut edges: Vec<Vec<(u32, u32)>> = Vec::new();
  let mut edge_buf: Vec<(u32, u32)> = Vec::new();
  for (s, &t) in times.iter().enumerate() {
    let downbeat = pr.measure > 0 && t % pr.measure == 0;
    let chord = plan_at(&pr.plan, t);
    let broke = |a: usize, b: usize| s == 0 || !sounds[s - 1][a] || !sounds[s - 1][b];
    let mut cur = Layer::new();
    edge_buf.clear();

    for i in 0..prev.nodes.len() {
      if work > MAX_WORK {
        break; // the budget is spent; the check below turns this into a refusal
      }
      let st = prev.nodes[i];

      // --- each free voice on its own, against every fixed voice ------------
      let mut feasible = true;
      for (k, &v) in freeix.iter().enumerate() {
        opts[k].clear();
        let mode = match cells[s][v] {
          Err(m) => m,
          Ok(_) => unreachable!(),
        };
        // Iterated by index rather than collected. The obvious `Vec` here is
        // one heap allocation per live state per free voice per slice — some
        // hundreds of millions over a corpus run, and by a wide margin the
        // slowest thing in the search before it was removed.
        let ncand = if mode == Mode::Strike { doms[s][k].len() } else { 1 };
        for ci in 0..ncand {
          let p = match mode {
            Mode::Rest => None,
            Mode::Hold => st.now[k],
            Mode::Strike => Some(doms[s][k][ci]),
          };
          if mode == Mode::Hold && p.is_none() {
            continue; // a held voice whose predecessor was silent sounds nothing
          }
          let Some((harm, flags)) = voice_ok(st.now[k], st.harm[k], p, mode, chord, hard_melodic) else {
            continue;
          };
          // pairs against the fixed voices
          let mut o = Opt { p, harm, cost: 0.0, flags: 0, st: [State::default(); MAXPAIR] };
          let mut ok = true;
          for (pi, &(a, b)) in pairs.iter().enumerate() {
            let Kind::Fixed(kk, fv) = kinds[pi] else { continue };
            if kk != k {
              continue;
            }
            let (Some(pp), Some((pf, sf))) = (p, cells[s][fv].unwrap_or(None)) else {
              o.st[pi] = State::default();
              continue;
            };
            let br = broke(a, b);
            let was_free = if br { None } else { st.now[k] };
            let was_fix = if br { None } else { pitch_at[s - 1][fv] };
            let me = (pp, mode == Mode::Strike, Move::of(was_free, pp));
            let it = (pf, sf, Move::of(was_fix, pf));
            let sym = if v < fv { corpus::pair_sym(me, it, downbeat) } else { corpus::pair_sym(it, me, downbeat) };
            let next = automaton::step_into(st.pair[pi], sym, &mut fired);
            for r in &fired {
              if pr.tier.contains(r) {
                ok = false;
                break;
              }
              if let Some(x) = SOFT.iter().position(|s| s == r) {
                o.cost += pr.weights[x];
              }
            }
            if !ok {
              break;
            }
            o.st[pi] = next;
          }
          if !ok {
            continue;
          }
          o.flags = flags as u32;
          opts[k].push(o);
        }
        if opts[k].is_empty() {
          feasible = false;
          break;
        }
      }
      if !feasible {
        continue;
      }

      // --- and now the product, over the free-against-free pairs only -------
      let mut counts = [1usize; MAXFREE];
      let mut total = 1usize;
      for k in 0..freeix.len() {
        counts[k] = opts[k].len();
        total *= counts[k];
      }
      for combo in 0..total {
        let mut idx = combo;
        let mut chosen: [usize; MAXFREE] = [0; MAXFREE];
        for k in 0..freeix.len() {
          chosen[k] = idx % counts[k];
          idx /= counts[k];
        }
        let mut node = Node { now: [None; MAXFREE], pair: st.pair, harm: [H_FREE; MAXFREE] };
        let mut cost = 0.0f64;
        let mut flags = 0u32;
        for k in 0..freeix.len() {
          let o = &opts[k][chosen[k]];
          node.now[k] = o.p;
          node.harm[k] = o.harm;
          cost += o.cost;
          flags += o.flags;
          for (pi, kd) in kinds.iter().enumerate() {
            if matches!(kd, Kind::Fixed(kk, _) if *kk == k) {
              node.pair[pi] = o.st[pi];
            }
          }
        }
        let mut ok = true;
        for (pi, &(a, b)) in pairs.iter().enumerate() {
          let Kind::Both(k1, k2) = kinds[pi] else { continue };
          let (Some(p1), Some(p2)) = (node.now[k1], node.now[k2]) else {
            node.pair[pi] = State::default();
            continue;
          };
          let br = broke(a, b);
          let m1 = matches!(cells[s][freeix[k1]], Err(Mode::Strike));
          let m2 = matches!(cells[s][freeix[k2]], Err(Mode::Strike));
          let sym = corpus::pair_sym(
            (p1, m1, Move::of(if br { None } else { st.now[k1] }, p1)),
            (p2, m2, Move::of(if br { None } else { st.now[k2] }, p2)),
            downbeat,
          );
          let next = automaton::step_into(st.pair[pi], sym, &mut fired);
          for r in &fired {
            if pr.tier.contains(r) {
              ok = false;
              break;
            }
            if let Some(x) = SOFT.iter().position(|s| s == r) {
              cost += pr.weights[x];
            }
          }
          if !ok {
            break;
          }
          node.pair[pi] = next;
        }
        if !ok {
          continue;
        }
        work += 1;
        let to = cur.relax(node, prev.cost[i] + cost, i as u32, node.now, prev.count[i], prev.flags[i] + flags);
        if pr.samples > 0 {
          edge_buf.push((i as u32, to as u32));
        }
      }
    }

    peak = peak.max(cur.nodes.len());
    if work > MAX_WORK {
      return Err(format!(
        "state explosion at slice {s}: {work} edges relaxed, budget {MAX_WORK} — §2.7's wall, with {} free voices",
        freeix.len()
      ));
    }
    if cur.nodes.is_empty() {
      return Err(format!("no legal fill: dead at slice {s} of {} (tick {t})", times.len()));
    }
    if cur.nodes.len() > MAX_STATES {
      return Err(format!(
        "state explosion at slice {s}: {} live states, cap {MAX_STATES} — §2.7's wall, with {} free voices",
        cur.nodes.len(),
        freeix.len()
      ));
    }
    // A finished layer is only ever read through its backpointers, so its hash
    // index — the largest thing in it by far — is dead the moment the next
    // layer is built. Keeping it made a long search hold hundreds of megabytes
    // of dead map and spend its time in the allocator rather than the automaton.
    if pr.samples > 0 {
      edge_buf.sort_unstable_by_key(|&(_, to)| to);
      edges.push(std::mem::take(&mut edge_buf));
    }
    let mut done = std::mem::replace(&mut prev, cur);
    done.index = Map::default();
    done.nodes = vec![];
    layers.push(done);
  }
  layers.push(prev);

  // --- traceback -----------------------------------------------------------
  let last = layers.last().unwrap();
  let total: u128 = last.count.iter().fold(0u128, |a, &b| a.saturating_add(b));
  let mut j = (0..last.nodes.len())
    .min_by(|&a, &b| last.cost[a].partial_cmp(&last.cost[b]).unwrap())
    .ok_or("no final state")?;
  let cost = last.cost[j];
  let flags = last.flags[j] as usize;

  let mut chosen: Vec<[Option<Pitch>; MAXFREE]> = vec![[None; MAXFREE]; times.len()];
  for s in (0..times.len()).rev() {
    let l = &layers[s + 1];
    chosen[s] = l.took[j];
    j = l.back[j] as usize;
  }

  // --- assemble one chosen path into voices --------------------------------
  let assemble = |chosen: &[[Option<Pitch>; MAXFREE]]| -> Vec<Voice> {
    let mut out = pr.voices.clone();
    for (k, &v) in freeix.iter().enumerate() {
      let mut notes: Vec<Note> = vec![];
      for (s, &t) in times.iter().enumerate() {
        let Some(p) = chosen[s][k] else { continue };
        let end = times.get(s + 1).copied().unwrap_or_else(|| {
          pr.voices[v].notes.last().map(|n| n.onset + n.dur).unwrap_or(t + 1).max(t + 1)
        });
        let strike = matches!(cells[s][v], Err(Mode::Strike));
        match notes.last_mut() {
          Some(n) if !strike && n.onset + n.dur == t && n.pitch == p => n.dur += end - t,
          _ => notes.push(Note { onset: t, dur: end - t, pitch: p, attack: strike }),
        }
      }
      out[v] = Voice { notes };
    }
    out
  };

  // --- uniform samples from the legal set ----------------------------------
  //
  // Choose a final node in proportion to how many paths reach it, then walk
  // back choosing each predecessor in proportion to *its* count. The factors
  // telescope — `count[j]` is by construction the sum over j's predecessors —
  // so every complete path comes out with probability exactly `1/total`. That
  // is §7.1's Weak C2 with the half that needs a corpus removed: typical rather
  // than optimal, and asserting nothing.
  //
  // Weights go through `f64`, which is exact enough while the counts fit; a
  // saturated total would silently distort them, so `Solution::saturated` says
  // when that has happened.
  let mut sampled: Vec<Vec<Voice>> = Vec::new();
  if pr.samples > 0 && total > 0 && !last.count.is_empty() {
    let mut rng = pr.seed | 1;
    let mut unif = move || {
      rng = rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
      let mut z = rng;
      z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
      z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
      (((z ^ (z >> 31)) >> 11) as f64) / ((1u64 << 53) as f64)
    };
    let final_total: f64 = last.count.iter().map(|&c| c as f64).sum();
    for _ in 0..pr.samples {
      let mut r = unif() * final_total;
      let mut j = last.count.len() - 1;
      for (i, &c) in last.count.iter().enumerate() {
        r -= c as f64;
        if r <= 0.0 {
          j = i;
          break;
        }
      }
      let mut path = vec![[None; MAXFREE]; times.len()];
      for s in (0..times.len()).rev() {
        path[s] = layers[s + 1].took[j];
        if s == 0 {
          break;
        }
        let e = &edges[s];
        let lo = e.partition_point(|&(_, to)| (to as usize) < j);
        let hi = e.partition_point(|&(_, to)| (to as usize) <= j);
        if lo >= hi {
          return Err(format!("sampler: node {j} of layer {} has no predecessor", s + 1));
        }
        let w: f64 = e[lo..hi].iter().map(|&(f, _)| layers[s].count[f as usize] as f64).sum();
        let mut r = unif() * w;
        let mut pick = e[lo].0 as usize;
        for &(f, _) in &e[lo..hi] {
          pick = f as usize;
          r -= layers[s].count[f as usize] as f64;
          if r <= 0.0 {
            break;
          }
        }
        j = pick;
      }
      sampled.push(assemble(&path));
    }
  }

  Ok(Solution {
    voices: assemble(&chosen),
    cost,
    legal_fills: total,
    saturated: total == u128::MAX,
    peak_states: peak,
    slices: times.len(),
    melodic_flags: flags,
    sampled,
  })
}

/// Everything about one free voice's candidate note that does not involve
/// another free voice: its melodic interval, and the harmonic automaton.
///
/// Returns the new harmonic debt and the count of permitted-but-flagged melodic
/// intervals, or `None` if the note is refused.
fn voice_ok(
  prev: Option<Pitch>,
  debt: u8,
  now: Option<Pitch>,
  mode: Mode,
  chord: Option<Chord>,
  hard_melodic: bool,
) -> Option<(u8, usize)> {
  let mut flags = 0usize;
  if let (Some(a), Some(b)) = (prev, now) {
    if mode == Mode::Strike && a != b && Interval::between(a, b).is_forbidden_melodic() {
      if hard_melodic {
        return None;
      }
      flags += 1;
    }
  }

  let mut harm = H_FREE;
  if debt != H_FREE {
    match mode {
      // Leaving a dissonance by falling silent discharges nothing.
      Mode::Rest => return None,
      Mode::Hold => harm = debt, // still owed; the voice has not moved
      Mode::Strike => {
        let (a, b) = (prev?, now?);
        let mv = Move::of(Some(a), b);
        if !(if debt == H_DOWN { mv == Move::StepDown } else { mv.is_step() }) {
          return None;
        }
      }
    }
  }
  if harm != H_FREE {
    return Some((harm, flags)); // carried, not re-incurred
  }
  let (Some(c), Some(p)) = (chord, now) else { return Some((H_FREE, flags)) };
  if c.contains(p) {
    return Some((H_FREE, flags));
  }
  // a note foreign to the prevailing chord: prepared, or approached by step
  Some((
    match mode {
      Mode::Rest => H_FREE,
      // tied across a change of chord — a suspension, and it must fall
      Mode::Hold => H_DOWN,
      Mode::Strike => {
        if !Move::of(Some(prev?), p).is_step() {
          return None;
        }
        H_STEP
      }
    },
    flags,
  ))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::automaton::CONFIRMED;
  use crate::kern::TICKS_PER_QUARTER as Q;

  /// A voice of quarter notes from kern-style step numbers.
  fn line(steps: &[i16]) -> Voice {
    Voice {
      notes: steps
        .iter()
        .enumerate()
        .map(|(i, &s)| Note { onset: i as i64 * Q, dur: Q, pitch: Pitch::new(s, 0), attack: true })
        .collect(),
    }
  }

  fn problem<'a>(fixed: Voice, rhythm: Voice, tier: &'a [Rule]) -> Problem<'a> {
    Problem {
      voices: vec![fixed, rhythm],
      free: vec![false, true],
      compass: vec![(0, 0), (35, 39)], // C5..G5 for the free voice
      key: [0; 7],
      measure: 4 * Q,
      plan: vec![],
      tier,
      weights: [1.0; 6],
      samples: 0,
      seed: 0x5EED,
    }
  }

  /// The generator must not emit counterpoint its own checker then flags. Both
  /// go through `corpus::pair_sym` for exactly this reason, and this is the test
  /// that says so.
  #[test]
  fn what_the_search_accepts_the_checker_accepts() {
    let cf = line(&[28, 30, 29, 31, 28]); // C4 E4 D4 F4 C4
    let sol = fill(&problem(cf.clone(), line(&[35, 35, 35, 35, 35]), CONFIRMED)).expect("a fill exists");
    let t = corpus::check_voices(&sol.voices[0], &sol.voices[1], 4 * Q);
    for r in CONFIRMED {
      assert_eq!(t.by_rule.get(r.name()).copied().unwrap_or(0), 0, "generator emitted {}", r.name());
    }
  }

  /// The count of legal fills is checked against enumerating them all. If the
  /// two ever disagreed, §8.6's selectivity figure — the whole point of the
  /// exercise — would be a number with no referent.
  #[test]
  fn the_fill_count_matches_brute_force() {
    let cf = line(&[28, 30, 29]);
    let pr = problem(cf.clone(), line(&[35, 35, 35]), CONFIRMED);
    let sol = fill(&pr).expect("a fill exists");

    let lo = 35i16;
    let n = 5i16; // C5..G5, as in `problem`
    let mut brute = 0u128;
    for a in 0..n {
      for b in 0..n {
        for c in 0..n {
          let free = line(&[lo + a, lo + b, lo + c]);
          let t = corpus::check_voices(&cf, &free, 4 * Q);
          if CONFIRMED.iter().all(|r| t.by_rule.get(r.name()).copied().unwrap_or(0) == 0) {
            brute += 1;
          }
        }
      }
    }
    assert_eq!(sol.legal_fills, brute, "dynamic programme counted {} , enumeration {brute}", sol.legal_fills);
    assert!(brute > 1, "the instance is too constrained to be a test");
  }

  /// A uniform draw is still a legal fill. The sampler walks a different path
  /// through the same DAG, so if it could reach anything the hard tier refuses,
  /// the count it draws in proportion to would be counting illegal fills too and
  /// §8.6's headline number would be wrong.
  #[test]
  fn every_sampled_fill_is_legal() {
    let cf = line(&[28, 30, 29, 31, 28]);
    let mut pr = problem(cf, line(&[35, 35, 35, 35, 35]), CONFIRMED);
    pr.samples = 24;
    let sol = fill(&pr).expect("a fill exists");
    assert_eq!(sol.sampled.len(), 24);
    for (i, s) in sol.sampled.iter().enumerate() {
      let t = corpus::check_voices(&s[0], &s[1], 4 * Q);
      for r in CONFIRMED {
        assert_eq!(t.by_rule.get(r.name()).copied().unwrap_or(0), 0, "sample {i} breaks {}", r.name());
      }
    }
  }

  /// And it draws more than one of them. A sampler that always returned the same
  /// path would pass the test above and be worth nothing.
  #[test]
  fn sampling_explores_the_legal_set() {
    let cf = line(&[28, 30, 29]);
    let mut pr = problem(cf, line(&[35, 35, 35]), CONFIRMED);
    pr.samples = 60;
    let sol = fill(&pr).expect("a fill exists");
    let seen: std::collections::BTreeSet<Vec<i16>> =
      sol.sampled.iter().map(|s| s[1].notes.iter().map(|n| n.pitch.step).collect()).collect();
    assert!(seen.len() > 1, "sampler returned one distinct fill out of {}", sol.legal_fills);
    assert!(
      seen.len() as u128 <= sol.legal_fills,
      "drew {} distinct fills from a legal set of {}",
      seen.len(),
      sol.legal_fills
    );
  }

  /// The draw is **uniform**, which is the entire claim and the one thing the
  /// two tests above do not check: a sampler that always returned the cheapest
  /// path would pass "is legal", and one biased towards a corner would pass
  /// "explores". So enumerate a small instance and look at the histogram.
  ///
  /// 116 legal fills, 20 000 draws, seeded and therefore deterministic. For a
  /// uniform distribution chi-squared is expected to come out near its degrees
  /// of freedom, and it does — 113 against 115 — with every one of the 116 fills
  /// drawn at least once. The bound below is loose on purpose: it is there to
  /// catch a sampler that has become lopsided, not to re-measure the fit.
  #[test]
  fn the_draw_is_uniform_over_the_legal_set() {
    let cf = line(&[28, 30, 29]);
    let mut pr = problem(cf, line(&[35, 35, 35]), CONFIRMED);
    pr.samples = 20_000;
    let sol = fill(&pr).expect("a fill exists");

    let mut hist: std::collections::BTreeMap<Vec<i16>, usize> = Default::default();
    for s in &sol.sampled {
      *hist.entry(s[1].notes.iter().map(|n| n.pitch.step).collect()).or_default() += 1;
    }
    assert_eq!(
      hist.len() as u128,
      sol.legal_fills,
      "drew {} distinct fills of {} legal ones",
      hist.len(),
      sol.legal_fills
    );
    let exp = pr.samples as f64 / hist.len() as f64;
    let chi: f64 = hist.values().map(|&c| (c as f64 - exp).powi(2) / exp).sum();
    let df = (hist.len() - 1) as f64;
    assert!(chi < 2.0 * df, "chi-squared {chi:.1} against {df:.0} degrees of freedom: not uniform");
  }

  /// Rhythm is data: the fill articulates where it was told to and nowhere else.
  #[test]
  fn the_search_never_chooses_a_rhythm() {
    let cf = line(&[28, 30, 29, 31]);
    let rhythm = Voice {
      notes: vec![
        Note { onset: 0, dur: 2 * Q, pitch: Pitch::new(35, 0), attack: true },
        Note { onset: 2 * Q, dur: 2 * Q, pitch: Pitch::new(35, 0), attack: true },
      ],
    };
    let sol = fill(&problem(cf, rhythm, CONFIRMED)).expect("a fill exists");
    let onsets: Vec<i64> = sol.voices[1].notes.iter().filter(|n| n.attack).map(|n| n.onset).collect();
    assert_eq!(onsets, vec![0, 2 * Q]);
  }

  /// A note foreign to the plan's chord has to be prepared or approached by
  /// step, and has to resolve — the second automaton of §2.3, running.
  #[test]
  fn a_non_chord_tone_must_be_approached_and_left_by_step() {
    // C major throughout; the free voice may only leave the chord by step
    let plan = vec![Segment { start: 0, end: 8 * Q, chord: Some(Chord { root: 0, quality: 0 }), fit: 1.0 }];
    let cf = line(&[28, 28, 28, 28]);
    let mut pr = problem(cf, line(&[35, 35, 35, 35]), CONFIRMED);
    pr.plan = plan;
    let sol = fill(&pr).expect("a fill exists");
    let c = Chord { root: 0, quality: 0 };
    let notes = &sol.voices[1].notes;
    for (i, n) in notes.iter().enumerate() {
      if c.contains(n.pitch) {
        continue;
      }
      let prev = i.checked_sub(1).map(|j| notes[j].pitch);
      let next = notes.get(i + 1).map(|m| m.pitch);
      assert!(prev.map_or(false, |p| Move::of(Some(p), n.pitch).is_step()), "{} unprepared", n.pitch.name());
      assert!(next.map_or(true, |m| Move::of(Some(n.pitch), m).is_step()), "{} unresolved", n.pitch.name());
    }
  }
}
