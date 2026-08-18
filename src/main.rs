//! Drivers for every measurement readme §8 reports.
//!
//!   cargo run --release -- states    the reachable state count
//!   cargo run --release -- verdict   the three tests the roughness field failed
//!   cargo run --release -- corpus    how often Bach violates the rulebook
//!   cargo run --release -- stretto   BWV 867's five entries, as a clique
//!   cargo run --release -- sweep     the harmonic analyser, penalty swept
//!   cargo run --release -- realise   the fill, its measurement, and the MIDI
//!
//! Run with no argument for the four that take seconds. §10.2 of the readme maps
//! each section to its command.

mod automaton;
mod corpus;
mod experiments;
mod harmony;
mod midi;
mod plan;
mod realise;
mod refdata;
mod shape;
mod species;
mod step5;
mod stretto;
mod kern;
mod pitch;

use automaton::{Move, Rule, State, Sym, Vert};
use pitch::{parse_kern_pitch, Interval};

const KERN: &str = "corpus/bach-wtc-fugues/kern";

fn main() {
  let what = std::env::args().nth(1).unwrap_or_else(|| "all".into());
  match what.as_str() {
    "states" => states(),
    "verdict" => verdict(),
    "corpus" => corpus_run(),
    "diag" => diag(),
    "stretto" => stretto(),
    "rank" => rank(),
    "probe" => probe(),
    "exp" => { exp_density(); exp_pareto(); exp_renaissance(); exp_chromatic(); exp_harmony(); }
    "exp1" => exp_density(),
    "exp2" => exp_pareto(),
    "exp3" => exp_renaissance(),
    "exp4" => exp_chromatic(),
    "exp5" => exp_harmony(),
    "design" => step4_design(),
    "harmony" => { harmony_run(); harmony_design(); harmony_corpus(); }
    "h1" => harmony_run(),
    "h2" => harmony_design(),
    "h3" => harmony_corpus(),
    "cad" => cadence_check(),
    "hren" => harmony_renaissance(),
    "seg" => segmentation_sensitivity(),
    "revisit" => step4_revisit(),
    "sweep" => analyser_sweep(),
    "holdout" => analyser_holdout(),
    "hren2" => analyser_renaissance(),
    "func" => functional_test(),
    "realise" => step5::run(),
    "r1" => step5::render_stretto(),
    "r2" => step5::reconstruct(),
    "r3" => step5::scalarisations(),
    "gen" => step5::generality(),
    "species" => step5::species(),
    "shape" => step5::shape_test(),
    "plan" => step5::plan_test(),
    "s17" => { analyser_sweep(); analyser_holdout(); analyser_renaissance(); }
    "s16" => { cadence_check(); harmony_renaissance(); segmentation_sensitivity(); step4_revisit(); }
    _ => {
      states();
      verdict();
      corpus_run();
      stretto();
    }
  }
}

fn states() {
  let (reachable, crude) = automaton::reachable_states();
  println!("\n== reachable states ==");
  println!("alphabet           {}", automaton::alphabet().len());
  println!("crude product      {crude}   (5 values of `prev` x 256 obligation sets)");
  println!("reachable          {}", reachable.len());
  let owed: std::collections::BTreeSet<u8> = reachable.iter().map(|s| s.owed).collect();
  println!("distinct owings    {}  of 256", owed.len());
  // Three counts, not two, because the middle one is a *measurement* (§8.2) and
  // reporting only "hard" and "soft" invites the reader to reconcile a rulebook
  // of 11 with a hard tier of 2 and get neither.
  println!(
    "rules transcribed  {}   ({} written as hard, {} as soft)",
    automaton::HARD.len() + automaton::SOFT.len(),
    automaton::HARD.len(),
    automaton::SOFT.len()
  );
  println!("confirmed hard     {}   by both corpora, per readme §8.2", automaton::CONFIRMED.len());
}

/// A tiny score written by hand, as `(lower, upper)` pairs of kern tokens with
/// a tie flag on each, so the verdict tests read like the music they describe.
fn walk(events: &[(&str, &str, bool, bool)], downbeats: &[bool]) -> Vec<Rule> {
  let mut st = State::default();
  let mut prev: Option<(pitch::Pitch, pitch::Pitch)> = None;
  let mut all = vec![];
  for (i, &(lo, hi, lo_tied, hi_tied)) in events.iter().enumerate() {
    let (lp, hp) = (parse_kern_pitch(lo).unwrap(), parse_kern_pitch(hi).unwrap());
    let sym = Sym {
      vert: Vert::of(Interval::between(lp, hp)),
      lo: Move::of(prev.map(|p: (pitch::Pitch, _)| p.0), lp),
      hi: Move::of(prev.map(|p| p.1), hp),
      lo_tied,
      hi_tied,
      downbeat: downbeats.get(i).copied().unwrap_or(true),
      crossed: false,
    };
    let (fired, next) = automaton::step(st, sym);
    all.extend(fired);
    st = next;
    prev = Some((lp, hp));
  }
  all
}

fn verdict() {
  println!("\n== verdict tests ==");
  println!("(ricercar §7.2 had to substitute its own test because the roughness");
  println!(" field could not do the first of these)\n");

  // 1. Parallel fifths, the canonical forbidden edge.
  let par = walk(&[("c", "g", false, false), ("d", "a", false, false)], &[true, true]);
  let flagged = par.contains(&Rule::ParallelPerfect);
  println!("1. parallel fifths flagged                    {}", yes(flagged));

  // 2. A bare fifth is a consonance. The roughness field rated it 0.089 —
  //    among the *least* rough intervals there are — and so could never
  //    flag the parallel above.
  let fifth = Interval::between(parse_kern_pitch("c").unwrap(), parse_kern_pitch("g").unwrap());
  let consonant = fifth.quality().is_consonant();
  println!("2. a bare fifth is consonant                  {}", yes(consonant));

  // 3. The same dissonance, prepared and unprepared. Both sound a seventh on
  //    the downbeat; only the second is a suspension. This is the distinction
  //    readme §8 calls the device most of the repertoire is built from.
  let suspension = walk(
    &[
      ("c", "cc", false, false),  // preparation: C4/C5, an octave, consonant
      ("d", "cc", false, true),   // bass steps up, C5 held: a 7th, prepared
      ("d", "b", false, false),   // upper resolves down by step to a 6th
    ],
    &[false, true, false],
  );
  let accident = walk(
    &[
      ("c", "a", false, false),   // C4/A4, a 6th
      ("d", "cc", false, false),  // upper LEAPS into the same 7th, unprepared
      ("d", "b", false, false),
    ],
    &[false, true, false],
  );
  let s_bad = suspension.iter().any(|r| r.is_hard());
  let a_bad = accident.contains(&Rule::UnpreparedDissonance);
  println!("3. suspension accepted                        {}", yes(!s_bad));
  println!("   same interval, struck, rejected            {}", yes(a_bad));
  println!("   -> distinguished                           {}", yes(!s_bad && a_bad));
}

fn yes(b: bool) -> &'static str {
  if b { "PASS" } else { "FAIL" }
}

/// Which melodic intervals the checker is objecting to. Written because 39
/// flagged intervals per 1000 slices is not a plausible thing to believe about
/// Bach, and the question is which interval the rule has wrong.
fn diag() {
  println!("
== flagged melodic intervals ==");
  let dir = std::path::Path::new(KERN);
  let mut files: Vec<_> = std::fs::read_dir(dir).expect("kern")
    .filter_map(|e| e.ok().map(|e| e.path()))
    .filter(|p| p.file_name().map(|n| n.to_string_lossy().starts_with("wtc1f")).unwrap_or(false))
    .collect();
  files.sort();
  let mut total = corpus::Tally::default();
  for f in &files {
    if let Ok(p) = kern::read(f) { total.merge(&corpus::check_piece(&p)); }
  }
  let mut v: Vec<_> = total.melodic.iter().collect();
  v.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
  println!("  diatonic  semitones   count   name");
  for ((st, se), n) in v.iter().take(12) {
    println!("  {st:>8} {se:>10} {n:>7}   {}", name_interval(*st, *se));
  }
}

pub fn name_interval(st: i16, se: i16) -> &'static str {
  match (st.abs(), se.abs()) {
    (1, 3) => "augmented second",
    (2, 2) => "diminished third",
    (3, 6) => "augmented fourth (tritone)",
    (4, 6) => "diminished fifth (tritone)",
    (4, 8) => "augmented fifth",
    (5, 7) => "diminished sixth",
    (6, _) => "seventh",
    (7, 11) => "diminished octave",
    (2, 5) => "augmented third",
    (0, 1) => "augmented unison (chromatic)",
    _ => "other",
  }
}

fn corpus_run() {
  println!("\n== Bach against the rulebook ==");
  let dir = std::path::Path::new(KERN);
  if !dir.exists() {
    println!("{KERN} not found - run `git submodule update --init`");
    return;
  }
  let mut files: Vec<_> = std::fs::read_dir(dir)
    .expect("read kern dir")
    .filter_map(|e| e.ok().map(|e| e.path()))
    .filter(|p| {
      p.file_name().map(|n| n.to_string_lossy().starts_with("wtc1f")).unwrap_or(false)
    })
    .collect();
  files.sort();

  let mut total = corpus::Tally::default();
  let mut poly = 0usize;
  println!("\n  fugue  vv  slices   hard   per 1k");
  for f in &files {
    match kern::read(f) {
      Ok(p) => {
        let t = corpus::check_piece(&p);
        poly += p.polyphonic_instants;
        let rate = if t.slices > 0 { 1000.0 * t.hard_total() as f64 / t.slices as f64 } else { 0.0 };
        println!(
          "  {:<7} {:>2} {:>7} {:>6} {:>8.1}",
          p.id.trim_start_matches("wtc1f"),
          p.voices.len(),
          t.slices,
          t.hard_total(),
          rate
        );
        total.merge(&t);
      }
      Err(e) => println!("  {}: {e}", f.display()),
    }
  }

  println!("\n  {} fugues, {} voice pairs, {} slices", files.len(), total.pairs, total.slices);
  if poly > 0 {
    println!("  {poly} instants where a voice sounded >1 pitch (split spines, not interpreted)");
  }
  println!("  {} note-to-note moves within voices", total.melodic_moves);
  // Melody is counted per voice and everything else per slice, so the two have
  // different denominators and saying so is the difference between a rate and
  // a number that looks like one.
  println!("\n  rule                             count      per 1k");
  println!("                                          (slices, or moves for melody)");
  for tier in [automaton::HARD, automaton::SOFT] {
    for r in tier {
      let n = total.by_rule.get(r.name()).copied().unwrap_or(0);
      // melody has its own denominator: moves within a voice, not slices
      let den = if *r == Rule::ForbiddenMelodic { total.melodic_moves } else { total.slices };
      let rate = 1000.0 * n as f64 / den.max(1) as f64;
      println!(
        "  {:<28} {} {:>7}  {:>8.1}",
        r.name(),
        if r.is_hard() { "H" } else { "s" },
        n,
        rate
      );
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parallel_fifths_are_flagged() {
    let v = walk(&[("c", "g", false, false), ("d", "a", false, false)], &[true, true]);
    assert!(v.contains(&Rule::ParallelPerfect));
  }

  #[test]
  fn contrary_motion_between_fifths_is_not_parallel() {
    // Both sound a fifth, but the voices move in opposite directions.
    let v = walk(&[("c", "g", false, false), ("B", "ff", false, false)], &[true, true]);
    assert!(!v.contains(&Rule::ParallelPerfect));
  }

  #[test]
  fn a_prepared_suspension_outranks_the_same_struck_interval() {
    let prepared = walk(
      &[("c", "cc", false, false), ("d", "cc", false, true), ("d", "b", false, false)],
      &[false, true, false],
    );
    let struck = walk(
      &[("c", "a", false, false), ("d", "cc", false, false), ("d", "b", false, false)],
      &[false, true, false],
    );
    assert!(!prepared.iter().any(|r| r.is_hard()), "suspension rejected: {prepared:?}");
    assert!(struck.contains(&Rule::UnpreparedDissonance));
  }

  #[test]
  fn the_state_space_is_small_and_finite() {
    let (reachable, crude) = automaton::reachable_states();
    assert!(reachable.len() < crude, "reachability bought nothing");
    assert!(reachable.len() > 1);
  }
}

// ---------------------------------------------------------------- step 2 ---

/// BWV 867's five final entries, in quarters from the start of the fugue,
/// straight from the algomus ground truth (`22-bwv867-ref.dez`). One per voice.
const STRETTO_Q: [i64; 5] = [266, 268, 270, 272, 274];
/// The subject's two contested lengths, in quarters — readme §3.3. Keller and
/// Bruhn read three measures ("female ending"), Prout and Bruhn two ("male").
const LEN_FEMALE_Q: i64 = 12;
const LEN_MALE_Q: i64 = 8;

fn stretto() {
  let path = std::path::Path::new(KERN).join("wtc1f22.krn");
  let p = match kern::read(&path) {
    Ok(p) => p,
    Err(e) => return println!("{e}"),
  };
  let q = kern::TICKS_PER_QUARTER;
  println!("
== step 2: BWV 867, the clique test ==");
  println!("{} voices, measure {} ticks, entries at quarters {:?}", p.voices.len(), p.measure, STRETTO_Q);

  for (label, len_q) in
    [("female (Keller, Bruhn), 3 measures", LEN_FEMALE_Q), ("male (Prout, Bruhn), 2 measures", LEN_MALE_Q)]
  {
    let top = p.voices.len() - 1;
    let sub = stretto::Subject::cut(&p.voices[top], 0, len_q * q);
    println!("
-- subject read as {label} --");
    print!("   ");
    for n in &sub.notes {
      print!("{} ", n.pitch.name());
    }
    println!("({} notes)", sub.notes.len());

    let mut entries = vec![];
    for (i, &sq) in STRETTO_Q.iter().enumerate() {
      let t = sq * q;
      let v = p.voices.len() - 1 - i;
      let Some(n) = p.voices[v].notes.iter().find(|n| n.onset >= t) else { continue };
      let (ds, dc) = stretto::interval_from(&sub, n.pitch);
      entries.push(stretto::Entry { d: t - STRETTO_Q[0] * q, dsteps: ds, dsemis: dc });
      println!("   +{:>2}q  voice {}  {}  ({:+} steps)", (t - STRETTO_Q[0] * q) / q, v, n.pitch.name(), ds);
    }

    let ix: Vec<usize> = (0..entries.len()).collect();
    for (tier_name, tier) in
      [("full hard tier (5 rules)", automaton::HARD), ("confirmed tier (2 rules, §8.2)", automaton::CONFIRMED)]
    {
      let table = stretto::build(&sub, &entries, p.measure, tier);
      let ok = table.is_clique(&ix);
      println!("
   {tier_name:<32} clique: {}   max {} of {}",
        yes(ok), table.max_clique(p.voices.len()).len(), entries.len());
      if !ok {
        for i in 0..entries.len() {
          for j in (i + 1)..entries.len() {
            if !table.ok[i][j] {
              let a = sub.place(entries[i].d, entries[i].dsteps, entries[i].dsemis);
              let b = sub.place(entries[j].d, entries[j].dsteps, entries[j].dsemis);
              let v = stretto::compatible(&a, &b, p.measure, tier);
              let why: Vec<String> =
                v.worst.iter().map(|(n, c)| format!("{n} x{c}")).collect();
              println!("      {} vs {}  {}", entries[i].label(), entries[j].label(), why.join(", "));
            }
          }
        }
      }
    }
  }

  // The control. The table above places an idealised *template*; Bach wrote
  // actual notes. If the template fails where the real passage passes, the
  // fault is in the model of an entry, not in the rulebook.
  println!("
-- control: Bach's actual notes, measures 67.5-71 --");
  let (t0, t1) = (STRETTO_Q[0] * q, (STRETTO_Q[4] + LEN_FEMALE_Q) * q);
  let mut worst_full = 0usize;
  let mut worst_conf = 0usize;
  let mut named: Vec<String> = vec![];
  for a in 0..p.voices.len() {
    for b in (a + 1)..p.voices.len() {
      let (va, vb) = (window(&p.voices[a], t0, t1), window(&p.voices[b], t0, t1));
      worst_full += stretto::compatible(&va, &vb, p.measure, automaton::HARD).hard;
      let c = stretto::compatible(&va, &vb, p.measure, automaton::CONFIRMED);
      worst_conf += c.hard;
      for (n, k) in &c.worst {
        if *k > 0 {
          named.push(format!("voices {a}-{b}: {n} x{k}"));
        }
      }
    }
  }
  println!("   10 real voice pairs: {worst_full} violations on the full tier, {worst_conf} on the confirmed tier");
  for n in &named {
    println!("      {n}");
  }
}

fn window(v: &kern::Voice, t0: i64, t1: i64) -> kern::Voice {
  kern::Voice {
    notes: v.notes.iter().filter(|n| n.onset >= t0 && n.onset < t1)
      .map(|n| kern::Note { onset: n.onset - t0, ..*n }).collect(),
  }
}

// ---------------------------------------------------------------- step 3 ---

/// Contrapuntal capacity for every Bach subject in the ground truth, ranked.
///
/// Ricercar §6.1 wanted this and was blocked twice — once on an unpinned
/// threshold, once on cost. Here it is a table lookup and a bounded clique
/// search, and the whole corpus runs in seconds.
fn rank() {
  let refpath = std::path::Path::new("corpus/algomus-data/fugues/fugues.ref");
  let dir = std::path::Path::new(KERN);
  // load every piece first: the annotations are in bars, and a bar is only a
  // duration once the score has told us the time signature
  let mut pieces: std::collections::BTreeMap<String, kern::Piece> = Default::default();
  for n in 1..=24 {
    let f = dir.join(format!("wtc1f{n:02}.krn"));
    if let Ok(p) = kern::read(&f) {
      pieces.insert(format!("wtc-i-{n:02}"), p);
    }
  }
  let specs = match refdata::read(refpath, &|id| pieces.get(id).map(|p| p.measure)) {
    Ok(s) => s,
    Err(e) => return println!("{e}"),
  };

  println!("\n== step 3: contrapuntal capacity of 24 Bach subjects ==");
  println!("(diatonic transpositions -7..+7, offsets every quarter within the");
  println!(" subject, one entry per offset, clique anchored at the subject)");
  println!("Judged on the FULL 5-rule tier, not the confirmed 2-rule tier of §8.2:");
  println!("see §8.4 - under the 2-rule tier capacity does not converge at all, so");
  println!("the only tier that measures anything is one Bach himself violates.\n");
  println!("  fugue        vv  subj  notes  cap  tightest stretto");
  let mut rows: Vec<(usize, String, i64, usize, String)> = vec![];

  for spec in &specs {
    let Some(p) = pieces.get(&spec.id) else { continue };
    if spec.len == 0 || spec.entries.is_empty() {
      continue;
    }
    // the first annotated entry, in the voice that states it
    let (letter, start) = spec.entries.iter().min_by_key(|(_, t)| *t).copied().unwrap();
    let vi = voice_of(p, letter);
    let sub = stretto::Subject::cut(&p.voices[vi], start, spec.len);
    if sub.notes.len() < 2 {
      continue;
    }
    let (cap, set, dens) = stretto::capacity(
      &sub, &p.key, p.measure, automaton::HARD, kern::TICKS_PER_QUARTER, 12);
    let mut tight: Vec<i64> = set.iter().map(|(d, _)| *d / kern::TICKS_PER_QUARTER).collect();
    tight.sort_unstable();
    let span = tight.last().copied().unwrap_or(0);
    let _ = span;
    rows.push((
      cap,
      spec.id.clone(),
      spec.len / kern::TICKS_PER_QUARTER,
      sub.notes.len(),
      format!("{dens:.2}  {tight:?}q"),
    ));
  }

  rows.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
  for (cap, id, len, notes, tight) in &rows {
    let vv = pieces.get(id).map(|p| p.voices.len()).unwrap_or(0);
    println!("  {id:<12} {vv:>2} {len:>5}q {notes:>5} {cap:>5}  {tight}");
  }

  // §3.3: the eight contested subjects, at both readings
  println!("\n  -- capacity against the contested endings (§3.3) --");
  println!("  fugue        primary          alternatives");
  for spec in &specs {
    if spec.alternatives.is_empty() {
      continue;
    }
    let Some(p) = pieces.get(&spec.id) else { continue };
    let (letter, start) = spec.entries.iter().min_by_key(|(_, t)| *t).copied().unwrap();
    let vi = voice_of(p, letter);
    let at = |len: i64| {
      let s = stretto::Subject::cut(&p.voices[vi], start, len);
      stretto::capacity(&s, &p.key, p.measure, automaton::HARD,
        kern::TICKS_PER_QUARTER, 12).0
    };
    let alts: Vec<String> = spec
      .alternatives
      .iter()
      .map(|&l| format!("{}q -> {}", l / kern::TICKS_PER_QUARTER, at(l)))
      .collect();
    println!(
      "  {:<12} {:>2}q -> {:<8}  {}",
      spec.id,
      spec.len / kern::TICKS_PER_QUARTER,
      at(spec.len),
      alts.join(", ")
    );
  }
}

/// Ground truth names voices top-down (S A T B C); kern spines run low to high.
fn voice_of(p: &kern::Piece, letter: char) -> usize {
  let order = ['S', 'A', 'T', 'B', 'C'];
  let i = order.iter().position(|&c| c == letter).unwrap_or(0);
  p.voices.len().saturating_sub(1 + i).min(p.voices.len() - 1)
}

/// One fugue, timed, with the graph's edge density — written because the
/// ranking did not finish in ten minutes, and the first question is whether
/// the clique search is slow or the measure is vacuous.
fn probe() {
  let p = kern::read(&std::path::Path::new(KERN).join("wtc1f22.krn")).expect("kern");
  let sub = stretto::Subject::cut(&p.voices[p.voices.len() - 1], 0, 12 * kern::TICKS_PER_QUARTER);
  println!("\n== probe: BWV 867, capacity search ==");
  for (name, tier) in [("confirmed (2 rules)", automaton::CONFIRMED), ("full (5 rules)", automaton::HARD)] {
    for cap in [4usize, 6, 8, 10, 12] {
      let t0 = std::time::Instant::now();
      let (n, _set, dens) =
        stretto::capacity(&sub, &p.key, p.measure, tier, kern::TICKS_PER_QUARTER, cap);
      println!("  {name:<20} cap {cap}: capacity {n}, density {dens:.3}, {:?}", t0.elapsed());
      if t0.elapsed().as_secs() > 30 {
        println!("  (stopping: too slow to continue)");
        break;
      }
    }
  }
}

// ------------------------------------------------------------ experiments ---

fn load_bach() -> std::collections::BTreeMap<String, kern::Piece> {
  let dir = std::path::Path::new(KERN);
  let mut out = std::collections::BTreeMap::new();
  for n in 1..=24 {
    if let Ok(p) = kern::read(&dir.join(format!("wtc1f{n:02}.krn"))) {
      out.insert(format!("wtc-i-{n:02}"), p);
    }
  }
  out
}

fn subjects() -> Vec<(String, kern::Piece, stretto::Subject, Vec<i64>)> {
  let pieces = load_bach();
  let specs = refdata::read(
    std::path::Path::new("corpus/algomus-data/fugues/fugues.ref"),
    &|id| pieces.get(id).map(|p| p.measure),
  )
  .unwrap_or_default();
  let mut out = vec![];
  for spec in specs {
    let Some(p) = pieces.get(&spec.id) else { continue };
    if spec.len == 0 || spec.entries.is_empty() {
      continue;
    }
    let (letter, start) = spec.entries.iter().min_by_key(|(_, t)| *t).copied().unwrap();
    let sub = stretto::Subject::cut(&p.voices[voice_of(p, letter)], start, spec.len);
    if sub.notes.len() >= 2 {
      out.push((spec.id.clone(), p.clone(), sub, spec.alternatives.clone()));
    }
  }
  out
}

/// Experiment 1: does graph *density* discriminate where clique size saturates?
fn exp_density() {
  println!("\n== experiment 1: density instead of clique size ==");
  println!("  fugue        notes/q   dens(2-rule)  dens(5-rule)");
  let subs = subjects();
  let (mut d2, mut d5, mut nq) = (vec![], vec![], vec![]);
  let mut rows = vec![];
  for (id, p, sub, _) in &subs {
    let q = kern::TICKS_PER_QUARTER;
    let (_, _, a) = stretto::capacity(sub, &p.key, p.measure, automaton::CONFIRMED, q, 2);
    let (_, _, b) = stretto::capacity(sub, &p.key, p.measure, automaton::HARD, q, 2);
    let dens = sub.notes.len() as f64 / (sub.len / q).max(1) as f64;
    rows.push((a, id.clone(), dens, b));
    d2.push(a);
    d5.push(b);
    nq.push(dens);
  }
  rows.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
  for (a, id, dens, b) in &rows {
    println!("  {id:<12} {dens:>7.2}   {a:>10.3}   {b:>10.3}");
  }
  let lo = d2.iter().cloned().fold(f64::MAX, f64::min);
  let hi = d2.iter().cloned().fold(0.0, f64::max);
  println!("\n  spread of 2-rule density: {lo:.3} .. {hi:.3}");
  println!("  2-rule density vs note density   r = {:+.3}", experiments::pearson(&nq, &d2));
  println!("  5-rule density vs note density   r = {:+.3}", experiments::pearson(&nq, &d5));
  println!("  2-rule vs 5-rule density         r = {:+.3}", experiments::pearson(&d2, &d5));
}

/// Experiment 2: capacity under the permissive tier, with the soft criteria
/// calibrated against Bach's own stretto.
fn exp_pareto() {
  println!("\n== experiment 2: capacity at Bach's own soft level ==");
  let pieces = load_bach();
  let Some(p867) = pieces.get("wtc-i-22") else { return };
  let q = kern::TICKS_PER_QUARTER;
  let limit = experiments::bach_soft_limit(p867, 266 * q, (274 + 12) * q);
  println!("  Bach's Stretto II, worst of its 10 real pairs, per slice:");
  for (r, v) in automaton::SOFT.iter().zip(&limit) {
    println!("     {:<26} {v:.3}", r.name());
  }
  println!("\n  fugue        cap(2-rule + Pareto)");
  let mut rows = vec![];
  for (id, p, sub, _) in &subjects() {
    let c = experiments::capacity_pareto(sub, &p.key, p.measure, &limit, q, 12);
    rows.push((c, id.clone()));
  }
  rows.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
  for (c, id) in &rows {
    println!("  {id:<12} {c:>6}");
  }
}

/// Experiment 3: the same rulebook on the repertoire Fux is about.
fn exp_renaissance() {
  println!("\n== experiment 3: Fux's rules on 16th-century polyphony ==");
  let mut files: Vec<std::path::PathBuf> = vec![];
  for d in ["Jos", "Oke", "Obr", "Duf", "Bus", "Mar"] {
    let dir = std::path::Path::new("corpus/jrp-scores").join(d);
    if let Ok(rd) = std::fs::read_dir(&dir) {
      files.extend(
        rd.filter_map(|e| e.ok().map(|e| e.path()))
          .filter(|p| p.extension().map(|x| x == "krn").unwrap_or(false)),
      );
    }
  }
  files.sort();
  files.truncate(200);
  if files.is_empty() {
    return println!("  corpus/jrp-scores not found");
  }
  let mut total = corpus::Tally::default();
  let mut ok = 0;
  for f in &files {
    if let Ok(p) = kern::read(f) {
      total.merge(&corpus::check_piece(&p));
      ok += 1;
    }
  }
  println!(
    "  {ok} of {} files parsed, {} slices, {} melodic moves",
    files.len(),
    total.slices,
    total.melodic_moves
  );

  let mut bach = corpus::Tally::default();
  for (_, p) in load_bach() {
    bach.merge(&corpus::check_piece(&p));
  }
  println!("\n  rule                          Renaissance      Bach   (per 1k)");
  for r in automaton::HARD {
    let melodic = *r == automaton::Rule::ForbiddenMelodic;
    let dr = if melodic { total.melodic_moves } else { total.slices };
    let db = if melodic { bach.melodic_moves } else { bach.slices };
    let ren = 1000.0 * total.by_rule.get(r.name()).copied().unwrap_or(0) as f64 / dr.max(1) as f64;
    let bac = 1000.0 * bach.by_rule.get(r.name()).copied().unwrap_or(0) as f64 / db.max(1) as f64;
    println!("  {:<28} {ren:>9.1} {bac:>9.1}", r.name());
  }
}

/// Experiment 4: is the melodic rule objecting to chromaticism?
fn exp_chromatic() {
  println!("\n== experiment 4: chromaticism against the melodic rule ==");
  let (mut chrom, mut mel, mut diss) = (vec![], vec![], vec![]);
  println!("  fugue        chromatic  melodic/1k  dissonance/1k");
  let mut rows = vec![];
  for (id, p) in load_bach() {
    let c = experiments::chromaticism(&p);
    let (rates, _) = experiments::rates(&p);
    let pick = |n: &str| rates.iter().find(|(k, _)| *k == n).map(|(_, v)| *v).unwrap_or(0.0);
    rows.push((c, id, pick("forbidden melodic interval"), pick("unresolved dissonance")));
  }
  rows.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
  for (c, id, m, d) in &rows {
    println!("  {id:<12} {:>8.1}% {m:>11.1} {d:>14.1}", c * 100.0);
    chrom.push(*c);
    mel.push(*m);
    diss.push(*d);
  }
  println!("\n  chromaticism vs melodic rule     r = {:+.3}", experiments::pearson(&chrom, &mel));
  println!("  chromaticism vs dissonance rule  r = {:+.3}", experiments::pearson(&chrom, &diss));
}

/// Experiment 5: does a harmonic constraint bind where the others do not?
fn exp_harmony() {
  println!("\n== experiment 5: harmony as the binding constraint ==");
  println!("  fraction of >=3-note sonorities explained by a triad or 7th chord\n");
  let (mut f, mut t) = (0usize, 0usize);
  for (id, p) in load_bach() {
    let (a, b) = experiments::harmonic_fit(&p.voices);
    if b > 0 {
      println!("  {id:<12} {:>6.1}%  ({a} of {b})", 100.0 * a as f64 / b as f64);
    }
    f += a;
    t += b;
  }
  println!("\n  Bach, all 24 fugues:            {:>6.1}%  ({f} of {t})", 100.0 * f as f64 / t as f64);

  let q = kern::TICKS_PER_QUARTER;
  let (mut cf, mut ct, mut n_sets) = (0usize, 0usize, 0usize);
  for (_, p, sub, _) in &subjects() {
    for d in [1i64, 2, 3, 5] {
      for k in [-4i16, -2, 2, 4] {
        let voices = vec![
          sub.place_diatonic(0, 0, &p.key),
          sub.place_diatonic(d * q, k, &p.key),
          sub.place_diatonic(2 * d * q, 2 * k, &p.key),
        ];
        let (a, b) = experiments::harmonic_fit(&voices);
        cf += a;
        ct += b;
        n_sets += 1;
      }
    }
  }
  println!(
    "  arbitrary 3-entry strettos:     {:>6.1}%  ({cf} of {ct}, {n_sets} placements)",
    100.0 * cf as f64 / ct.max(1) as f64
  );
}

// ------------------------------------------------------------ step 4 -------

/// Deterministic PRNG, so a reported contour can be reproduced exactly.
struct Rng(u64);
impl Rng {
  fn next(&mut self) -> u64 {
    self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = self.0;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
  }
  fn below(&mut self, n: usize) -> usize {
    (self.next() % n as u64) as usize
  }
}

/// Rewrite a subject's pitches from a contour of scale degrees, keeping its
/// rhythm. The head is fixed: it is what the ear recognises on re-entry, which
/// is readme §3.2's weighting intuition surviving as a search order.
fn from_contour(base: &stretto::Subject, contour: &[i16], key: &[i8; 7]) -> stretto::Subject {
  let head = base.notes[0].pitch;
  let notes = base
    .notes
    .iter()
    .zip(contour)
    .map(|(n, &deg)| {
      let step = head.step + deg;
      let letter = step.rem_euclid(7) as usize;
      kern::Note { pitch: pitch::Pitch::new(step, key[letter]), ..*n }
    })
    .collect();
  stretto::Subject { notes, len: base.len }
}

fn contour_of(sub: &stretto::Subject) -> Vec<i16> {
  let head = sub.notes[0].pitch.step;
  sub.notes.iter().map(|n| n.pitch.step - head).collect()
}

/// How much melody a contour actually has — the diagnostic that decides whether
/// an optimum is musical or degenerate.
fn liveliness(c: &[i16]) -> (usize, usize, f64) {
  let distinct: std::collections::BTreeSet<i16> = c.iter().copied().collect();
  let holds = c.windows(2).filter(|w| w[0] == w[1]).count();
  let motion = c.windows(2).map(|w| (w[1] - w[0]).abs() as f64).sum::<f64>()
    / (c.len() - 1).max(1) as f64;
  (distinct.len(), holds, motion)
}

fn step4_design() {
  println!("\n== step 4: designing a subject against the §8.4 measure ==");
  let pieces = load_bach();
  let Some(p) = pieces.get("wtc-i-22") else { return };
  let q = kern::TICKS_PER_QUARTER;
  let base = stretto::Subject::cut(&p.voices[p.voices.len() - 1], 0, 12 * q);
  let n = base.notes.len();
  let score = |c: &[i16]| {
    stretto::density(&from_contour(&base, c, &p.key), &p.key, p.measure, automaton::CONFIRMED, 2 * q)
  };

  let bach = contour_of(&base);
  let (bd, bh, bm) = liveliness(&bach);
  println!(
    "  Bach's own subject      density {:.3}   {bd} distinct degrees, {bh} repeats, mean step {bm:.2}",
    score(&bach)
  );

  // a random control
  let mut rng = Rng(0x5EED);
  let trials = 400;
  let mut samples = Vec::with_capacity(trials);
  for _ in 0..trials {
    let mut c = vec![0i16; n];
    for x in c.iter_mut().skip(1) {
      *x = rng.below(15) as i16 - 7;
    }
    samples.push(score(&c));
  }
  let mean: f64 = samples.iter().sum::<f64>() / trials as f64;
  let sd = (samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / trials as f64).sqrt();
  println!("  random contours ({trials})   density {mean:.3} +- {sd:.3}");
  println!("  -> Bach sits {:+.2} standard deviations above random",
    (score(&bach) - mean) / sd.max(1e-9));

  // hill-climb with restarts, head fixed
  let mut best = (score(&bach), bach.clone());
  for restart in 0..12 {
    let mut c = vec![0i16; n];
    if restart > 0 {
      for x in c.iter_mut().skip(1) {
        *x = rng.below(15) as i16 - 7;
      }
    }
    let mut cur = score(&c);
    loop {
      let mut improved = false;
      for i in 1..n {
        let old = c[i];
        for d in -7i16..=7 {
          if d == old {
            continue;
          }
          c[i] = d;
          let s = score(&c);
          if s > cur + 1e-9 {
            cur = s;
            improved = true;
          } else {
            c[i] = old;
          }
        }
      }
      if !improved {
        break;
      }
    }
    if cur > best.0 {
      best = (cur, c.clone());
    }
  }

  let (d, h, m) = liveliness(&best.1);
  println!("\n  best contour found      density {:.3}", best.0);
  println!("  degrees                 {:?}", best.1);
  println!("  {d} distinct degrees, {h} repeated-note transitions, mean step {m:.2}");
  let sub = from_contour(&base, &best.1, &p.key);
  print!("  as pitches              ");
  for note in &sub.notes {
    print!("{} ", note.pitch.name());
  }
  println!();

  // Does constraining the search to musical contours rescue it? Require at
  // least five distinct scale degrees and no run of three identical notes.
  let musical = |c: &[i16]| {
    let (d, _, _) = liveliness(c);
    d >= 5 && !c.windows(3).any(|w| w[0] == w[1] && w[1] == w[2])
  };
  let mut cbest = (0.0f64, bach.clone());
  for restart in 0..16 {
    let mut c = bach.clone();
    if restart > 0 {
      for x in c.iter_mut().skip(1) {
        *x = rng.below(15) as i16 - 7;
      }
    }
    if !musical(&c) {
      continue;
    }
    let mut cur = score(&c);
    loop {
      let mut improved = false;
      for i in 1..n {
        let old = c[i];
        for d in -7i16..=7 {
          if d == old {
            continue;
          }
          c[i] = d;
          let v = score(&c);
          if musical(&c) && v > cur + 1e-9 {
            cur = v;
            improved = true;
          } else {
            c[i] = old;
          }
        }
      }
      if !improved {
        break;
      }
    }
    if cur > cbest.0 {
      cbest = (cur, c.clone());
    }
  }
  let (cd, ch, cm) = liveliness(&cbest.1);
  println!("\n  constrained to >=5 degrees and no triple repeat:");
  println!("  best density {:.3}   {cd} distinct, {ch} repeats, mean step {cm:.2}", cbest.0);
  println!("  degrees      {:?}", cbest.1);

  println!("\n  -- the same optimisation over all 24 subjects' rhythms --");
  println!("  fugue        Bach   random    best   distinct  repeats");
  let mut wins = 0;
  let mut rows = vec![];
  for (id, pp, s, _) in &subjects() {
    if s.notes.len() < 3 || s.notes.len() > 24 {
      continue;
    }
    let sc = |c: &[i16]| {
      stretto::density(&from_contour(s, c, &pp.key), &pp.key, pp.measure, automaton::CONFIRMED, 2 * q)
    };
    let own = contour_of(s);
    let b = sc(&own);
    let mut c = vec![0i16; s.notes.len()];
    let mut cur = sc(&c);
    loop {
      let mut improved = false;
      for i in 1..s.notes.len() {
        let old = c[i];
        for dd in -7i16..=7 {
          if dd == old {
            continue;
          }
          c[i] = dd;
          let v = sc(&c);
          if v > cur + 1e-9 {
            cur = v;
            improved = true;
          } else {
            c[i] = old;
          }
        }
      }
      if !improved {
        break;
      }
    }
    // The decisive control: the mean density of random contours on this very
    // rhythm. If Bach's own contour scores no better, the measure is blind to
    // contour and its across-subject spread is a fact about rhythm.
    let mut rsum = 0.0;
    for _ in 0..60 {
      let mut rc = vec![0i16; s.notes.len()];
      for x in rc.iter_mut().skip(1) {
        *x = rng.below(15) as i16 - 7;
      }
      rsum += sc(&rc);
    }
    let rmean = rsum / 60.0;
    let (dd, hh, _) = liveliness(&c);
    if cur > b {
      wins += 1;
    }
    rows.push((id.clone(), b, cur, dd, hh, rmean));
  }
  for (id, b, c, dd, hh, rm) in &rows {
    println!("  {id:<12} {b:.3}  {rm:.3}   {c:.3}   {dd:>6}   {hh:>6}");
  }
  let bachs: Vec<f64> = rows.iter().map(|r| r.1).collect();
  let rands: Vec<f64> = rows.iter().map(|r| r.5).collect();
  let better = rows.iter().filter(|r| r.1 > r.5).count();
  println!("
  Bach's contour beats a random one on the same rhythm: {better} of {}", rows.len());
  println!("  Bach density vs random-on-same-rhythm   r = {:+.3}", experiments::pearson(&bachs, &rands));
  let md: f64 = rows.iter().map(|r| r.1 - r.5).sum::<f64>() / rows.len() as f64;
  println!("  mean advantage of Bach's contour        {md:+.4}");
  println!("\n  the optimiser beat Bach on {wins} of {} subjects", rows.len());
  let avg_d: f64 = rows.iter().map(|r| r.3 as f64).sum::<f64>() / rows.len() as f64;
  println!("  mean distinct degrees in an optimised contour: {avg_d:.1}");
}

// ---------------------------------------------------------------- §2.3 -----

/// The decisive test for experiment 5: does accounting for non-chord tones take Bach
/// from 78% to something a hard rule could be built on?
fn harmony_run() {
  println!("\n== §2.3: harmony, with non-chord tones accounted for ==");
  println!("  segmented at the notated beat; every note is a chord tone or a");
  println!("  classified dissonance against the prevailing chord\n");
  println!("  fugue        explained   fit   CT     susp  pass  neigh  app  esc  UNTREATED");
  let mut tot = harmony::Report::default();
  let mut fits = vec![];
  for (id, p) in load_bach() {
    let r = harmony::report_piece(&p);
    println!(
      "  {id:<12} {:>7.1}%  {:>4.2} {:>5}  {:>5} {:>5} {:>6} {:>4} {:>4} {:>10}",
      100.0 * r.explained(), r.mean_fit, r.chord_tones,
      r.suspension, r.passing, r.neighbour, r.appoggiatura, r.escape, r.untreated
    );
    fits.push(r.mean_fit);
    tot.merge(&r);
  }
  println!(
    "\n  Bach, all 24 fugues:      {:>6.1}%   ({} of {} notes)",
    100.0 * tot.explained(), tot.chord_tones + tot.explained_ncts(), tot.total()
  );
  println!("  bare chord membership (experiment 5):  78.0%");
  println!("  chord tones alone:              {:>5.1}%",
    100.0 * tot.chord_tones as f64 / tot.total().max(1) as f64);

  // the control: the same measure on arbitrary strettos
  let q = kern::TICKS_PER_QUARTER;
  let mut ctl = harmony::Report::default();
  for (_, p, sub, _) in &subjects() {
    for d in [1i64, 2, 3, 5] {
      for k in [-4i16, -2, 2, 4] {
        let voices = vec![
          sub.place_diatonic(0, 0, &p.key),
          sub.place_diatonic(d * q, k, &p.key),
          sub.place_diatonic(2 * d * q, 2 * k, &p.key),
        ];
        ctl.merge(&harmony::report(&voices, p.beat));
      }
    }
  }
  println!("\n  arbitrary 3-entry strettos:     {:>5.1}%   ({} untreated of {})",
    100.0 * ctl.explained(), ctl.untreated, ctl.total());
  println!("  -> separation on this measure:  {:.1} points", 100.0 * (tot.explained() - ctl.explained()));

  // The binary measure saturates: widening the rule until Bach passes makes it
  // pass almost anything, which is §8.2's deadlock arriving in the harmonic domain. The
  // graded statistics underneath it are the ones to look at.
  let ctfrac = |r: &harmony::Report| r.chord_tones as f64 / r.total().max(1) as f64;
  let mut cfits = vec![];
  let q2 = kern::TICKS_PER_QUARTER;
  for (_, p, sub, _) in &subjects() {
    for d in [1i64, 2, 3, 5] {
      for k in [-4i16, -2, 2, 4] {
        let voices = vec![
          sub.place_diatonic(0, 0, &p.key),
          sub.place_diatonic(d * q2, k, &p.key),
          sub.place_diatonic(2 * d * q2, 2 * k, &p.key),
        ];
        cfits.push(harmony::report(&voices, p.beat).mean_fit);
      }
    }
  }
  let mf = |v: &Vec<f64>| v.iter().sum::<f64>() / v.len().max(1) as f64;
  println!("\n  graded statistics            Bach    control   separation");
  println!(
    "  chord tones (not NCTs)      {:>5.1}%   {:>5.1}%     {:>5.1} pts",
    100.0 * ctfrac(&tot),
    100.0 * ctfrac(&ctl),
    100.0 * (ctfrac(&tot) - ctfrac(&ctl))
  );
  println!(
    "  mean chord fit              {:>6.3}   {:>6.3}     {:>5.3}",
    mf(&fits),
    mf(&cfits),
    mf(&fits) - mf(&cfits)
  );
  let untr = |r: &harmony::Report| 1000.0 * r.untreated as f64 / r.total().max(1) as f64;
  println!(
    "  untreated per 1000 notes    {:>6.1}   {:>6.1}     {:>5.1}",
    untr(&tot),
    untr(&ctl),
    untr(&ctl) - untr(&tot)
  );
}

/// The test §8.4 asks for: an objective that *rewards* a subject working at
/// the fifth, where the contrapuntal one penalised it.
fn harmony_design() {
  println!("\n== §2.3 as a design objective ==");
  println!("  objective: fraction of notes explained when the subject sounds");
  println!("  against its own answer at the fifth below\n");
  let pieces = load_bach();
  let Some(p) = pieces.get("wtc-i-22") else { return };
  let q = kern::TICKS_PER_QUARTER;
  let base = stretto::Subject::cut(&p.voices[p.voices.len() - 1], 0, 12 * q);
  let n = base.notes.len();

  // the answer enters half way through, a fifth below - the fugal relationship
  let score = |c: &[i16]| {
    let s = from_contour(&base, c, &p.key);
    let voices = vec![s.place_diatonic(0, 0, &p.key), s.place_diatonic(4 * q, -4, &p.key)];
    harmony::report(&voices, p.beat).explained()
  };

  let bach = contour_of(&base);
  println!("  Bach's own subject      {:.3}", score(&bach));

  let mut rng = Rng(0xC0FFEE);
  let trials = 400;
  let mut samples = Vec::with_capacity(trials);
  for _ in 0..trials {
    let mut c = vec![0i16; n];
    for x in c.iter_mut().skip(1) {
      *x = rng.below(15) as i16 - 7;
    }
    samples.push(score(&c));
  }
  let mean: f64 = samples.iter().sum::<f64>() / trials as f64;
  let sd = (samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / trials as f64).sqrt();
  println!("  random contours ({trials})   {mean:.3} +- {sd:.3}");
  println!("  -> Bach sits {:+.2} standard deviations above random", (score(&bach) - mean) / sd.max(1e-9));

  let mut best = (score(&bach), bach.clone());
  for restart in 0..14 {
    let mut c = bach.clone();
    if restart > 0 {
      for x in c.iter_mut().skip(1) {
        *x = rng.below(15) as i16 - 7;
      }
    }
    let mut cur = score(&c);
    loop {
      let mut improved = false;
      for i in 1..n {
        let old = c[i];
        for d in -7i16..=7 {
          if d == old {
            continue;
          }
          c[i] = d;
          let v = score(&c);
          if v > cur + 1e-9 {
            cur = v;
            improved = true;
          } else {
            c[i] = old;
          }
        }
      }
      if !improved {
        break;
      }
    }
    if cur > best.0 {
      best = (cur, c.clone());
    }
  }
  let (d, h, m) = liveliness(&best.1);
  println!("\n  best contour found      {:.3}", best.0);
  println!("  degrees                 {:?}", best.1);
  println!("  {d} distinct degrees, {h} repeated-note transitions, mean step {m:.2}");
  let sub = from_contour(&base, &best.1, &p.key);
  print!("  as pitches              ");
  for note in &sub.notes {
    print!("{} ", note.pitch.name());
  }
  println!();

  // is a monotone still optimal?
  let flat = vec![0i16; n];
  println!("\n  a monotone scores       {:.3}   (it won outright under step 4's first objective)", score(&flat));
}

/// Per-subject: does Bach's contour beat random on the harmonic objective?
fn harmony_corpus() {
  println!("\n== §2.3 objective, all subjects ==");
  println!("  fugue        Bach   random    best   distinct");
  let q = kern::TICKS_PER_QUARTER;
  let mut rng = Rng(0xBEEF);
  let (mut nb, mut rows) = (0usize, vec![]);
  for (id, pp, s, _) in &subjects() {
    if s.notes.len() < 3 || s.notes.len() > 24 {
      continue;
    }
    let sc = |c: &[i16]| {
      let sub = from_contour(s, c, &pp.key);
      let voices = vec![sub.place_diatonic(0, 0, &pp.key), sub.place_diatonic(s.len / 3, -4, &pp.key)];
      harmony::report(&voices, pp.beat).explained()
    };
    let own = contour_of(s);
    let b = sc(&own);
    let mut rsum = 0.0;
    for _ in 0..60 {
      let mut rc = vec![0i16; s.notes.len()];
      for x in rc.iter_mut().skip(1) {
        *x = rng.below(15) as i16 - 7;
      }
      rsum += sc(&rc);
    }
    let rmean = rsum / 60.0;
    let mut c = own.clone();
    let mut cur = b;
    loop {
      let mut improved = false;
      for i in 1..s.notes.len() {
        let old = c[i];
        for dd in -7i16..=7 {
          if dd == old {
            continue;
          }
          c[i] = dd;
          let v = sc(&c);
          if v > cur + 1e-9 {
            cur = v;
            improved = true;
          } else {
            c[i] = old;
          }
        }
      }
      if !improved {
        break;
      }
    }
    let (dd, _, _) = liveliness(&c);
    if b > rmean {
      nb += 1;
    }
    rows.push((id.clone(), b, rmean, cur, dd));
    let _ = q;
  }
  for (id, b, r, c, dd) in &rows {
    println!("  {id:<12} {b:.3}  {r:.3}   {c:.3}   {dd:>6}");
  }
  let bs: Vec<f64> = rows.iter().map(|r| r.1).collect();
  let rs: Vec<f64> = rows.iter().map(|r| r.2).collect();
  let md: f64 = rows.iter().map(|r| r.1 - r.2).sum::<f64>() / rows.len() as f64;
  println!("\n  Bach's contour beats random on the same rhythm: {nb} of {}", rows.len());
  println!("  mean advantage of Bach's contour: {md:+.4}   (the first step-4 run gave -0.0763)");
  println!("  Bach vs random across subjects    r = {:+.3}", experiments::pearson(&bs, &rs));
  let avg: f64 = rows.iter().map(|r| r.4 as f64).sum::<f64>() / rows.len() as f64;
  println!("  mean distinct degrees in an optimised contour: {avg:.1}   (the first step-4 run gave 1.0)");
}

// ----------------------------------------------------- validating the analyser ------

fn specs_with(pieces: &std::collections::BTreeMap<String, kern::Piece>) -> Vec<refdata::SubjectSpec> {
  refdata::read(
    std::path::Path::new("corpus/algomus-data/fugues/fugues.ref"),
    &|id| pieces.get(id).map(|p| p.measure),
  )
  .unwrap_or_default()
}

/// Test 1. The only external check available on the harmonic analyser: does it
/// find the cadences the ground truth annotates?
///
/// Every number in the first harmonic attempt measures whether my chord templates explain notes, not
/// whether the labels are right. An analyser picking plausible-but-wrong chords
/// would produce exactly those figures.
fn cadence_check() {
  println!("
== cadence validation against the ground truth ==");
  println!("  The label names the KEY of the cadence, not only its type: of 106");
  println!("  annotated cadences only 39 are in the home tonic. `III:PAC` is a");
  println!("  perfect cadence in the mediant, and its arrival chord is III.
");
  let pieces = load_bach();
  let specs = specs_with(&pieces);
  println!("  fugue        key    annot  arrival  V->arrival");
  let (mut n_ann, mut n_hit, mut n_vi, mut n_parsed) = (0usize, 0usize, 0usize, 0usize);
  for spec in &specs {
    let Some(p) = pieces.get(&spec.id) else { continue };
    if spec.cadences.is_empty() { continue }
    let Some((tonic, minor)) = p.tonic else { continue };
    let end = p.voices.iter().flat_map(|v| v.notes.iter().map(|n| n.onset + n.dur)).max().unwrap_or(0);
    let segs = harmony::analyse(&p.voices, p.beat, end);
    let (mut hit, mut vi, mut parsed) = (0usize, 0usize, 0usize);
    for (tick, label) in &spec.cadences {
      let Some((deg, kind)) = label.split_once(':') else { continue };
      let Some(off) = roman(deg, minor) else { continue };
      parsed += 1;
      // where the cadence arrives depends on its type: a half cadence stops on
      // the dominant, a deceptive one lands on vi of the local key.
      let local = (tonic as i16 + off).rem_euclid(12) as u8;
      let arrival = match kind {
        "HC" => (local as i16 + 7).rem_euclid(12) as u8,
        "DC" => (local as i16 + 9).rem_euclid(12) as u8,
        _ => local,
      };
      let dom = (local as i16 + 7).rem_euclid(12) as u8;
      // allow the arrival to land in the annotated segment or the next one
      let i = (*tick / p.beat).max(0) as usize;
      let found = (i..=i + 1).any(|j| segs.get(j).and_then(|s| s.chord).map(|c| c.root == arrival).unwrap_or(false));
      if found { hit += 1 }
      let before = i.checked_sub(1).and_then(|j| segs.get(j)).and_then(|s| s.chord);
      if found && before.map(|c| c.root == dom).unwrap_or(false) { vi += 1 }
    }
    n_ann += spec.cadences.len();
    n_parsed += parsed;
    n_hit += hit;
    n_vi += vi;
    println!(
      "  {:<12} {:<6} {:>5} {:>8} {:>11}",
      spec.id,
      format!("{}{}", ["C","C#","D","E-","E","F","F#","G","A-","A","B-","B"][tonic as usize], if minor { "m" } else { "" }),
      parsed, hit, vi
    );
  }
  println!("
  {n_parsed} cadences parsed of {n_ann} annotated");
  println!("  arrival chord correct:               {n_hit}  ({:.0}%)", 100.0 * n_hit as f64 / n_parsed.max(1) as f64);
  println!("  and preceded by its dominant:        {n_vi}  ({:.0}%)", 100.0 * n_vi as f64 / n_parsed.max(1) as f64);

  // The baseline: how often a *randomly chosen* segment would match, which is
  // what the hit rate has to beat to mean anything.
  let (mut t, mut all) = (0usize, 0usize);
  for spec in &specs {
    let Some(p) = pieces.get(&spec.id) else { continue };
    let Some((tonic, minor)) = p.tonic else { continue };
    let end = p.voices.iter().flat_map(|v| v.notes.iter().map(|n| n.onset + n.dur)).max().unwrap_or(0);
    let segs = harmony::analyse(&p.voices, p.beat, end);
    for (_, label) in &spec.cadences {
      let Some((deg, kind)) = label.split_once(':') else { continue };
      let Some(off) = roman(deg, minor) else { continue };
      let local = (tonic as i16 + off).rem_euclid(12) as u8;
      let arrival = match kind {
        "HC" => (local as i16 + 7).rem_euclid(12) as u8,
        "DC" => (local as i16 + 9).rem_euclid(12) as u8,
        _ => local,
      };
      // probability that two consecutive random segments contain that root
      let n = segs.iter().filter(|s| s.chord.map(|c| c.root == arrival).unwrap_or(false)).count();
      t += n * 2;
      all += segs.len();
    }
  }
  println!("  chance rate for the same lookup:     {:.0}%   <- the number to beat", 100.0 * t as f64 / all.max(1) as f64);
}

/// Roman numeral to semitones above the tonic, per mode. Upper case is a major
/// triad on that degree, lower case minor, which for the scale degrees that
/// differ between modes is what disambiguates them.
fn roman(d: &str, minor: bool) -> Option<i16> {
  let up = d.to_ascii_uppercase();
  let maj = [("I", 0), ("II", 2), ("III", 4), ("IV", 5), ("V", 7), ("VI", 9), ("VII", 11)];
  let min = [("I", 0), ("II", 2), ("III", 3), ("IV", 5), ("V", 7), ("VI", 8), ("VII", 10)];
  let table: &[(&str, i16)] = if minor { &min } else { &maj };
  // a case mismatch means the degree is borrowed: III in a major key is the
  // flattened mediant, iii in a minor key the raised one
  let base = table.iter().find(|(k, _)| *k == up).map(|(_, v)| *v)?;
  let other = (if minor { &maj } else { &min }).iter().find(|(k, _)| *k == up).map(|(_, v)| *v)?;
  let is_upper = d.chars().next().map(|c| c.is_uppercase()).unwrap_or(true);
  let expect_upper = matches!(up.as_str(), "I" | "IV" | "V") != minor;
  Some(if is_upper == expect_upper { base } else { other })
}

/// Test 2. A tonal analyser should fit modal polyphony *worse*. If it does not,
/// it is not measuring tonality.
fn harmony_renaissance() {
  println!("\n== the harmonic analyser on modal polyphony ==");
  let mut files: Vec<std::path::PathBuf> = vec![];
  for d in ["Jos", "Oke", "Obr", "Duf", "Bus", "Mar"] {
    if let Ok(rd) = std::fs::read_dir(std::path::Path::new("corpus/jrp-scores").join(d)) {
      files.extend(rd.filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "krn").unwrap_or(false)));
    }
  }
  files.sort();
  files.truncate(200);
  let mut ren = harmony::Report::default();
  let (mut rfit, mut n) = (0.0f64, 0usize);
  for f in &files {
    if let Ok(p) = kern::read(f) {
      let r = harmony::report(&p.voices, p.beat);
      if r.total() > 0 {
        rfit += r.mean_fit;
        n += 1;
        ren.merge(&r);
      }
    }
  }
  let mut bach = harmony::Report::default();
  let (mut bfit, mut bn) = (0.0f64, 0usize);
  for (_, p) in load_bach() {
    let r = harmony::report_piece(&p);
    bfit += r.mean_fit;
    bn += 1;
    bach.merge(&r);
  }
  let ct = |r: &harmony::Report| 100.0 * r.chord_tones as f64 / r.total().max(1) as f64;
  let ex = |r: &harmony::Report| 100.0 * r.explained();
  let un = |r: &harmony::Report| 1000.0 * r.untreated as f64 / r.total().max(1) as f64;
  println!("  {n} Renaissance works, {} notes; 24 Bach fugues, {} notes\n", ren.total(), bach.total());
  println!("  statistic                    Renaissance     Bach   difference");
  println!("  mean chord fit                   {:>7.3}  {:>7.3}   {:>+7.3}", rfit / n.max(1) as f64, bfit / bn.max(1) as f64, rfit / n.max(1) as f64 - bfit / bn.max(1) as f64);
  println!("  chord tones                      {:>6.1}%  {:>6.1}%   {:>+7.1}", ct(&ren), ct(&bach), ct(&ren) - ct(&bach));
  println!("  explained (binary)               {:>6.1}%  {:>6.1}%   {:>+7.1}", ex(&ren), ex(&bach), ex(&ren) - ex(&bach));
  println!("  untreated per 1000 notes         {:>7.1}  {:>7.1}   {:>+7.1}", un(&ren), un(&bach), un(&ren) - un(&bach));
  println!("\n  prediction: a tonal vocabulary should fit modal music WORSE.");
}

/// Test 3. The beat is an unjustified free parameter, and this project has been
/// burned four times by exactly that.
fn segmentation_sensitivity() {
  println!("\n== does the segmentation window change the answer? ==");
  println!("  window            fit    chord tones   explained   untreated/1k");
  for (name, div) in [("half beat", 2i64), ("beat (as used)", 1), ("half measure", -2), ("measure", -1)] {
    let mut tot = harmony::Report::default();
    let (mut fit, mut n) = (0.0, 0usize);
    for (_, p) in load_bach() {
      let w = if div > 0 { p.beat / div } else if div == -2 { p.measure / 2 } else { p.measure };
      if w == 0 { continue }
      let r = harmony::report(&p.voices, w);
      fit += r.mean_fit;
      n += 1;
      tot.merge(&r);
    }
    println!(
      "  {name:<16} {:>5.3}     {:>7.1}%    {:>7.1}%       {:>7.1}",
      fit / n.max(1) as f64,
      100.0 * tot.chord_tones as f64 / tot.total().max(1) as f64,
      100.0 * tot.explained(),
      1000.0 * tot.untreated as f64 / tot.total().max(1) as f64
    );
  }
}

/// A **graded** harmonic objective, replacing the first attempt's binary one which
/// saturated at 1.000 for Bach and for a monotone alike.
fn graded(voices: &[kern::Voice], beat: i64) -> f64 {
  let r = harmony::report(voices, beat);
  let t = r.total().max(1) as f64;
  // chord fit, minus the untreated-dissonance rate that the first attempt found is the one
  // statistic separating Bach from arbitrary placement. Nothing is weighted
  // against anything else: these are on the same scale, both fractions of the
  // same denominator.
  r.mean_fit - r.untreated as f64 / t
}

/// Tests 4 and 5: the graded objective, and step 4 re-run against it.
fn step4_revisit() {
  println!("\n== a graded harmonic objective, and step 4 again ==");
  let pieces = load_bach();
  let Some(p) = pieces.get("wtc-i-22") else { return };
  let q = kern::TICKS_PER_QUARTER;
  let base = stretto::Subject::cut(&p.voices[p.voices.len() - 1], 0, 12 * q);
  let n = base.notes.len();
  let score = |c: &[i16]| {
    let s = from_contour(&base, c, &p.key);
    let voices = vec![s.place_diatonic(0, 0, &p.key), s.place_diatonic(4 * q, -4, &p.key)];
    graded(&voices, p.beat)
  };
  let bach = contour_of(&base);
  let flat = vec![0i16; n];
  println!("  Bach's own subject      {:.4}", score(&bach));
  println!("  a monotone              {:.4}   (both were 1.000 under the binary objective)", score(&flat));

  let mut rng = Rng(0xF00D);
  let trials = 400;
  let mut samples = Vec::with_capacity(trials);
  for _ in 0..trials {
    let mut c = vec![0i16; n];
    for x in c.iter_mut().skip(1) {
      *x = rng.below(15) as i16 - 7;
    }
    samples.push(score(&c));
  }
  let mean: f64 = samples.iter().sum::<f64>() / trials as f64;
  let sd = (samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / trials as f64).sqrt();
  println!("  random contours ({trials})   {mean:.4} +- {sd:.4}");
  println!("  -> Bach sits {:+.2} sd above random   (binary gave +0.91, step 4 gave +0.23)",
    (score(&bach) - mean) / sd.max(1e-9));

  let mut best = (score(&bach), bach.clone());
  for restart in 0..14 {
    let mut c = bach.clone();
    if restart > 0 {
      for x in c.iter_mut().skip(1) {
        *x = rng.below(15) as i16 - 7;
      }
    }
    let mut cur = score(&c);
    loop {
      let mut improved = false;
      for i in 1..n {
        let old = c[i];
        for d in -7i16..=7 {
          if d == old { continue }
          c[i] = d;
          let v = score(&c);
          if v > cur + 1e-12 { cur = v; improved = true } else { c[i] = old }
        }
      }
      if !improved { break }
    }
    if cur > best.0 { best = (cur, c.clone()) }
  }
  let (d, h, m) = liveliness(&best.1);
  println!("\n  best contour found      {:.4}", best.0);
  println!("  degrees                 {:?}", best.1);
  println!("  {d} distinct degrees, {h} repeats, mean step {m:.2}");
  println!("  beats Bach?             {}", if best.0 > score(&bach) + 1e-12 { "yes" } else { "no" });

  println!("\n  -- across all subjects --");
  println!("  fugue        Bach    random    best   distinct");
  let mut rng2 = Rng(0xD00D);
  let (mut nb, mut rows) = (0usize, vec![]);
  for (id, pp, s, _) in &subjects() {
    if s.notes.len() < 3 || s.notes.len() > 24 { continue }
    let sc = |c: &[i16]| {
      let sub = from_contour(s, c, &pp.key);
      let voices = vec![sub.place_diatonic(0, 0, &pp.key), sub.place_diatonic(s.len / 3, -4, &pp.key)];
      graded(&voices, pp.beat)
    };
    let own = contour_of(s);
    let b = sc(&own);
    let mut rsum = 0.0;
    for _ in 0..60 {
      let mut rc = vec![0i16; s.notes.len()];
      for x in rc.iter_mut().skip(1) { *x = rng2.below(15) as i16 - 7 }
      rsum += sc(&rc);
    }
    let rmean = rsum / 60.0;
    let mut c = own.clone();
    let mut cur = b;
    loop {
      let mut improved = false;
      for i in 1..s.notes.len() {
        let old = c[i];
        for dd in -7i16..=7 {
          if dd == old { continue }
          c[i] = dd;
          let v = sc(&c);
          if v > cur + 1e-12 { cur = v; improved = true } else { c[i] = old }
        }
      }
      if !improved { break }
    }
    let (dd, _, _) = liveliness(&c);
    if b > rmean { nb += 1 }
    rows.push((id.clone(), b, rmean, cur, dd));
  }
  for (id, b, r, c, dd) in &rows {
    println!("  {id:<12} {b:.4}  {r:.4}  {c:.4}   {dd:>6}");
  }
  let md: f64 = rows.iter().map(|r| r.1 - r.2).sum::<f64>() / rows.len() as f64;
  let beat_bach = rows.iter().filter(|r| r.3 > r.1 + 1e-12).count();
  let avg: f64 = rows.iter().map(|r| r.4 as f64).sum::<f64>() / rows.len() as f64;
  println!("\n  Bach beats random on the same rhythm: {nb} of {}   (step 4: 5/20, graded: 17/20)", rows.len());
  println!("  mean advantage of Bach's contour:     {md:+.4}   (step 4: -0.0763, graded: +0.0552)");
  println!("  optimiser beats Bach on:              {beat_bach} of {}", rows.len());
  println!("  mean distinct degrees in an optimum:  {avg:.1}   (step 4: 1.0, graded: 6.8)");
}

// ------------------------------------------------------- the analyser rebuilt ------

/// Cadence accuracy of the Viterbi analyser, swept over the change penalty.
///
/// The penalty is not fitted. Fitting one scalar to the corpus would be exactly
/// the thing this project was written to avoid, and testing on the data it was
/// fitted to would be worse. The whole curve is reported instead, so a reader
/// can see how much of the result is the parameter — which is the complaint
/// the validation pass made about the window it replaces.
fn analyser_sweep() {
  println!("\n== the Viterbi analyser, swept over the change penalty ==");
  let pieces = load_bach();
  let specs = specs_with(&pieces);
  println!("  lambda   arrival  +dominant   chance   harm.rhythm   chord tones");
  for &lambda in &[0.0f64, 0.05, 0.1, 0.2, 0.3, 0.5, 0.75, 1.0, 1.5, 2.0] {
    let (mut hit, mut vi, mut tot) = (0usize, 0usize, 0usize);
    let (mut chance_n, mut chance_d) = (0usize, 0usize);
    let (mut hr, mut nhr) = (0.0f64, 0usize);
    let mut rep = harmony::Report::default();
    for spec in &specs {
      let Some(p) = pieces.get(&spec.id) else { continue };
      let Some((tonic, minor)) = p.tonic else { continue };
      let segs = harmony::analyse_viterbi(&p.voices, p.beat, lambda);
      if segs.is_empty() {
        continue;
      }
      hr += harmony::harmonic_rhythm(&segs);
      nhr += 1;
      rep.merge(&harmony::report_viterbi(&p.voices, p.beat, lambda));
      for (tick, label) in &spec.cadences {
        let Some((deg, kind)) = label.split_once(':') else { continue };
        let Some(off) = roman(deg, minor) else { continue };
        tot += 1;
        let local = (tonic as i16 + off).rem_euclid(12) as u8;
        let arrival = match kind {
          "HC" => (local as i16 + 7).rem_euclid(12) as u8,
          "DC" => (local as i16 + 9).rem_euclid(12) as u8,
          _ => local,
        };
        let dom = (local as i16 + 7).rem_euclid(12) as u8;
        let at = segs.iter().position(|s| s.start <= *tick && *tick < s.end);
        let found = at
          .and_then(|i| segs.get(i))
          .and_then(|s| s.chord)
          .map(|c| c.root == arrival)
          .unwrap_or(false);
        if found {
          hit += 1;
          if let Some(i) = at {
            let here = segs[i].chord;
            let prev = segs[..i].iter().rev().find(|s| s.chord != here).and_then(|s| s.chord);
            if prev.map(|c| c.root == dom).unwrap_or(false) {
              vi += 1;
            }
          }
        }
        let n = segs.iter().filter(|s| s.chord.map(|c| c.root == arrival).unwrap_or(false)).count();
        chance_n += n;
        chance_d += segs.len();
      }
    }
    println!(
      "  {lambda:>6.2}   {:>6.0}%  {:>8.0}%  {:>6.0}%   {:>10.0}t   {:>10.1}%",
      100.0 * hit as f64 / tot.max(1) as f64,
      100.0 * vi as f64 / tot.max(1) as f64,
      100.0 * chance_n as f64 / chance_d.max(1) as f64,
      hr / nhr.max(1) as f64,
      100.0 * rep.chord_tones as f64 / rep.total().max(1) as f64
    );
  }
  println!("\n  (fixed-window analyser: 38% arrival, 18% +dominant, 23% chance)");
  println!("  a quarter note is {} ticks; the beat varies by piece", kern::TICKS_PER_QUARTER);
}

/// Held-out check: does the best lambda on one half of the corpus hold on the
/// other? The sweep reports a curve; this asks whether picking a point on it
/// generalises, which is the only honest way to quote a single number.
fn analyser_holdout() {
  println!("\n== held-out validation ==");
  let pieces = load_bach();
  let specs = specs_with(&pieces);
  let score = |lambda: f64, which: usize| {
    let (mut hit, mut tot) = (0usize, 0usize);
    for (k, spec) in specs.iter().enumerate() {
      if k % 2 != which {
        continue;
      }
      let Some(p) = pieces.get(&spec.id) else { continue };
      let Some((tonic, minor)) = p.tonic else { continue };
      let segs = harmony::analyse_viterbi(&p.voices, p.beat, lambda);
      for (tick, label) in &spec.cadences {
        let Some((deg, kind)) = label.split_once(':') else { continue };
        let Some(off) = roman(deg, minor) else { continue };
        tot += 1;
        let local = (tonic as i16 + off).rem_euclid(12) as u8;
        let arrival = match kind {
          "HC" => (local as i16 + 7).rem_euclid(12) as u8,
          "DC" => (local as i16 + 9).rem_euclid(12) as u8,
          _ => local,
        };
        if segs
          .iter()
          .find(|s| s.start <= *tick && *tick < s.end)
          .and_then(|s| s.chord)
          .map(|c| c.root == arrival)
          .unwrap_or(false)
        {
          hit += 1;
        }
      }
    }
    100.0 * hit as f64 / tot.max(1) as f64
  };
  let grid: Vec<f64> = vec![0.0, 0.05, 0.1, 0.2, 0.3, 0.5, 0.75, 1.0, 1.5, 2.0];
  for (name, fit_on, test_on) in [("odd-numbered", 0usize, 1usize), ("even-numbered", 1, 0)] {
    let mut best = (f64::NAN, -1.0f64);
    for &l in &grid {
      let v = score(l, fit_on);
      if v > best.1 {
        best = (l, v);
      }
    }
    println!(
      "  chosen on {name:<14} lambda = {:.2}  ({:.0}% there)  -> {:.0}% on the other half",
      best.0, best.1, score(best.0, test_on)
    );
  }
}

/// Does the new analyser still prefer modal music? the modal falsification, re-run.
fn analyser_renaissance() {
  println!("\n== modal control, on the new analyser ==");
  let lambda = 0.3;
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
  files.truncate(200);
  let (mut ren, mut rf, mut rn) = (harmony::Report::default(), 0.0f64, 0usize);
  for f in &files {
    if let Ok(p) = kern::read(f) {
      let r = harmony::report_viterbi(&p.voices, p.beat, lambda);
      if r.total() > 0 {
        rf += r.mean_fit;
        rn += 1;
        ren.merge(&r);
      }
    }
  }
  let (mut bach, mut bf, mut bn) = (harmony::Report::default(), 0.0f64, 0usize);
  for (_, p) in load_bach() {
    let r = harmony::report_viterbi(&p.voices, p.beat, lambda);
    bf += r.mean_fit;
    bn += 1;
    bach.merge(&r);
  }
  let ct = |r: &harmony::Report| 100.0 * r.chord_tones as f64 / r.total().max(1) as f64;
  let un = |r: &harmony::Report| 1000.0 * r.untreated as f64 / r.total().max(1) as f64;
  println!("  at lambda = {lambda}\n");
  println!("  statistic                    Renaissance     Bach   difference");
  println!(
    "  mean fit                         {:>7.3}  {:>7.3}   {:>+7.3}",
    rf / rn.max(1) as f64, bf / bn.max(1) as f64, rf / rn.max(1) as f64 - bf / bn.max(1) as f64
  );
  println!("  chord tones                      {:>6.1}%  {:>6.1}%   {:>+7.1}", ct(&ren), ct(&bach), ct(&ren) - ct(&bach));
  println!("  untreated per 1000 notes         {:>7.1}  {:>7.1}   {:>+7.1}", un(&ren), un(&bach), un(&ren) - un(&bach));
  println!("\n  (fixed window: fit +0.061, chord tones +7.0, untreated -3.3 - all wrong-signed)");
}

/// The sharper modal test — functional progression, not chord fit.
fn functional_test() {
  println!("\n== functional progression: the test the modal control wanted ==");
  let lambda = 1.0;
  let (mut bo, mut bt) = (0usize, 0usize);
  for (_, p) in load_bach() {
    let (o, t) = harmony::functional_rate(&harmony::analyse_viterbi(&p.voices, p.beat, lambda));
    bo += o;
    bt += t;
  }
  let mut files: Vec<std::path::PathBuf> = vec![];
  for d in ["Jos", "Oke", "Obr", "Duf", "Bus", "Mar"] {
    if let Ok(rd) = std::fs::read_dir(std::path::Path::new("corpus/jrp-scores").join(d)) {
      files.extend(rd.filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "krn").unwrap_or(false)));
    }
  }
  files.sort();
  files.truncate(200);
  let (mut ro, mut rt) = (0usize, 0usize);
  for f in &files {
    if let Ok(p) = kern::read(f) {
      let (o, t) = harmony::functional_rate(&harmony::analyse_viterbi(&p.voices, p.beat, lambda));
      ro += o;
      rt += t;
    }
  }
  println!("  at lambda = {lambda}, root motion by 4th/5th/2nd/3rd counted as functional\n");
  println!("  Bach        {:>6.1}%   ({bo} of {bt} chord changes)", 100.0 * bo as f64 / bt.max(1) as f64);
  println!("  Renaissance {:>6.1}%   ({ro} of {rt})", 100.0 * ro as f64 / rt.max(1) as f64);
  println!("  difference  {:>+6.1} points", 100.0 * bo as f64 / bt.max(1) as f64 - 100.0 * ro as f64 / rt.max(1) as f64);
  println!("\n  prediction: tonal music should be MORE functional than modal.");
}
