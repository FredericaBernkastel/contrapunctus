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
  answer,
  cli::{self, CONF_MEL},
  plan, refdata, shape, species, stretto,
};

/// The Bach `**kern` directory and the MIDI output directory, both settable —
/// §10.3's table is [`crate::cli::Params`] now, and these read it.
fn kern_dir() -> &'static std::path::Path {
  cli::params().kern.as_path()
}

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
  let d = cli::params().out.clone();
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
  let Ok(p) = kern::read(&kern_dir().join("wtc1f22.krn")) else {
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
  let dir = kern_dir();
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
      let segs = harmony::analyse_viterbi(&clipped, p.beat, cli::params().lambda);
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
  /// The same three, over the sampled fills rather than the cheapest.
  s_notes: usize,
  s_exact: usize,
  s_pc: usize,
  /// `(exact, notes)` per span over the sampled fills. The eight draws from one
  /// span share its fixed voices, its plan and its rhythm, so they are not eight
  /// independent observations; the span is the unit that replicates, and a
  /// standard error computed per note would be far too small.
  spanwise: Vec<(usize, usize)>,
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
#[allow(clippy::too_many_arguments)]
fn one(
  p: &Piece,
  sp: &Span,
  tier: &[Rule],
  which: Plan,
  w: f64,
  samples: usize,
  beta: f64,
  sc: &mut Score,
) -> Option<Vec<Voice>> {
  let all = &sp.clipped;
  let plan = match which {
    Plan::None => vec![],
    Plan::Leaky => sp.segs.clone(),
    Plan::Clean => {
      let source: Vec<Voice> =
        all.iter().enumerate().filter(|(i, _)| !sp.freeflag[*i]).map(|(_, v)| v.clone()).collect();
      harmony::analyse_viterbi(&source, p.beat, cli::params().lambda)
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
    prescribe: [0.0; 3],
    samples,
    // Deterministic and per span, so a rerun draws the same fills: this is a
    // measurement, not a demo, and §10 says nothing here is unseeded.
    seed: cli::params().seed ^ (sp.start as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
    beta,
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

  let (mut sn, mut se) = (0, 0);
  for s in &sol.sampled {
    let (n_, ex, pc) = agree(s);
    sc.s_notes += n_;
    sc.s_exact += ex;
    sc.s_pc += pc;
    sn += n_;
    se += ex;
  }
  if sn > 0 {
    sc.spanwise.push((se, sn));
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
        let got = one(&pieces[s.piece], s, tier, which, 1.0, 0, 0.0, &mut sc);
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
      one(&pieces[s.piece], s, CONF_MEL, Plan::Clean, w, 0, 0.0, &mut sc);
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
      one(&pieces[s.piece], s, CONF_MEL, Plan::Clean, 1.0, cli::params().samples, 0.0, &mut sc);
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
    println!("   ({} uniform draws per span, seeded per span; `notes` counts every sampled note)", cli::params().samples);
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
  let plan = harmony::analyse_viterbi(&source, p.beat, cli::params().lambda);
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
      prescribe: [0.0; 3],
      samples: 0,
      seed: 0,
      beta: 0.0,
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

// ------------------------------------------------ step 6: the generality test ---

/// Span length for the generality test: eight quarters, the same size of problem
/// as §8.6's reconstruction, so the two are commensurable.
const GEN_SPAN: i64 = 8 * TICKS_PER_QUARTER;
/// Inverse temperatures swept. `0` is the uniform draw and large values approach
/// the cheapest fill, so the curve interpolates between two figures §8.6 already
/// reports and the question is whether anything in between beats both.
const GEN_BETA: [f64; 6] = [0.0, 0.25, 0.5, 1.0, 2.0, 4.0];

/// Spans taken from a score **without annotations**, so that one protocol runs on
/// both corpora.
///
/// The Renaissance corpus has no subject annotations and never will, so §8.6's
/// entry-driven spans cannot be used for a comparison across repertoires. This
/// takes their place: hold the **top voice**, free up to two others that sound
/// throughout, and window at a fixed tick length. Applied to Bach it is a
/// slightly harder problem than §8.6's — the held voice is whatever is on top
/// rather than a subject entry — which is the price of measuring the same thing
/// in both centuries.
fn windows(pieces: &[Piece], want: usize) -> Vec<Span> {
  let mut out = vec![];
  for (pi, p) in pieces.iter().enumerate() {
    if p.voices.len() < 2 {
      continue;
    }
    let end = p.voices.iter().flat_map(|v| v.notes.iter().map(|n| n.onset + n.dur)).max().unwrap_or(0);
    if end < 4 * GEN_SPAN {
      continue;
    }
    // spread the windows through the piece rather than taking the opening, which
    // in a fugue is one voice alone and in a mass is often homophonic
    let stride = (end - GEN_SPAN) / want.max(1) as i64;
    let mut taken = 0;
    for k in 0..want {
      let start = GEN_SPAN + stride * k as i64;
      if start + GEN_SPAN > end {
        break;
      }
      let clipped: Vec<Voice> = p.voices.iter().map(|v| clip(v, start, start + GEN_SPAN)).collect();
      // a voice counts as present only if it sounds for most of the window
      let held: Vec<i64> =
        clipped.iter().map(|v| v.notes.iter().map(|n| n.dur).sum::<i64>()).collect();
      let present: Vec<usize> = (0..clipped.len()).filter(|&v| held[v] * 2 >= GEN_SPAN).collect();
      if present.len() < 2 {
        continue;
      }
      // the top voice by mean pitch is the one held; it is the part a listener
      // tracks, and holding it is the nearest annotation-free analogue of §8.6
      let mean = |v: usize| -> f64 {
        let n = &clipped[v].notes;
        if n.is_empty() { f64::MIN } else { n.iter().map(|x| x.pitch.chroma() as f64).sum::<f64>() / n.len() as f64 }
      };
      let top = *present.iter().max_by(|&&a, &&b| mean(a).partial_cmp(&mean(b)).unwrap()).unwrap();
      let free: Vec<usize> = present.iter().copied().filter(|&v| v != top).take(2).collect();
      if free.is_empty() {
        continue;
      }
      let mut freeflag = vec![false; p.voices.len()];
      for &v in &free {
        freeflag[v] = true;
      }
      let segs = harmony::analyse_viterbi(&clipped, p.beat, cli::params().lambda);
      out.push(Span { piece: pi, id: p.id.clone(), start, len: GEN_SPAN, free, clipped, freeflag, segs });
      taken += 1;
    }
    let _ = taken;
  }
  out
}

fn renaissance(limit: usize) -> Vec<Piece> {
  let mut files: Vec<std::path::PathBuf> = vec![];
  for d in ["Jos", "Oke", "Obr", "Duf", "Bus", "Mar"] {
    if let Ok(rd) = std::fs::read_dir(std::path::Path::new("corpus/jrp-scores").join(d)) {
      files.extend(
        rd.filter_map(|e| e.ok().map(|e| e.path()))
          .filter(|p| p.extension().map(|x| x == "krn").unwrap_or(false)),
      );
    }
  }
  files.sort();
  files.truncate(limit);
  files.iter().filter_map(|f| kern::read(f).ok()).collect()
}

/// **Does a treatise-derived weighting generalise, or is it Bach's?**
///
/// §7.1 asked whether WaveFunctionCollapse's Weak C2 — sample in proportion to
/// how much the model likes a pattern — could be had without a corpus. Fux
/// supplies the *directions*: the six soft criteria are the things he says to
/// avoid. He supplies no *magnitudes*, which is exactly Komosinski's objection to
/// Schottstaedt's weights, so there is one number here rather than six, and it is
/// swept and reported as a curve on §8.5's precedent rather than chosen.
///
/// The instrument is §8.2's. That section stratified *rules* into universal and
/// repertoire-specific by measuring them against two corpora three centuries
/// apart; this measures a *weighting* the same way.
///
/// **The decision rule, fixed before the numbers and printed with them.** Let
/// `β*` be the best inverse temperature on each corpus.
///
/// - improves over `β = 0` on **both** corpora → the weighting encodes general
///   voice leading, and it stays;
/// - improves on **one** → it is repertoire-specific, and it is rolled back and
///   documented, exactly as §8.2 did with the melodic rule;
/// - improves on **neither** → it is useless and it is rolled back.
pub fn generality() {
  println!("
== step 6: does a treatise weighting generalise, or is it Bach's? ==");
  println!("  Fux names the six things to avoid and no magnitudes, so one temperature is swept.");
  println!("  beta = 0 is the uniform draw of §8.6; large beta approaches the cheapest fill.
");
  println!("  DECIDED BEFORE THE RUN: keep only if some single beta beats beta=0 on BOTH corpora");
  println!("  by more than twice the standard error of the paired per-span difference. Helping one");
  println!("  is repertoire-specific and gets rolled back, per §8.2.
");

  let bach: Vec<Piece> = (1..=24)
    .filter_map(|n| kern::read(&kern_dir().join(format!("wtc1f{n:02}.krn"))).ok())
    .collect();
  let ren = renaissance(cli::params().ren_works);
  if bach.is_empty() || ren.is_empty() {
    return println!("  (corpus missing)");
  }
  // §8.6 ran three windows per fugue; §8.8 onwards runs thirty. Keeping this
  // one at its own number is what makes the default reproduce the published row.
  let bs = windows(&bach, cli::params().gen_windows);
  let rs = windows(&ren, cli::params().ren_windows);
  println!(
    "  Bach {} spans from {} fugues; Renaissance {} spans from {} works",
    bs.len(),
    bach.len(),
    rs.len(),
    ren.len()
  );
  println!("  one protocol for both: hold the top voice, free up to two others, 8 quarters");
  println!("  the span is the unit of replication, since eight draws share one span's context
");

  // per corpus, per beta: the spanwise (exact, notes) vector
  let mut runs: Vec<Vec<Vec<(usize, usize)>>> = vec![];
  for (pieces, spans) in [(&bach, &bs), (&ren, &rs)] {
    let mut per_beta = vec![];
    for &beta in GEN_BETA.iter() {
      let mut sc = Score::default();
      for s in spans.iter() {
        if s.free.len() > 2 {
          continue;
        }
        one(&pieces[s.piece], s, CONF_MEL, Plan::Clean, 1.0, cli::params().samples, beta, &mut sc);
      }
      per_beta.push(sc.spanwise);
    }
    runs.push(per_beta);
  }

  // paired per-span difference against beta = 0, which is the powerful test:
  // the same spans, the same seeds, the same note counts
  let paired = |a: &Vec<(usize, usize)>, b: &Vec<(usize, usize)>| -> (f64, f64) {
    let d: Vec<f64> = a
      .iter()
      .zip(b)
      .filter(|((_, n1), (_, n2))| *n1 > 0 && n1 == n2)
      .map(|((e1, n1), (e0, _))| (*e1 as f64 - *e0 as f64) / *n1 as f64)
      .collect();
    if d.len() < 2 {
      return (0.0, 0.0);
    }
    let m = d.iter().sum::<f64>() / d.len() as f64;
    let var = d.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (d.len() - 1) as f64;
    (100.0 * m, 100.0 * (var / d.len() as f64).sqrt())
  };
  let rate = |v: &Vec<(usize, usize)>| -> f64 {
    let (e, n): (usize, usize) = v.iter().fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
    if n == 0 { 0.0 } else { 100.0 * e as f64 / n as f64 }
  };

  println!("   beta  |     Bach exact   gain vs beta=0  |  Renaissance exact   gain vs beta=0");
  let mut shared: Vec<(f64, f64, f64, f64, f64)> = vec![];
  for (bi, &beta) in GEN_BETA.iter().enumerate() {
    let (bg, bse) = paired(&runs[0][bi], &runs[0][0]);
    let (rg, rse) = paired(&runs[1][bi], &runs[1][0]);
    println!(
      "   {:>4.2}  |         {:>5.1}%   {:>+5.2} +/- {:>4.2}  |          {:>5.1}%   {:>+5.2} +/- {:>4.2}",
      beta,
      rate(&runs[0][bi]),
      bg,
      bse,
      rate(&runs[1][bi]),
      rg,
      rse
    );
    if bi > 0 {
      shared.push((beta, bg, bse, rg, rse));
    }
  }

  println!("
  gains are paired per-span differences against the uniform draw, +/- one standard error.");
  let winners: Vec<&(f64, f64, f64, f64, f64)> =
    shared.iter().filter(|(_, bg, bse, rg, rse)| *bg > 2.0 * *bse && *rg > 2.0 * *rse).collect();
  match winners.first() {
    Some((beta, bg, _, rg, _)) => {
      println!(
        "
  VERDICT: GENERAL. beta = {beta:.2} beats the uniform draw in both centuries by more than
           two standard errors — Bach {bg:+.2} points, Renaissance {rg:+.2}. The direction Fux states
           carries across three hundred years; only the magnitude is local. Keep."
      );
      if winners.len() > 1 {
        println!("  ({} of the swept values clear the bar, so the result does not hinge on one.)", winners.len());
      }
    }
    None => {
      let b_any = shared.iter().any(|(_, g, se, _, _)| *g > 2.0 * *se);
      let r_any = shared.iter().any(|(_, _, _, g, se)| *g > 2.0 * *se);
      println!(
        "
  VERDICT: {} Roll back, per the rule stated above.",
        match (b_any, r_any) {
          (true, false) => "REPERTOIRE-SPECIFIC — clears the bar on Bach only.",
          (false, true) => "REPERTOIRE-SPECIFIC — clears the bar on the Renaissance only.",
          (true, true) => "NOT SIMULTANEOUS — each corpus has a beta that helps, but no single one helps both.",
          (false, false) => "NO EFFECT — nothing clears two standard errors on either corpus.",
        }
      );
    }
  }
}

// ------------------------------------------- step 6: the species whitelist ---

/// **Does Fux's own enumeration account for the music?**
///
/// §9 step 6's second proposal, and its gate. The whitelist is only a tighter
/// rulebook if real counterpoint stays inside it; if it does not, it is simply a
/// wrong one, and §8.2 is the instrument that settles which — the same
/// measurement on two corpora three centuries apart.
///
/// The comparison to keep in view is §8.2's own column. The two dissonance rules
/// this would replace flag **8.0 and 71.1** per thousand slices in the
/// Renaissance and **21.4 and 90.9** in Bach, which is why they were stratified
/// out of the hard tier. A whitelist worth having must do very much better than
/// that in both centuries.
pub fn species() {
  println!("\n== step 6: Fux's species as a whitelist, checked before it is used ==");
  println!("  Four figures, transcribed and nothing else: consonance, passing, neighbour, suspension.");
  println!("  The question is what fraction of the dissonances real music writes they account for.\n");

  let bach: Vec<Piece> = (1..=24)
    .filter_map(|n| kern::read(&kern_dir().join(format!("wtc1f{n:02}.krn"))).ok())
    .collect();
  let ren = renaissance(cli::params().ren_works);
  if bach.is_empty() || ren.is_empty() {
    return println!("  (corpus missing)");
  }

  println!("   corpus         reading         slices   dissonant   explained   UNLISTED /1k");
  let mut keep: Vec<(String, species::Tally)> = vec![];
  for (name, corpus) in [("Bach", &bach), ("Renaissance", &ren)] {
    for (label, metric, fourth) in
      [("strict", true, false), ("figures only", false, false), ("4th consonant", false, true)]
    {
      let mut t = species::Tally::default();
      for p in corpus.iter() {
        t.merge(&species::check_piece(p, metric, fourth));
      }
      println!(
        "   {name:<13}  {label:<14} {:>7} {:>11} {:>10.1}% {:>9.1}",
        t.slices,
        t.dissonant,
        100.0 * t.explained(),
        t.per_thousand()
      );
      if !metric && fourth {
        keep.push((name.to_string(), t.clone()));
      }
    }
  }

  println!("\n  `strict` enforces Fux's metric condition — suspensions on the beat, passing tones off it.");
  println!("  `figures only` drops it and asks about the figures alone.");
  println!("  The rules this replaces flag 8.0 and 71.1 per thousand in the Renaissance, 21.4 and 90.9 in Bach.\n");

  for (name, t) in &keep {
    let mut fig: Vec<(&&str, &usize)> = t.by_figure.iter().filter(|(k, _)| **k != "consonance").collect();
    fig.sort_by_key(|(_, v)| std::cmp::Reverse(**v));
    let parts: Vec<String> = fig.iter().map(|(k, v)| format!("{k} {:.0}%", 100.0 * **v as f64 / t.dissonant.max(1) as f64)).collect();
    println!("  {name:<12} dissonances: {}", parts.join(", "));
    let mut un: Vec<(&(i16, i16), &usize)> = t.unlisted.iter().collect();
    un.sort_by_key(|(_, v)| std::cmp::Reverse(**v));
    let top: Vec<String> = un
      .iter()
      .take(5)
      .map(|((st, se), v)| {
        let pct = 100.0 * **v as f64 / t.dissonant.max(1) as f64;
        format!("{}({st},{se}) {pct:.0}%", crate::name_interval(*st, *se))
      })
      .collect();
    if !top.is_empty() {
      println!("  {:<12} unlisted, commonest first: {}", "", top.join(", "));
    }
  }
}

// ------------------------------------ step 6: a criterion that is not local ---


/// **Does a criterion over a whole line do what the local tier cannot?**
///
/// §8.6 says what is missing and where to look: pitch class is recovered about
/// twice as often as pitch, so the octave is wrong, and register is a property
/// of a phrase that no one-slice criterion can see. §2.5 says the accumulators
/// that would express it are finite-state but would multiply the search state by
/// a few hundred. So the criterion is applied **after** the search instead:
/// draw whole legal fills uniformly and let a shape criterion pick among them.
///
/// The control is the same draws, unranked. Whatever separates the two is the
/// criterion's doing — same spans, same graph, same seeds, same scoring
/// function. And the bar is §8.2's, fixed before the run: keep a criterion only
/// if it beats the unranked draw on **both** corpora by more than twice the
/// standard error of the paired per-span difference.
pub fn shape_test() {
  println!("\n== step 6: a criterion that is not local ==");
  println!("  Every soft criterion looks at one slice or two. These look at the whole line:");
  println!("  one climax, a compass inside a tenth, and not standing on one note — all Fux's.");
  println!("  They rerank {} uniform draws per span rather than entering the search,", cli::params().rerank);
  println!("  because §2.5 says carrying a running range would multiply the state by a few hundred.\n");
  println!("  DECIDED BEFORE THE RUN: keep a criterion only if it beats the unranked draw on BOTH");
  println!("  corpora by more than twice the standard error of the paired per-span difference.\n");

  let bach: Vec<Piece> = (1..=24)
    .filter_map(|n| kern::read(&kern_dir().join(format!("wtc1f{n:02}.krn"))).ok())
    .collect();
  let ren = renaissance(cli::params().ren_works);
  if bach.is_empty() || ren.is_empty() {
    return println!("  (corpus missing)");
  }
  // 24 fugues against 200 works gives 67 spans against 577, and an effect that
  // clears two standard errors on the larger corpus need not on the smaller for
  // any reason but its size. So Bach is sampled far more densely — 30 windows a
  // fugue against 3 a work — to bring the counts within reach. A null result on
  // 67 spans would have been a statement about the sample rather than the criterion.
  let bs = windows(&bach, cli::params().bach_windows);
  let rs = windows(&ren, cli::params().ren_windows);
  println!("  Bach {} spans, Renaissance {} spans; windows are denser in Bach to equalise power
", bs.len(), rs.len());

  // per corpus: for each span, the unranked mean and each criterion's pick
  let mut per_corpus: Vec<(Vec<f64>, Vec<Vec<f64>>)> = vec![];
  for (pieces, spans) in [(&bach, &bs), (&ren, &rs)] {
    let mut base: Vec<f64> = vec![];
    let mut picked: Vec<Vec<f64>> = vec![vec![]; shape::CRITERIA.len()];
    for s in spans.iter() {
      if s.free.len() > 2 {
        continue;
      }
      let p = &pieces[s.piece];
      let all = &s.clipped;
      let source: Vec<Voice> =
        all.iter().enumerate().filter(|(i, _)| !s.freeflag[*i]).map(|(_, v)| v.clone()).collect();
      let pr = Problem {
        voices: all.clone(),
        free: s.freeflag.clone(),
        compass: p.voices.iter().map(compass).collect(),
        key: p.key,
        measure: p.measure,
        plan: harmony::analyse_viterbi(&source, p.beat, cli::params().lambda),
        tier: cli::params().tier.rules(),
        weights: [1.0; 6],
        prescribe: [0.0; 3],
        samples: cli::params().rerank,
        seed: cli::params().seed ^ (s.start as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
        beta: 0.0,
      };
      let Ok(sol) = realise::fill(&pr) else { continue };
      if sol.sampled.is_empty() {
        continue;
      }

      // agreement of one candidate fill with Bach, note for note
      let agree = |cand: &[Voice]| -> f64 {
        let (mut n_, mut ex) = (0usize, 0usize);
        for &v in &s.free {
          for n in all[v].notes.iter().filter(|n| n.attack) {
            if let Some((got, _)) = kern::sounding(&cand[v], n.onset) {
              n_ += 1;
              if got == n.pitch {
                ex += 1;
              }
            }
          }
        }
        if n_ == 0 { f64::NAN } else { ex as f64 / n_ as f64 }
      };

      let rates: Vec<f64> = sol.sampled.iter().map(|c| agree(c)).collect();
      if rates.iter().any(|x| x.is_nan()) {
        continue;
      }
      base.push(rates.iter().sum::<f64>() / rates.len() as f64);
      for (ci, (_, f)) in shape::CRITERIA.iter().enumerate() {
        // the criterion scores the free voices it actually wrote
        let best = sol
          .sampled
          .iter()
          .enumerate()
          .max_by(|a, b| {
            let sc = |c: &(usize, &Vec<Voice>)| -> f64 {
              s.free.iter().map(|&v| f(&c.1[v])).sum::<f64>() / s.free.len() as f64
            };
            sc(a).partial_cmp(&sc(b)).unwrap()
          })
          .map(|(i, _)| i)
          .unwrap_or(0);
        picked[ci].push(rates[best]);
      }
    }
    per_corpus.push((base, picked));
  }

  let paired = |a: &[f64], b: &[f64]| -> (f64, f64) {
    let d: Vec<f64> = a.iter().zip(b).map(|(x, y)| x - y).collect();
    if d.len() < 2 {
      return (0.0, 0.0);
    }
    let m = d.iter().sum::<f64>() / d.len() as f64;
    let var = d.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (d.len() - 1) as f64;
    (100.0 * m, 100.0 * (var / d.len() as f64).sqrt())
  };
  let mean = |v: &[f64]| -> f64 { 100.0 * v.iter().sum::<f64>() / v.len().max(1) as f64 };

  println!(
    "   criterion    |     Bach   gain vs unranked  |  Renaissance   gain vs unranked"
  );
  println!(
    "   {:<12} |    {:>5.1}%                  |       {:>5.1}%",
    "unranked",
    mean(&per_corpus[0].0),
    mean(&per_corpus[1].0)
  );
  let mut verdicts = vec![];
  for (ci, (name, _)) in shape::CRITERIA.iter().enumerate() {
    let (bg, bse) = paired(&per_corpus[0].1[ci], &per_corpus[0].0);
    let (rg, rse) = paired(&per_corpus[1].1[ci], &per_corpus[1].0);
    println!(
      "   {:<12} |    {:>5.1}%   {:>+5.2} +/- {:>4.2}  |       {:>5.1}%   {:>+5.2} +/- {:>4.2}",
      name,
      mean(&per_corpus[0].1[ci]),
      bg,
      bse,
      mean(&per_corpus[1].1[ci]),
      rg,
      rse
    );
    verdicts.push((*name, bg > 2.0 * bse, rg > 2.0 * rse, bg, rg));
  }

  println!("\n  gains are paired per-span differences against the same draws unranked, +/- one standard error.");
  let keep: Vec<&(&str, bool, bool, f64, f64)> = verdicts.iter().filter(|v| v.1 && v.2).collect();
  if keep.is_empty() {
    let any = verdicts.iter().filter(|v| v.1 || v.2).count();
    println!(
      "\n  VERDICT: nothing clears the bar on both corpora ({any} of {} clears it on one). Not adopted.",
      verdicts.len()
    );
  } else {
    println!("\n  VERDICT: {} clears the bar on both corpora:", keep.len());
    for (n, _, _, b, r) in &keep {
      println!("    {n:<12} Bach {b:+.2}, Renaissance {r:+.2}");
    }
  }
}

// -------------------------------------------- step 6: a better harmonic plan ---

/// The plans compared, in the order they are built and printed. `clean` rows see
/// the fixed voices only and are the candidates; `oracle` rows see the answer key
/// and are the ceiling.
const PLANS: [&str; 9] = [
  "none",
  "clean λ=1",
  "clean λ=0",
  "clean λ=2",
  "clean fit≥.6",
  "clean fit≥.8",
  "oracle",
  "oracle/beat",
  "oracle/bar",
];
/// Index of the condition every other one is measured against: §8.6's own plan.
const PLAN_BASE: usize = 1;
/// The rows that do not see the answer key, and so are eligible to be adopted.
const PLAN_CAND: [usize; 4] = [2, 3, 4, 5];
/// The rows that do, and so are a ceiling rather than a candidate.
const PLAN_CEIL: [usize; 3] = [6, 7, 8];

/// Fraction of a plan's duration on which it actually names a chord.
fn covered(pl: &[harmony::Segment]) -> f64 {
  let total: i64 = pl.iter().map(|s| s.end - s.start).sum();
  if total == 0 {
    return 0.0;
  }
  let named: i64 = pl.iter().filter(|s| s.chord.is_some()).map(|s| s.end - s.start).sum();
  named as f64 / total as f64
}

/// Note-for-note agreement of one candidate fill with the composer's own notes:
/// `(notes compared, exact matches)`.
fn agreement(cand: &[Voice], all: &[Voice], free: &[usize]) -> (usize, usize) {
  let (mut n_, mut ex) = (0usize, 0usize);
  for &v in free {
    for n in all[v].notes.iter().filter(|n| n.attack) {
      let Some((got, _)) = kern::sounding(&cand[v], n.onset) else { continue };
      n_ += 1;
      if got == n.pitch {
        ex += 1;
      }
    }
  }
  (n_, ex)
}

/// **What is a better harmonic plan worth, and can one be had without cheating?**
///
/// §9 step 6's fourth proposal. §8.6 already runs the plan three ways and the
/// `leaky` row scores three points above the honest one, which is the largest
/// single effect in that whole table — three times what reversing the objective
/// buys. That is why this item is on the list, and it is also why the number
/// cannot be read off the table as it stands.
///
/// Two faults, both fixed here.
///
/// **The rows are not paired.** A tighter plan solves spans a looser one refuses
/// and refuses spans a looser one solves — `clean` finishes 99 of 117 and
/// `leaky` 110 — so `9.3%` against `7.8%` compares two different sets of notes.
/// Everything below is a **paired per-span difference against `clean` on the
/// spans both conditions finished**, which is §8.2's protocol and the one §8.8
/// used.
///
/// **The oracle is not a plan a grammar could supply.** §2.4's productions name
/// a key plan and a cadence schedule; they cannot name a chord per onset, since
/// the onsets belong to the notes the grammar is asking for. So the oracle is
/// also run **coarsened** to a beat and to a bar, which is the resolution a form
/// grammar could actually deliver, and that is the honest ceiling on step 7.
///
/// Two candidate improvements run beside them, neither of which sees the answer.
/// `λ` is varied because §8.5 swept it against a **full** texture and this plan
/// is analysed from one or two voices out of three or four — the same question
/// asked of half the evidence. And the plan is **gated on its own fit**, because
/// a plan is a hard constraint and a wrong one forbids the right note.
///
/// The bar is §8.2's, fixed before the run: keep a candidate only if it beats
/// `clean` on **both** corpora by more than twice the standard error.
pub fn plan_test() {
  println!("\n== step 6: a better harmonic plan ==");
  println!("  §8.6's `leaky` row is three points above its `clean` row — the largest single effect in");
  println!("  that table. It is also unpaired, and it is not a plan any grammar could emit. Both here.\n");
  println!("  `clean` rows see the fixed voices only and are the candidates. `oracle` rows see the");
  println!("  answer key and are the ceiling; `oracle/beat` and `oracle/bar` are that ceiling coarsened");
  println!("  to the resolution §2.4's grammar could actually deliver.\n");
  println!("  DECIDED BEFORE THE RUN: keep a candidate only if it beats `clean λ=1` on BOTH corpora");
  println!("  by more than twice the standard error of the paired per-span difference.\n");

  let bach: Vec<Piece> = (1..=24)
    .filter_map(|n| kern::read(&kern_dir().join(format!("wtc1f{n:02}.krn"))).ok())
    .collect();
  let ren = renaissance(cli::params().ren_works);
  if bach.is_empty() || ren.is_empty() {
    return println!("  (corpus missing)");
  }
  let bs = windows(&bach, cli::params().bach_windows);
  let rs = windows(&ren, cli::params().ren_windows);

  // per corpus, per plan: the paired gain, its standard error, and how much of
  // the plan names the chord the answer-key analysis names
  let mut summary: Vec<Vec<(f64, f64, f64)>> = vec![];
  let names = ["Bach", "Renaissance"];

  for (cname, pieces, spans) in [(names[0], &bach, &bs), (names[1], &ren, &rs)] {
    let t0 = std::time::Instant::now();
    let mut rows: Vec<Vec<Option<f64>>> = vec![];
    let mut cov = vec![0.0f64; PLANS.len()];
    let mut ov = vec![0.0f64; PLANS.len()];
    let mut fills: Vec<Vec<f64>> = vec![vec![]; PLANS.len()];
    let mut built = 0usize;

    for s in spans.iter() {
      if s.free.len() > 2 {
        continue;
      }
      let p = &pieces[s.piece];
      let all = &s.clipped;
      let source: Vec<Voice> =
        all.iter().enumerate().filter(|(i, _)| !s.freeflag[*i]).map(|(_, v)| v.clone()).collect();
      let base = plan::viterbi(&source, p.beat, cli::params().lambda);
      let oracle = s.segs.clone();
      let variants: Vec<Vec<harmony::Segment>> = vec![
        vec![],
        base.clone(),
        plan::viterbi(&source, p.beat, 0.0),
        plan::viterbi(&source, p.beat, 2.0),
        plan::gated(&base, 0.6),
        plan::gated(&base, 0.8),
        oracle.clone(),
        plan::coarsen(&oracle, p.beat, s.start),
        plan::coarsen(&oracle, p.measure, s.start),
      ];
      built += 1;
      let mut row: Vec<Option<f64>> = vec![None; PLANS.len()];
      for (i, pl) in variants.iter().enumerate() {
        cov[i] += covered(pl);
        ov[i] += plan::overlap(pl, &oracle);
        let pr = Problem {
          voices: all.clone(),
          free: s.freeflag.clone(),
          compass: p.voices.iter().map(compass).collect(),
          key: p.key,
          measure: p.measure,
          plan: pl.clone(),
          tier: cli::params().tier.rules(),
          weights: [1.0; 6],
          prescribe: [0.0; 3],
          samples: 0,
          seed: cli::params().seed ^ (s.start as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
          beta: 0.0,
        };
        let Ok(sol) = realise::fill(&pr) else { continue };
        let (n_, ex) = agreement(&sol.voices, all, &s.free);
        if n_ == 0 {
          continue;
        }
        row[i] = Some(ex as f64 / n_ as f64);
        fills[i].push(if sol.saturated { f64::INFINITY } else { (sol.legal_fills as f64).max(1.0).log10() });
      }
      rows.push(row);
    }
    if rows.is_empty() {
      println!("  {cname}: no spans");
      summary.push(vec![(0.0, 0.0, 0.0); PLANS.len()]);
      continue;
    }

    println!(
      "  {cname}: {built} spans of eight quarters, at most two free voices, tier {}",
      cli::params().tier.label()
    );
    println!("   plan            done   covered   vs oracle   log10 fills    exact   gain vs clean     n");
    let mut here = vec![];
    for i in 0..PLANS.len() {
      let done = rows.iter().filter(|r| r[i].is_some()).count();
      let mine: Vec<f64> = rows.iter().filter_map(|r| r[i]).collect();
      // paired against `clean` on the spans both finished
      let d: Vec<f64> = rows
        .iter()
        .filter_map(|r| match (r[i], r[PLAN_BASE]) {
          (Some(a), Some(b)) => Some(a - b),
          _ => None,
        })
        .collect();
      let (m, se) = if d.len() < 2 {
        (0.0, 0.0)
      } else {
        let m = d.iter().sum::<f64>() / d.len() as f64;
        let var = d.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (d.len() - 1) as f64;
        (100.0 * m, 100.0 * (var / d.len() as f64).sqrt())
      };
      let overlap = 100.0 * ov[i] / built as f64;
      here.push((m, se, overlap));
      let gain =
        if i == PLAN_BASE { "      —      ".to_string() } else { format!("{m:>+6.2} +/- {se:>4.2}") };
      println!(
        "   {:<14} {:>4}    {:>5.0}%      {:>5.0}%       {:>7.1}    {:>5.1}%  {gain}  {:>4}",
        PLANS[i],
        done,
        100.0 * cov[i] / built as f64,
        overlap,
        median(&fills[i]),
        100.0 * mine.iter().sum::<f64>() / mine.len().max(1) as f64,
        d.len(),
      );
    }
    summary.push(here);
    println!("   ({:.0}s)\n", t0.elapsed().as_secs_f64());
  }

  println!("  `covered` is the fraction of the span on which the plan names a chord at all; `vs oracle`");
  println!("  is the fraction of it naming the same chord as the answer-key analysis. `exact` is over");
  println!("  each row's own solved spans and is not comparable across rows — `gain` is, and it is the");
  println!("  paired per-span difference on the spans that row and `clean λ=1` both finished.\n");

  if summary.len() < 2 {
    return;
  }

  // --- what a point of correct harmony is worth ------------------------------
  //
  // The three ceiling rows differ from `clean` in one measurable way — how much
  // of the plan is right — so dividing the gain by that difference converts
  // "build a better analyser" into an exchange rate. It is quoted rather than
  // fitted: three points and a mean, not a regression.
  println!("  Each of the ceiling rows differs from `clean` in exactly one measurable respect, which is");
  println!("  how much of the plan is right. The gain per point of chord agreement:\n");
  for (c, name) in names.iter().enumerate() {
    let base = summary[c][PLAN_BASE].2;
    let rates: Vec<f64> = PLAN_CEIL
      .iter()
      .filter(|&&i| summary[c][i].2 - base > 1.0)
      .map(|&i| summary[c][i].0 / (summary[c][i].2 - base))
      .collect();
    if rates.is_empty() {
      continue;
    }
    let m = rates.iter().sum::<f64>() / rates.len() as f64;
    println!(
      "    {name:<12} {m:.3} points of note agreement per point of chord agreement  ({})",
      rates.iter().map(|r| format!("{r:.3}")).collect::<Vec<_>>().join(", ")
    );
  }

  // --- the verdict, on the bar stated above ---------------------------------
  let clears = |i: usize| -> bool {
    (0..2).all(|c| summary[c][i].0 > 2.0 * summary[c][i].1 && summary[c][i].1 > 0.0)
  };
  let keep: Vec<usize> = PLAN_CAND.iter().copied().filter(|&i| clears(i)).collect();
  println!();
  if keep.is_empty() {
    let any = PLAN_CAND
      .iter()
      .filter(|&&i| (0..2).any(|c| summary[c][i].0 > 2.0 * summary[c][i].1 && summary[c][i].1 > 0.0))
      .count();
    println!(
      "  VERDICT: no plan that stays inside the fixed voices clears the bar on both corpora ({any} of {}",
      PLAN_CAND.len()
    );
    println!("  clears it on one). Nothing adopted; §8.6's plan stands.");
  } else {
    println!("  VERDICT: {} clears the bar on both corpora:", keep.len());
    for i in keep {
      println!(
        "    {:<12} Bach {:+.2}, Renaissance {:+.2}",
        PLANS[i], summary[0][i].0, summary[1][i].0
      );
    }
  }
  let ceil: Vec<usize> = PLAN_CEIL.iter().copied().filter(|&i| clears(i)).collect();
  println!(
    "\n  {} of the {} ceiling rows clear it, {} included — so the plan is worth having and the",
    ceil.len(),
    PLAN_CEIL.len(),
    if ceil.contains(&8) { "`oracle/bar`" } else { "the finest" }
  );
  println!("  analyser is what stands between the search and it.");
}

// ------------------------------- step 6: replacing the soft tier, not reweighting it ---


/// Mean melodic interval of a line, in scale steps, counting a repeated note as
/// zero. §8.6's diagnosis is that the fills are too narrow and §8.8's is that
/// they do not go anywhere; this and `spread` are those two faults as numbers,
/// and the composer's own notes are in the table beside them.
fn mean_step(v: &Voice) -> Option<f64> {
  let p: Vec<i16> = v.notes.iter().filter(|n| n.attack).map(|n| n.pitch.step).collect();
  if p.len() < 2 {
    return None;
  }
  Some(p.windows(2).map(|w| (w[1] - w[0]).abs() as f64).sum::<f64>() / (p.len() - 1) as f64)
}

/// A line's range over the span, in scale steps.
fn spread(v: &Voice) -> Option<f64> {
  let p: Vec<i16> = v.notes.iter().filter(|n| n.attack).map(|n| n.pitch.step).collect();
  let (lo, hi) = (p.iter().min()?, p.iter().max()?);
  Some((hi - lo) as f64)
}

/// The mean of `mean_step` and `spread` over the free voices of one fill.
fn shapes(cand: &[Voice], free: &[usize]) -> (f64, f64) {
  let s: Vec<f64> = free.iter().filter_map(|&v| mean_step(&cand[v])).collect();
  let r: Vec<f64> = free.iter().filter_map(|&v| spread(&cand[v])).collect();
  let m = |x: &[f64]| if x.is_empty() { 0.0 } else { x.iter().sum::<f64>() / x.len() as f64 };
  (m(&s), m(&r))
}

/// **Is the soft tier one criterion or six, and does saying the same thing
/// positively beat saying it as a prohibition?**
///
/// §9 step 6's last proposal, and the one whose stated destination this
/// repository cannot reach: it points at **Marpurg** and **Kirnberger**, and
/// `literature/` holds neither. What can be done without them is the two
/// questions above, and they are the two that have to be answered before any
/// replacement is worth transcribing.
///
/// **First, an ablation.** The tier is six prohibitions charged at equal weight
/// and worth about a point over not optimising at all (§8.6). Nobody has asked
/// which of the six that point comes from. Six one-hot runs answer it. This is
/// not the reweighting §5 refuses — no weight vector is being proposed for
/// adoption — it is the measurement that says whether "the soft tier" names one
/// thing or six.
///
/// **Then a replacement.** §8.8 closed with the treatise stating only what a
/// line must not do, and §8.9 with the harmony under the line being the one
/// lever that moves. Three positive criteria follow from those two sentences and
/// need no text this repository lacks: **move by step**, **move against the
/// other voice**, and **state the harmony**. Each replaces the whole tier rather
/// than joining it — `weights` goes to zero — and each is reported alone,
/// because combining them needs magnitudes and §5 is about exactly that. The
/// combination is reported too, at equal weight, since equal weight is what the
/// incumbent tier already assumes.
///
/// Every row searches **the same graph**: a prescription reorders the legal set
/// and never shrinks it, which is asserted in `realise`'s tests. So the `done`
/// column is constant by construction and any difference is the objective's.
///
/// The bar is §8.2's, fixed before the run: keep a replacement only if it beats
/// the six-criterion tier on **both** corpora by more than twice the standard
/// error of the paired per-span difference.
pub fn soft_test() {
  println!("\n== step 6: replacing the soft tier rather than reweighting it ==");
  println!("  The tier is six prohibitions at equal weight, worth about a point over not optimising.");
  println!("  Two questions: which of the six is that point, and does stating the same thing");
  println!("  positively do better? The three positive criteria replace the tier — `weights` goes to");
  println!("  zero — rather than joining it.\n");
  println!("  Marpurg and Kirnberger are where §9 points and `literature/` holds neither, so what is");
  println!("  transcribed here is what §8.8 and §8.9 imply and no text this repository lacks.\n");
  println!("  DECIDED BEFORE THE RUN: keep a replacement only if it beats `soft(6)` on BOTH corpora");
  println!("  by more than twice the standard error of the paired per-span difference.\n");

  // (label, soft weights, prescription weights, draws)
  //
  // Two controls, and the second is easy to leave out and necessary. A one-hot
  // ablation leaves most paths **tied** at zero cost, so what it reports is
  // partly its criterion and partly whichever of the tied paths the search
  // happens to keep. `tie-break only` is that arbitrary choice with no criterion
  // at all, and an ablation is worth reading against it rather than against the
  // uniform draw.
  let mut conds: Vec<(String, [f64; 6], [f64; 3], usize)> = vec![
    ("no objective".into(), [0.0; 6], [0.0; 3], cli::params().samples),
    ("tie-break only".into(), [0.0; 6], [0.0; 3], 0),
    ("soft(6)".into(), [1.0; 6], [0.0; 3], 0),
  ];
  /// Index of the row every other one is measured against.
  const BASE: usize = 2;
  /// The six one-hot ablations of the tier.
  const ABL: std::ops::Range<usize> = BASE + 1..BASE + 7;
  for (i, r) in SOFT.iter().enumerate() {
    let mut w = [0.0; 6];
    w[i] = 1.0;
    let short = match r {
      Rule::DirectToPerfect => "direct→perfect",
      Rule::PerfectConsonance => "perfect cons.",
      Rule::DirectMotion => "direct motion",
      Rule::VoiceCrossing => "crossing",
      Rule::UnrecoveredLeap => "leap",
      _ => "repetition",
    };
    conds.push((format!("only {short}"), w, [0.0; 3], 0));
  }
  for (i, name) in realise::PRESCRIPTIONS.iter().enumerate() {
    let mut w = [0.0; 3];
    w[i] = 1.0;
    conds.push((format!("→ {name}"), [0.0; 6], w, 0));
  }
  conds.push(("→ all three".into(), [0.0; 6], [1.0; 3], 0));

  let bach: Vec<Piece> = (1..=24)
    .filter_map(|n| kern::read(&kern_dir().join(format!("wtc1f{n:02}.krn"))).ok())
    .collect();
  let ren = renaissance(cli::params().ren_works);
  if bach.is_empty() || ren.is_empty() {
    return println!("  (corpus missing)");
  }
  let bs = windows(&bach, cli::params().bach_windows);
  let rs = windows(&ren, cli::params().ren_windows);

  let mut summary: Vec<Vec<(f64, f64)>> = vec![];
  let names = ["Bach", "Renaissance"];

  for (cname, pieces, spans) in [(names[0], &bach, &bs), (names[1], &ren, &rs)] {
    let t0 = std::time::Instant::now();
    let mut rows: Vec<Vec<Option<f64>>> = vec![];
    let mut shape: Vec<Vec<f64>> = vec![vec![]; conds.len() * 2];
    let mut theirs: Vec<Vec<f64>> = vec![vec![]; 2];

    for s in spans.iter() {
      if s.free.len() > 2 {
        continue;
      }
      let p = &pieces[s.piece];
      let all = &s.clipped;
      let source: Vec<Voice> =
        all.iter().enumerate().filter(|(i, _)| !s.freeflag[*i]).map(|(_, v)| v.clone()).collect();
      let plan = harmony::analyse_viterbi(&source, p.beat, cli::params().lambda);
      let mut row: Vec<Option<f64>> = vec![None; conds.len()];
      for (i, (_, w, pw, draws)) in conds.iter().enumerate() {
        let pr = Problem {
          voices: all.clone(),
          free: s.freeflag.clone(),
          compass: p.voices.iter().map(compass).collect(),
          key: p.key,
          measure: p.measure,
          plan: plan.clone(),
          tier: cli::params().tier.rules(),
          weights: *w,
          prescribe: *pw,
          samples: *draws,
          seed: cli::params().seed ^ (s.start as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
          beta: 0.0,
        };
        let Ok(sol) = realise::fill(&pr) else { continue };
        // the sampled rows are scored over their draws, the rest over the one
        // fill the objective chose; either way it is the same scoring function
        let cands: Vec<&Vec<Voice>> =
          if *draws > 0 { sol.sampled.iter().collect() } else { vec![&sol.voices] };
        if cands.is_empty() {
          continue;
        }
        let mut rates = vec![];
        let (mut ms, mut sp) = (0.0, 0.0);
        for c in &cands {
          let (n_, ex) = agreement(c, all, &s.free);
          if n_ == 0 {
            continue;
          }
          rates.push(ex as f64 / n_ as f64);
          let (a, b) = shapes(c, &s.free);
          ms += a;
          sp += b;
        }
        if rates.is_empty() {
          continue;
        }
        row[i] = Some(rates.iter().sum::<f64>() / rates.len() as f64);
        shape[i * 2].push(ms / cands.len() as f64);
        shape[i * 2 + 1].push(sp / cands.len() as f64);
      }
      // the answer key's own shape on the very same voices
      if row[BASE].is_some() {
        let (a, b) = shapes(all, &s.free);
        theirs[0].push(a);
        theirs[1].push(b);
      }
      rows.push(row);
    }
    if rows.is_empty() {
      println!("  {cname}: no spans");
      summary.push(vec![(0.0, 0.0); conds.len()]);
      continue;
    }

    let avg = |v: &[f64]| -> f64 { if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 } };
    println!(
      "  {cname}: {} spans of eight quarters, at most two free voices, tier {}, clean plan",
      rows.len(),
      cli::params().tier.label()
    );
    println!("   objective            done    |step|   range    exact   gain on soft(6)      n");
    let mut here = vec![];
    for i in 0..conds.len() {
      let done = rows.iter().filter(|r| r[i].is_some()).count();
      let mine: Vec<f64> = rows.iter().filter_map(|r| r[i]).collect();
      let d: Vec<f64> = rows
        .iter()
        .filter_map(|r| match (r[i], r[BASE]) {
          (Some(a), Some(b)) => Some(a - b),
          _ => None,
        })
        .collect();
      let (m, se) = if d.len() < 2 {
        (0.0, 0.0)
      } else {
        let m = d.iter().sum::<f64>() / d.len() as f64;
        let var = d.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (d.len() - 1) as f64;
        (100.0 * m, 100.0 * (var / d.len() as f64).sqrt())
      };
      here.push((m, se));
      let gain = if i == BASE { "      —      ".into() } else { format!("{m:>+6.2} +/- {se:>4.2}") };
      println!(
        "   {:<20} {:>4}    {:>5.2}   {:>5.2}    {:>5.1}%   {gain}   {:>4}",
        conds[i].0,
        done,
        avg(&shape[i * 2]),
        avg(&shape[i * 2 + 1]),
        100.0 * avg(&mine),
        d.len(),
      );
    }
    println!(
      "   {:<20} {:>4}    {:>5.2}   {:>5.2}    {:>5.1}%",
      "the composer's own",
      theirs[0].len(),
      avg(&theirs[0]),
      avg(&theirs[1]),
      100.0
    );
    summary.push(here);
    println!("   ({:.0}s)\n", t0.elapsed().as_secs_f64());
  }

  println!("  `|step|` is the mean melodic interval of the free voices in scale steps and `range` their");
  println!("  compass over the span — §8.6's narrowness and §8.8's deficiency as numbers, with the");
  println!("  answer key's own values on the same voices in the last row. `done` is constant because");
  println!("  every row searches the same graph: an objective reorders the legal set, it does not prune it.");
  println!("  `only X` charges one soft criterion and ignores the other five, and is read against");
  println!("  `tie-break only` rather than the uniform draw, since most paths tie under it. `→` rows");
  println!("  charge a positive criterion **instead of** the tier, not beside it.\n");

  if summary.len() < 2 {
    return;
  }
  let clears = |i: usize| -> bool {
    (0..2).all(|c| summary[c][i].0 > 2.0 * summary[c][i].1 && summary[c][i].1 > 0.0)
  };
  let pres: Vec<usize> = (conds.len() - 4..conds.len()).collect();
  let keep: Vec<usize> = pres.iter().copied().filter(|&i| clears(i)).collect();
  if keep.is_empty() {
    let any = pres
      .iter()
      .filter(|&&i| (0..2).any(|c| summary[c][i].0 > 2.0 * summary[c][i].1 && summary[c][i].1 > 0.0))
      .count();
    println!(
      "  VERDICT: no prescription beats the six-criterion tier on both corpora ({any} of {} beats it on",
      pres.len()
    );
    println!("  one). Nothing adopted; §8.6's soft tier stands.");
  } else {
    println!("  VERDICT: {} beats the six-criterion tier on both corpora:", keep.len());
    for i in keep {
      println!("    {:<20} Bach {:+.2}, Renaissance {:+.2}", conds[i].0, summary[0][i].0, summary[1][i].0);
    }
  }

  // which of the six the tier's point actually is
  let abl: Vec<usize> = ABL.collect();
  println!("\n  And of the six prohibitions, measured one at a time against all six together:");
  for c in 0..2 {
    let mut v: Vec<(f64, f64, &str)> =
      abl.iter().map(|&i| (summary[c][i].0, summary[c][i].1, conds[i].0.as_str())).collect();
    v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let best = v.first().unwrap();
    println!(
      "    {:<12} best alone is `{}` at {:+.2} +/- {:.2}; worst is `{}` at {:+.2}",
      names[c],
      best.2,
      best.0,
      best.1,
      v.last().unwrap().2,
      v.last().unwrap().0
    );
  }
}

// ------------------------- step 6: is the objective worth anything, restated paired ---

/// **§8.6's three-row control, on §8.6's own spans, paired.**
///
/// §8.10 found the six-criterion tier *losing* to no objective at all on both
/// corpora, which is the reverse of what §8.6 reports. Two things differ between
/// the two measurements and only one of them can be the cause, so this changes
/// one at a time.
///
/// §8.10 runs on §8.8's windows; §8.6 runs on the annotated entry spans. §8.10
/// pairs per span; §8.6 pools over notes, and pooling weights a span by how many
/// notes it has *and* counts each of the eight uniform draws' notes separately.
/// This runs §8.6's rows on §8.6's spans and reports both accountings side by
/// side, so that whichever of the two is responsible says so.
///
/// A pooled figure and a paired one answering differently is not a tie. The
/// paired one is correct: the span is the unit of replication, which is the
/// point §8.2 established and §8.8 and §8.9 have used since.
pub fn objective_check() {
  println!("\n== step 6: what the objective is worth, on §8.6's own spans and paired ==");
  println!("  §8.10 has the six-criterion tier losing to no objective at all. §8.6 has it winning.");
  println!("  §8.10 uses §8.8's windows and pairs per span; §8.6 uses the annotated entry spans and");
  println!("  pools over notes. This holds the spans at §8.6's and reports both accountings.\n");

  let (pieces, sp) = spans();
  if sp.is_empty() {
    return println!("  (corpus or ground truth missing)");
  }
  let conds: [(&str, [f64; 6], usize); 4] = [
    ("no objective", [0.0; 6], cli::params().samples),
    ("tie-break only", [0.0; 6], 0),
    ("soft(6) minimised", [1.0; 6], 0),
    ("soft(6) maximised", [-1.0; 6], 0),
  ];
  const BASE: usize = 2;

  let mut rows: Vec<Vec<Option<(f64, usize, usize)>>> = vec![];
  let mut shape: Vec<Vec<f64>> = vec![vec![]; conds.len() * 2];
  let mut theirs: Vec<Vec<f64>> = vec![vec![]; 2];

  for s in sp.iter() {
    if s.free.len() > 2 {
      continue;
    }
    let p = &pieces[s.piece];
    let all = &s.clipped;
    let source: Vec<Voice> =
      all.iter().enumerate().filter(|(i, _)| !s.freeflag[*i]).map(|(_, v)| v.clone()).collect();
    let plan = harmony::analyse_viterbi(&source, p.beat, cli::params().lambda);
    let mut row: Vec<Option<(f64, usize, usize)>> = vec![None; conds.len()];
    for (i, (_, w, draws)) in conds.iter().enumerate() {
      let pr = Problem {
        voices: all.clone(),
        free: s.freeflag.clone(),
        compass: p.voices.iter().map(compass).collect(),
        key: p.key,
        measure: p.measure,
        plan: plan.clone(),
        tier: cli::params().tier.rules(),
        weights: *w,
        prescribe: [0.0; 3],
        samples: *draws,
        seed: cli::params().seed ^ (s.start as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
        beta: 0.0,
      };
      let Ok(sol) = realise::fill(&pr) else { continue };
      let cands: Vec<&Vec<Voice>> =
        if *draws > 0 { sol.sampled.iter().collect() } else { vec![&sol.voices] };
      if cands.is_empty() {
        continue;
      }
      let (mut tn, mut te) = (0usize, 0usize);
      let (mut rate, mut ms, mut spd) = (0.0, 0.0, 0.0);
      let mut k = 0usize;
      for c in &cands {
        let (n_, ex) = agreement(c, all, &s.free);
        if n_ == 0 {
          continue;
        }
        tn += n_;
        te += ex;
        rate += ex as f64 / n_ as f64;
        let (a, b) = shapes(c, &s.free);
        ms += a;
        spd += b;
        k += 1;
      }
      if k == 0 {
        continue;
      }
      // the per-span rate is the paired quantity; the raw counts are what
      // pooling adds up, and §8.6 reports the second
      row[i] = Some((rate / k as f64, te, tn));
      shape[i * 2].push(ms / k as f64);
      shape[i * 2 + 1].push(spd / k as f64);
    }
    if row[BASE].is_some() {
      let (a, b) = shapes(all, &s.free);
      theirs[0].push(a);
      theirs[1].push(b);
    }
    rows.push(row);
  }
  if rows.is_empty() {
    return println!("  (no spans)");
  }

  let avg = |v: &[f64]| -> f64 { if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 } };
  let pool = |i: usize| -> f64 {
    let (e, n) = rows.iter().filter_map(|r| r[i]).fold((0usize, 0usize), |(a, b), (_, e, n)| (a + e, b + n));
    100.0 * e as f64 / n.max(1) as f64
  };
  let pooled_base = pool(BASE);
  println!("  {} annotated entry spans searched, tier {}, clean plan\n", rows.len(), cli::params().tier.label());
  println!("   objective             |step|   range   POOLED over notes   PAIRED per span      n");
  for i in 0..conds.len() {
    let d: Vec<f64> = rows
      .iter()
      .filter_map(|r| match (r[i], r[BASE]) {
        (Some(a), Some(b)) => Some(a.0 - b.0),
        _ => None,
      })
      .collect();
    let (m, se) = if d.len() < 2 {
      (0.0, 0.0)
    } else {
      let m = d.iter().sum::<f64>() / d.len() as f64;
      let var = d.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (d.len() - 1) as f64;
      (100.0 * m, 100.0 * (var / d.len() as f64).sqrt())
    };
    let mine: Vec<f64> = rows.iter().filter_map(|r| r[i]).map(|x| x.0).collect();
    let (pl, pr_) = if i == BASE {
      (format!("{:>5.1}%      —   ", pooled_base), "      —      ".to_string())
    } else {
      (format!("{:>5.1}%   {:>+5.2}", pool(i), pool(i) - pooled_base), format!("{m:>+6.2} +/- {se:>4.2}"))
    };
    println!(
      "   {:<18}    {:>5.2}   {:>5.2}   {pl:<18}  {pr_}   {:>4}   [{:>5.1}%]",
      conds[i].0,
      avg(&shape[i * 2]),
      avg(&shape[i * 2 + 1]),
      d.len(),
      100.0 * avg(&mine),
    );
  }
  println!(
    "   {:<18}    {:>5.2}   {:>5.2}",
    "the composer's own",
    avg(&theirs[0]),
    avg(&theirs[1])
  );
  println!("\n  `POOLED` adds every note in every span together, which is what §8.6 reports and which");
  println!("  weights a span by its note count — and, for the sampled row, counts all eight draws.");
  println!("  `PAIRED` is the mean of the per-span differences, with the span as the unit of");
  println!("  replication. The bracketed figure is the unweighted mean of the per-span rates.");
}

// --------------------------------------- step 7: Marpurg's tonal answer, tested ---

/// **Does the treatise of Bach's own circle predict Bach's own answers?**
///
/// §9's standing open problem is that Fux is 1725 and Palestrina-style vocal
/// while the WTC is 1722 and keyboard, and that the fugue treatise of Bach's
/// circle had never been read. `answer.rs` transcribes its third chapter. This
/// measures it, on §8.2's instrument and §8.7's question: **a rule is worth
/// having only if the music stays inside it, and only if staying inside it means
/// something.**
///
/// The unit is the exposition's `Führer`/`Gefährte` pair — the first two
/// annotated entries of each fugue, which is the object Marpurg's chapter is
/// about. They are compared **by scale degree**, since an answer sits in another
/// voice at another octave and only its degrees are the claim.
///
/// Three conditions, and the first is the null the other two must beat:
///
/// - `real 5th` — the subject transposed bodily up a fifth, which needs no
///   treatise at all and is what [§8.3](../readme.md)'s stretto placement
///   already does;
/// - `real 4th` — the same up a fourth, the other plain transposition;
/// - `Marpurg` — the set his stated rules admit, which contains both of those
///   and at most `2n` sequences besides.
///
/// The set's **size** is reported beside its coverage for the reason §8.7 gives:
/// a whitelist that admits everything explains nothing.
pub fn answer_test() {
  println!("\n== step 7: Marpurg's tonal answer, against Bach's own ==");
  println!("  Hauptstück 3 of the Abhandlung von der Fuge (1753), transcribed in `answer.rs`:");
  println!("  two Grundsätze, two rules pinning the ends of the subject, and one Vertauschung that");
  println!("  changes exactly one melodic interval by exactly one degree. Where Marpurg settles the");
  println!("  mutation's place by worked example rather than by rule, this enumerates the places.\n");
  println!("  The unit is the exposition's Führer/Gefährte pair — the first two annotated entries —");
  println!("  compared by scale degree, since the answer sits in another voice at another octave.\n");

  let dir = kern_dir();
  let mut pieces = std::collections::BTreeMap::new();
  for n in 1..=24 {
    if let Ok(p) = kern::read(&dir.join(format!("wtc1f{n:02}.krn"))) {
      pieces.insert(format!("wtc-i-{n:02}"), p);
    }
  }
  if pieces.is_empty() {
    return println!("  (corpus missing)");
  }
  let specs = refdata::read(
    std::path::Path::new("corpus/algomus-data/fugues/fugues.ref"),
    &|id| pieces.get(id).map(|p| p.measure),
  )
  .unwrap_or_default();
  if specs.is_empty() {
    return println!("  (ground truth missing)");
  }

  let (mut pairs, mut no_tonic, mut ragged) = (0usize, 0usize, 0usize);
  let (mut fifth, mut fourth, mut marpurg) = (0usize, 0usize, 0usize);
  // the case that says the rules are not merely loose but *wrong*: Bach wrote a
  // plain transposition and Marpurg's rules refuse it
  let mut excluded: Vec<String> = vec![];
  let mut sizes: Vec<f64> = vec![];
  let mut open_sizes: Vec<f64> = vec![];
  let (mut loose, mut r1_n, mut r1_ok, mut r2_ok) = (0usize, 0usize, 0usize, 0usize);
  // A rule only earns anything where it *differs* from plain transposition. For
  // a subject opening on the tonic, Rule I says the answer opens on the fifth —
  // which is what transposing up a fifth does anyway, so those cases are free.
  // The discriminating ones are where the rule says `Fourth`.
  let (mut r1_d, mut r1_d_ok, mut r2_d, mut r2_d_ok) = (0usize, 0usize, 0usize, 0usize);
  // §3.3: where the subject *ends* is an annotation choice and the ground truth
  // records the dissenting readings. Rule II is a claim about that very note, so
  // it is tested at every boundary offered rather than at one of them.
  let (mut r2_any_n, mut r2_any_ok) = (0usize, 0usize);
  let mut shown = false;
  // the discriminating cases: those a plain transposition gets wrong
  let mut tonal: Vec<(String, bool, usize)> = vec![];

  for spec in &specs {
    let Some(p) = pieces.get(&spec.id) else { continue };
    if spec.len == 0 || spec.entries.len() < 2 {
      continue;
    }
    let Some((pc, _)) = p.tonic else {
      no_tonic += 1;
      continue;
    };
    let Some(tonic) = answer::tonic_letter(pc, &p.key) else {
      no_tonic += 1;
      continue;
    };
    let mut ents = spec.entries.clone();
    ents.sort_by_key(|e| e.1);
    let ents: Vec<&(char, i64)> = ents.iter().filter(|e| e.1 >= 0).take(2).collect();
    if ents.len() < 2 {
      continue;
    }
    let cut_at = |e: &(char, i64), l: i64| clip(&p.voices[voice_of(p, e.0)], e.1, e.1 + l);
    let cut = |e: &(char, i64)| cut_at(e, spec.len);
    let (dux, comes) = (cut(ents[0]), cut(ents[1]));
    let want = answer::degrees(&comes, tonic);
    if want.is_empty() || want.len() != answer::degrees(&dux, tonic).len() {
      // the annotation's window does not bracket the two entries alike; a
      // comparison note for note would be comparing different numbers of notes
      ragged += 1;
      continue;
    }
    pairs += 1;

    let hit = |v: &Voice| answer::degrees(v, tonic) == want;
    let is5 = hit(&answer::real(&dux, answer::Leg::Fifth, &p.key));
    let is4 = hit(&answer::real(&dux, answer::Leg::Fourth, &p.key));
    let set = answer::admissible(&dux, &p.key, tonic);
    let inset = set.iter().any(hit);
    sizes.push(set.len() as f64);
    // Rule II is hedged in the source, so the same set without it
    let open = answer::admissible_opt(&dux, &p.key, tonic, false);
    open_sizes.push(open.len() as f64);
    loose += open.iter().any(hit) as usize;

    // and each rule checked on the one note it is about, which is the claim
    // Marpurg actually makes and is independent of where a mutation falls
    let dd = answer::degrees(&dux, tonic);
    if let Some(l) = answer::first_leg(dd[0]) {
      let ok = answer::answered(dd[0], l) == want[0];
      r1_n += 1;
      r1_ok += ok as usize;
      if l == answer::Leg::Fourth {
        r1_d += 1;
        r1_d_ok += ok as usize;
      }
    }
    if let Some(l) = answer::last_leg(dd[dd.len() - 1]) {
      let ok = answer::answered(dd[dd.len() - 1], l) == want[want.len() - 1];
      r2_ok += ok as usize;
      if l == answer::Leg::Fourth {
        r2_d += 1;
        r2_d_ok += ok as usize;
      }
    }
    for &l in std::iter::once(&spec.len).chain(spec.alternatives.iter()) {
      if l <= 0 {
        continue;
      }
      let (a, b) = (cut_at(ents[0], l), cut_at(ents[1], l));
      let (da, db) = (answer::degrees(&a, tonic), answer::degrees(&b, tonic));
      if da.is_empty() || da.len() != db.len() {
        continue;
      }
      if answer::last_leg(da[da.len() - 1]) == Some(answer::Leg::Fourth) {
        r2_any_n += 1;
        r2_any_ok +=
          (answer::answered(da[da.len() - 1], answer::Leg::Fourth) == db[db.len() - 1]) as usize;
      }
    }
    fifth += is5 as usize;
    fourth += is4 as usize;
    marpurg += inset as usize;
    if (is5 || is4) && !inset {
      excluded.push(spec.id.clone());
    }
    if !is5 && !is4 {
      tonal.push((spec.id.clone(), inset, set.len()));
    }
    // one pair shown whole, because a percentage over 24 cases is worth nothing
    // without an instance a reader can check by eye
    if !shown && !is5 && inset {
      shown = true;
      let row = |label: &str, v: &[usize]| {
        let cells: Vec<String> = v.iter().take(18).map(|d| format!("{:>3}", d + 1)).collect();
        println!("     {label:<22}{}", cells.join(""));
      };
      println!("   {} in {}, degrees of the subject and of the answer:", spec.id, spec.id);
      row("Fuhrer", &answer::degrees(&dux, tonic));
      row("Gefahrte, Bach's", &want);
      row("plain fifth", &answer::degrees(&answer::real(&dux, answer::Leg::Fifth, &p.key), tonic));
      let k = set.iter().position(hit).unwrap();
      row("Marpurg, member", &answer::degrees(&set[k], tonic));
      println!("     ({} of {} members matches; degrees are 1-based, 1 the tonic)
", k + 1, set.len());
    }
  }

  if pairs == 0 {
    return println!("  (no usable Führer/Gefährte pairs)");
  }
  let pct = |k: usize| 100.0 * k as f64 / pairs as f64;
  println!("   {pairs} usable pairs of the {} annotated fugues", specs.len());
  if no_tonic + ragged > 0 {
    println!("   ({no_tonic} without a key interpretation, {ragged} whose two entries are annotated to");
    println!("    different note counts and cannot be compared note for note)\n");
  }
  println!("
   Each rule on the one note it is about — the claim Marpurg makes, independent of");
  println!("   where a mutation falls:
");
  let rate = |k: usize, n: usize| 100.0 * k as f64 / n.max(1) as f64;
  println!("                                                        all cases      where it differs");
  println!("                                                                       from a plain fifth");
  println!(
    "   Rule I,  first note: tonic and dominant answer      {:>5.1}%  of {r1_n:<4}    {:>5.1}%  of {r1_d}",
    rate(r1_ok, r1_n),
    rate(r1_d_ok, r1_d)
  );
  println!(
    "   Rule II, last note:  tonic/dominant, third/third    {:>5.1}%  of {pairs:<4}    {:>5.1}%  of {r2_d}",
    rate(r2_ok, pairs),
    rate(r2_d_ok, r2_d)
  );
  println!(
    "   Rule II again, at every subject end the ground truth offers    {:>5.1}%  of {r2_any_n}",
    rate(r2_any_ok, r2_any_n)
  );
  println!("
   The right-hand column is the one that counts. Where a rule says `answer at the fifth`");
  println!("   it is saying what transposition does anyway, and only the cases where it says");
  println!("   `answer at the fourth` are the rule earning something. The last line answers the");
  println!("   objection §3.3 raises against the one before it: Rule II is a claim about the");
  println!("   subject's final note, and where that note falls is a reading rather than a fact, so");
  println!("   the rule is retried at every reading the ground truth records.");

  println!("
   And whole answers, note for note:
");
  println!("   condition                          agrees with Bach   median set");
  println!("   real answer, up a fifth                     {:>5.1}%            1", pct(fifth));
  println!("   real answer, up a fourth                    {:>5.1}%            1", pct(fourth));
  println!(
    "   Marpurg, Rules I and II                     {:>5.1}%         {:>4.0}",
    pct(marpurg),
    median(&sizes)
  );
  println!(
    "   Marpurg, Rule I only (II is hedged)         {:>5.1}%         {:>4.0}",
    pct(loose),
    median(&open_sizes)
  );
  if !excluded.is_empty() {
    println!(
      "
   And the set **refuses** {} answer(s) Bach wrote as a plain transposition: {}.",
      excluded.len(),
      excluded.join(", ")
    );
    println!("   A rule that admits too little is wrong in a way a loose one is not, and this is the");
    println!("   figure that keeps the row above from being read as coverage.");
  }

  println!("\n   {} answers are neither plain transposition — the tonal ones, which are the whole", tonal.len());
  println!("   point of the chapter and the only place a treatise can earn anything:");
  if tonal.is_empty() {
    println!("     (none)");
  } else {
    let got = tonal.iter().filter(|t| t.1).count();
    for (id, ok, n) in &tonal {
      println!("     {id}   {}   set of {n}", if *ok { "inside Marpurg's set" } else { "OUTSIDE it" });
    }
    println!(
      "\n   {got} of {} inside, which is {:.0}% of the cases the null cannot reach.",
      tonal.len(),
      100.0 * got as f64 / tonal.len() as f64
    );
  }
  println!("\n  `median set` is how many answers the rules admit for one subject, against §8.6's");
  println!("  10^12 fills of three bars. A set of one is a prediction; a set of ten is a shortlist.");
}
