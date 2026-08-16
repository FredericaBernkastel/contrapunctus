//! Roadmap step 1: the two-voice automaton, measured and tested.
//!
//!   cargo run --release -- states    the reachable state count
//!   cargo run --release -- verdict   the three tests the roughness field failed
//!   cargo run --release -- corpus    how often Bach violates the rulebook

mod automaton;
mod corpus;
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
    _ => {
      states();
      verdict();
      corpus_run();
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
