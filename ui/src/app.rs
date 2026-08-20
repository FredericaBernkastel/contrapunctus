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
use crate::audio::Player;
use crate::files::{self, Imported, Loaded, Note};
use crate::schedule;
use crate::task::Slot;
use crate::{report, score, strip, theme};

pub struct App {
  cat: Catalog,
  /// Which catalogue entry the design came from, if it came from one. A design
  /// loaded from a file came from none, and saying so is better than pointing at
  /// whichever entry happens to be first.
  chosen: Option<usize>,
  /// The file a subject was imported from, kept so that a different voice of it
  /// can be chosen afterwards. A multi-voice file has to give up *one* line and
  /// nothing in a bare `**kern` file says which — so the interface takes the
  /// first, says so, and offers the others.
  imported: Option<Imported>,
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
  importing: Slot<Result<Imported, Note>>,

  /// Built on the first press of Play, never before. A test constructs this
  /// application headlessly and a build machine has no sound card; more to the
  /// point, a browser may only open an audio context in response to a gesture,
  /// and pressing Play is one.
  player: Option<Player>,
  /// Why there is no sound, if there is none. Spec 6.3's rule, applied to the
  /// synth as well: absence is stated, never silent.
  no_sound: Option<String>,
  /// A bit per voice, set to silence it while listening. Interface state, not
  /// music: it changes what is audible and nothing about what is written.
  mute: u32,
  /// The very score the player holds. Kept so that every tick-to-sample
  /// conversion goes through the tempo the sound was actually built at, rather
  /// than through whatever the tempo control says at the moment of asking.
  sound: Option<std::sync::Arc<schedule::Score>>,
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
      imported: None,
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
      importing: Slot::default(),
      player: None,
      no_sound: None,
      mute: 0,
      sound: None,
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
    self.resound();
  }

  /// Apply an edit from the plan strip.
  ///
  /// **The two classes must not be blurred, and here they are two branches.**
  /// A key change keeps the piece the same length, so `compose::refill_span`
  /// rewrites the blocks that return owns and every other note stays exactly
  /// where it was. An episode length, a link, a reordering: everything after
  /// moves in time, there is no sense in which those bars are unchanged, and
  /// the piece is recomposed. The strip has been drawing the new plan, faded,
  /// for the length of the drag, so the second is not a surprise by the time it
  /// happens.
  fn apply(&mut self, e: strip::Edit) {
    let next = e.applied(&self.layout);
    if next == self.layout {
      return;
    }
    match e.touches(&self.design, &next) {
      Some(blocks) => self.recolour(e, blocks, next),
      None => self.rebuild(e, next),
    }
  }

  /// An edit that rewrites some blocks and moves none: local if it can be,
  /// recomposed if it cannot, and it says which.
  ///
  /// The affected blocks go through as **one span**, not one after another. A
  /// return owns its episode and its entry, and refilling those separately pins
  /// the seam between them to notes chosen for the key being edited away.
  fn recolour(&mut self, e: strip::Edit, blocks: std::ops::RangeInclusive<usize>, l: Layout) {
    let Some(prev) = self.out.take() else { return };
    let (first, last) = (*blocks.start(), *blocks.end());
    if last >= prev.blocks.len() {
      self.out = Some(prev);
      return;
    }

    let said = describe_edit(e, &l);
    let t0 = web_time::Instant::now();
    let next = match compose::refill_span(&self.design, &l, self.tier.rules(), self.seed, &prev, first, last) {
      Ok(o) => o,
      Err(why) => {
        // Not *local* — not impossible. About a quarter of key changes come
        // back this way, so stopping would refuse an ordinary request on a
        // technicality. Recompose, and say which of the two happened: the
        // difference is exactly the promise the interface made.
        match compose::fugue(&self.design, &l, self.tier.rules(), self.seed) {
          Ok(o) => {
            self.status =
              Some(format!("{said} — not reachable from where the piece was, so the whole fugue was rewritten ({why})"));
            self.layout = l;
            self.out = Some(o);
            self.fidelity = None;
          }
          Err(e) => {
            self.status = Some(format!("that will not fill: {e}"));
            self.out = Some(prev);
          }
        }
        return;
      }
    };
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    self.status =
      Some(format!("{said} — {} blocks rewritten in {ms:.0} ms, the rest untouched", last - first + 1));
    self.layout = l;
    self.out = Some(next);
    self.fidelity = None;
  }

  /// A span-changing edit: the piece is a different length, so it is a
  /// different piece.
  fn rebuild(&mut self, e: strip::Edit, l: Layout) {
    let t0 = web_time::Instant::now();
    match compose::fugue(&self.design, &l, self.tier.rules(), self.seed) {
      Ok(o) => {
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        let what = describe_edit(e, &l);
        self.status =
          Some(format!("{what} — every bar after it moves, so the piece was rewritten: {} bars in {ms:.0} ms", o.bars));
        self.layout = l;
        self.out = Some(o);
        self.fidelity = None;
      }
      Err(why) => self.status = Some(format!("that will not fill: {why}")),
    }
  }

  /// Open the sound card, once, when something first needs it.
  ///
  /// Failure is a message and not a panic: a machine with no output device is an
  /// ordinary machine, and everything else in this program still works on it.
  fn wake(&mut self) -> bool {
    if self.player.is_some() {
      return true;
    }
    match Player::open() {
      Ok(p) => {
        p.set_mute(self.mute);
        self.player = Some(p);
        self.no_sound = None;
        self.reload_sound();
        true
      }
      Err(e) => {
        self.no_sound = Some(e);
        false
      }
    }
  }

  /// Hand the player the music now on screen, at the tempo now set.
  ///
  /// Called after every recompose and every edit, so what is heard is what is
  /// shown. The position is not reset — an edit during playback continues from
  /// where the ear already is, which is the whole reason to edit while listening.
  fn reload_sound(&mut self) {
    let (Some(p), Some(out)) = (self.player.as_ref(), self.out.as_ref()) else { return };
    let score = std::sync::Arc::new(schedule::schedule(&out.voices, self.qpm, p.rate()));
    p.load(score.clone());
    self.sound = Some(score);
  }

  /// Put the sound back after something blocked the frame.
  ///
  /// **On the web the stream does not survive a stall, and cannot recover by
  /// itself.** `cpal`'s WebAudio backend schedules each buffer from an `onended`
  /// callback on the main thread, and its cursor synchronises with the context
  /// clock **only for the first buffer** — after that it just advances by one
  /// buffer step. Block the main thread for longer than what is already queued
  /// and every later buffer is scheduled at a time that has passed. The sound
  /// chops, and it chops for ever, because nothing ever re-synchronises. Only
  /// reloading the page fixed it, which is a fair description of a bug.
  ///
  /// A generate is half a second, so it stalls the frame by design (7.3), and
  /// the stream is therefore rebuilt around it. Dropping `cpal::Stream` closes
  /// the `AudioContext`, so this does not accumulate contexts against the
  /// browser's per-page limit — which is the thing that would have made this
  /// cure worse than the disease.
  ///
  /// Native audio runs on its own thread and none of this applies, so it keeps
  /// its stream and its continuity. That is one of the very few places this
  /// program does something different by target, and 7.5 asks for a reason
  /// rather than for uniformity.
  fn resound(&mut self) {
    let was = self.player.as_ref().map(|p| (p.is_playing(), p.position()));

    #[cfg(target_arch = "wasm32")]
    if was.is_some() {
      self.player = None; // Drop closes the AudioContext
      self.sound = None;
    }

    if was.is_some() && self.player.is_none() {
      self.wake(); // rebuilds the stream, and loads the current music into it
    } else {
      self.reload_sound();
    }

    if let (Some((playing, at)), Some(p), Some(sc)) = (was, self.player.as_ref(), self.sound.as_ref()) {
      // Clamp, because the music may have got shorter while we were away.
      p.seek(at.min(sc.samples));
      p.set_playing(playing);
    }
  }

  /// Where the ear is, in ticks. `None` when nothing has ever played.
  fn playhead(&self) -> Option<i64> {
    let p = self.player.as_ref()?;
    Some(self.sound.as_ref()?.tick_of(p.position()))
  }

  fn seek(&mut self, tick: i64) {
    if !self.wake() {
      return;
    }
    if let (Some(p), Some(sc)) = (self.player.as_ref(), self.sound.as_ref()) {
      p.seek(sc.sample_of(tick.max(0)));
    }
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
          // `Settings::reproduce` ran the search, and on the web that ran on
          // this thread — `spawn_local` is not another one.
          self.resound();
        }
        Err(Note::Cancelled) => self.status = Some("nothing opened".into()),
        Err(Note::Failed(e)) => self.status = Some(format!("could not open: {e}")),
        Err(Note::Saved(_)) => {}
      }
    }
    if let Some(r) = self.importing.take() {
      match r {
        Ok(im) => {
          let took = im.took;
          self.imported = Some(im);
          self.take_voice(took);
        }
        Err(Note::Cancelled) => self.status = Some("nothing imported".into()),
        Err(Note::Failed(e)) => self.status = Some(format!("could not import: {e}")),
        Err(Note::Saved(_)) => {}
      }
    }
  }

  /// Take voice `v` of the imported file as the subject.
  ///
  /// Everything the design needs comes from the file: the key signature, the
  /// metre, and the tonic as a **letter of that file's own signature** rather
  /// than as a pitch class, because §2.1's whole argument is that those are
  /// different things. What the file cannot supply, the message names.
  fn take_voice(&mut self, v: usize) {
    let Some(im) = self.imported.as_ref() else { return };
    let Some(voice) = im.piece.voices.get(v).cloned() else { return };
    let notes = voice.notes.iter().filter(|n| n.attack).count();
    let ticks = voice.notes.iter().map(|n| n.onset + n.dur).max().unwrap_or(0);
    let bars = ticks as f64 / im.piece.measure.max(1) as f64;
    let (name, of, no_key) = (im.name.clone(), im.of, im.piece.tonic.is_none());

    self.design = Design {
      subject: voice,
      voices: self.design.voices,
      key: im.piece.key,
      tonic: im
        .piece
        .tonic
        .and_then(|(pc, _)| contrapunctus::answer::tonic_letter(pc, &im.piece.key))
        .unwrap_or(0),
      measure: im.piece.measure,
      beat: im.piece.beat,
      compass: catalog::compass(self.design.voices),
    };
    self.chosen = None;
    if let Some(im) = self.imported.as_mut() {
      im.took = v;
    }

    let mut said = format!("{name}: {notes} notes over {bars:.1} bars");
    if of > 1 {
      said.push_str(&format!(" — voice {} of {of}", v + 1));
    }
    if bars > 8.0 {
      said.push_str(" — that is long for a subject; the whole line is the subject here");
    }
    if no_key {
      said.push_str(" — no key interpretation in the file, so the tonic was taken as C");
    }
    self.status = Some(said);
    self.compose();
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
    let mut transport = false;
    let qpm_was = self.qpm;
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
          let playing = self.player.as_ref().is_some_and(|p| p.is_playing());
          transport |= ui
            .add_enabled(self.out.is_some(), egui::Button::new(if playing { "■" } else { "▶" }))
            .on_hover_text(if playing { "Stop" } else { "Play, through the built-in synth" })
            .clicked();
          ui.add(egui::DragValue::new(&mut self.qpm).range(30..=200).suffix(" bpm"))
            .on_hover_text("Quarter notes per minute. §8.16's own figures are quoted at 76.");
        });
      });
    });

    if transport {
      let playing = self.player.as_ref().is_some_and(|p| p.is_playing());
      if playing {
        if let Some(p) = self.player.as_ref() {
          p.set_playing(false);
        }
      } else if self.wake() {
        // Starting from the end starts again from the beginning, which is what
        // pressing play after a piece has finished plainly means.
        if let Some(p) = self.player.as_ref() {
          let done = self.sound.as_ref().map(|s| s.samples).unwrap_or(0);
          if done > 0 && p.position() >= done {
            p.seek(0);
          }
          p.set_playing(true);
        }
      }
    }
    if self.qpm != qpm_was {
      // The tempo is part of the conversion, so the notes and the playhead both
      // move with it. Seeking to the tick we were on keeps the ear in the music
      // rather than at whatever sample that tick used to be.
      let was = self.playhead();
      self.reload_sound();
      if let Some(t) = was {
        self.seek(t);
      }
    }

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
    let mut seek = None;
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

      ui.add_space(4.0);
      ui.horizontal(|ui| {
        ui.label(RichText::new("PLAN").monospace().weak().small());
        ui.add_space(12.0);
        ui.label(RichText::new("hear").monospace().weak().small());
        // Silencing a voice is how anyone learns to follow one, which is the
        // whole point of a synth whose job is clarity rather than beauty.
        for v in 0..self.design.voices {
          let on = self.mute & (1 << v) == 0;
          let c = theme::voice(v, dark);
          let tag = RichText::new(format!("{}", v + 1)).color(if on { c } else { ui.visuals().weak_text_color() });
          if ui
            .selectable_label(on, tag)
            .on_hover_text(if on { "Silence this voice" } else { "Hear this voice again" })
            .clicked()
          {
            self.mute ^= 1 << v;
            if let Some(p) = self.player.as_ref() {
              p.set_mute(self.mute);
            }
          }
        }
      });
      let head = self.playhead();
      let asked = strip::Strip { out, design: &self.design, layout: &self.layout, playhead: head }.show(ui);
      edit = asked.edit;
      seek = asked.seek;

      ui.add_space(10.0);
      ui.label(RichText::new("SCORE").monospace().weak().small());
      egui::ScrollArea::both().max_height(score::height(out.voices.len()) + 12.0).show(ui, |ui| {
        let want = (out.bars as f32 * 46.0).max(ui.available_width());
        let following = self.player.as_ref().is_some_and(|p| p.is_playing());
        if let Some(t) = score::show(ui, &out.voices, &self.design.key, measure, want, head, following) {
          seek = Some(t);
        }
      });

      ui.add_space(10.0);
      ui.separator();
      ui.label(RichText::new("HOW IT TURNED OUT").monospace().weak().small());
      ui.add_space(2.0);
      report::show(ui, out, self.tier, self.advanced);

      if let Some(why) = &self.no_sound {
        ui.add_space(6.0);
        ui.colored_label(theme::warn(dark), format!("No sound: {why}"));
        ui.label(
          RichText::new("Everything else works; the score and the report do not need a sound card.")
            .weak()
            .small(),
        );
      }
      if let Some(line) = &self.status {
        ui.add_space(6.0);
        ui.label(RichText::new(line).weak().small());
      }
    });

    // Applied after the panel, not inside it: the panel holds a borrow of the
    // outcome it is drawing, and an edit replaces that outcome.
    if let Some(e) = edit {
      self.apply(e);
      self.resound();
    }
    if let Some(t) = seek {
      self.seek(t);
    }

    // Repaint while the sound is moving, because the playhead is read off the
    // audio callback and an immediate-mode frame that is not drawn does not read
    // anything. Nothing polls when nothing is playing.
    if self.player.as_ref().is_some_and(|p| p.is_playing()) {
      ui.ctx().request_repaint();
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
      self.imported = None;
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
    if ui
      .button("Import…")
      .on_hover_text(
        "A Humdrum **kern file, whose contents are taken as the subject.          Not MIDI: MIDI does not spell its pitches, and a subject that arrived          respelled would be a different subject.",
      )
      .clicked()
    {
      files::import_subject(self.importing.clone());
    }
    // Which line of a multi-voice file is the subject. Nothing in a bare
    // `**kern` file says, so the first is taken and the rest are offered rather
    // than the choice being made silently and permanently.
    let voices_in_file = self.imported.as_ref().map(|im| (im.of, im.took));
    if let Some((of, took)) = voices_in_file.filter(|(of, _)| *of > 1) {
      let mut want = None;
      ui.horizontal(|ui| {
        ui.label(RichText::new("from voice").weak().small());
        for v in 0..of {
          let empty = self.imported.as_ref().is_some_and(|im| im.piece.voices[v].notes.is_empty());
          let clicked = if empty {
            ui.add_enabled(false, egui::Button::new(format!("{}", v + 1)))
              .on_disabled_hover_text("this line has no notes in it");
            false
          } else {
            ui.selectable_label(v == took, format!("{}", v + 1)).clicked()
          };
          if clicked {
            want = Some(v);
          }
        }
      });
      if let Some(v) = want {
        self.take_voice(v);
      }
    }

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

/// What an edit did, in the words the plan strip uses for the same things.
fn describe_edit(e: strip::Edit, l: &Layout) -> String {
  let order = || l.middles.iter().map(|d| catalog::degree_name(*d)).collect::<Vec<_>>().join(" · ");
  match e {
    strip::Edit::Key(k, deg) => format!("return {} sent to {}", k + 1, catalog::degree_name(deg)),
    strip::Edit::MoveMiddle(..) => format!("returns reordered to {}", order()),
    strip::Edit::EpisodeBars(_) => format!("episodes of {} bars", l.episode_bars),
    strip::Edit::LinkBars(_) => match l.link {
      Some((_, n)) => format!("a link of {n} bars"),
      None => "no link in the exposition".to_string(),
    },
    strip::Edit::Reroll { id, .. } => {
      let n = l.rerolls.iter().find(|(k, _)| *k == id).map_or(0, |(_, n)| *n);
      format!("those bars written again (draw {})", n + 1)
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
    let brief = Layout { middles: vec![4], episode_bars: 2, link: None, ..Layout::default() };
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

  /// **Asking for one block again rewrites that block and nothing else.**
  ///
  /// The claim 4.2's reroll makes, and the one that would be easiest to fake by
  /// recomposing with a different seed — which would look almost right and
  /// differ everywhere. Checked over the whole piece, note for note, outside the
  /// block's own bars.
  #[test]
  fn a_reroll_rewrites_one_block_and_no_others() {
    let mut app = App::default();
    app.compose();
    let start = app.out.clone().expect("a fugue");
    let l0 = app.layout.clone();

    let mut rewrote = 0;
    for block in 0..start.blocks.len() {
      let id = compose::identities(&start.blocks)[block];
      let mut probe = probe_from(&start, &l0);
      probe.apply(strip::Edit::Reroll { block, id });
      let after = probe.out.as_ref().expect("still a fugue");
      if !probe.status.as_deref().unwrap_or("").contains("the rest untouched") {
        continue; // it refused and recomposed, which the next test covers
      }
      rewrote += 1;

      let (from, to) = (start.blocks[block].at, start.blocks[block].at + start.blocks[block].len);
      let outside = |v: &contrapunctus::kern::Voice| -> Vec<(i64, i64, i16, i8)> {
        v.notes
          .iter()
          .filter(|n| n.onset < from || n.onset >= to)
          .map(|n| (n.onset, n.dur, n.pitch.step, n.pitch.alter))
          .collect()
      };
      for (a, b) in start.voices.iter().zip(&after.voices) {
        assert_eq!(outside(a), outside(b), "rerolling block {block} changed notes outside it");
      }
      assert_eq!(start.bars, after.bars, "a reroll must not change the length");
      assert_eq!(probe.layout.rerolls, vec![(id, 1)], "the reroll was not recorded in the layout");
      // recorded in the layout is what makes it survive a save — spec 8
      assert_ne!(
        contrapunctus::settings::fingerprint(&start.voices),
        contrapunctus::settings::fingerprint(&after.voices),
        "rerolling block {block} changed nothing at all"
      );
    }
    assert!(rewrote > 0, "no reroll was local, so the promise was never tested");
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

  /// An imported subject becomes the design, and the design still composes.
  ///
  /// The dialog is the only part not exercised: what arrives from it is a name
  /// and some bytes, and this puts those straight into the slot the dialog would
  /// have filled. What is being checked is the half that can be wrong — that the
  /// key, the metre and the tonic come from the imported file rather than from
  /// whatever was selected before it.
  #[test]
  fn an_imported_subject_replaces_the_design() {
    let text = contrapunctus::embedded::FUGUES[10].1; // No. 11 in F major, a different key
    let piece = contrapunctus::kern::parse(text, "imported").expect("it parses");
    let took = piece.voices.iter().position(|v| !v.notes.is_empty()).unwrap_or(0);
    let of = piece.voices.len();

    let mut app = App::default();
    app.compose();
    let before = app.design.key;
    assert!(app.chosen.is_some(), "the default design comes from the catalogue");

    app.importing.put(Ok(Imported { name: "wtc1f11.krn".into(), piece, took, of }));
    app.collect();

    assert!(app.chosen.is_none(), "an imported subject is not a catalogue entry");
    assert_ne!(app.design.key, before, "the key did not come from the imported file");
    assert!(!app.design.subject.notes.is_empty(), "the subject is empty");
    // it names which line of how many it took, rather than choosing silently
    assert!(app.status.as_deref().unwrap_or("").contains("voice 1 of"), "{:?}", app.status);
    // and it says the file is long, because a whole fugue is not a subject
    assert!(app.status.as_deref().unwrap_or("").contains("long for a subject"), "{:?}", app.status);

    // and another line of the same file can be taken afterwards
    let first = contrapunctus::settings::fingerprint(std::slice::from_ref(&app.design.subject));
    app.take_voice(1);
    let second = contrapunctus::settings::fingerprint(std::slice::from_ref(&app.design.subject));
    assert_ne!(first, second, "taking a different voice gave the same subject");
    assert!(app.status.as_deref().unwrap_or("").contains("voice 2 of"), "{:?}", app.status);
    assert!(app.out.is_some(), "the second voice did not compose: {:?}", app.refused);
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
