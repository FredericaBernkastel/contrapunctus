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
use crate::synth::Mix;
use crate::task::Slot;
use crate::{compass, farm, report, score, strip, theme};
#[cfg(feature = "midi-out")]
use crate::midi;

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
  /// The design and layout the piece in `out` was actually written from.
  ///
  /// **Not the same as `design` and `layout` once a control has moved**, and the
  /// plan strip draws the *piece*, so it has to ask these. The gap is the window
  /// between changing a control and pressing Compose, which the "↻" is there to
  /// advertise — and in that window the live controls describe a piece that does
  /// not exist yet. Handing them to the strip drew a plan of one piece over the
  /// blocks of another: at four voices it drew a fourth lane over a three-voice
  /// fugue and looked up every block's identity in a thirteen-block plan against
  /// a twelve-block one, so a click in a lane asked for a rest in the wrong
  /// block, or in a voice that had never entered and was refused.
  ///
  /// It was reported as the rest gesture not working at four voices. It was the
  /// strip being lied to, and every gesture had the same fault where a control
  /// had moved — the voice count only made it visible, because that is the
  /// control that changes how many lanes there are.
  shown: Option<(Design, Layout)>,
  /// Where generating happens — spec 7.3. A worker in a browser that has them,
  /// and this thread otherwise, and it always says which.
  farm: farm::Farm,
  /// The design and layout a worker was asked with, because it answers later and
  /// the controls may have moved by then — the same reason `shown` exists, and
  /// the same fault it was written for. And when, so that a worker which never
  /// answers becomes a fallback rather than a wait with no end.
  asked: Option<(Design, Layout, web_time::Instant)>,
  /// A generate in progress **here**, a block per frame — spec 7.3.
  ///
  /// Six tenths of a second is rude on a desktop and freezes a browser tab, and
  /// the blockwise fill already had the shape to be stopped between two blocks.
  /// The previous piece stays on screen and stays playable while this runs; what
  /// the strip draws is the *new* plan with the unwritten tail faded, which is a
  /// better progress indicator than a bar because it is the thing itself.
  ///
  /// `'static` because `Tier::rules` is, and `compose::Run` owns its design.
  work: Option<compose::Run<'static>>,

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
  /// What is audible, as against what is written — which voices are silenced
  /// and how the register is balanced. Interface state and not music: none of it
  /// changes a note, and none of it goes in a settings file.
  mix: Mix,
  /// Where the score is looking and how closely. The plan strip has no
  /// equivalent and wants none: it fits the whole piece, which is its entire
  /// job, and a view that shows everything has nothing to zoom to.
  ///
  /// Driven here rather than left to the scroll area, because the wheel means
  /// *pan* over this view and a scroll area would spend it going nowhere
  /// vertically.
  view: score::View,
  /// Where the score was drawn, which is what a wheel has to be over to mean
  /// anything. Kept because a test cannot aim at a rectangle it cannot see.
  page: Option<egui::Rect>,
  /// What part of the piece the score is showing, as fractions — the plan strip
  /// draws it, so that zooming in does not lose the reader.
  seen: Option<(f32, f32)>,
  /// The system MIDI port chosen, and the connection to it — spec 6.3.
  ///
  /// **The sound card keeps running while this is open**, with every voice
  /// muted, because it is the clock: `midi::Out` is told where the playhead is
  /// and works out what changed, so it wants a position and there is already one
  /// that counts samples in an audio callback. One scheduler, two sinks.
  #[cfg(feature = "midi-out")]
  midi: Option<midi::Out>,
  /// The ports there were when the panel last looked, or why there were none.
  ///
  /// Not enumerated every frame, because each call opens a client — and **not
  /// cached when the answer is nothing**, because on the web nothing is what it
  /// says before the browser has granted Web MIDI. `midir` asks for access
  /// asynchronously and reports an empty list until the promise resolves and the
  /// reader has answered the prompt, so a panel that believed the first answer
  /// would say "no ports" for ever on a machine with two.
  #[cfg(feature = "midi-out")]
  midi_ports: Option<Result<Vec<String>, String>>,
  /// When that was asked, so that an empty answer can be asked again without
  /// asking sixty times a second.
  #[cfg(feature = "midi-out")]
  midi_asked: Option<web_time::Instant>,
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
      shown: None,
      farm: farm::Farm::start(),
      asked: None,
      #[cfg(feature = "midi-out")]
      midi: None,
      #[cfg(feature = "midi-out")]
      midi_ports: None,
      #[cfg(feature = "midi-out")]
      midi_asked: None,
      work: None,
      status: None,
      fidelity: None,
      saving: Slot::default(),
      loading: Slot::default(),
      importing: Slot::default(),
      player: None,
      no_sound: None,
      mix: Mix::default(),
      view: score::View::default(),
      page: None,
      seen: None,
      sound: None,
    }
  }
}

impl App {
  /// Start a generate. It runs a block per frame in [`App::grind`] and finishes
  /// there; nothing here blocks.
  fn compose(&mut self) {
    // Somewhere else first, if there is anywhere else. `Farm::ask` says whether
    // it took the work; `false` is a browser without workers or a desktop, and
    // both mean the same thing here — do it in the frame.
    self.work = None;
    self.asked = None;
    if self.farm.ask(&self.design, &self.layout, self.tier, self.seed) {
      self.asked = Some((self.design.clone(), self.layout.clone(), web_time::Instant::now()));
      self.refused = None;
      self.stale = false;
      self.fidelity = None;
      return;
    }
    match compose::Run::new(&self.design, &self.layout, self.tier.rules(), self.seed) {
      Ok(run) => {
        self.work = Some(run);
        self.refused = None;
      }
      Err(e) => {
        self.work = None;
        self.refused = Some(e);
      }
    }
    self.stale = false;
    self.fidelity = None;
  }

  /// Fill what fits in a frame, if a generate is running.
  ///
  /// **A time budget rather than a fixed block count**, because blocks are not
  /// the same size: an episode of one bar and an entry of four are both one
  /// call. Six milliseconds leaves the rest of a sixteen-millisecond frame for
  /// drawing, and a block that runs long simply overruns — there is no way to
  /// stop inside one, and pretending otherwise would mean a second place that
  /// writes a block.
  ///
  /// **This is necessary and not sufficient on the web.** `cpal`'s WebAudio
  /// backend queues about 85 ms and one block is tens of milliseconds, so the
  /// sound can still be crowded; 7.3 wants a worker and 6.4's stream rebuild is
  /// what stands in until there is one.
  fn grind(&mut self, ctx: &egui::Context) {
    // Whatever the worker has said since the last frame, first: it answers all
    // at once and there is nothing to do in the frame while it is working.
    if let Some((d, l, since)) = self.asked.clone() {
      // Give up before taking, so that a worker which answers nothing at all is
      // a fallback with a reason rather than a panel that says "writing" for ever.
      if farm::overdue(since.elapsed().as_secs_f64()) {
        self.asked = None;
        self.farm.give_up("it was asked and never answered");
        self.status = Some(format!(
          "the worker did not answer in {:.0} s, so this is being written in the page instead",
          farm::PATIENCE
        ));
        self.compose();
        ctx.request_repaint();
        return;
      }
      if let Some(done) = self.farm.take(&d, &l) {
        self.asked = None;
        match done {
          Ok(o) => {
            self.status = Some(format!("{} blocks written in a worker, in {:.0} ms", o.blocks.len(), o.seconds * 1000.0));
            self.out = Some(o);
            self.shown = Some((d, l));
            self.refused = None;
            self.resound();
          }
          Err(e) => self.refused = Some(e),
        }
        ctx.request_repaint();
      } else {
        // It is working and will not tell us how far along it is. A worker that
        // reported progress would have to send a message a block, which is the
        // cost the worker exists to avoid paying.
        ctx.request_repaint();
      }
      return;
    }
    if self.grind_for(0.006) {
      ctx.request_repaint();
    }
  }

  /// Run the generate to completion, blocking.
  ///
  /// What a caller wants when there is no frame to be polite to — a test, and
  /// anything that must not proceed without a finished piece. It is the same
  /// loop with the budget taken off, not a second way of generating.
  #[allow(dead_code)] // used by the tests, which are the callers with no frame
  pub fn settle(&mut self) {
    // A worker cannot be waited for from here — there is no thread to block and
    // a browser would deadlock if there were. Tests run on the desktop, where
    // `Farm::ask` always declines, so this is the whole of it; a caller that
    // reached here with a worker outstanding is told rather than hung.
    if self.asked.is_some() {
      self.status = Some("waiting on the worker; there is nothing to settle here".into());
      return;
    }
    while self.grind_for(f64::INFINITY) {}
  }

  /// Fill blocks until `budget` seconds are spent or the run ends. Returns
  /// whether one is still going.
  fn grind_for(&mut self, budget: f64) -> bool {
    if self.work.is_none() {
      return false;
    }
    let t0 = web_time::Instant::now();
    loop {
      let Some(run) = self.work.as_mut() else { return false };
      match run.step() {
        Err(e) => {
          self.refused = Some(e);
          self.work = None;
          return false;
        }
        Ok(true) if t0.elapsed().as_secs_f64() < budget => continue,
        Ok(true) => return true,
        Ok(false) => break,
      }
    }
    let Some(run) = self.work.take() else { return false };
    let (_, all) = run.progress();
    match run.finish() {
      Ok(o) => {
        self.status = Some(format!("{all} blocks written in {:.0} ms", o.seconds * 1000.0));
        self.out = Some(o);
        self.shown = Some((self.design.clone(), self.layout.clone()));
        self.refused = None;
        self.resound();
      }
      Err(e) => self.refused = Some(e),
    }
    false
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

  /// An edit that rewrites some blocks and moves none.
  ///
  /// **It rewrites from the first affected block to the end of the piece, and
  /// that is not laziness.** Refilling only the edited blocks means pinning the
  /// last of them to what the piece sounded *before* the edit, and that pin is
  /// information a settings file does not carry — so a piece reached by editing
  /// was not one the generator would produce from its own settings, and saving
  /// it and opening it again gave a different fugue. Running to the end takes no
  /// pin, and then the result is exactly what `compose::fugue` writes.
  ///
  /// What survives of 4.2's locality is the half worth having: **every bar
  /// before the edit is untouched**, which is the part already heard. The bars
  /// after it follow from the change, which is what they should do anyway — a
  /// return sent somewhere else and everything after it pretending otherwise was
  /// never the more musical answer.
  fn recolour(&mut self, e: strip::Edit, blocks: std::ops::RangeInclusive<usize>, l: Layout) {
    let Some(prev) = self.out.take() else { return };
    let first = *blocks.start();
    let last = prev.blocks.len().saturating_sub(1);
    if first > last {
      self.out = Some(prev);
      return;
    }

    let said = describe_edit(e, &l);
    let t0 = web_time::Instant::now();
    let next = match compose::refill_span(&self.design, &l, self.tier.rules(), self.seed, &prev, first, last) {
      Ok(o) => o,
      Err(why) => {
        self.status = Some(format!("{said} will not fill: {why}"));
        self.out = Some(prev);
        return;
      }
    };
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    self.status = Some(format!(
      "{said} — {} blocks rewritten in {ms:.0} ms, and every bar before them is untouched",
      last - first + 1
    ));
    self.layout = l;
    self.shown = Some((self.design.clone(), self.layout.clone()));
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
        self.shown = Some((self.design.clone(), self.layout.clone()));
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
        p.set_mix(heard(self.mix, self.to_midi()));
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
  /// The port picker — spec 6.3, whose whole insistence is that absence is
  /// stated. There are three states and they are three different sentences: no
  /// MIDI on this machine at all, MIDI but nothing to send to, and a list.
  #[cfg(feature = "midi-out")]
  fn midi_out(&mut self, ui: &mut Ui) {
    // Asked again while the answer is nothing, and left alone once it is
    // something. On the web the first answer is *always* nothing: `midir` asks
    // the browser for Web MIDI access asynchronously and reports an empty list
    // until that resolves and the reader has answered the permission prompt. A
    // panel that believed the first answer would say "no ports" for ever.
    //
    // A port can also be plugged in later and nothing announces it, so the
    // refresh button stays for the case where the answer *was* something.
    let waited = self.midi_asked.map_or(f64::INFINITY, |t| t.elapsed().as_secs_f64());
    if ask_again(self.midi_ports.as_ref(), waited) {
      self.midi_ports = Some(midi::ports());
      self.midi_asked = Some(web_time::Instant::now());
    }
    ui.horizontal(|ui| {
      ui.label(RichText::new("send to").monospace().weak().small());
      let chosen = self.midi.as_ref().map(|m| m.name().to_owned());
      match self.midi_ports.as_ref() {
        Some(Err(why)) => {
          ui.add_enabled(false, egui::Button::new("no MIDI")).on_disabled_hover_text(why.clone());
        }
        Some(Ok(names)) if names.is_empty() => {
          // Two different absences, and on the web the likelier one is that
          // nothing has been granted yet rather than that nothing is there.
          let (label, why) = if cfg!(target_arch = "wasm32") {
            (
              "asking…",
              "Web MIDI asks permission the first time and answers a moment later, so this may                become a list. Not every browser has it at all — Chromium does, and the others                vary. Nothing is sent until a port is chosen.",
            )
          } else {
            ("no ports", "This machine has MIDI but nothing to send it to.")
          };
          ui.add_enabled(false, egui::Button::new(label)).on_disabled_hover_text(why);
        }
        Some(Ok(names)) => {
          let names = names.clone();
          let mut want: Option<Option<usize>> = None;
          egui::ComboBox::from_id_salt("midi")
            .width(190.0)
            .selected_text(chosen.clone().unwrap_or_else(|| "the built-in synth".into()))
            .show_ui(ui, |ui| {
              if ui.selectable_label(chosen.is_none(), "the built-in synth").clicked() {
                want = Some(None);
              }
              for (i, name) in names.iter().enumerate() {
                if ui.selectable_label(chosen.as_deref() == Some(name.as_str()), name).clicked() {
                  want = Some(Some(i));
                }
              }
            });
          if let Some(which) = want {
            self.choose_port(which);
          }
        }
        None => {}
      }
      if ui.small_button("↻").on_hover_text("Look for ports again").clicked() {
        self.midi_ports = None;
        self.midi_asked = None;
      }
    });
    if self.midi.is_some() {
      ui.label(
        RichText::new(
          "The sound card keeps the time and says nothing; the notes go out of the port. Events            are sent once a frame, so a note can be up to about 16 ms late — the built-in synth is            the one that keeps better time and this is the one that sounds better.",
        )
        .weak()
        .small(),
      );
    }
  }

  /// Open a port, or go back to the synth. Either way, whatever was sounding on
  /// the old sink stops first — a port dropped mid-note holds it forever.
  #[cfg(feature = "midi-out")]
  fn choose_port(&mut self, which: Option<usize>) {
    if let Some(m) = self.midi.as_mut() {
      m.silence();
    }
    self.midi = None;
    let Some(i) = which else {
      self.status = Some("back to the built-in synth".into());
      self.push_mix();
      return;
    };
    match midi::Out::open(i) {
      Ok(m) => {
        self.status = Some(format!("sending to {}", m.name()));
        self.midi = Some(m);
      }
      Err(e) => self.status = Some(e),
    }
    // After, not before: the rule reads `self.midi`, and until it is set the
    // answer is the one that was wrong.
    self.push_mix();
  }

  /// Put the port where the playhead is. Once a frame, from the sound card's own
  /// sample count — see `midi.rs` for why a position rather than a stream of
  /// events, and why that is what makes a stuck note impossible.
  #[cfg(feature = "midi-out")]
  fn pump_midi(&mut self) {
    let Some(m) = self.midi.as_mut() else { return };
    let (Some(p), Some(score)) = (self.player.as_ref(), self.sound.as_ref()) else {
      m.silence();
      return;
    };
    if p.is_playing() {
      m.at(score, p.position());
    } else {
      m.silence();
    }
  }

  /// Send the mix the card should be playing. Every caller goes through here, so
  /// that "a MIDI port silences the synth" is one rule in one place rather than a
  /// thing three call sites have to remember.
  fn push_mix(&self) {
    if let Some(p) = self.player.as_ref() {
      p.set_mix(heard(self.mix, self.to_midi()));
    }
  }

  /// Whether the notes are going out of a port rather than to the card.
  fn to_midi(&self) -> bool {
    #[cfg(feature = "midi-out")]
    {
      self.midi.is_some()
    }
    #[cfg(not(feature = "midi-out"))]
    {
      false
    }
  }

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
          self.shown = Some((self.design.clone(), self.layout.clone()));
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
    self.grind(ui.ctx());
    #[cfg(feature = "midi-out")]
    self.pump_midi();
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

    let mut panelled = None;
    egui::Panel::left("controls").default_size(288.0).show(ui, |ui| {
      panelled = egui::ScrollArea::vertical().show(ui, |ui| self.controls(ui)).inner;
    });

    let mut edit = None;
    let mut seek = None;
    // Where the score ended up, so the wheel over it can be read afterwards.
    let mut viewport: Option<(egui::Rect, f32, f32, f32)> = None;
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

      // What a generate looks like while it runs. The plan strip below fills in
      // block by block from the same run, but on the very first one there is no
      // previous piece and therefore no strip, so this line is the only thing
      // saying anything is happening.
      if self.asked.is_some() {
        ui.add_space(6.0);
        ui.label(RichText::new("Writing — in a worker").strong());
        ui.spinner();
        ui.label(
          RichText::new(
            "The page is free while it works, and so is the sound: nothing here is doing the              arithmetic. What was on screen before is still on screen and still plays.",
          )
          .weak()
          .small(),
        );
        ui.add_space(6.0);
      }
      if let Some(run) = &self.work {
        let (done, all) = run.progress();
        ui.add_space(6.0);
        ui.label(RichText::new(format!("Writing — block {done} of {all}")).strong());
        ui.add(egui::ProgressBar::new(done as f32 / all.max(1) as f32).desired_height(6.0));
        ui.label(
          RichText::new(
            "A few blocks a frame, so the window stays answerable while it works. Whatever              was here before is still on screen and still plays.",
          )
          .weak()
          .small(),
        );
        ui.add_space(6.0);
      }

      // Before the piece is borrowed, because it needs all of `self` and the
      // views below hold `out` for the rest of the panel.
      #[cfg(feature = "midi-out")]
      self.midi_out(ui);

      let Some(out) = &self.out else { return };
      let measure = self.design.measure;

      ui.add_space(4.0);
      // Set by the controls below and acted on after the row closes, because
      // `push_mix` needs all of `self` and the row borrows pieces of it.
      let mut push = false;
      ui.horizontal(|ui| {
        ui.label(RichText::new("PLAN").monospace().weak().small());
        ui.add_space(12.0);
        ui.label(RichText::new("hear").monospace().weak().small());
        // Silencing a voice is how anyone learns to follow one, which is the
        // whole point of a synth whose job is clarity rather than beauty.
        for v in 0..self.design.voices {
          let on = self.mix.mute & (1 << v) == 0;
          let c = theme::voice(v, dark);
          let tag = RichText::new(format!("{}", v + 1)).color(if on { c } else { ui.visuals().weak_text_color() });
          if ui
            .selectable_label(on, tag)
            .on_hover_text(if on { "Silence this voice" } else { "Hear this voice again" })
            .clicked()
          {
            self.mix.mute ^= 1 << v;
            push = true;
          }
        }

        // The one-knob equalizer. A tilt is the simplest useful shape an
        // equalizer has, and it is the shape this particular complaint has: the
        // ear hears high notes as louder than low ones at the same amplitude, so
        // what wants adjusting is the slope across the register and not a set of
        // bands. A listener reported it; the default came from that report.
        ui.add_space(12.0);
        ui.label(RichText::new("balance").monospace().weak().small());
        let slider = egui::Slider::new(&mut self.mix.tilt, 0.0..=6.0).suffix(" dB/8ve").fixed_decimals(1);
        if ui
          .add_sized([132.0, 18.0], slider)
          .on_hover_text(
            "How much the high voices are quietened against the low ones. Equal amplitude is not \
             equal loudness — the ear is far more sensitive high up — so at zero the soprano \
             dominates. Three is where this landed by ear.",
          )
          .changed()
        {
          push = true;
        }
      });
      if push {
        self.push_mix();
      }
      let head = self.playhead();
      let seen = self.seen.filter(|(a, b)| *b - *a < 0.999);
      let writing = self.work.as_ref().map(|r| (r.blocks(), r.progress().0));
      // **The design and layout the piece came from**, not the ones the controls
      // are showing — see `App::shown`. A strip drawing one piece from another's
      // plan is a strip whose every gesture lands in the wrong place.
      let (design, layout) = match &self.shown {
        Some((d, l)) => (d, l),
        None => (&self.design, &self.layout),
      };
      let asked = strip::Strip { out, design, layout, playhead: head, window: seen, writing }.show(ui);
      edit = asked.edit;
      seek = asked.seek;

      ui.add_space(10.0);
      ui.horizontal(|ui| {
        ui.label(RichText::new("SCORE").monospace().weak().small());
        ui.label(RichText::new("wheel to pan · ctrl and wheel to zoom").weak().small());
        if self.view != score::View::default()
          && ui.small_button("fit").on_hover_text("Back to the whole piece").clicked()
        {
          self.view = score::View::default();
        }
      });

      let window = ui.available_width();
      let bars = out.bars as f32;
      let content = self.view.content(bars, window);

      // Keep the playhead on the page while the sound moves. Done here rather
      // than by `scroll_to_rect` inside the score, because two things setting
      // the offset is two things disagreeing about it.
      if self.player.as_ref().is_some_and(|p| p.is_playing()) {
        if let Some(t) = head {
          let at = (t as f32 / compose::length(&out.blocks).max(1) as f32) * content;
          self.view.keep_in_sight(at, bars, window);
        }
      }
      self.view.pan(0.0, bars, window);

      let scrolled = egui::ScrollArea::horizontal()
        // The wheel belongs to this application over this view: it pans, and
        // with ctrl it zooms. The bar still drags, because taking that away
        // would be taking something and giving nothing.
        .scroll_source(egui::scroll_area::ScrollSource {
          scroll_bar: true,
          drag: egui::scroll_area::DragScroll::OnTouch,
          mouse_wheel: false,
        })
        .scroll_offset(egui::Vec2::new(self.view.look, 0.0))
        .max_height(score::height(out.voices.len()) + 12.0)
        .show(ui, |ui| {
          let sheet = score::Sheet {
            voices: &out.voices,
            key: &self.design.key,
            measure,
            beat: self.design.beat,
            width: content,
            playhead: head,
          };
          sheet.show(ui)
        });
      if let Some(t) = scrolled.inner {
        seek = Some(t);
      }
      self.view.look = scrolled.state.offset.x;
      // The wheel is read after the view is drawn, because the rect it has to be
      // over is what the drawing returns. One frame of lag, and scrolling
      // repaints anyway.
      viewport = Some((scrolled.inner_rect, content, window, bars));
      self.page = Some(scrolled.inner_rect);
      // For the strip, next frame: it is drawn above this and cannot know yet.
      self.seen = Some((self.view.look / content.max(1.0), (self.view.look + window) / content.max(1.0)));

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

    // **Wheel to pan, ctrl and wheel to zoom**, over the score and nowhere else.
    // The plan strip fits the whole piece by design and has nothing to zoom to,
    // so the wheel there is left to the page.
    //
    // Read after the view is drawn, because the rectangle the pointer must be
    // over is what the drawing returns — but in the same frame, so there is no
    // lag in it.
    //
    // **The zoom comes from `zoom_delta` and not from the scroll delta**, which
    // is not a preference. When a wheel event carries egui's zoom modifier, egui
    // puts the wheel into `zoom_factor_delta` and leaves `smooth_scroll_delta`
    // at zero — so reading the scroll and the ctrl key separately, as this did
    // at first, gives a zoom branch that can never run while panning goes on
    // working. It also means a trackpad pinch arrives here for nothing, since
    // `zoom_delta` prefers a touch gesture when there is one.
    if let Some((rect, _content, window, bars)) = viewport {
      let (scroll, zoom, at) = ui.input(|i| (i.smooth_scroll_delta, i.zoom_delta(), i.pointer.hover_pos()));
      if let Some(p) = at.filter(|p| rect.contains(*p)) {
        if zoom != 1.0 {
          self.view.zoom_about(p.x - rect.left(), zoom, bars, window);
          ui.ctx().request_repaint();
        } else if scroll != egui::Vec2::ZERO {
          // A score is a horizontal thing, so a vertical wheel moves along it.
          // The horizontal delta counts too, for whoever has a wheel or a
          // trackpad that sends one.
          self.view.pan(-scroll.y - scroll.x, bars, window);
          ui.ctx().request_repaint();
        }
      }
    }

    // Applied after the panel, not inside it: the panel holds a borrow of the
    // outcome it is drawing, and an edit replaces that outcome.
    if let Some(e) = edit.or(panelled) {
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
  fn controls(&mut self, ui: &mut Ui) -> Option<strip::Edit> {
    let mut changed = false;
    // An edit the panel asked for, applied by the caller once the panel is done
    // drawing — the same one the strip's edits go through.
    let mut from_panel: Option<strip::Edit> = None;

    group(ui, "THE SUBJECT");
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
      // Four is offered now, and it was not: readme §8.17 measured the search's
      // wall against the voices it must *choose* rather than the voices sounding,
      // and a rest takes one out of the choosing. What four voices needs is a
      // rest wherever a block would otherwise have three free, so asking for four
      // sets one — `rests_that_fit` — rather than handing over a refusal.
      for n in [2usize, 3, 4] {
        if ui.selectable_label(self.design.voices == n, format!("{n}")).clicked() {
          self.design.voices = n;
          // The compass has one range per voice, so it moves with the count.
          self.design.compass = catalog::compass(n);
          // A rest pattern that was legal at three voices is not at four, and one
          // written for four is merely redundant at three, so it is recomputed
          // either way rather than carried across.
          self.layout.rests = vec![];
          self.layout.rests = compose::rests_that_fit(&self.design, &self.layout);
          changed = true;
        }
      }
    });
    if self.design.voices > 3 {
      ui.label(
        RichText::new(
          "Four voices state the subject; three sound at once. The exact search can choose two            voices at a time, so one always rests — click a faint bar in the plan to move which.            Four sounding together is what §9's solver is for.",
        )
        .weak()
        .small(),
      );
    }

    ui.add_space(8.0);
    labelled(ui, "Times the subject comes back", "Layout::middles.len()");
    let mut returns = self.layout.middles.len();
    if ui
      .add(egui::Slider::new(&mut returns, 1..=6).show_value(true))
      .on_hover_text(
        "Statements of the subject after the exposition — middle entries, in the usual name for them.          §8.15 measured a median of three across the book, and a range of none to nine.",
      )
      .changed()
    {
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
      group(ui, "COMPASS");
      labelled(ui, "Where each voice may go", "Design::compass");
      // The key is copied out because the widget takes the compass mutably and
      // both live in `design`. Seven bytes, and it saves a split borrow.
      let key = self.design.key;
      let least = compass::least_for(&self.design.subject);
      if (compass::Compass { ranges: &mut self.design.compass, key, least }).show(ui) {
        changed = true;
      }

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

      let mut close = self.layout.close_at_home;
      if ui.checkbox(&mut close, "Close at home").changed() {
        from_panel = Some(strip::Edit::CloseAtHome(close));
      }

      ui.add_space(12.0);
      // The same three edits the plan strip offers, and **through the same
      // path**: `apply` takes them and the piece changes at once. They used to
      // set the layout directly and wait for Compose, which meant taking a
      // return out here and taking one out there did visibly different things.
      group(ui, "MIDDLES");
      for i in 0..self.layout.middles.len() {
        ui.horizontal(|ui| {
          ui.label(RichText::new(format!("{}.", i + 1)).monospace().weak());
          let mut deg = self.layout.middles[i];
          egui::ComboBox::from_id_salt(("deg", i))
            .width(96.0)
            .selected_text(catalog::degree_name(deg))
            .show_ui(ui, |ui| {
              for d in 0..7i16 {
                if ui.selectable_value(&mut deg, d, catalog::degree_name(d)).clicked() {
                  from_panel = Some(strip::Edit::Key(i, d));
                }
              }
            });
          if ui.small_button("✕").on_hover_text("Take this return out").clicked() {
            from_panel = Some(strip::Edit::RemoveMiddle(i));
          }
        });
      }
      if ui.small_button("＋ add a return").clicked() {
        from_panel = Some(strip::Edit::AddMiddle { at: self.layout.middles.len(), degree: 4 });
      }

      ui.add_space(12.0);
      group(ui, "SEARCH");

      labelled(ui, "Let the search choose who rests", "Layout::texture");
      let mut drawn = self.layout.texture == compose::Texture::Drawn;
      if ui
        .checkbox(&mut drawn, "")
        .on_hover_text(
          "Every way a block could rest its voices is a search of its own, and the fill is drawn            from all of them at once, in proportion to how much music each admits. That invents no            rule — it is the same draw the notes come from.",
        )
        .changed()
      {
        self.layout.texture = if drawn { compose::Texture::Drawn } else { compose::Texture::Given };
        changed = true;
      }
      ui.label(
        RichText::new(
          "Off, and measured rather than cautious: a pattern with one more voice playing admits            thousands of times more music, so the draw returns the fullest texture the search can            afford and almost never anything thinner. What it does change is *which* voice rests            where one has to — readme §8.17.",
        )
        .weak()
        .small(),
      );
      ui.add_space(8.0);
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

    // **Where the work happens, said out loud.** A worker that quietly failed to
    // start and a worker that quietly worked look identical from a chair, and
    // the difference is the whole feature — spec 7.3.
    ui.label(RichText::new(format!("writing happens {}", self.farm.where_it_runs())).weak().small());
    if self.farm.stalls_the_sound() {
      ui.label(
        RichText::new(
          "so a piece already playing will break up while a new one is written — 6.4 rebuilds            the stream afterwards, which is a cure for the symptom.",
        )
        .weak()
        .small(),
      );
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
    from_panel
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
    strip::Edit::AddMiddle { at, .. } => format!("a return added at {} — now {}", at + 1, order()),
    strip::Edit::RemoveMiddle(_) => match l.middles.len() {
      0 => "no returns at all, which §8.15 found in the book too".to_string(),
      _ => format!("a return taken out — now {}", order()),
    },
    strip::Edit::Turn { id, .. } => {
      let n = l.turns.iter().find(|(k, _)| *k == id).map_or(0, |(_, n)| *n);
      match n {
        0 => "the voices back where they were".to_string(),
        n => format!("that block and the rest moved {} {}", n.abs(), if n.abs() == 1 { "voice" } else { "voices" }),
      }
    }
    strip::Edit::Rest { id, voice, .. } => {
      let resting = l.rests.iter().find(|(k, _)| *k == id).is_some_and(|(_, vs)| vs.contains(&voice));
      format!("voice {} {} there", voice + 1, if resting { "rests" } else { "plays again" })
    }
    strip::Edit::CloseAtHome(true) => "closing at home".to_string(),
    strip::Edit::CloseAtHome(false) => "stopping after the last return".to_string(),
  }
}

/// Whether to look for MIDI ports again — spec 6.3, and the rule that an empty
/// answer is not one.
///
/// **On the web the first answer is always nothing.** `midir` asks the browser
/// for Web MIDI access asynchronously and reports an empty list until that
/// resolves and the reader has answered the permission prompt, which is at best
/// a moment later and at worst never. A panel that took the first answer said
/// "no ports" for ever on a machine with two, which is how it was reported.
///
/// So nothing is asked again, a reason is asked again more slowly — a browser
/// with no Web MIDI at all will keep saying so and there is no point hurrying it
/// — and a list is left alone, because that is an answer.
#[cfg(feature = "midi-out")]
fn ask_again(ports: Option<&Result<Vec<String>, String>>, waited: f64) -> bool {
  match ports {
    None => true,
    Some(Ok(names)) if names.is_empty() => waited > 1.0,
    Some(Err(_)) => waited > 2.0,
    Some(Ok(_)) => false,
  }
}

/// What the sound card should actually play, given what is audible and where the
/// notes are going — spec 6.3.
///
/// **The card keeps running when a MIDI port is open, and says nothing.** It is
/// the clock: `midi::Out` is told where the playhead is, and the only thing that
/// counts samples is the audio callback. So the stream stays up and every voice
/// is muted, which is a different thing from stopping it and the reason the
/// playhead still moves.
///
/// A free function because it is a rule and not a step, and because the rule was
/// **written down and never applied** — `midi.rs` said "with every voice muted"
/// from the first commit and three `set_mix` calls sent `self.mix` unchanged, so
/// both sinks played at once. A sentence in a doc comment is not an
/// implementation, and the way to tell the difference is to give it somewhere to
/// be tested.
fn heard(mix: Mix, to_midi: bool) -> Mix {
  if to_midi {
    Mix { mute: !0, ..mix }
  } else {
    mix
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
    // With the clefs installed, so the score's real drawing path is the one that
    // runs — `__run_test_ui` would install an empty font set instead.
    let ctx = egui::Context::default();
    crate::glyph::install(&ctx);
    for _ in 0..2 {
      let mut once = Some(&mut app);
      ctx
        .run_ui(Default::default(), |ui| {
          if let Some(app) = once.take() {
            app.draw(ui);
          }
        })
        .drop_without_applying_deltas();
    }
    // The generate runs a block per frame now, and this paints a handful of
    // frames rather than however many the piece needs. `settle` is the same
    // loop with the frame budget taken off.
    app.settle();
    assert!(app.out.is_some(), "the default settings did not produce a fugue: {:?}", app.refused);
  }

  /// **An empty port list is not an answer, and is asked again.**
  ///
  /// Reported as no dropdown at all on the web. Two faults: the browser build
  /// did not ask for the `midi-out` feature, and the panel cached the first
  /// answer — which on the web is *always* nothing, because `midir` requests Web
  /// MIDI asynchronously and reports an empty list until the reader has answered
  /// a permission prompt. Either alone was enough to show no ports for ever.
  #[cfg(feature = "midi-out")]
  #[test]
  fn an_empty_port_list_is_asked_again_and_a_real_one_is_not() {
    let none: Option<&Result<Vec<String>, String>> = None;
    assert!(ask_again(none, 0.0), "never having looked is a reason to look");

    let empty = Ok(vec![]);
    assert!(!ask_again(Some(&empty), 0.1), "asking sixty times a second is not asking again, it is spinning");
    assert!(ask_again(Some(&empty), 1.5), "an empty list was believed, which is the reported bug");

    let refused: Result<Vec<String>, String> = Err("no MIDI on this system".into());
    assert!(!ask_again(Some(&refused), 1.5), "a browser with no Web MIDI does not need hurrying");
    assert!(ask_again(Some(&refused), 2.5), "and is still worth one more try, since nothing else will");

    let found = Ok(vec!["Microsoft GS Wavetable Synth".to_string()]);
    assert!(!ask_again(Some(&found), 600.0), "a list is an answer and re-opening a client is not free");
  }

  /// **A MIDI port silences the card, and the card keeps running.**
  ///
  /// Reported: both sinks played at once. `midi.rs` had said "with every voice
  /// muted" from its first line and nothing did it — three `set_mix` calls sent
  /// the mix unchanged, so choosing a port added a second sound instead of
  /// moving the notes to it. A sentence in a doc comment is not an
  /// implementation.
  ///
  /// Muted rather than stopped, which is the other half: the card is the clock,
  /// `midi::Out` is told where the playhead is, and a stopped stream is a
  /// playhead that does not move.
  #[test]
  fn a_midi_port_silences_the_card_without_stopping_it() {
    let quiet = Mix { mute: 0b010, tilt: 4.5 };
    // to the card, what the listener asked for and nothing else
    assert_eq!(heard(quiet, false).mute, 0b010, "the synth stopped hearing what the mute buttons said");
    assert_eq!(heard(quiet, true).tilt, quiet.tilt, "a port changed the register balance, which is not its business");

    // to a port, everything silent — and *every* voice, not only the ones the
    // listener had already muted
    let to_port = heard(quiet, true);
    for v in 0..8 {
      assert!(to_port.mute & (1 << v) != 0, "voice {v} still sounds on the card while a port is open");
    }

    // The listener's own mutes survive a port being opened and closed, because
    // the silencing happens **on the way out** and never to the stored value —
    // `push_mix` always reads `self.mix`, which no port writes to. Chaining
    // `heard` into itself is not that property and does not hold: it is not
    // invertible and was never meant to be.
    let app = App { mix: quiet, ..App::default() };
    assert_eq!(heard(app.mix, true).mute, !0, "a port did not silence the card");
    assert_eq!(app.mix.mute, 0b010, "opening a port changed what the listener had asked for");
    assert_eq!(heard(app.mix, false).mute, 0b010, "closing it did not give them back what they asked for");
  }

  /// **The strip draws the piece, not the controls** — and every gesture in it
  /// lands where the piece is.
  ///
  /// Reported as manual resting not working once four voices were enabled. It was
  /// not the rest gesture: between changing a control and pressing Compose the
  /// panel described a piece that did not exist yet, and the strip was handed
  /// those controls rather than the ones the piece on screen came from. At four
  /// voices that drew a fourth lane over a three-voice fugue — a lane whose voice
  /// had entered nowhere, so a click in it was refused — and looked every block's
  /// identity up in a thirteen-block plan against a twelve-block one, so the
  /// three real lanes asked for rests in the wrong blocks.
  ///
  /// The voice count only made it visible. Every gesture had it, for any control.
  #[test]
  fn the_strip_is_given_the_piece_it_is_drawing() {
    let mut app =
      App { layout: Layout { middles: vec![4], episode_bars: 2, ..Layout::default() }, ..App::default() };
    app.compose();
    app.settle();
    let made = app.out.clone().expect("a three-voice fugue");

    // exactly what the voice picker does, and no Compose after it
    app.design.voices = 4;
    app.design.compass = catalog::compass(4);
    app.layout.rests = compose::rests_that_fit(&app.design, &app.layout);
    assert!(app.stale || app.shown.is_some(), "the piece is stale and nothing says so");

    let (design, layout) = app.shown.clone().expect("the piece remembers what wrote it");
    // the whole of the fix: what the strip is handed is *not* what the controls
    // say, and a `shown` that merely aliased the live values would pass every
    // assertion below while reproducing the bug exactly
    assert_ne!(design.voices, app.design.voices, "`shown` is following the controls rather than the piece");
    assert_eq!(design.voices, made.voices.len(), "the strip would draw the wrong number of lanes");
    assert_eq!(
      compose::resting(&design, &layout).len(),
      made.blocks.len(),
      "the strip would ask about a different number of blocks than it draws"
    );
    assert_eq!(
      compose::identities_of(&design, &layout).len(),
      made.blocks.len(),
      "the strip would name blocks by identities from another plan"
    );

    // and once it is composed, the two agree again
    app.compose();
    app.settle();
    let now = app.out.clone().unwrap_or_else(|| panic!("four voices refused: {:?}", app.refused));
    let (design, layout) = app.shown.clone().expect("still remembered");
    assert_eq!(design.voices, 4);
    assert_eq!(compose::resting(&design, &layout).len(), now.blocks.len());
    assert!(!layout.rests.is_empty(), "four voices with no rests would not have composed");
  }

  /// **Asking for four voices produces a fugue, not a refusal.**
  ///
  /// The button was disabled and said so, and readme §8.17 is why it is not any
  /// more: the search's wall is on the voices it must *choose*, and a rest takes
  /// one out of the choosing. So choosing four also sets a rest pattern —
  /// `rests_that_fit` — rather than handing over a refusal nobody can act on,
  /// which is the trap 4.4 names.
  ///
  /// Through the panel rather than the library, because the thing that could
  /// break is the wiring: a voice count that changes without the pattern
  /// following it composes at three and refuses at four.
  #[test]
  fn asking_for_four_voices_gives_four_voices() {
    let mut app =
      App { layout: Layout { middles: vec![4], episode_bars: 2, ..Layout::default() }, ..App::default() };

    for n in [3usize, 4] {
      app.design.voices = n;
      app.design.compass = catalog::compass(n);
      app.layout.rests = vec![];
      app.layout.rests = compose::rests_that_fit(&app.design, &app.layout);
      app.compose();
      app.settle();
      let out = app.out.clone().unwrap_or_else(|| panic!("{n} voices refused: {:?}", app.refused));
      assert_eq!(out.voices.len(), n);
      assert!(out.verdict.exposition_covers_the_voices, "{n} voices: the exposition does not cover them");
      for (v, voice) in out.voices.iter().enumerate() {
        assert!(!voice.notes.is_empty(), "{n} voices: voice {v} never sounds");
      }
      // three voices need no rests at all, four need some — which is the whole
      // difference the button now has to carry
      assert_eq!(app.layout.rests.is_empty(), n <= 3, "{n} voices: rests were {:?}", app.layout.rests);
    }
  }

  /// A refusal for the one reason a rest would cure says so. Otherwise four
  /// voices without a pattern is a dead end rather than a limit, and §9's
  /// disabled button was at least honest about being one.
  #[test]
  fn a_refusal_about_free_voices_names_its_own_cure() {
    let mut app = App::default();
    app.design.voices = 4;
    app.design.compass = catalog::compass(4);
    app.layout = Layout { middles: vec![4], episode_bars: 2, ..Layout::default() };
    app.layout.rests = vec![]; // deliberately none
    app.compose();
    app.settle();
    let why = app.refused.clone().expect("four voices with nobody resting must refuse");
    assert!(why.contains("Rest one in this block"), "the refusal does not say what to do: {why}");
  }

  /// **Every compass the widget can produce still composes.**
  ///
  /// The drag can put the voices anywhere: all three inside one octave, the bass
  /// above the soprano, or three boxes that never meet. None of those is a
  /// sensible fugue and all of them are two seconds of dragging away, so the
  /// question is whether the search survives them. It does — which is why the
  /// widget clamps only the two things that are not arrangements but errors: an
  /// end passing its partner, and a bound leaving the staff.
  ///
  /// Written after the inverted case turned out to **panic** in the library
  /// rather than refuse. `realise::a_compass_the_wrong_way_round_is_an_error_and_not_a_panic`
  /// is that fix; this is the interface end of the same finding.
  #[test]
  fn any_compass_the_drag_can_reach_still_composes() {
    let cat = catalog::load();
    let s = &cat.subjects[0];
    let brief = Layout { middles: vec![4], episode_bars: 2, link: None, ..Layout::default() };
    let (floor, ceil) = (compass::FLOOR, compass::CEIL);
    let arrangements = [
      ("the default", vec![(33, 45), (28, 40), (21, 33)]),
      ("all three in one octave", vec![(28, 35), (28, 35), (28, 35)]),
      ("upside down: the bass on top", vec![(21, 33), (28, 40), (33, 45)]),
      ("three that never meet", vec![(38, 45), (28, 35), (14, 21)]),
      ("squeezed to the floor", vec![(36, 43), (31, 38), (24, 31)]),
      ("against both stops", vec![(ceil - 7, ceil), (28, 40), (floor, floor + 7)]),
    ];
    let mut refused = vec![];
    for (what, cs) in arrangements {
      // First that the case is one the widget could actually reach, so this
      // never quietly becomes a test of compasses no drag can produce.
      for &(lo, hi) in &cs {
        assert!(hi - lo >= 7, "{what}: {lo}..{hi} is narrower than the widget's floor");
        assert!(lo >= floor && hi <= ceil, "{what}: {lo}..{hi} is off the staff the widget draws");
      }
      let mut d = s.design(3);
      d.compass = cs;
      if let Err(e) = compose::fugue(&d, &brief, Tier::Full.rules(), 0x5EED) {
        refused.push(format!("{what}: {e}"));
      }
    }
    assert!(refused.is_empty(), "a compass the drag can reach will not compose:
  {}", refused.join("
  "));
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

  /// **Every edit leaves the bars before it alone, and produces a piece the
  /// settings reproduce.**
  ///
  /// Two claims and the second is the one that was false. A settings file
  /// records the design, the layout, the tier and the seed, and section 8
  /// promises that loading one gives the same fugue note for note — so what is
  /// on screen has to be what `compose::fugue` writes from those. It was not:
  /// an edit refilled only the blocks it touched and pinned the last of them to
  /// what the piece sounded *before* the edit, and that pin is history the file
  /// does not carry. Saving an edited fugue and opening it reported a mismatch
  /// between engine 0.1.0 and engine 0.1.0.
  ///
  /// So an edit now rewrites to the end of the piece, and this asserts both
  /// halves of what that buys: the bars before are untouched, and the whole
  /// thing regenerates from its own settings.
  #[test]
  fn every_edit_keeps_the_past_and_stays_reproducible() {
    let mut app = App::default();
    app.compose();
    app.settle();
    let start = app.out.clone().expect("a fugue");
    let l0 = app.layout.clone();
    let ids = compose::identities_of(&app.design, &l0);

    let mut tried = 0;
    let mut edits: Vec<strip::Edit> = vec![strip::Edit::MoveMiddle(0, 2), strip::Edit::MoveMiddle(2, 0)];
    edits.extend((0..l0.middles.len()).flat_map(|k| (0..7i16).map(move |d| strip::Edit::Key(k, d))));
    edits.extend((0..start.blocks.len()).map(|b| strip::Edit::Reroll { block: b, id: ids[b] }));
    edits.extend((0..=l0.middles.len()).map(|at| strip::Edit::AddMiddle { at, degree: 4 }));
    edits.extend((0..l0.middles.len()).map(strip::Edit::RemoveMiddle));
    edits.push(strip::Edit::CloseAtHome(false));

    for e in edits {
      let mut probe = probe_from(&start, &l0);
      probe.apply(e);
      let Some(after) = probe.out.as_ref() else { continue };
      if probe.layout == l0 {
        continue; // a no-op, or it refused and left everything alone
      }
      tried += 1;

      // 1. the piece is what its own settings produce
      let fresh = compose::fugue(&probe.design, &probe.layout, probe.tier.rules(), probe.seed)
        .expect("the settings must generate something");
      assert_eq!(
        contrapunctus::settings::fingerprint(&after.voices),
        contrapunctus::settings::fingerprint(&fresh.voices),
        "{e:?} left a piece its own settings do not reproduce"
      );

      // 2. and the bars before the edit did not move
      let Some(range) = e.touches(&probe.design, &probe.layout) else { continue };
      let at = start.blocks[*range.start()].at;
      let before = |o: &Outcome, v: usize| -> Vec<(i64, i64, i16, i8)> {
        o.voices[v]
          .notes
          .iter()
          .filter(|n| n.onset < at)
          .map(|n| (n.onset, n.dur, n.pitch.step, n.pitch.alter))
          .collect()
      };
      for v in 0..probe.design.voices {
        assert_eq!(before(&start, v), before(after, v), "{e:?} moved a bar before the edit");
      }
    }
    assert!(tried > 20, "only {tried} edits actually applied, so this proves little");
  }

  /// **A piece with no returns, and a piece that does not close at home.**
  ///
  /// The handle row can reach both, so both have to be shapes the generator
  /// accepts rather than states an interface can get stuck in. §8.15 measured a
  /// range of nought to nine returns across the book, so an empty journey is
  /// something the book has; a fugue that stops after its last return is not,
  /// and it is offered anyway because 4.2 lists the close as a parameter and a
  /// parameter with one legal value is not one.
  #[test]
  fn the_shapes_the_handles_can_reach_all_compose() {
    let d = catalog::load().subjects[1].design(3);
    let shapes = [
      ("no returns", Layout { middles: vec![], ..Layout::default() }),
      ("no close", Layout { close_at_home: false, ..Layout::default() }),
      ("neither", Layout { middles: vec![], close_at_home: false, ..Layout::default() }),
      ("nine returns", Layout { middles: vec![4, 5, 3, 1, 6, 2, 4, 5, 3], ..Layout::default() }),
    ];
    for (what, l) in shapes {
      let out = compose::fugue(&d, &l, Tier::Full.rules(), 0x5EED)
        .unwrap_or_else(|e| panic!("{what} did not compose: {e}"));
      assert!(out.bars > 0, "{what} came out empty");
      assert!(!out.blocks.is_empty(), "{what} has no blocks");
    }
  }

  /// **The wheel reaches the score, with ctrl and without it.**
  ///
  /// This is the layer the arithmetic tests do not cover, and it is where the
  /// bug was: `score::View` was right all along and got no wheel to act on.
  /// egui moves a ctrl-modified wheel into `zoom_factor_delta` and leaves
  /// `smooth_scroll_delta` at zero, so reading the scroll and the ctrl key
  /// separately gave a zoom branch that could never run. Panning worked, which
  /// is what made it look like a zoom problem rather than an input one.
  ///
  /// Synthetic events through the real path, because the mistake was in the
  /// path.
  #[test]
  fn the_wheel_pans_and_ctrl_and_the_wheel_zooms() {
    use egui::{Event, Modifiers, MouseWheelUnit, Pos2, TouchPhase, Vec2};

    let ctx = egui::Context::default();
    crate::glyph::install(&ctx);
    let mut app = App::default();

    let screen = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1200.0, 900.0));
    // **The clock has to move.** egui smooths the wheel by the frame time, and
    // at a `dt` of nought the smoothing applies nought — a synthetic frame that
    // never advances receives every event and acts on none of them.
    let clock = std::cell::Cell::new(1.0f64);
    let frame = |app: &mut App, events: Vec<Event>| {
      clock.set(clock.get() + 1.0 / 60.0);
      let input =
        egui::RawInput { screen_rect: Some(screen), time: Some(clock.get()), events, ..Default::default() };
      let mut once = Some(app);
      ctx
        .run_ui(input, |ui| {
          if let Some(app) = once.take() {
            app.draw(ui);
          }
        })
        .drop_without_applying_deltas();
    };
    // The modifiers that matter travel on the wheel event itself: egui decides
    // whether a wheel is a zoom from `wheel.modifiers`, not from the frame's.
    let wheel = |at: Pos2, dy: f32, modifiers: Modifiers| {
      vec![
        Event::PointerMoved(at),
        Event::MouseWheel {
          unit: MouseWheelUnit::Point,
          delta: Vec2::new(0.0, dy),
          phase: TouchPhase::Move,
          modifiers,
        },
      ]
    };

    // one frame to start the generate, then finish it, then a frame to lay the
    // score out — there is no score to aim a wheel at until there is a piece
    frame(&mut app, vec![]);
    app.settle();
    frame(&mut app, vec![]);
    let page = app.page.expect("the score was drawn");
    assert!(page.width() > 100.0, "the score came out {page:?}");
    let at = page.center();

    // zoom in far enough that there is somewhere to pan to
    // `Modifiers::COMMAND` is what egui matches its zoom modifier against.
    let ctrl = Modifiers::COMMAND;
    let was = app.view.zoom;
    for _ in 0..8 {
      frame(&mut app, wheel(at, 40.0, ctrl));
    }
    assert!(app.view.zoom > was, "ctrl and the wheel did not zoom: {was} is still {}", app.view.zoom);

    // and the plain wheel moves along the piece
    let look = app.view.look;
    for _ in 0..8 {
      frame(&mut app, wheel(at, -40.0, Modifiers::default()));
    }
    assert!(app.view.look > look, "the wheel did not pan: {look} is still {}", app.view.look);

    // a wheel that is nowhere near the score changes nothing
    let (zoom, look) = (app.view.zoom, app.view.look);
    frame(&mut app, wheel(Pos2::new(20.0, 20.0), 60.0, ctrl));
    assert_eq!(app.view.zoom, zoom, "a wheel over the side panel zoomed the score");
    assert_eq!(app.view.look, look, "a wheel over the side panel panned the score");
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
    app.settle();
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
    app.settle();
    // **A piece or a stated reason, and never neither.**
    //
    // The assertion that used to be here was that `out` is `Some`, and it
    // passed for the wrong reason: this file is a whole fugue, its lines span a
    // sixteenth, and every one of them hits §2.7's wall on the first block. What
    // `out` held was the piece the *previous* import left behind, while
    // `refused` held this one's explosion — both true at once, and only one of
    // them about the voice just taken. Keeping the old piece and saying why is
    // the right behaviour and the panel shows the refusal in warning colour; the
    // test was simply reading the wrong half of it.
    assert!(
      app.out.is_some() || app.refused.is_some(),
      "taking a voice neither wrote a piece nor said why not"
    );
    if let Some(why) = &app.refused {
      assert!(why.contains("§2.7") || why.contains("state explosion"), "refused for an unexpected reason: {why}");
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
