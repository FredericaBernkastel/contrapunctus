//! The application — spec 3's frame, and the state behind it.
//!
//! One layout at two densities. Every control keeps its position between Simple
//! and Advanced, and Advanced *reveals* rather than rearranges, so nobody has to
//! learn a second layout to gain a parameter. A plain-language label sits above
//! the library's own name for the thing, in mono, so a beginner reads *How far
//! it travels* and an expert reads `middles = [4, 5, 3]` underneath. Nobody is
//! lied to.
//!
//! **The design is the state, not the subject picker.** A settings file carries
//! its own `Design` — the subject as notes, in the library's own units — so a
//! file can be opened whose subject is not one of the 24 and everything still
//! works. The picker *sets* the design; it does not define it.

use contrapunctus::{
  automaton::Tier,
  compose::{self, Design, Layout, Outcome},
  settings::Fidelity,
};
use egui::{RichText, Ui};

use crate::catalog::{self, Catalog, Journey};
use crate::files::{self, Loaded, Note};
use crate::task::Slot;
use crate::{report, score, strip, theme};

pub struct App {
  cat: Catalog,
  /// Which catalogue entry the design came from, if it came from one. A design
  /// loaded from a file came from none, and saying so is better than pointing at
  /// whichever entry happens to be first.
  chosen: Option<usize>,
  design: Design,
  layout: Layout,
  tier: Tier,
  seed: u64,
  advanced: bool,
  /// Quarter notes per minute, for the MIDI export.
  qpm: u32,

  out: Option<Outcome>,
  /// The search refused, and refusing is a result — §2.5 does not beam.
  refused: Option<String>,
  /// The controls have moved since what is on screen was written.
  stale: bool,
  /// Compose once on the first frame, so the window appears before the work.
  first: bool,

  /// What the last file operation or edit did, in one line.
  status: Option<String>,
  /// Whether a loaded file reproduced the fugue it recorded. `None` once the
  /// music has been changed since, because the answer would no longer be about
  /// anything on screen.
  fidelity: Option<Fidelity>,

  saving: Slot<Note>,
  loading: Slot<Result<Loaded, Note>>,
}

impl Default for App {
  fn default() -> Self {
    let cat = catalog::load();
    // BWV 847, which is §8.16's own subject and therefore the one whose figures
    // this interface can be checked against.
    let chosen = cat.subjects.iter().position(|s| s.id == "wtc-i-02").unwrap_or(0);
    let design = cat.subjects[chosen].design(3);
    App {
      cat,
      chosen: Some(chosen),
      design,
      layout: Layout::default(),
      tier: Tier::Full,
      seed: 0x5EED,
      advanced: false,
      qpm: 76,
      out: None,
      refused: None,
      stale: false,
      first: true,
      status: None,
      fidelity: None,
      saving: Slot::default(),
      loading: Slot::default(),
    }
  }
}

impl App {
  fn compose(&mut self) {
    match compose::fugue(&self.design, &self.layout, self.tier.rules(), self.seed) {
      Ok(o) => {
        self.out = Some(o);
        self.refused = None;
      }
      Err(e) => self.refused = Some(e),
    }
    self.stale = false;
    self.fidelity = None;
  }

  /// Apply an edit from the plan strip.
  ///
  /// **Span-preserving, and the interface must not blur that.** A key change
  /// keeps the piece the same length, so `compose::refill` rewrites the two
  /// blocks that middle owns — its episode and its entry, because changing where
  /// a return goes changes both the journey and the arrival — and every other
  /// note stays exactly where it was. If any part of it refuses, nothing is
  /// applied: a half-edited piece is worse than an unedited one.
  fn apply(&mut self, e: strip::Edit) {
    let strip::Edit::Key(k, deg) = e;
    if self.layout.middles.get(k) == Some(&deg) {
      return;
    }
    let Some(prev) = self.out.take() else { return };
    let mut l = self.layout.clone();
    l.middles[k] = deg;

    // The two blocks as one span, not one after the other. A middle owns its
    // episode and its entry, and refilling them separately pins the seam between
    // them to notes chosen for the key being edited away — which is usually
    // unsatisfiable, so an ordinary edit came back refused.
    let owned = compose::blocks_of_middle(&self.design, &l, k);
    let (Some(&first), Some(&last)) = (owned.first(), owned.last()) else { return };

    let t0 = std::time::Instant::now();
    let next = match compose::refill_span(&self.design, &l, self.tier.rules(), self.seed, &prev, first, last) {
      Ok(o) => o,
      Err(why) => {
        // A refusal here means the edit is not *local* — not that it is
        // impossible. About a quarter of key changes come back this way, so
        // stopping would refuse an ordinary request on a technicality. Recompose
        // the whole piece instead and say which of the two happened, because the
        // difference is exactly the promise the interface made: everything else
        // stayed where it was, or it did not.
        match compose::fugue(&self.design, &l, self.tier.rules(), self.seed) {
          Ok(o) => {
            self.status = Some(format!(
              "return {} sent to {} — not reachable from where the piece was, so the whole                fugue was rewritten ({why})",
              k + 1,
              catalog::degree_name(deg)
            ));
            self.layout = l;
            self.out = Some(o);
            self.fidelity = None;
          }
          Err(e) => {
            self.status = Some(format!("that key will not fill: {e}"));
            self.out = Some(prev);
          }
        }
        return;
      }
    };
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    self.status = Some(format!(
      "return {} sent to {} — {} blocks rewritten in {ms:.0} ms, the rest untouched",
      k + 1,
      catalog::degree_name(deg),
      last - first + 1,
    ));
    self.layout = l;
    self.out = Some(next);
    self.fidelity = None;
  }

  /// Take whatever the file tasks have finished, once per frame.
  fn collect(&mut self) {
    if let Some(n) = self.saving.take() {
      self.status = Some(match n {
        Note::Saved(name) => format!("saved {name}"),
        Note::Cancelled => "nothing saved".into(),
        Note::Failed(e) => format!("could not save: {e}"),
      });
    }
    if let Some(r) = self.loading.take() {
      match r {
        Ok(l) => {
          let want = contrapunctus::settings::fingerprint(std::slice::from_ref(&l.settings.design.subject));
          self.chosen = self
            .cat
            .subjects
            .iter()
            .position(|s| contrapunctus::settings::fingerprint(std::slice::from_ref(&s.notes)) == want);
          self.design = l.settings.design;
          self.layout = l.settings.layout;
          self.tier = l.settings.tier;
          self.seed = l.settings.seed;
          self.status = Some(match &l.how {
            Fidelity::Exact => "opened — the same fugue, note for note".to_string(),
            other => other.message().unwrap_or_default(),
          });
          self.fidelity = Some(l.how);
          self.out = Some(l.outcome);
          self.refused = None;
          self.stale = false;
        }
        Err(Note::Cancelled) => self.status = Some("nothing opened".into()),
        Err(Note::Failed(e)) => self.status = Some(format!("could not open: {e}")),
        Err(Note::Saved(_)) => {}
      }
    }
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
    self.collect();

    let mut save = false;
    let mut open = false;
    let mut export = false;
    egui::Panel::top("bar").show(ui, |ui| {
      ui.horizontal(|ui| {
        ui.heading("Contrapunctus");
        ui.add_space(10.0);
        ui.selectable_value(&mut self.advanced, false, "Simple");
        ui.selectable_value(&mut self.advanced, true, "Advanced");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
          open |= ui.button("Open").on_hover_text("A settings file, which writes the same fugue back.").clicked();
          save |= ui
            .add_enabled(self.out.is_some(), egui::Button::new("Save"))
            .on_hover_text("Everything that determines this fugue, as JSON, with a fingerprint over the notes.")
            .clicked();
          export |= ui
            .add_enabled(self.out.is_some(), egui::Button::new("Export MIDI"))
            .on_hover_text("Tracks top voice first, each named with how many entries it carries.")
            .clicked();
          ui.add_enabled(false, egui::Button::new("▶"))
            .on_disabled_hover_text("Sound is not wired up yet — it is next on the roadmap.");
        });
      });
    });

    if open {
      files::load_settings(self.loading.clone());
    }
    if let (true, Some(out)) = (save, self.out.as_ref()) {
      files::save_settings(&self.design, &self.layout, self.tier, self.seed, out, self.saving.clone());
    }
    if let (true, Some(out)) = (export, self.out.as_ref()) {
      files::export_midi(out, &self.design, self.qpm, self.saving.clone());
    }

    egui::Panel::left("controls").default_size(288.0).show(ui, |ui| {
      egui::ScrollArea::vertical().show(ui, |ui| self.controls(ui));
    });

    let mut edit = None;
    egui::CentralPanel::default_margins().show(ui, |ui| {
      let dark = ui.visuals().dark_mode;

      // A loaded file that did not reproduce says so, above the music rather
      // than beside it. The music is shown either way — refusing to show it
      // would hide the very thing the reader needs in order to judge.
      if let Some(f) = &self.fidelity {
        if let Some(msg) = f.message() {
          ui.add_space(4.0);
          ui.colored_label(theme::warn(dark), RichText::new("Not the fugue this file recorded").strong());
          ui.label(msg);
          ui.label(
            RichText::new(
              "The settings are the ones the file holds; what they now produce is below.                Saving again records this engine and this music.",
            )
            .weak()
            .small(),
          );
          ui.add_space(6.0);
        }
      }

      if let Some(why) = &self.refused {
        ui.add_space(6.0);
        ui.colored_label(theme::warn(dark), RichText::new("Refused").strong());
        ui.label(why);
        ui.label(
          RichText::new(
            "The search is exact: where no legal filling exists it says so rather than              writing an approximate one.",
          )
          .weak()
          .small(),
        );
        ui.add_space(8.0);
      }

      let Some(out) = &self.out else { return };
      let measure = self.design.measure;
      let origins = compose::origins(&self.design, &self.layout);

      ui.add_space(4.0);
      ui.label(RichText::new("PLAN").monospace().weak().small());
      let (_, asked) = strip::show(ui, out, self.design.voices, measure, &origins);
      edit = asked;

      ui.add_space(10.0);
      ui.label(RichText::new("SCORE").monospace().weak().small());
      egui::ScrollArea::both().max_height(score::height(out.voices.len()) + 12.0).show(ui, |ui| {
        let want = (out.bars as f32 * 46.0).max(ui.available_width());
        score::show(ui, &out.voices, &self.design.key, measure, want);
      });

      ui.add_space(10.0);
      ui.separator();
      ui.label(RichText::new("HOW IT TURNED OUT").monospace().weak().small());
      ui.add_space(2.0);
      report::show(ui, out, self.tier, self.advanced);

      if let Some(line) = &self.status {
        ui.add_space(6.0);
        ui.label(RichText::new(line).weak().small());
      }
    });

    // Applied after the panel, not inside it: the panel holds a borrow of the
    // outcome it is drawing, and an edit replaces that outcome.
    if let Some(e) = edit {
      self.apply(e);
    }
  }
}

impl App {
  fn controls(&mut self, ui: &mut Ui) {
    let mut changed = false;

    group(ui, "THE TUNE");
    let name = self
      .chosen
      .and_then(|i| self.cat.subjects.get(i))
      .map(|s| s.name.clone())
      // A design that came from a file is not one of the 24, and the picker says
      // so rather than pointing at whichever entry happens to be selected.
      .unwrap_or_else(|| "from a file".to_string());
    let mut pick = None;
    egui::ComboBox::from_id_salt("subject").width(248.0).selected_text(name).show_ui(ui, |ui| {
      for (i, s) in self.cat.subjects.iter().enumerate() {
        if ui.selectable_label(self.chosen == Some(i), &s.name).clicked() {
          pick = Some(i);
        }
      }
    });
    if let Some(i) = pick {
      self.chosen = Some(i);
      self.design = self.cat.subjects[i].design(self.design.voices);
      changed = true;
    }
    let notes = self.design.subject.notes.iter().filter(|n| n.attack).count();
    let bars = self.design.subject.notes.iter().map(|n| n.onset + n.dur).max().unwrap_or(0) as f64
      / self.design.measure.max(1) as f64;
    ui.label(
      RichText::new(match self.chosen.and_then(|i| self.cat.subjects.get(i)) {
        Some(s) => format!("{notes} notes over {bars:.1} bars · Bach set it for {}", s.scored_for),
        None => format!("{notes} notes over {bars:.1} bars"),
      })
      .weak()
      .small(),
    );
    ui.add_enabled(false, egui::Button::new("Import…"))
      .on_disabled_hover_text("Importing a subject from a file is on the roadmap.");

    ui.add_space(12.0);
    group(ui, "THE SHAPE");

    labelled(ui, "How many voices", "Design::voices");
    ui.horizontal(|ui| {
      for n in [2usize, 3] {
        if ui.selectable_label(self.design.voices == n, format!("{n}")).clicked() {
          self.design.voices = n;
          // The compass has one range per voice, so it moves with the count.
          self.design.compass = catalog::compass(n);
          changed = true;
        }
      }
      // Present and disabled, with the reason — spec 9. Hiding it would conceal
      // a fact about the program, and this repository's habit is the opposite.
      ui.add_enabled(false, egui::Button::new("4")).on_disabled_hover_text(
        "Four voices needs three free voices at once. The search is exact up to two and          refuses beyond rather than beaming. It is §9's solver item.",
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

  /// A second application holding a given piece, for trying an edit against it
  /// without disturbing the first.
  fn probe_from(start: &Outcome, layout: &Layout) -> App {
    App { out: Some(start.clone()), layout: layout.clone(), ..App::default() }
  }

  /// A key change that stays local rewrites only the blocks that middle owns
  /// and **leaves every other note exactly where it was**.
  ///
  /// This is the claim spec 4.2 makes to the user and the one an interface is
  /// most tempted to fudge: recomposing would look almost right, differ
  /// everywhere, and be much easier. Checked over the whole piece rather than at
  /// the seams, because a splice that tore one bar later would pass a boundary
  /// check and still be wrong on screen.
  ///
  /// About a quarter of key changes are *not* local — the pinned ending is not
  /// reachable from where the piece happens to be — and those are recomposed
  /// instead. The test asserts the promise on whichever ones hold it, and that
  /// at least one does, so it cannot pass by never taking the local path.
  #[test]
  fn a_local_key_change_leaves_every_other_note_alone() {
    let mut app = App::default();
    app.compose();
    let start = app.out.clone().expect("a fugue");
    let l0 = app.layout.clone();
    let mut local = 0;

    for k in 0..l0.middles.len() {
      for deg in 0..7i16 {
        if l0.middles[k] == deg {
          continue;
        }
        let mut probe = probe_from(&start, &l0);
        probe.apply(strip::Edit::Key(k, deg));
        let after = probe.out.as_ref().expect("still a fugue");
        let said = probe.status.clone().unwrap_or_default();

        if said.contains("the rest untouched") {
          local += 1;
          let owned = compose::blocks_of_middle(&probe.design, &probe.layout, k);
          let from = start.blocks[owned[0]].at;
          let to = start.blocks[owned[owned.len() - 1]].at + start.blocks[owned[owned.len() - 1]].len;
          let outside = |v: &contrapunctus::kern::Voice| -> Vec<(i64, i64, i16, i8)> {
            v.notes
              .iter()
              .filter(|n| n.onset < from || n.onset >= to)
              .map(|n| (n.onset, n.dur, n.pitch.step, n.pitch.alter))
              .collect()
          };
          for (a, b) in start.voices.iter().zip(&after.voices) {
            assert_eq!(outside(a), outside(b), "middle {k} to degree {deg} changed notes outside its span");
          }
          assert_eq!(start.bars, after.bars, "a span-preserving edit must not change the length");
        } else {
          // not local: the piece was rewritten, and the layout still took the edit
          assert_eq!(probe.layout.middles[k], deg, "neither local nor rewritten: {said}");
        }
      }
    }
    assert!(local > 0, "no key change was local, so the promise was never tested");
  }

  /// Whatever happens, an edit either applies or leaves nothing behind.
  ///
  /// The half-applied case is the one worth ruling out: a layout that took the
  /// edit beside music that did not, which would show a plan and a score saying
  /// different things about where the piece is.
  #[test]
  fn an_edit_is_all_or_nothing() {
    let mut app = App::default();
    app.compose();
    let start = app.out.clone().expect("a fugue");
    let l0 = app.layout.clone();

    for deg in 0..7i16 {
      let mut probe = probe_from(&start, &l0);
      probe.apply(strip::Edit::Key(0, deg));
      let after = probe.out.as_ref().expect("still a fugue");
      let took = probe.layout.middles[0] == deg;
      let same = contrapunctus::settings::fingerprint(&start.voices)
        == contrapunctus::settings::fingerprint(&after.voices);
      if l0.middles[0] == deg {
        continue;
      }
      assert!(
        took != same,
        "degree {deg}: layout took the edit = {took}, music unchanged = {same} — one of those is a lie"
      );
    }
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
