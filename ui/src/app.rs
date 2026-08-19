//! The application — spec 3's frame, and the state behind it.
//!
//! One layout at two densities. Every control keeps its position between Simple
//! and Advanced, and Advanced *reveals* rather than rearranges, so nobody has to
//! learn a second layout to gain a parameter. A plain-language label sits above
//! the library's own name for the thing, in mono, so a beginner reads *How far
//! it travels* and an expert reads `middles = [4, 5, 3]` underneath. Nobody is
//! lied to.

use contrapunctus::{
  automaton::Tier,
  compose::{self, Layout, Outcome},
};
use egui::{RichText, Ui};

use crate::catalog::{self, Catalog, Journey};
use crate::{report, score, strip, theme};

pub struct App {
  cat: Catalog,
  chosen: usize,
  voices: usize,
  layout: Layout,
  tier: Tier,
  seed: u64,
  advanced: bool,

  out: Option<Outcome>,
  /// The search refused, and refusing is a result — §2.5 does not beam.
  refused: Option<String>,
  /// The controls have moved since what is on screen was written.
  stale: bool,
  /// Compose once on the first frame, so the window appears before the work.
  first: bool,
}

impl Default for App {
  fn default() -> Self {
    let cat = catalog::load();
    // BWV 847, which is §8.16's own subject and therefore the one whose figures
    // this interface can be checked against.
    let chosen = cat.subjects.iter().position(|s| s.id == "wtc-i-02").unwrap_or(0);
    App {
      cat,
      chosen,
      voices: 3,
      layout: Layout::default(),
      tier: Tier::Full,
      seed: 0x5EED,
      advanced: false,
      out: None,
      refused: None,
      stale: false,
      first: true,
    }
  }
}

impl App {
  fn design(&self) -> Option<compose::Design> {
    self.cat.subjects.get(self.chosen).map(|s| s.design(self.voices))
  }

  fn compose(&mut self) {
    let Some(d) = self.design() else {
      self.refused = Some("no subject".into());
      return;
    };
    match compose::fugue(&d, &self.layout, self.tier.rules(), self.seed) {
      Ok(o) => {
        self.out = Some(o);
        self.refused = None;
      }
      Err(e) => self.refused = Some(e),
    }
    self.stale = false;
  }
}

impl eframe::App for App {
  fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
    self.draw(ui);
  }
}

impl App {
  /// The whole interface, given nothing but a `Ui`.
  ///
  /// Split out from the trait method so that a test can paint every view
  /// headlessly — `eframe::Frame` cannot be constructed outside a running
  /// window, and a smoke test that cannot run is not a smoke test.
  pub fn draw(&mut self, ui: &mut Ui) {
    if self.first {
      self.first = false;
      self.compose();
    }

    egui::Panel::top("bar").show(ui, |ui| {
      ui.horizontal(|ui| {
        ui.heading("Contrapunctus");
        ui.add_space(10.0);
        ui.selectable_value(&mut self.advanced, false, "Simple");
        ui.selectable_value(&mut self.advanced, true, "Advanced");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
          ui.add_enabled(false, egui::Button::new("Open"))
            .on_disabled_hover_text("Settings files are next — the library writes them already.");
          ui.add_enabled(false, egui::Button::new("Save"))
            .on_disabled_hover_text("Settings files are next — the library writes them already.");
          ui.add_enabled(false, egui::Button::new("▶"))
            .on_disabled_hover_text("Sound is not wired up yet.");
        });
      });
    });

    egui::Panel::left("controls").default_size(288.0).show(ui, |ui| {
      egui::ScrollArea::vertical().show(ui, |ui| self.controls(ui));
    });

    egui::CentralPanel::default_margins().show(ui, |ui| {
      if let Some(why) = &self.refused {
        ui.add_space(6.0);
        ui.colored_label(theme::warn(ui.visuals().dark_mode), RichText::new("Refused").strong());
        ui.label(why);
        ui.label(
          RichText::new(
            "The search is exact: where no legal filling exists it says so rather than \
             writing an approximate one.",
          )
          .weak()
          .small(),
        );
        ui.add_space(8.0);
      }

      let Some(out) = &self.out else { return };
      let measure = self.cat.subjects.get(self.chosen).map(|s| s.measure).unwrap_or(960);

      ui.add_space(4.0);
      ui.label(RichText::new("PLAN").monospace().weak().small());
      strip::show(ui, out, self.voices, measure);

      ui.add_space(10.0);
      ui.label(RichText::new("SCORE").monospace().weak().small());
      let key = self.cat.subjects.get(self.chosen).map(|s| s.key).unwrap_or([0; 7]);
      egui::ScrollArea::both().max_height(score::height(out.voices.len()) + 12.0).show(ui, |ui| {
        let want = (out.bars as f32 * 46.0).max(ui.available_width());
        score::show(ui, &out.voices, &key, measure, want);
      });

      ui.add_space(10.0);
      ui.separator();
      ui.label(RichText::new("HOW IT TURNED OUT").monospace().weak().small());
      ui.add_space(2.0);
      report::show(ui, out, self.tier, self.advanced);
    });
  }
}

impl App {
  fn controls(&mut self, ui: &mut Ui) {
    let mut changed = false;

    group(ui, "THE TUNE");
    let name = self.cat.subjects.get(self.chosen).map(|s| s.name.clone()).unwrap_or_default();
    egui::ComboBox::from_id_salt("subject").width(248.0).selected_text(name).show_ui(ui, |ui| {
      for (i, s) in self.cat.subjects.iter().enumerate() {
        if ui.selectable_value(&mut self.chosen, i, &s.name).changed() {
          changed = true;
        }
      }
    });
    if let Some(s) = self.cat.subjects.get(self.chosen) {
      ui.label(
        RichText::new(format!(
          "{} notes over {:.1} bars · Bach set it for {}",
          s.notes.notes.iter().filter(|n| n.attack).count(),
          s.bars,
          s.scored_for
        ))
        .weak()
        .small(),
      );
    }
    ui.add_enabled(false, egui::Button::new("Import…"))
      .on_disabled_hover_text("Importing a subject from a file is on the roadmap.");

    ui.add_space(12.0);
    group(ui, "THE SHAPE");

    labelled(ui, "How many voices", "Design::voices");
    ui.horizontal(|ui| {
      for n in [2usize, 3] {
        if ui.selectable_value(&mut self.voices, n, format!("{n}")).changed() {
          changed = true;
        }
      }
      // Present and disabled, with the reason — spec 9. Hiding it would conceal
      // a fact about the program, and this repository's habit is the opposite.
      ui.add_enabled(false, egui::Button::new("4")).on_disabled_hover_text(
        "Four voices needs three free voices at once. The search is exact up to two and \
         refuses beyond rather than beaming. It is §9's solver item.",
      );
    });

    ui.add_space(8.0);
    labelled(ui, "Times the tune comes back", "Layout::middles.len()");
    let mut returns = self.layout.middles.len();
    if ui.add(egui::Slider::new(&mut returns, 1..=6).show_value(true)).changed() {
      resize(&mut self.layout.middles, returns);
      changed = true;
    }

    ui.add_space(8.0);
    labelled(ui, "How far it travels", "Layout::middles");
    let now = Journey::of(&self.layout.middles);
    ui.horizontal_wrapped(|ui| {
      for j in Journey::ALL {
        if ui.selectable_label(now == Some(j), j.label()).on_hover_text(j.hint()).clicked() {
          self.layout.middles = j.middles();
          changed = true;
        }
      }
    });
    if now.is_none() {
      ui.label(RichText::new("edited — no longer one of the three").weak().small());
    }
    ui.label(
      RichText::new(
        self
          .layout
          .middles
          .iter()
          .map(|d| catalog::degree_name(*d))
          .collect::<Vec<_>>()
          .join(" · "),
      )
      .monospace()
      .small(),
    );

    ui.add_space(8.0);
    labelled(ui, "How strictly it follows the rules", "tier");
    for t in [Tier::Confirmed, Tier::ConfMelodic, Tier::Full] {
      let text = match t {
        Tier::Confirmed => "Loose (2 rules)",
        Tier::ConfMelodic => "Middling (3 rules)",
        Tier::Full => "Strict (5 rules) — recommended",
      };
      if ui.selectable_value(&mut self.tier, t, text).changed() {
        changed = true;
      }
    }
    ui.label(
      RichText::new(
        "Strict is the default and is not the tier the repository endorses for describing \
         Bach: on the looser ones the generator writes 366 dissonances per thousand and a \
         listener called it cacophony; on all five it writes about 74, below Bach's own 112.",
      )
      .weak()
      .small(),
    );

    if self.advanced {
      ui.add_space(12.0);
      group(ui, "LAYOUT");

      labelled(ui, "Episode length", "Layout::episode_bars");
      if ui.add(egui::Slider::new(&mut self.layout.episode_bars, 1..=6).suffix(" bars")).changed() {
        changed = true;
      }

      let mut linked = self.layout.link.is_some();
      if ui.checkbox(&mut linked, "Exposition takes a link").changed() {
        self.layout.link = if linked { Some((1, 1)) } else { None };
        changed = true;
      }
      if let Some((after, bars)) = &mut self.layout.link {
        ui.horizontal(|ui| {
          changed |= ui.add(egui::DragValue::new(after).range(1..=4).prefix("after entry ")).changed();
          changed |= ui.add(egui::DragValue::new(bars).range(1..=4).suffix(" bars")).changed();
        });
        ui.label(
          RichText::new("§2.4's grammar forbids this and 82% of real expositions contain one.")
            .weak()
            .small(),
        );
      }

      if ui.checkbox(&mut self.layout.close_at_home, "Close at home").changed() {
        changed = true;
      }

      ui.add_space(12.0);
      group(ui, "MIDDLES");
      let mut remove = None;
      for i in 0..self.layout.middles.len() {
        ui.horizontal(|ui| {
          ui.label(RichText::new(format!("{}.", i + 1)).monospace().weak());
          egui::ComboBox::from_id_salt(("deg", i))
            .width(96.0)
            .selected_text(catalog::degree_name(self.layout.middles[i]))
            .show_ui(ui, |ui| {
              for d in 0..7i16 {
                if ui.selectable_value(&mut self.layout.middles[i], d, catalog::degree_name(d)).changed() {
                  changed = true;
                }
              }
            });
          if ui.small_button("✕").clicked() && self.layout.middles.len() > 1 {
            remove = Some(i);
          }
        });
      }
      if let Some(i) = remove {
        self.layout.middles.remove(i);
        changed = true;
      }
      if ui.small_button("＋ add a return").clicked() {
        self.layout.middles.push(4);
        changed = true;
      }

      ui.add_space(12.0);
      group(ui, "SEARCH");
      ui.horizontal(|ui| {
        ui.label(RichText::new("seed").monospace().small());
        let mut hex = format!("{:X}", self.seed);
        if ui.add(egui::TextEdit::singleline(&mut hex).desired_width(110.0).font(egui::TextStyle::Monospace)).changed() {
          if let Ok(v) = u64::from_str_radix(hex.trim_start_matches("0x"), 16) {
            self.seed = v;
            changed = true;
          }
        }
      });

      if !self.cat.missing.is_empty() {
        ui.add_space(12.0);
        group(ui, "NOT OFFERED");
        for (id, why) in &self.cat.missing {
          ui.label(RichText::new(format!("{id} — {why}")).weak().small());
        }
      }
    }

    ui.add_space(14.0);
    let label = if self.stale { "Compose ↻" } else { "Compose" };
    if ui.add_sized([ui.available_width(), 30.0], egui::Button::new(RichText::new(label).strong())).clicked() {
      self.compose();
    }
    if ui
      .add_sized([ui.available_width(), 26.0], egui::Button::new("Try a different one"))
      .on_hover_text(
        "A different seed writes different notes and about equally good ones — over twelve \
         seeds the rate runs 70 to 78 per thousand, all far below Bach's 112. This explores \
         the legal set; it does not hunt a better score.",
      )
      .clicked()
    {
      self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
      self.compose();
    }

    if changed {
      self.stale = true;
    }
    if self.stale {
      ui.label(
        RichText::new("The controls have moved since this was written.")
          .weak()
          .small(),
      );
    }
  }
}

/// A group heading, in the mono voice the sketch uses for structure.
fn group(ui: &mut Ui, title: &str) {
  ui.label(RichText::new(title).monospace().small().weak());
  ui.separator();
}

/// The plain label, and the library's own name for the thing underneath it.
fn labelled(ui: &mut Ui, plain: &str, real: &str) {
  ui.label(plain);
  ui.label(RichText::new(real).monospace().small().weak());
}

/// Grow or shrink the middles to `n`, keeping what is there. A grown plan
/// repeats the dominant, which is the book's commonest return and the least
/// surprising thing to add.
fn resize(middles: &mut Vec<i16>, n: usize) {
  while middles.len() > n.max(1) {
    middles.pop();
  }
  while middles.len() < n {
    middles.push(4);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Paint the whole interface, headlessly, twice.
  ///
  /// Not a test of what it looks like — nothing here can see. It is a test that
  /// every view can be drawn from a real `Outcome` without an index running off
  /// the end of a lane, a voice, or a block list, which is the failure mode of
  /// drawing code that indexes three collections against each other. Twice
  /// because the first pass composes and the second draws what the first made,
  /// and those are different code paths.
  #[test]
  fn every_view_paints() {
    let mut app = App::default();
    assert!(!app.cat.subjects.is_empty(), "the embedded corpus supplied no subjects");
    for _ in 0..2 {
      egui::__run_test_ui(|ui| app.draw(ui));
    }
    assert!(app.out.is_some(), "the default settings did not produce a fugue: {:?}", app.refused);
  }

  /// Every subject the picker offers must produce a design the generator will
  /// take. A picker listing a subject that cannot be composed is a picker that
  /// wastes the one click a beginner is sure to make.
  ///
  /// On the shortest layout that is still a fugue — one return, two-bar
  /// episodes, no link. What this checks is the *design*: a compass that does
  /// not admit the subject, a tonic that is not a letter of the signature, a
  /// clip that came back empty. Those fail on any layout, and this is the
  /// cheapest one that exercises them: two and a half minutes of debug build
  /// across 22 subjects, against five for `Layout::default()`. An exact search
  /// is not fast unoptimised, and the alternative to paying it is a picker
  /// whose entries are unchecked.
  #[test]
  fn every_offered_subject_composes() {
    let cat = catalog::load();
    assert_eq!(cat.subjects.len(), 24, "not offered: {:?}", cat.missing);
    let brief = Layout { middles: vec![4], episode_bars: 2, link: None, close_at_home: true };
    let mut refused = vec![];
    for s in &cat.subjects {
      let d = s.design(3);
      if let Err(e) = compose::fugue(&d, &brief, Tier::Full.rules(), 0x5EED) {
        refused.push(format!("{}: {e}", s.id));
      }
    }
    assert!(refused.is_empty(), "subjects offered that will not compose:\n  {}", refused.join("\n  "));
  }

  /// The presets are the ones spec 3.2 names, and `Journey::of` recognises each
  /// of them — otherwise the interface would show *edited* the moment a preset
  /// was clicked.
  #[test]
  fn every_preset_is_recognised_as_itself() {
    for j in Journey::ALL {
      assert_eq!(Journey::of(&j.middles()), Some(j), "{} was not recognised", j.label());
    }
    assert_eq!(Journey::of(&[2, 2, 2]), None, "an edited plan must not read as a preset");
    assert_eq!(Journey::Wander.middles(), Layout::default().middles, "the default is Wanders");
  }
}
