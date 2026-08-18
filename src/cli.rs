//! The command line: readme §10.2's table as a program, and §10.3's as flags.
//!
//! Every measurement this repository reports has a command, and until now the
//! mapping lived in a `match` on `argv[1]` that answered an unknown argument by
//! silently running something else. Two things follow from that being wrong.
//!
//! **Every command says which section it produces.** `--help` is §10.2, and
//! [`Cmd::section`] is the same string the table uses, so the two cannot drift
//! apart without one of them being obviously wrong.
//!
//! **The parameters are flags rather than constants.** §10.3 is a table of
//! numbers — `λ`, the draw counts, how many windows per fugue, how many works
//! are read — and every one of them was a `const` that had to be edited and
//! recompiled to vary. They are [`Params`] now.
//!
//! **The defaults are exactly the published runs.** That is the contract of this
//! module: `contrapunctus <command>` with no flags reproduces the figures in
//! §8, and any flag given is a departure from the record. Nothing in §8 was
//! produced with a non-default flag.

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::{path::PathBuf, sync::OnceLock};

use crate::automaton::{Rule, CONFIRMED, HARD};

#[derive(Parser, Debug)]
#[command(
  name = "contrapunctus",
  version,
  about = "Counterpoint is a regular language — the measurements behind readme.md §8.",
  long_about = "Counterpoint is a regular language.\n\n\
    Every command reproduces one section of readme.md, and `list` prints the whole\n\
    map. Defaults reproduce the published figures exactly; every flag is a\n\
    departure from the record.\n\n\
    With no command, the four measurements that take seconds are run.",
  disable_help_subcommand = true
)]
pub struct Cli {
  #[command(subcommand)]
  pub cmd: Option<Cmd>,
  #[command(flatten)]
  pub params: Params,
}

/// §10.3's parameter table, settable.
///
/// Every field's default is the value the published run used, so the struct
/// doubles as the machine-readable form of that table. They are `global` so that
/// `contrapunctus --lambda 0.3 sweep` and `contrapunctus sweep --lambda 0.3` are
/// the same command.
#[derive(Args, Debug, Clone)]
pub struct Params {
  /// Chord-change penalty for the harmonic analyser — §8.5's λ, swept there and
  /// left at the middle of the plausible band everywhere else.
  #[arg(long, global = true, default_value_t = 1.0, value_name = "COST")]
  pub lambda: f64,

  /// Uniform draws per span where a section reports a sampled row.
  #[arg(long, global = true, default_value_t = 8, value_name = "N")]
  pub samples: usize,

  /// Draws per span for §8.8, which ranks them rather than averaging them and so
  /// wants more of them.
  #[arg(long, global = true, default_value_t = 32, value_name = "N")]
  pub rerank: usize,

  /// Windows taken per Bach fugue from §8.8 onwards. Denser than the
  /// Renaissance's to equalise the power of the paired comparison, since 24
  /// fugues have to stand against 200 works.
  #[arg(long, global = true, default_value_t = 30, value_name = "N")]
  pub bach_windows: usize,

  /// Windows per Bach fugue for §8.6's treatise-weighting table alone, which was
  /// run before that equalisation and reports 67 spans rather than 690. It keeps
  /// its own number so that the default still reproduces the published row.
  #[arg(long, global = true, default_value_t = 3, value_name = "N")]
  pub gen_windows: usize,

  /// Windows taken per 15th-century work.
  #[arg(long, global = true, default_value_t = 3, value_name = "N")]
  pub ren_windows: usize,

  /// 15th-century works read. §10.4 records which they turn out to be.
  #[arg(long, global = true, default_value_t = 200, value_name = "N")]
  pub ren_works: usize,

  /// Inverse temperature for the sampler. §8.6 measured this as
  /// repertoire-specific and every published run passes zero.
  #[arg(long, global = true, default_value_t = 0.0, value_name = "β")]
  pub beta: f64,

  /// Base seed. Every span's seed is this mixed with its start tick, so a run is
  /// reproducible and two spans are not correlated.
  #[arg(long, global = true, default_value_t = 0x5EED, value_name = "N")]
  pub seed: u64,

  /// Which transcribed rules the realiser treats as hard, where a section uses
  /// one tier rather than crossing all three.
  #[arg(long, global = true, value_enum, default_value_t = TierArg::ConfMelodic)]
  pub tier: TierArg,

  /// Where MIDI is written. Not tracked by git.
  #[arg(long, global = true, default_value = "out", value_name = "DIR")]
  pub out: PathBuf,

  /// The Bach `**kern` directory.
  #[arg(long, global = true, default_value = "corpus/bach-wtc-fugues/kern", value_name = "DIR")]
  pub kern: PathBuf,

  /// The 15th-century `**kern` directory.
  #[arg(long, global = true, default_value = "corpus/jrp-scores", value_name = "DIR")]
  pub jrp: PathBuf,
}

/// The three tiers §8.6 crosses, as a flag.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TierArg {
  /// The two rules §8.2 found universal — this document's endorsed tier.
  Confirmed,
  /// Those two plus the melodic prohibition, which is what §8.6 onwards uses.
  ConfMelodic,
  /// All five written hard, dissonance rules included.
  Full,
}

/// The melodic prohibition added to the confirmed pair — §8.6's `conf+melodic`.
pub const CONF_MEL: &[Rule] = &[Rule::ParallelPerfect, Rule::DirectPerfectOnDownbeat, Rule::ForbiddenMelodic];

impl TierArg {
  pub fn rules(self) -> &'static [Rule] {
    match self {
      TierArg::Confirmed => CONFIRMED,
      TierArg::ConfMelodic => CONF_MEL,
      TierArg::Full => HARD,
    }
  }
  pub fn label(self) -> &'static str {
    match self {
      TierArg::Confirmed => "confirmed(2)",
      TierArg::ConfMelodic => "conf+melodic",
      TierArg::Full => "full(5)",
    }
  }
}

impl Default for Params {
  fn default() -> Self {
    // `Parser::parse_from` with no arguments applies every `default_value_t`,
    // so the defaults live in one place rather than two. A second hand-written
    // list here is exactly the kind of thing that drifts.
    Cli::parse_from(["contrapunctus"]).params
  }
}

static PARAMS: OnceLock<Params> = OnceLock::new();

/// The parameters this run was invoked with.
///
/// Set once by `main` before any driver runs. A test or a `cargo run` that never
/// calls [`set_params`] gets the published defaults, which is what makes the
/// drivers callable from a unit test without a command line.
pub fn params() -> &'static Params {
  PARAMS.get_or_init(Params::default)
}

/// Install the parsed parameters. Called once, from `main`, before dispatch.
pub fn set_params(p: Params) {
  let _ = PARAMS.set(p);
}

/// The three blocks `list` prints in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
  /// Produces a figure §8 reports.
  Reported,
  /// Reproduces a superseded figure recorded in `CHANGELOG.md`.
  Superseded,
  /// Runs several of the above.
  Batch,
}

/// One command per measurement, named for what it produces and aliased to the
/// short name §10.2 has always used.
#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmd {
  /// Reachable states of the counterpoint automaton
  States,
  /// The three tests ricercar's roughness field failed
  Verdict,
  /// How often Bach breaks each transcribed rule
  Corpus,
  /// The melodic rule broken down by interval
  Diag,
  /// The same rules on 15th-century polyphony
  #[command(alias = "exp3")]
  Renaissance,
  /// Chromaticism against the melodic rule
  #[command(alias = "exp4")]
  Chromatic,
  /// BWV 867's five entries as a clique
  Stretto,
  /// Contrapuntal capacity of 24 Bach subjects
  Rank,
  /// BWV 867 alone, capacity searched
  Probe,
  /// Density instead of clique size
  #[command(alias = "exp1")]
  Density,
  /// Capacity at Bach's own soft level
  #[command(alias = "exp2")]
  Pareto,
  /// Designing a subject against the §8.4 measure
  Design,
  /// A graded harmonic objective, and step 4 again
  Revisit,
  /// Harmony with non-chord tones accounted for
  #[command(alias = "h1")]
  Ncts,
  /// Harmony as a design objective
  #[command(alias = "h2")]
  HarmonyDesign,
  /// That objective over all subjects
  #[command(alias = "h3")]
  HarmonyCorpus,
  /// The analyser against 106 typed cadence annotations
  #[command(alias = "cad")]
  Cadence,
  /// The first analyser on modal polyphony
  Hren,
  /// Does the segmentation window change the answer?
  Seg,
  /// The Viterbi analyser, swept over the change penalty
  Sweep,
  /// Held-out validation of that sweep
  Holdout,
  /// Modal control, on the rebuilt analyser
  #[command(alias = "hren2")]
  ModalControl,
  /// Functional progression, the test the modal control wanted
  Func,
  /// Harmony as the binding constraint
  #[command(alias = "exp5")]
  BindingHarmony,
  /// The stretto rendered to MIDI
  #[command(alias = "r1")]
  Render,
  /// Reconstructing Bach's free voices
  #[command(alias = "r2")]
  Reconstruct,
  /// What a change of weighting costs, note by note
  #[command(alias = "r3")]
  Scalarisations,
  /// A treatise weighting, measured against two corpora
  #[command(alias = "gen")]
  Generality,
  /// Fux's species as a whitelist, checked before it is used
  Species,
  /// Criteria over a whole line, ranking uniform draws
  Shape,
  /// Harmonic plans, priced paired and on both corpora
  Plan,
  /// The soft tier ablated, and three positive criteria in its place
  Soft,
  /// What the objective is worth, on §8.6's own spans and paired
  #[command(alias = "obj")]
  Objective,
  /// Marpurg's tonal answer, against Bach's own
  Answer,
  /// The fourth, and the scope a dissonance is judged in
  Fourth,
  /// Are episodes sequences?
  Episode,
  /// Key-finding, against the cadence annotations
  Key,
  /// Does the form grammar derive the book?
  Form,
  /// Render, reconstruct and scalarisations together
  Realise,
  /// All five of the tier-deadlock experiments
  Exp,
  /// The three first-attempt harmony runs together
  Harmony,
  /// The superseded step-4 revisit batch
  S16,
  /// The superseded analyser batch
  S17,
  /// The four measurements that take seconds — the default
  All,
  /// Print the section-to-command map that readme §10.2 tabulates
  List,
}

impl Cmd {
  /// The readme section this command produces, or `""` for the batches and for
  /// the runs kept only so a superseded figure stays reproducible.
  ///
  /// This is §10.2's first column and it has to stay equal to it — a test in
  /// `tests/references.rs` reads both and compares them, which is the only way
  /// a table in prose and a `match` in code stay in step.
  pub fn section(self) -> &'static str {
    use Cmd::*;
    match self {
      States | Verdict => "§8.1",
      Corpus | Diag | Renaissance | Chromatic => "§8.2",
      Stretto => "§8.3",
      Density | Design => "§8.4",
      Sweep | Holdout => "§8.5",
      Render | Reconstruct | Scalarisations | Generality => "§8.6",
      Species => "§8.7",
      Shape => "§8.8",
      Plan => "§8.9",
      Soft | Objective => "§8.10",
      Answer => "§8.11",
      Fourth => "§8.12",
      Episode => "§8.13",
      Key => "§8.14",
      Form => "§8.15",
      _ => "",
    }
  }

  /// Which of `list`'s three blocks this belongs to.
  ///
  /// The middle one is not a dumping ground: §10.2 names those commands as the
  /// ones reproducing **superseded** measurements, kept because a figure
  /// recorded in `CHANGELOG.md` that cannot be re-run is a figure nobody can
  /// check.
  pub fn group(self) -> Group {
    use Cmd::*;
    match self {
      Realise | Exp | Harmony | S16 | S17 | All | List => Group::Batch,
      _ if !self.section().is_empty() => Group::Reported,
      _ => Group::Superseded,
    }
  }

  /// Every command, in the order `list` prints them.
  pub fn all() -> &'static [Cmd] {
    use Cmd::*;
    &[
      // reported, in section order
      States, Verdict, Corpus, Diag, Renaissance, Chromatic, Stretto, Density, Design, Sweep,
      Holdout, Render, Reconstruct, Scalarisations, Generality, Species, Shape, Plan, Soft,
      Objective, Answer, Fourth, Episode, Key, Form,
      // superseded, kept reproducible
      Rank, Probe, Pareto, Revisit, Ncts, HarmonyDesign, HarmonyCorpus, Cadence, Hren, Seg,
      ModalControl, Func, BindingHarmony,
      // batches
      Realise, Exp, Harmony, S16, S17, All, List,
    ]
  }

  /// The name as typed. Clap renames variants to kebab-case, and this has to
  /// agree with it or `list` prints commands that do not run.
  pub fn name(self) -> String {
    let d = format!("{self:?}");
    let mut out = String::new();
    for (i, c) in d.chars().enumerate() {
      if c.is_uppercase() && i > 0 {
        out.push('-');
      }
      out.extend(c.to_lowercase());
    }
    out
  }
}

/// `list` — §10.2's table, printed from the same data `--help` is built from.
pub fn list() {
  println!("\n== every command, and the section it produces ==");
  println!("\n  Defaults reproduce the published figures exactly; any flag is a departure from the");
  println!("  record. `cargo run --release -- <command>`, and `--help` for §10.3's parameters.");
  for (g, title, note) in [
    (Group::Reported, "reported in §8", "each produces the section named beside it"),
    (Group::Superseded, "superseded", "kept runnable so CHANGELOG.md's figures can be checked"),
    (Group::Batch, "batches", "several of the above, in one run"),
  ] {
    println!("\n  -- {title} --   {note}\n");
    let mut last = "";
    for c in Cmd::all().iter().filter(|c| c.group() == g) {
      let s = c.section();
      if g == Group::Reported && s != last && !last.is_empty() {
        println!();
      }
      last = s;
      println!("   {:<8} {:<16} {}", s, c.name(), about(*c));
    }
  }
  println!("\n  Sections of §8 without a command are argued rather than measured, and §10.5 lists");
  println!("  what cannot be reproduced from this repository at all.");
}

/// The one-line description, kept beside the `#[doc]` clap uses so that `list`
/// and `--help` cannot disagree.
fn about(c: Cmd) -> &'static str {
  use Cmd::*;
  match c {
    States => "reachable states of the counterpoint automaton",
    Verdict => "the three tests ricercar's roughness field failed",
    Corpus => "how often Bach breaks each transcribed rule",
    Diag => "the melodic rule broken down by interval",
    Renaissance => "the same rules on 15th-century polyphony",
    Chromatic => "chromaticism against the melodic rule",
    Stretto => "BWV 867's five entries as a clique",
    Rank => "contrapuntal capacity of 24 Bach subjects",
    Probe => "BWV 867 alone, capacity searched",
    Density => "density instead of clique size",
    Pareto => "capacity at Bach's own soft level",
    Design => "designing a subject against the §8.4 measure",
    Revisit => "a graded harmonic objective, and step 4 again",
    Ncts => "harmony with non-chord tones accounted for",
    HarmonyDesign => "harmony as a design objective",
    HarmonyCorpus => "that objective over all subjects",
    Cadence => "the analyser against 106 typed cadence annotations",
    Hren => "the first analyser on modal polyphony",
    Seg => "does the segmentation window change the answer?",
    Sweep => "the Viterbi analyser, swept over the change penalty",
    Holdout => "held-out validation of that sweep",
    ModalControl => "modal control, on the rebuilt analyser",
    Func => "functional progression, the test the modal control wanted",
    BindingHarmony => "harmony as the binding constraint",
    Render => "the stretto rendered to MIDI",
    Reconstruct => "reconstructing Bach's free voices",
    Scalarisations => "what a change of weighting costs, note by note",
    Generality => "a treatise weighting, measured against two corpora",
    Species => "Fux's species as a whitelist, checked before it is used",
    Shape => "criteria over a whole line, ranking uniform draws",
    Plan => "harmonic plans, priced paired and on both corpora",
    Soft => "the soft tier ablated, and three positive criteria in its place",
    Objective => "what the objective is worth, on §8.6's own spans and paired",
    Answer => "Marpurg's tonal answer, against Bach's own",
    Fourth => "the fourth, and the scope a dissonance is judged in",
    Episode => "are episodes sequences?",
    Key => "key-finding, against the cadence annotations",
    Form => "does §2.4's grammar derive the book?",
    Realise => "render, reconstruct and scalarisations together",
    Exp => "all five of the tier-deadlock experiments",
    Harmony => "the three first-attempt harmony runs together",
    S16 => "the superseded step-4 revisit batch",
    S17 => "the superseded analyser batch",
    All => "the four measurements that take seconds — the default",
    List => "this map",
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Clap's own validation of the derived tree: conflicting flags, duplicated
  /// aliases, a `default_value_t` that does not parse. It is one call and it
  /// fails at build time rather than at the user's.
  #[test]
  fn the_command_tree_is_well_formed() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
  }

  /// `list` must not print a command that does not run. The names are derived
  /// from the `Debug` impl and clap derives its own from the variant, so the two
  /// agree only as long as both are kebab-case — this is the test that says so.
  #[test]
  fn every_listed_command_parses() {
    for c in Cmd::all() {
      let got = Cli::try_parse_from(["contrapunctus", &c.name()])
        .unwrap_or_else(|e| panic!("`{}` does not parse: {e}", c.name()));
      assert_eq!(got.cmd, Some(*c), "`{}` parsed as something else", c.name());
    }
  }

  /// Every short name readme §10.2 and CHANGELOG.md have ever cited still works.
  /// These are in the published record, so they are part of the interface.
  #[test]
  fn the_published_short_names_still_work() {
    for (short, want) in [
      ("exp1", Cmd::Density),
      ("exp2", Cmd::Pareto),
      ("exp3", Cmd::Renaissance),
      ("exp4", Cmd::Chromatic),
      ("exp5", Cmd::BindingHarmony),
      ("h1", Cmd::Ncts),
      ("h2", Cmd::HarmonyDesign),
      ("h3", Cmd::HarmonyCorpus),
      ("cad", Cmd::Cadence),
      ("hren2", Cmd::ModalControl),
      ("r1", Cmd::Render),
      ("r2", Cmd::Reconstruct),
      ("r3", Cmd::Scalarisations),
      ("gen", Cmd::Generality),
      ("obj", Cmd::Objective),
    ] {
      let got = Cli::try_parse_from(["contrapunctus", short])
        .unwrap_or_else(|e| panic!("published name `{short}` no longer parses: {e}"));
      assert_eq!(got.cmd, Some(want), "`{short}` now means something else");
    }
  }

  /// The contract this module states in its own doc comment: no flag given
  /// changes any published number.
  #[test]
  fn the_defaults_are_the_published_run() {
    let p = Params::default();
    assert_eq!(p.lambda, 1.0);
    assert_eq!(p.beta, 0.0);
    assert_eq!(p.samples, 8);
    assert_eq!(p.rerank, 32);
    assert_eq!(p.bach_windows, 30);
    assert_eq!(p.ren_windows, 3);
    assert_eq!(p.ren_works, 200);
    assert_eq!(p.seed, 0x5EED);
    assert_eq!(p.tier, TierArg::ConfMelodic);
    assert_eq!(p.tier.rules(), CONF_MEL);
  }

  /// A flag may be given before or after the command, since a reader who has
  /// just typed a long command name should not have to retype it.
  #[test]
  fn parameters_are_accepted_on_either_side_of_the_command() {
    let a = Cli::try_parse_from(["contrapunctus", "--lambda", "0.3", "sweep"]).unwrap();
    let b = Cli::try_parse_from(["contrapunctus", "sweep", "--lambda", "0.3"]).unwrap();
    assert_eq!(a.params.lambda, 0.3);
    assert_eq!(b.params.lambda, 0.3);
    assert_eq!(a.cmd, b.cmd);
  }

  /// An unknown command must be an error. It used to run the default four
  /// silently, which is how a mistyped command comes back as a measurement of
  /// something else.
  #[test]
  fn an_unknown_command_is_refused_rather_than_ignored() {
    assert!(Cli::try_parse_from(["contrapunctus", "specie"]).is_err());
    assert!(Cli::try_parse_from(["contrapunctus"]).unwrap().cmd.is_none());
  }
}
