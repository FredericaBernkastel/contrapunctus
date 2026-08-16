//! Roadmap step 1: the two-voice automaton, measured and tested.
//!
//!   cargo run --release -- states    the reachable state count
//!   cargo run --release -- verdict   the three tests the roughness field failed
//!   cargo run --release -- corpus    how often Bach violates the rulebook

mod automaton;
mod corpus;
mod experiments;
mod refdata;
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
  println!("hard rules {}   soft criteria {}", automaton::HARD.len(), automaton::SOFT.len());
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

fn name_interval(st: i16, se: i16) -> &'static str {
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
      [("full hard tier (5 rules)", automaton::HARD), ("confirmed tier (2 rules, §9.4)", automaton::CONFIRMED)]
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
  println!("Judged on the FULL 5-rule tier, not the confirmed 2-rule tier of §9.4:");
  println!("see §11 - under the 2-rule tier capacity does not converge at all, so");
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
  println!("\n== step 4: designing a subject against the §12.1 measure ==");
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
