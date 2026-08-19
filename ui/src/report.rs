//! *How it turned out* — spec 3.1's third view, and spec 11's one rule about
//! numbers.
//!
//! > **Report a number with a yardstick or not at all.** *70 per thousand* means
//! > nothing to anyone. *70 per thousand, where Bach averages 112* means
//! > something immediately, and the second is barely longer.
//!
//! `compose::fugue` returns the notes *and* every judgement §8 can pass on them,
//! which is what makes this view possible without a second pass over the music —
//! and, more to the point, is what makes it hard to show a fugue *without* its
//! verdict.

use contrapunctus::{
  automaton::{Tier, CONFIRMED},
  compose::Outcome,
};
use egui::{RichText, Ui};

use crate::theme;

/// Bach's own rate on the same five rules, over the 24 fugues — §8.12. The
/// yardstick, and the reason any of the numbers here mean anything.
pub const BACH: f64 = 112.3;

pub fn show(ui: &mut Ui, out: &Outcome, tier: Tier, advanced: bool) {
  let dark = ui.visuals().dark_mode;
  let rate = out.per_thousand(Tier::Full.rules());
  let clean = out.clean_on(CONFIRMED);

  ui.horizontal_wrapped(|ui| {
    ui.spacing_mut().item_spacing.x = 4.0;
    ui.label(RichText::new(format!("{:.0} dissonances per thousand", rate)).strong());
    ui.label("where Bach averages");
    ui.label(RichText::new(format!("{BACH:.0}")).strong());
    ui.label("— so this is");
    ui.label(
      RichText::new(if rate < BACH { "smoother than Bach" } else { "rougher than Bach" })
        .color(if rate < BACH { theme::good(dark) } else { theme::warn(dark) }),
    );
    ui.label(".");
  });

  ui.horizontal_wrapped(|ui| {
    ui.spacing_mut().item_spacing.x = 4.0;
    if clean {
      ui.colored_label(theme::good(dark), "It breaks none");
    } else {
      ui.colored_label(theme::warn(dark), "It breaks some");
    }
    ui.label("of the two rules that hold in every century tested.");
  });

  ui.horizontal_wrapped(|ui| {
    ui.spacing_mut().item_spacing.x = 4.0;
    ui.label(format!("{} bars,", out.bars));
    ui.label(format!("{} blocks,", out.blocks.len()));
    ui.label(format!("written in {:.1} seconds.", out.seconds));
    if out.relaxed.blocks > 0 {
      ui.label(
        RichText::new(format!(
          "{} needed a constraint dropped.",
          out.relaxed.blocks
        ))
        .color(theme::warn(dark)),
      );
    }
  });

  if !advanced {
    return;
  }

  ui.add_space(8.0);
  ui.separator();

  egui::Grid::new("verdict").num_columns(2).spacing([18.0, 3.0]).show(ui, |ui| {
    let v = &out.verdict;
    for (name, ok, note) in [
      ("exposition covers the voices", v.exposition_covers_the_voices, ""),
      ("exposition alternates", v.exposition_alternates, "tonic with dominant"),
      ("exposition runs unbroken", v.exposition_is_unbroken, "the link fails this on purpose"),
      ("has a middle", v.has_a_middle, ""),
      ("ends at home", v.ends_at_home, ""),
    ] {
      ui.colored_label(if ok { theme::good(dark) } else { theme::warn(dark) }, if ok { "yes" } else { "no" });
      ui.label(if note.is_empty() { name.to_string() } else { format!("{name} — {note}") });
      ui.end_row();
    }
  });

  ui.add_space(8.0);
  ui.label(RichText::new("firings by rule").weak());
  egui::Grid::new("tally").num_columns(2).spacing([18.0, 2.0]).show(ui, |ui| {
    for r in Tier::Full.rules() {
      let n = out.tally.by_rule.get(r.name()).copied().unwrap_or(0);
      ui.monospace(format!("{n:>5}"));
      ui.label(r.name());
      ui.end_row();
    }
    ui.monospace(format!("{:>5}", out.tally.slices));
    ui.label("slices judged");
    ui.end_row();
  });

  if out.relaxed.blocks > 0 {
    ui.add_space(8.0);
    ui.label(RichText::new("what was relaxed").weak());
    ui.label(format!(
      "{} of {} blocks: {} lost the join, {} the harmonic plan as well.",
      out.relaxed.blocks,
      out.blocks.len(),
      out.relaxed.without_prior,
      out.relaxed.without_plan
    ));
    ui.label(
      RichText::new(
        "The join goes first because it is this generator's own convenience; the plan is \
         both an obligation system and what keeps the search tractable.",
      )
      .weak()
      .small(),
    );
  }

  ui.add_space(8.0);
  ui.label(
    RichText::new(format!(
      "Generated against the {} tier. The blockwise fill resets the automaton at each block \
       edge, so a violation at a join is expected and is the same fact that makes editing local.",
      tier.label()
    ))
    .weak()
    .small(),
  );
}
