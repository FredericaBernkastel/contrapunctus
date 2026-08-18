//! Step 5's drivers: the first notes this project has produced, and the
//! measurements that say what they are worth — readme §8.6.
//!
//! Three things happen here.
//!
//! **The stretto is rendered.** §8.3's clique is a set of five placements of one
//! subject; placing them and writing the file is the first audible output of the
//! object the whole document turns on. No search is involved — with five entries
//! in five voices there are no free voices at all, which is §2.5's cost profile
//! arriving exactly as predicted.
//!
//! **The free voices are reconstructed.** For every annotated subject entry in
//! the Well-Tempered Clavier, hold the entry, discard the other voices' pitches
//! while keeping their rhythm, and fill them by §8.6's search. Bach's own notes
//! are then the answer key. The chance baseline is computed *before* the
//! comparison and per note, because §8.2 and §8.4 both turned on a measurement
//! that looked impressive until its baseline was worked out.
//!
//! **The scalarisation is shown to matter.** The same problem is solved several
//! times under different weightings of the soft criteria; the number of notes
//! that change is §5's claim about taste, stated as a number instead of an
//! opinion.
//!
//! One methodological point governs the second of these. §8.5 says the analyser
//! is the instrument that *judges* a realiser rather than the model it writes
//! against, and deriving the harmonic plan from the very notes being
//! reconstructed would be exactly that mistake. So the plan is taken from the
//! **fixed voices alone**, and the leaky version is run beside it to show what
//! the leak is worth.

use crate::{
  automaton::{self, Move, Rule, CONFIRMED, HARD, SOFT},
  corpus, harmony, kern,
  kern::{Note, Piece, Voice, TICKS_PER_QUARTER},
  pitch::{Interval, Pitch},
  realise::{self, Problem},
  refdata, stretto,
};

const KERN: &str = "corpus/bach-wtc-fugues/kern";
const OUT: &str = "out";
/// The middle of §8.5's plausible band: a chord change about every 1.4 quarters.
const LAMBDA: f64 = 1.0;
/// Uniform draws per span for the §9 step 6 row. Eight averages a per-span
/// figure without making the edge recording the dominant cost.
const SAMPLES: usize = 8;

/// Clip a voice to `[t0, t1)`, keeping notes that sound across the boundary
/// rather than dropping them. A note that began earlier is *held* into the span,
/// so it is not an attack there.
fn clip(v: &Voice, t0: i64, t1: i64) -> Voice {
  Voice {
    notes: v
      .notes
      .iter()
      .filter(|n| n.onset < t1 && n.onset + n.dur > t0)
      .map(|n| {
        let a = n.onset.max(t0);
        Note { onset: a - t0, dur: (n.onset + n.dur).min(t1) - a, pitch: n.pitch, attack: n.attack && n.onset >= t0 }
      })
      .collect(),
  }
}

/// A voice's compass over the whole piece, rounded outwards to whole octaves.
///
/// Taken from the piece rather than from the passage on purpose: how high a
/// given part goes is a fact about the fugue's layout, which a form grammar
/// would supply, whereas the passage's own range is a fact about the notes being
/// reconstructed and using it would be circular.
fn compass(v: &Voice) -> (i16, i16) {
  let lo = v.notes.iter().map(|n| n.pitch.step).min().unwrap_or(21);
  let hi = v.notes.iter().map(|n| n.pitch.step).max().unwrap_or(35);
  (lo, hi)
}

fn out_dir() -> std::path::PathBuf {
  let d = std::path::PathBuf::from(OUT);
  let _ = std::fs::create_dir_all(&d);
  d
}

/// The score's own time signature, recovered from the tick counts the reader
/// took off the `*M` interpretation: a bar of `measure` ticks divided into notes
/// of `beat` ticks each, with the whole note as the unit.
fn meter(p: &Piece) -> (u8, u8) {
  if p.beat <= 0 {
    return (4, 4);
  }
  (((p.measure / p.beat).max(1)) as u8, ((crate::kern::TICKS_PER_WHOLE / p.beat).max(1)) as u8)
}

/// Write a texture as MIDI **in score order**, top voice first, with each track
/// named by where it sits and what it is.
///
/// Both halves of this matter, and both were wrong first time. The tracks were
/// emitted in `**kern` spine order — lowest voice first — and named `voice 0`,
/// `voice 1`, `voice 2` after that index. Two consequences, both reported from a
/// DAW rather than found here. The top voice of three arrives as *voice 2*,
/// which reads as reversed to anyone expecting the ground truth's own top-down
/// `S A T B C`; and `stretto.mid` numbered its entries downward from the top
/// while `stretto-bach.mid` numbered its voices upward from the bottom, so
/// `entry 1` and `voice 4` were the same line and nothing said so.
///
/// The order is taken from the **mean sounding pitch**, not from the spine
/// index, so the label is a description of what will be heard rather than a
/// restatement of an assumption about the file. The range in each name makes the
/// pairing checkable by eye between two files without playing either.
fn write_score(
  path: &std::path::Path,
  voices: &[Voice],
  roles: &[String],
  qpm: u32,
  sig: (u8, u8),
) -> std::io::Result<()> {
  let mean = |v: &Voice| -> f64 {
    if v.notes.is_empty() {
      return f64::MIN;
    }
    v.notes.iter().map(|n| n.pitch.chroma() as f64).sum::<f64>() / v.notes.len() as f64
  };
  let mut order: Vec<usize> = (0..voices.len()).collect();
  order.sort_by(|&a, &b| mean(&voices[b]).partial_cmp(&mean(&voices[a])).unwrap());

  let n = order.len();
  let (mut out, mut names) = (Vec::with_capacity(n), Vec::with_capacity(n));
  for (pos, &v) in order.iter().enumerate() {
    let where_ = match pos {
      0 => "top",
      p if p + 1 == n => "bass",
      _ => "inner",
    };
    let lo = voices[v].notes.iter().map(|x| x.pitch).min_by_key(|p| p.chroma());
    let hi = voices[v].notes.iter().map(|x| x.pitch).max_by_key(|p| p.chroma());
    let span = match (lo, hi) {
      (Some(a), Some(b)) => format!("{}..{}", a.name(), b.name()),
      _ => "silent".into(),
    };
    names.push(format!("{} {where_} {span} {}", pos + 1, roles.get(v).map(|s| s.as_str()).unwrap_or("")));
    out.push(voices[v].clone());
  }
  crate::midi::write(path, &out, &names, qpm, sig)
}

// --------------------------------------------------------------- the demo ---

/// BWV 867's five entries, placed as §8.3's clique, written as sound.
pub fn render_stretto() {
  println!("\n== step 5a: the stretto, rendered ==");
  let Ok(p) = kern::read(&std::path::Path::new(KERN).join("wtc1f22.krn")) else {
    return println!("  (corpus missing)");
  };
  let q = TICKS_PER_QUARTER;
  let entries_q: [i64; 5] = [266, 268, 270, 272, 274];
  let sub = stretto::Subject::cut(&p.voices[p.voices.len() - 1], 0, 12 * q);

  // the transpositions Bach uses, recovered from the score as in §8.3
  let mut voices: Vec<Voice> = vec![];
  let mut names: Vec<String> = vec![];
  for (i, &sq) in entries_q.iter().enumerate() {
    let t = sq * q;
    let v = p.voices.len() - 1 - i;
    let Some(n) = p.voices[v].notes.iter().find(|n| n.onset >= t) else { continue };
    let (ds, dc) = stretto::interval_from(&sub, n.pitch);
    voices.push(sub.place((sq - entries_q[0]) * q, ds, dc));
    names.push(format!("- entry {} of 5, enters +{}q", i + 1, sq - entries_q[0]));
  }

  let hard = pairs_violating(&voices, p.measure, HARD);
  let conf = pairs_violating(&voices, p.measure, CONFIRMED);
  println!("  {} entries placed, {} notes", voices.len(), voices.iter().map(|v| v.notes.len()).sum::<usize>());
  println!("  violations: {hard} on the full tier, {conf} on the confirmed tier");

  let d = out_dir();
  match write_score(&d.join("stretto.mid"), &voices, &names, 66, meter(&p)) {
    Ok(()) => println!("  wrote {}", d.join("stretto.mid").display()),
    Err(e) => println!("  {e}"),
  }
  // Bach's own bars, for the comparison to be listenable rather than asserted.
  // The entry each voice carries is named here so that track `k` of one file and
  // track `k` of the other are the same line — which is the whole point of
  // writing both.
  let (t0, t1) = (entries_q[0] * q, (entries_q[4] + 12) * q);
  let bach: Vec<Voice> = p.voices.iter().map(|v| clip(v, t0, t1)).collect();
  let bn: Vec<String> = (0..bach.len())
    .map(|v| {
      // voice index runs low to high, entries are numbered from the top
      let e = bach.len() - v;
      format!("- carries entry {e} of 5, at +{}q, among everything else", (entries_q[e - 1] - entries_q[0]))
    })
    .collect();
  if write_score(&d.join("stretto-bach.mid"), &bach, &bn, 66, meter(&p)).is_ok() {
    println!("  wrote {}  (the same bars as Bach wrote them)", d.join("stretto-bach.mid").display());
  }
}

fn pairs_violating(voices: &[Voice], measure: i64, tier: &[Rule]) -> usize {
  let mut n = 0;
  for a in 0..voices.len() {
    for b in (a + 1)..voices.len() {
      n += stretto::compatible(&voices[a], &voices[b], measure, tier).hard;
    }
  }
  n
}

// ------------------------------------------------------- the reconstruction --

/// One annotated subject entry, and the problem it poses: hold this voice, fill
/// the others.
struct Span {
  piece: usize,
  id: String,
  start: i64,
  len: i64,
  free: Vec<usize>,
  /// Every voice clipped to the span, with the notes to be reconstructed still
  /// in it — they are the answer key, and the search never sees them.
  clipped: Vec<Voice>,
  freeflag: Vec<bool>,
  /// The analysis of the *whole* texture, for the chance baseline only.
  segs: Vec<harmony::Segment>,
}

fn spans() -> (Vec<Piece>, Vec<Span>) {
  let dir = std::path::Path::new(KERN);
  let mut pieces = std::collections::BTreeMap::new();
  for n in 1..=24 {
    if let Ok(p) = kern::read(&dir.join(format!("wtc1f{n:02}.krn"))) {
      pieces.insert(format!("wtc-i-{n:02}"), p);
    }
  }
  let specs = refdata::read(
    std::path::Path::new("corpus/algomus-data/fugues/fugues.ref"),
    &|id| pieces.get(id).map(|p| p.measure),
  )
  .unwrap_or_default();
  let ids: Vec<String> = pieces.keys().cloned().collect();
  let list: Vec<Piece> = pieces.into_values().collect();

  let mut out = vec![];
  for spec in &specs {
    let Some(pi) = ids.iter().position(|k| *k == spec.id) else { continue };
    let p = &list[pi];
    if spec.len == 0 {
      continue;
    }
    for &(letter, start) in &spec.entries {
      if start < 0 {
        continue;
      }
      let entry = voice_of(p, letter);
      let end = start + spec.len;
      // every other voice that sounds for at least half the span
      let free: Vec<usize> = (0..p.voices.len())
        .filter(|&v| v != entry)
        .filter(|&v| {
          let held: i64 = p.voices[v]
            .notes
            .iter()
            .filter(|n| n.onset < end && n.onset + n.dur > start)
            .map(|n| (n.onset + n.dur).min(end) - n.onset.max(start))
            .sum();
          held * 2 >= spec.len
        })
        .collect();
      if free.is_empty() {
        continue;
      }
      // the entry itself has to be there
      if !p.voices[entry].notes.iter().any(|n| n.onset == start) {
        continue;
      }
      let clipped: Vec<Voice> = p.voices.iter().map(|v| clip(v, start, end)).collect();
      let mut freeflag = vec![false; p.voices.len()];
      for &v in &free {
        freeflag[v] = true;
      }
      let segs = harmony::analyse_viterbi(&clipped, p.beat, LAMBDA);
      out.push(Span { piece: pi, id: spec.id.clone(), start, len: spec.len, free, clipped, freeflag, segs });
    }
  }
  (list, out)
}

/// Ground truth names voices top-down (S A T B C); kern spines run low to high.
fn voice_of(p: &Piece, letter: char) -> usize {
  let order = ['S', 'A', 'T', 'B', 'C'];
  let i = order.iter().position(|&c| c == letter).unwrap_or(0);
  p.voices.len().saturating_sub(1 + i).min(p.voices.len() - 1)
}

/// How many pitches were locally legal for voice `v` at tick `t`, holding every
/// other voice at Bach's notes and taking Bach's own preceding pitch.
///
/// This is the chance baseline, and it is the honest one: not "one pitch in a
/// two-octave compass" but "one of the pitches the rules and the plan actually
/// left open". It is computed from the same predicate the search uses, without
/// the outstanding obligations, which makes it a slight over-count and therefore
/// a slightly *generous* baseline rather than a flattering one.
fn local_choices(
  voices: &[Voice],
  v: usize,
  t: i64,
  prev: Option<Pitch>,
  key: &[i8; 7],
  measure: i64,
  tier: &[Rule],
  chord: Option<harmony::Chord>,
  comp: (i16, i16),
) -> usize {
  let mut n = 0;
  for step in comp.0..=comp.1 {
    let nat = Pitch::new(step, key[step.rem_euclid(7) as usize]);
    let mut cands = vec![nat];
    if let Some(c) = chord {
      if !c.contains(nat) {
        for d in [1i8, -1] {
          let alt = Pitch::new(step, nat.alter + d);
          if c.contains(alt) {
            cands.push(alt);
          }
        }
      }
    }
    for p in cands {
      if legal_here(voices, v, t, prev, p, measure, tier, chord) {
        n += 1;
      }
    }
  }
  n
}

#[allow(clippy::too_many_arguments)]
fn legal_here(
  voices: &[Voice],
  v: usize,
  t: i64,
  prev: Option<Pitch>,
  p: Pitch,
  measure: i64,
  tier: &[Rule],
  chord: Option<harmony::Chord>,
) -> bool {
  if tier.contains(&Rule::ForbiddenMelodic) {
    if let Some(a) = prev {
      if a != p && Interval::between(a, p).is_forbidden_melodic() {
        return false;
      }
    }
  }
  if let Some(c) = chord {
    if !c.contains(p) && !prev.map_or(false, |a| Move::of(Some(a), p).is_step()) {
      return false;
    }
  }
  for (u, vu) in voices.iter().enumerate() {
    if u == v {
      continue;
    }
    let Some((pu, su)) = kern::sounding(vu, t) else { continue };
    let pw = vu.notes.iter().rev().find(|n| n.onset < t).map(|n| n.pitch);
    let sym = corpus::pair_sym(
      (p, true, Move::of(prev, p)),
      (pu, su, Move::of(pw, pu)),
      measure > 0 && t % measure == 0,
    );
    let (fired, _) = automaton::step(automaton::State::default(), sym);
    if fired.iter().any(|r| tier.contains(r)) {
      return false;
    }
  }
  true
}

/// The two universal rules plus the melodic one.
///
/// §8.2 stratified the melodic prohibition as *repertoire-specific* — Bach
/// breaks it 37.6 times per thousand moves against the Renaissance's 1.0 — which
/// settles what it is worth as a **description** of Bach. It says nothing about
/// what it is worth as a **constraint on a generator**, and the two questions
/// have different answers: 96% of Bach's own melodic moves obey it, and without
/// it nothing whatever bounds a free voice's line, so the search is free to leap
/// two octaves between quavers. This tier is here because that turns out to
/// matter more than any other single decision in step 5.
const CONF_MEL: &[Rule] = &[Rule::ParallelPerfect, Rule::DirectPerfectOnDownbeat, Rule::ForbiddenMelodic];

#[derive(Default)]
struct Score {
  spans: usize,
  slices: usize,
  solved: usize,
  dead: usize,
  capped: usize,
  notes: usize,
  exact: usize,
  pc: usize,
  /// The same three, over the uniformly sampled fills rather than the cheapest.
  s_notes: usize,
  s_exact: usize,
  s_pc: usize,
  chance: f64,
  fills: Vec<f64>,
  choices: Vec<f64>,
  peak: usize,
  flags: usize,
}

/// Where the harmonic plan comes from.
#[derive(Clone, Copy, PartialEq)]
enum Plan {
  /// No plan at all — the control that shows what the counterpoint rules permit
  /// on their own, and therefore how much of the constraint §2.3 is carrying.
  None,
  /// From the fixed voices only. The honest condition: a form grammar would
  /// supply a plan, and §8.5 says the analyser judges a realiser rather than
  /// being the model it writes against.
  Clean,
  /// From the whole texture, the voices being reconstructed included. Cheating,
  /// and run precisely to price the cheat.
  Leaky,
}

/// Fill one span and score it against Bach.
fn one(p: &Piece, sp: &Span, tier: &[Rule], which: Plan, w: f64, samples: usize, sc: &mut Score) -> Option<Vec<Voice>> {
  let all = &sp.clipped;
  let plan = match which {
    Plan::None => vec![],
    Plan::Leaky => sp.segs.clone(),
    Plan::Clean => {
      let source: Vec<Voice> =
        all.iter().enumerate().filter(|(i, _)| !sp.freeflag[*i]).map(|(_, v)| v.clone()).collect();
      harmony::analyse_viterbi(&source, p.beat, LAMBDA)
    }
  };

  let pr = Problem {
    voices: all.clone(),
    free: sp.freeflag.clone(),
    compass: p.voices.iter().map(compass).collect(),
    key: p.key,
    measure: p.measure,
    plan: plan.clone(),
    tier,
    weights: [w; 6],
    samples,
    // Deterministic and per span, so a rerun draws the same fills: this is a
    // measurement, not a demo, and §10 says nothing here is unseeded.
    seed: 0x5EED ^ (sp.start as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
  };
  sc.spans += 1;
  let sol = match realise::fill(&pr) {
    Ok(s) => s,
    Err(e) => {
      if e.contains("explosion") {
        sc.capped += 1;
      } else {
        sc.dead += 1;
      }
      return None;
    }
  };
  sc.solved += 1;
  sc.peak = sc.peak.max(sol.peak_states);
  sc.flags += sol.melodic_flags;
  sc.slices += sol.slices;
  sc.fills.push(if sol.saturated { f64::INFINITY } else { (sol.legal_fills as f64).max(1.0).log10() });

  // --- score against Bach --------------------------------------------------
  //
  // The baseline is computed against **the same plan the search used**, and the
  // first version was not: it took the full texture's analysis in every row, so
  // it did not move with the condition it was supposed to be the control for,
  // and a row run with no plan at all was being scored against a baseline that
  // had one. The question a baseline answers here is "how often would picking at
  // random from what was open to the search have hit Bach", and *what was open
  // to the search* is not negotiable.
  // How often a candidate fill agrees with Bach, note for note. Used unchanged
  // for the cheapest fill and for every uniform sample, so the two figures are
  // never the product of two slightly different accountings.
  let agree = |cand: &[Voice]| -> (usize, usize, usize) {
    let (mut n_, mut ex, mut pc) = (0, 0, 0);
    for &v in &sp.free {
      for n in all[v].notes.iter().filter(|n| n.attack) {
        let Some((got, _)) = kern::sounding(&cand[v], n.onset) else { continue };
        n_ += 1;
        if got == n.pitch {
          ex += 1;
        }
        if got.chroma().rem_euclid(12) == n.pitch.chroma().rem_euclid(12) {
          pc += 1;
        }
      }
    }
    (n_, ex, pc)
  };

  let (n_, ex, pc) = agree(&sol.voices);
  sc.notes += n_;
  sc.exact += ex;
  sc.pc += pc;

  for s in &sol.sampled {
    let (n_, ex, pc) = agree(s);
    sc.s_notes += n_;
    sc.s_exact += ex;
    sc.s_pc += pc;
  }

  // the per-note baseline, which needs Bach's own preceding note and so cannot
  // be folded into `agree`
  for &v in &sp.free {
    let comp = compass(&p.voices[v]);
    let mut prev: Option<Pitch> = None;
    for n in &all[v].notes {
      if !n.attack {
        prev = Some(n.pitch);
        continue;
      }
      let ch = plan.iter().find(|s| s.start <= n.onset && n.onset < s.end).and_then(|s| s.chord);
      let k = local_choices(all, v, n.onset, prev, &p.key, p.measure, tier, ch, comp);
      sc.chance += if k == 0 { 0.0 } else { 1.0 / k as f64 };
      sc.choices.push(k as f64);
      prev = Some(n.pitch);
    }
  }
  Some(sol.voices)
}

fn mean(v: &[f64]) -> f64 {
  let f: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
  if f.is_empty() { 0.0 } else { f.iter().sum::<f64>() / f.len() as f64 }
}

fn median(v: &[f64]) -> f64 {
  let mut f: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
  if f.is_empty() {
    return 0.0;
  }
  f.sort_by(|a, b| a.partial_cmp(b).unwrap());
  f[f.len() / 2]
}

pub fn reconstruct() {
  println!("\n== step 5b: reconstructing Bach's free voices ==");
  let (pieces, sp) = spans();
  if sp.is_empty() {
    return println!("  (corpus or ground truth missing)");
  }
  let attempted = sp.iter().filter(|s| s.free.len() <= 2).count();
  println!("  {} annotated entry spans with at least one accompanying voice", sp.len());
  println!("  {attempted} have at most two free voices and are searched exactly; the rest are §2.7's wall");
  println!("  the answer key is Bach's own notes, which the search never sees\n");
  println!("   tier          plan    done  dead  cap   notes  exact    pc   chance  log10 fills  open/note   time");

  let mut best: Option<(Vec<Voice>, usize, String, i64)> = None;
  let (mut peak, mut flags, mut slices, mut solved) = (0usize, 0usize, 0usize, 0usize);
  for (tname, tier) in [("confirmed(2)", CONFIRMED), ("conf+melodic", CONF_MEL), ("full(5)", HARD)] {
    for (pname, which) in [("none", Plan::None), ("clean", Plan::Clean), ("leaky", Plan::Leaky)] {
      let t0 = std::time::Instant::now();
      let mut sc = Score::default();
      for s in &sp {
        if s.free.len() > 2 {
          continue;
        }
        let got = one(&pieces[s.piece], s, tier, which, 1.0, 0, &mut sc);
        if best.is_none() && which == Plan::Clean && tname == "conf+melodic" && s.free.len() == 2 {
          if let Some(f) = got {
            best = Some((f, s.piece, s.id.clone(), s.start));
          }
        }
      }
      let n = sc.notes.max(1) as f64;
      peak = peak.max(sc.peak);
      flags += sc.flags;
      slices += sc.slices;
      solved += sc.solved;
      println!(
        "   {tname:<12}  {pname:<6} {:>4} {:>5} {:>4} {:>6}  {:>4.1}% {:>4.1}%   {:>4.1}%   {:>8.1}    {:>7.1} {:>6.0}s",
        sc.solved,
        sc.dead,
        sc.capped,
        sc.notes,
        100.0 * sc.exact as f64 / n,
        100.0 * sc.pc as f64 / n,
        100.0 * sc.chance / n,
        median(&sc.fills),
        mean(&sc.choices),
        t0.elapsed().as_secs_f64(),
      );
    }
  }
  // --- the control that says what the objective is worth --------------------
  //
  // Minimising the soft criteria puts the search *below* the chance baseline, and
  // there are two readings of that: the objective contributes nothing, or it
  // contributes something and the something points away from Bach. Running the
  // identical search with the sign of the objective reversed separates them. If
  // maximising does no better, the criteria are noise; if it does better, they
  // are pointing the wrong way, and no choice of weights repairs that.
  for (label, w) in [("minimised", 1.0f64), ("maximised", -1.0f64)] {
    let mut sc = Score::default();
    for s in &sp {
      if s.free.len() > 2 {
        continue;
      }
      one(&pieces[s.piece], s, CONF_MEL, Plan::Clean, w, 0, &mut sc);
    }
    let n = sc.notes.max(1) as f64;
    println!(
      "   {:<12}  {:<9} {:>4} {:>5} {:>4} {:>6}  {:>4.1}% {:>4.1}%   {:>4.1}%   {:>8.1}    {:>7.1}",
      "conf+melodic", label, sc.solved, sc.dead, sc.capped, sc.notes,
      100.0 * sc.exact as f64 / n,
      100.0 * sc.pc as f64 / n,
      100.0 * sc.chance / n,
      median(&sc.fills),
      mean(&sc.choices),
    );
  }

  // --- §9 step 6: sample the legal set instead of optimising over it --------
  //
  // The like-for-like control that the `chance` column is not. `chance` is a
  // per-note quantity computed with **Bach's own preceding note** in hand, so it
  // solves an easier problem than any generator does and the comparison flatters
  // it. A uniform draw from the legal set gets no such help: it commits to a
  // whole path, its errors compound exactly as the search's do, and it is scored
  // by the same function. Whatever separates it from the optimised rows is the
  // objective's doing and nothing else.
  {
    let t0 = std::time::Instant::now();
    let mut sc = Score::default();
    for s in &sp {
      if s.free.len() > 2 {
        continue;
      }
      one(&pieces[s.piece], s, CONF_MEL, Plan::Clean, 1.0, SAMPLES, &mut sc);
    }
    let n = sc.s_notes.max(1) as f64;
    println!(
      "   {:<12}  {:<9} {:>4} {:>5} {:>4} {:>6}  {:>4.1}% {:>4.1}%   {:>4.1}%   {:>8.1}    {:>7.1} {:>5.0}s",
      "conf+melodic", "uniform", sc.solved, sc.dead, sc.capped, sc.s_notes,
      100.0 * sc.s_exact as f64 / n,
      100.0 * sc.s_pc as f64 / n,
      100.0 * sc.chance / sc.notes.max(1) as f64,
      median(&sc.fills),
      mean(&sc.choices),
      t0.elapsed().as_secs_f64(),
    );
    println!("   ({SAMPLES} uniform draws per span, seeded per span; `notes` counts every sampled note)");
  }

  println!(
    "
  peak live states in any one layer: {peak}, against about 225 pitch pairs for two free voices — §2.7's"
  );
  println!("  estimate was the pitch product, and the obligation set is what actually multiplies.");
  println!(
    "  {} slices searched over {solved} fills; {flags} melodic intervals Fux forbids were taken where the tier",
    slices
  );
  println!("  permitted them, which is what the `conf+melodic` rows exist to remove.");
  println!("\n  `open/note` is how many pitches the rules and the plan actually left free at each note,");
  println!("  counted from the same predicate the search uses. It is the denominator of `chance`.");
  println!("  `cap` counts spans whose exact search passed {} live states and was refused rather", realise::MAX_STATES);
  println!("  than beamed — §2.7's wall, arriving at two free voices rather than four.");

  if let Some((fill, pi, id, start)) = best {
    let d = out_dir();
    let p = &pieces[pi];
    // Which line is Bach's and which the program's is the one thing a listener
    // must not have to guess, so the track says it. The same roles are used for
    // both files: in `fill-bach.mid` every voice is Bach's, and saying which
    // *would have been* filled is what makes the pair comparable track by track.
    let sp2 = sp.iter().find(|s| s.piece == pi && s.start == start).unwrap();
    let roles: Vec<String> = (0..fill.len())
      .map(|v| {
        if sp2.freeflag[v] { "- FILLED by the search".into() } else { "- Bach's subject entry, held fixed".into() }
      })
      .collect();
    let bach_roles: Vec<String> = (0..fill.len())
      .map(|v| {
        if sp2.freeflag[v] { "- Bach (this is the line the search replaces)".into() } else { "- Bach's subject entry".into() }
      })
      .collect();
    let bach: Vec<Voice> = p.voices.iter().map(|v| clip(v, start, start + fill_len(&fill))).collect();
    let _ = write_score(&d.join("fill.mid"), &fill, &roles, 76, meter(p));
    let _ = write_score(&d.join("fill-bach.mid"), &bach, &bach_roles, 76, meter(p));
    println!("\n  {id} at bar {}, two free voices, conf+melodic:", start / p.measure + 1);
    // The percentages above are worth nothing without one instance shown whole.
    for &v in &sp2.free {
      let got: Vec<String> =
        bach[v].notes.iter().filter(|n| n.attack).take(16).map(|n| {
          match kern::sounding(&fill[v], n.onset) {
            Some((q, _)) if q == n.pitch => format!("{:>4}", q.name()),
            Some((q, _)) => format!("{:>4}", q.name()),
            None => "   -".into(),
          }
        }).collect();
      let want: Vec<String> =
        bach[v].notes.iter().filter(|n| n.attack).take(16).map(|n| format!("{:>4}", n.pitch.name())).collect();
      println!("     Bach  {}", want.join(""));
      println!("     fill  {}", got.join(""));
    }
    println!("\n  wrote {} and fill-bach.mid", d.join("fill.mid").display());
  }
}

fn fill_len(v: &[Voice]) -> i64 {
  v.iter().flat_map(|x| x.notes.iter().map(|n| n.onset + n.dur)).max().unwrap_or(0)
}

// ------------------------------------------------------ the scalarisation ---

/// §5 says no weighting of the soft criteria is defensible. Here is what that
/// costs: solve one span several times under different weights and count how
/// many notes change.
pub fn scalarisations() {
  println!("\n== step 5c: the weighting is a choice, and it changes the notes ==");
  let (pieces, sp) = spans();
  let Some(s) = sp.iter().find(|s| s.free.len() == 1) else {
    return println!("  (no single-free-voice span)");
  };
  let p = &pieces[s.piece];
  let (t0, t1) = (s.start, s.start + s.len);
  let all = s.clipped.clone();
  let free = s.freeflag.clone();
  let source: Vec<Voice> =
    all.iter().enumerate().filter(|(i, _)| !free[*i]).map(|(_, v)| v.clone()).collect();
  let plan = harmony::analyse_viterbi(&source, p.beat, LAMBDA);
  println!("  {} bars {}-{}, one free voice\n", s.id, t0 / p.measure + 1, t1 / p.measure + 1);

  let mut runs: Vec<(String, Vec<Pitch>, Vec<f64>)> = vec![];
  let mut named: Vec<(String, [f64; 6])> = vec![("uniform".into(), [1.0; 6])];
  for (i, r) in SOFT.iter().enumerate() {
    let mut w = [0.0; 6];
    w[i] = 1.0;
    named.push((r.name().into(), w));
  }
  println!("   {:<22} {:>5}   {}", "objective", "cost", "soft-criterion counts, and the line it chooses");

  for (name, weights) in &named {
    let pr = Problem {
      voices: all.clone(),
      free: free.clone(),
      compass: p.voices.iter().map(compass).collect(),
      key: p.key,
      measure: p.measure,
      plan: plan.clone(),
      tier: CONFIRMED,
      weights: *weights,
      samples: 0,
      seed: 0,
    };
    let Ok(sol) = realise::fill(&pr) else { continue };
    let notes: Vec<Pitch> = sol.voices[s.free[0]].notes.iter().filter(|n| n.attack).map(|n| n.pitch).collect();
    // the achieved soft vector, read back off the *checker* rather than off the
    // search's own accounting
    let mut vec6 = vec![0.0f64; SOFT.len()];
    for u in 0..sol.voices.len() {
      if u == s.free[0] {
        continue;
      }
      // **In index order.** `crossed` is "the higher-indexed voice sounds below
      // the lower", so swapping the arguments reports the exact negation of
      // voice crossing — which is how this line first showed a fill that scored
      // zero on crossing while apparently crossing at every slice.
      let (lo, hi) = if u < s.free[0] { (u, s.free[0]) } else { (s.free[0], u) };
      let t = corpus::check_voices(&sol.voices[lo], &sol.voices[hi], p.measure);
      for (i, r) in SOFT.iter().enumerate() {
        vec6[i] += t.by_rule.get(r.name()).copied().unwrap_or(0) as f64;
      }
    }
    println!(
      "   {:<22} {:>5.1}   [{}]  {}",
      name,
      sol.cost,
      vec6.iter().map(|x| format!("{x:>3.0}")).collect::<Vec<_>>().join(" "),
      notes.iter().take(10).map(|p| p.name()).collect::<Vec<_>>().join(" ")
    );
    runs.push((name.clone(), notes, vec6));
  }

  if runs.len() > 1 {
    let base = &runs[0].1;
    let mut worst = 0.0f64;
    for (_, r, _) in &runs[1..] {
      let n = base.len().min(r.len());
      if n == 0 {
        continue;
      }
      let d = (0..n).filter(|&i| base[i] != r[i]).count() as f64 / n as f64;
      worst = worst.max(d);
    }
    // how many of these are mutually non-dominated — §5's Pareto front, over the
    // solutions the scalarisations happen to reach
    let front: Vec<&(String, Vec<Pitch>, Vec<f64>)> = runs
      .iter()
      .filter(|(_, _, x)| {
        !runs.iter().any(|(_, _, y)| {
          y != x && y.iter().zip(x).all(|(a, b)| a <= b) && y.iter().zip(x).any(|(a, b)| a < b)
        })
      })
      .collect();
    println!("\n  A single objective disagrees with the uniform one on up to {:.0}% of notes.", 100.0 * worst);
    println!("  {} of these {} fills are mutually non-dominated: {}", front.len(), runs.len(),
      front.iter().map(|(n, _, _)| n.as_str()).collect::<Vec<_>>().join(", "));
    println!("  Every one of them is legal. Nothing in the rulebook chooses between them, which is");
    println!("  §5's position arriving as an obstacle rather than as a preference.");
  }
}

pub fn run() {
  render_stretto();
  reconstruct();
  scalarisations();
}
