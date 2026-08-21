//! Each voice's compass, drawn on a staff and dragged there — spec 3.3.
//!
//! `Design::compass` is one `(low, high)` pair of diatonic steps per voice, and
//! it is the only part of the design that is *geometry*: the realiser's domain
//! for a free voice is literally `compass.0..=compass.1` (`realise::domain`), so
//! these two numbers decide where every note that was not given can go. A pair
//! of spinners would state them. A staff shows them, and shows the thing the
//! numbers are actually about — **where the voices overlap**, which is what
//! decides whether they have room to cross and room to avoid one another.
//!
//! So the two staves are drawn once and the voices laid over them as bars, the
//! way an orchestration chart draws an instrument's range, rather than each
//! voice getting a staff of its own. Side by side on one pitch axis is the only
//! arrangement in which the overlap is visible at all.
//!
//! Placement is free, for §2.1's reason: `Pitch::step` *is* the staff position,
//! so a bound at step 33 sits on the line step 33 sits on and there is nothing
//! to convert. The clefs come from `crate::glyph`, at this file's smaller staff.

use egui::{Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};

use crate::{glyph, theme};

/// Pixels per diatonic step. Half a staff space, as on any staff — smaller than
/// the score's because this one shows six octaves at once and has to fit a
/// panel.
const STEP: f32 = 3.5;

/// C1 and C7: the window drawn, and the bounds a compass may not leave.
///
/// The same number does both jobs on purpose. A window that grew to follow a
/// drag would move the staff under the hand doing the dragging, and a bound
/// outside the window would be a bound with no handle to take hold of. Six
/// octaves is wider than any voice in this music, so the limit binds on
/// nonsense and nothing else.
pub const FLOOR: i16 = 7;
pub const CEIL: i16 = 49;

/// Middle line of each staff, and the note between them, in diatonic steps —
/// B4, D3 and C4. The score picks one staff per voice by where the voice sits;
/// here both are always drawn, because the voices are being compared.
const TREBLE: i16 = 34;
const BASS: i16 = 22;
const MIDDLE_C: i16 = 28;

/// Which end of a range a drag has hold of.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum End {
  Low,
  High,
}

/// Move one end of a range to `to`.
///
/// The clamp is what makes this worth having on its own: an end may not pass
/// its partner, may not leave the window, and may not squeeze the voice below
/// `least`. Written as three separate `if`s it would be three chances to get a
/// comparison backwards, and the first of the three is the one that matters —
/// a range with `low > high` used to take `realise::domain` down with a capacity
/// overflow, because it sizes a vector from `high - low + 1`. That is fixed in
/// the library now, but this is still where it is stopped from happening.
pub fn moved(range: (i16, i16), end: End, to: i16, least: i16) -> (i16, i16) {
  let (lo, hi) = range;
  let least = least.max(1);
  match end {
    End::Low => (to.clamp(FLOOR, hi - least), hi),
    End::High => (lo, to.clamp(lo + least, CEIL)),
  }
}

/// Move both ends together, keeping the span — transposing the voice's compass
/// rather than resizing it. Stops at the window without narrowing, which is why
/// the shift is clamped and not the ends.
pub fn shifted(range: (i16, i16), by: i16) -> (i16, i16) {
  let (lo, hi) = range;
  let by = by.clamp(FLOOR - lo, CEIL - hi);
  (lo + by, hi + by)
}

/// Where a step sits, and which step sits at a height. Inverses, and a test says
/// so — a drag reads one and draws through the other, so a disagreement between
/// them is a bar that does not follow the pointer.
pub fn y_of(step: i16, top: f32) -> f32 {
  top + (CEIL - step) as f32 * STEP
}

pub fn step_at(y: f32, top: f32) -> i16 {
  CEIL - ((y - top) / STEP).round() as i16
}

/// How tall the staff wants to be.
pub fn height() -> f32 {
  (CEIL - FLOOR) as f32 * STEP + 10.0
}

/// The compass panel, as one value.
pub struct Compass<'a> {
  /// One `(low, high)` per voice, top voice first — `Design::compass`.
  pub ranges: &'a mut Vec<(i16, i16)>,
  /// The key, so a bound is named the way this piece spells that letter.
  pub key: [i8; 7],
  /// The narrowest a voice may be squeezed to, in diatonic steps.
  pub least: i16,
}

impl Compass<'_> {
  /// Draw it, and take any drag. Returns whether a compass moved.
  pub fn show(&mut self, ui: &mut Ui) -> bool {
    let n = self.ranges.len().max(1);
    let (area, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height()), Sense::hover());
    let p = ui.painter_at(area);
    let dark = ui.visuals().dark_mode;
    let line = ui.visuals().weak_text_color().gamma_multiply(0.55);
    let top = area.top() + 5.0;
    let y = |step: i16| y_of(step, top);

    // A margin at the left for the two clefs, and a hair at the right.
    let x0 = area.left() + 30.0;
    let x1 = area.right() - 4.0;

    for centre in [TREBLE, BASS] {
      for k in -2..=2i16 {
        let ly = y(centre + k * 2);
        p.line_segment([Pos2::new(x0 - 6.0, ly), Pos2::new(x1, ly)], Stroke::new(1.0, line));
      }
    }
    // Middle C's ledger line — the one mark that says how far apart the two
    // staves are, and without it the gap between them is unreadable.
    let mc = y(MIDDLE_C);
    p.line_segment([Pos2::new(x0 - 6.0, mc), Pos2::new(x1, mc)], Stroke::new(1.0, line.gamma_multiply(0.4)));

    // Each clef's baseline on the line it names, which is where SMuFL puts its
    // origin: the G line, second from the bottom of the treble staff, and the F
    // line, second from the top of the bass.
    let space = STEP * 2.0;
    let ink = ui.visuals().text_color();
    glyph::clef(&p, glyph::G_CLEF, area.left() + 2.0, y(TREBLE - 2), space, ink);
    glyph::clef(&p, glyph::F_CLEF, area.left() + 2.0, y(BASS + 2), space, ink);

    let gap = 6.0;
    let w = (((x1 - x0 + gap) / n as f32) - gap).clamp(9.0, 38.0);
    let mut changed = false;

    for vi in 0..self.ranges.len() {
      let cx = x0 + vi as f32 * (w + gap);
      let (lo, hi) = self.ranges[vi];
      let c = theme::voice(vi, dark);
      let bar = Rect::from_min_max(Pos2::new(cx, y(hi) - STEP), Pos2::new(cx + w, y(lo) + STEP));
      p.rect_filled(bar, egui::CornerRadius::same(2), theme::wash(vi, dark, 64));
      p.rect_stroke(bar, egui::CornerRadius::same(2), Stroke::new(1.0, c), egui::StrokeKind::Inside);

      // The body, as a handle for transposing the whole compass. Its rect stops
      // short of both ends so it can never take a drag meant for one of them.
      let body = Rect::from_min_max(Pos2::new(cx, y(hi) + GRAB), Pos2::new(cx + w, y(lo) - GRAB));
      if body.height() > 3.0 {
        let id = ui.id().with(("compass-body", vi));
        let r = ui
          .interact(body, id, Sense::drag())
          .on_hover_cursor(egui::CursorIcon::ResizeVertical)
          .on_hover_text("Drag to move the whole compass, keeping its span");
        if let Some(at) = r.interact_pointer_pos() {
          // Absolute, from an offset taken when the drag began — accumulating
          // `drag_delta` instead would drop every movement smaller than a step,
          // and a slow drag is made entirely of those.
          if r.drag_started() {
            ui.data_mut(|d| d.insert_temp(id, step_at(at.y, top) - hi));
          }
          if r.dragged() {
            if let Some(off) = ui.data(|d| d.get_temp::<i16>(id)) {
              changed |= self.set(vi, shifted((lo, hi), step_at(at.y, top) - off - hi));
            }
          }
        }
      }

      for (end, step) in [(End::High, hi), (End::Low, lo)] {
        let grab = Rect::from_min_max(Pos2::new(cx, y(step) - GRAB), Pos2::new(cx + w, y(step) + GRAB));
        let id = ui.id().with(("compass-end", vi, end));
        let r = ui
          .interact(grab, id, Sense::drag())
          .on_hover_cursor(egui::CursorIcon::ResizeVertical)
          .on_hover_text(format!("Voice {} {} — {}", vi + 1, if end == End::High { "top" } else { "bottom" }, name(step, &self.key)));
        let lit = r.hovered() || r.dragged();
        p.rect_filled(
          Rect::from_min_max(Pos2::new(cx - 1.0, y(step) - 2.0), Pos2::new(cx + w + 1.0, y(step) + 2.0)),
          egui::CornerRadius::same(1),
          if lit { ui.visuals().strong_text_color() } else { c },
        );
        if r.dragged() {
          if let Some(at) = r.interact_pointer_pos() {
            changed |= self.set(vi, moved(self.ranges[vi], end, step_at(at.y, top), self.least));
          }
        }
      }
    }

    ui.horizontal_wrapped(|ui| {
      for (vi, (lo, hi)) in self.ranges.iter().enumerate() {
        ui.label(
          RichText::new(format!("{}  {}–{}", vi + 1, name(*lo, &self.key), name(*hi, &self.key)))
            .monospace()
            .small()
            .color(theme::voice(vi, dark)),
        );
      }
    });
    ui.label(
      RichText::new(format!(
        "Where each voice may go, and where they overlap. This bounds what the search \
         writes; a stated subject is given rather than searched, so it sounds where its \
         entry puts it either way. No voice narrower than {} steps, this subject's own span.",
        self.least
      ))
      .weak()
      .small(),
    );

    changed
  }

  /// Write a range back, reporting whether it was different. One place, so that
  /// `changed` cannot say yes to a drag that moved nothing — which is a
  /// recompose on every frame the pointer is held still.
  fn set(&mut self, vi: usize, to: (i16, i16)) -> bool {
    let was = std::mem::replace(&mut self.ranges[vi], to);
    was != to
  }
}

/// How far from an end a drag still counts as being on it. Half a handle, and
/// less than half the minimum span in pixels, so the two ends of even the
/// narrowest compass have separate grab zones.
const GRAB: f32 = 6.0;

/// A bound's name, spelled the way the key spells that letter.
///
/// A compass is diatonic steps and carries no accidental of its own, so there is
/// nothing to guess here: the letter comes from the step and the accidental from
/// the key signature, which is the same rule `realise::domain` uses to turn a
/// step into a pitch.
fn name(step: i16, key: &[i8; 7]) -> String {
  const LETTER: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
  let deg = step.rem_euclid(7) as usize;
  // Through `glyph`, because the sharp and the flat are characters no font egui
  // ships actually has — this label was where that turned up, as a tofu box.
  format!("{}{}{}", LETTER[deg], glyph::accidental(key[deg]), step.div_euclid(7))
}

/// The narrowest a voice may be, given the subject it has to state.
///
/// **This is a floor on usefulness, not on correctness, and the difference was
/// measured rather than assumed.** The first version of this comment said a
/// compass too small for the subject would make the search refuse. It does not:
/// the compass bounds the *free* voices only, and a stated subject's notes are
/// given rather than searched, so they are placed whether the compass admits
/// them or not. `any_compass_the_drag_can_reach_still_composes` tried every
/// arrangement a drag can produce, down to an octave and including three voices
/// stacked on the same octave, and all of them composed.
///
/// So the number below buys legibility, not legality: a voice with less than an
/// octave has almost nowhere to go, and a compass narrower than the subject is
/// a statement about the piece that the piece then ignores. The subject's own
/// span is the floor, an octave the floor under that. The one thing that really
/// is invalid — an end past its partner — is stopped by [`moved`] instead, and
/// by `realise::fill`, which used to panic on it.
pub fn least_for(subject: &contrapunctus::kern::Voice) -> i16 {
  let steps: Vec<i16> = subject.notes.iter().map(|n| n.pitch.step).collect();
  let span = match (steps.iter().min(), steps.iter().max()) {
    (Some(lo), Some(hi)) => hi - lo,
    _ => 0,
  };
  span.max(7)
}

#[cfg(test)]
mod tests {
  use super::*;

  /// **An end may not pass its partner, leave the window, or squeeze the voice
  /// below the minimum.** All three at both ends, because the two arms are
  /// separate expressions and a comparison the wrong way round in one of them
  /// looks right in the other.
  #[test]
  fn an_end_stops_where_it_has_to() {
    let r = (21, 33);

    // ordinary moves land where they were asked to
    assert_eq!(moved(r, End::Low, 25, 7), (25, 33));
    assert_eq!(moved(r, End::High, 40, 7), (21, 40));

    // the minimum span holds from either side, and it is the *other* end that
    // stays put
    assert_eq!(moved(r, End::Low, 30, 7), (26, 33), "the low end may not come within an octave of the high");
    assert_eq!(moved(r, End::High, 22, 7), (21, 28), "nor the high of the low");

    // and the window holds
    assert_eq!(moved(r, End::Low, -100, 7), (FLOOR, 33));
    assert_eq!(moved(r, End::High, 900, 7), (21, CEIL));

    // a range never comes back inverted, whatever it is asked for
    for to in (FLOOR - 5)..=(CEIL + 5) {
      for end in [End::Low, End::High] {
        let (lo, hi) = moved(r, end, to, 7);
        assert!(lo < hi, "moving the {end:?} end to {to} gave {lo}..{hi}");
        assert!(lo >= FLOOR && hi <= CEIL, "{lo}..{hi} left the window");
      }
    }
  }

  /// **Transposing keeps the span**, including where it runs into the window.
  /// Clamping the ends instead of the shift is the mistake here: it would let
  /// the window quietly narrow a voice that was only being moved.
  #[test]
  fn a_shift_moves_both_ends_and_keeps_the_span() {
    let r = (21, 33);
    assert_eq!(shifted(r, 4), (25, 37));
    assert_eq!(shifted(r, -4), (17, 29));

    for by in -60..=60 {
      let (lo, hi) = shifted(r, by);
      assert_eq!(hi - lo, r.1 - r.0, "shifting by {by} changed the span");
      assert!(lo >= FLOOR && hi <= CEIL, "shifting by {by} gave {lo}..{hi}");
    }
    assert_eq!(shifted(r, -100), (FLOOR, FLOOR + 12), "stops against the floor whole");
    assert_eq!(shifted(r, 100), (CEIL - 12, CEIL), "and against the ceiling");
  }

  /// **Height and step are inverses.** A drag reads one and the bar is drawn
  /// through the other, so if they disagree the bar does not follow the pointer
  /// — which is a bug you can only see, and only if you look.
  #[test]
  fn a_height_and_a_step_agree_with_each_other() {
    for top in [0.0f32, 7.5, 100.0] {
      for step in FLOOR..=CEIL {
        assert_eq!(step_at(y_of(step, top), top), step, "step {step} at top {top}");
      }
      // higher is further up the page, which is the whole convention
      assert!(y_of(CEIL, top) < y_of(FLOOR, top));
      // and the staff lines land on the lines
      assert_eq!(y_of(TREBLE, top) - y_of(TREBLE + 2, top), STEP * 2.0, "a staff space is two steps");
    }
  }

  /// **A drag on a handle moves the bound it is on**, through the widget rather
  /// than through the arithmetic.
  ///
  /// The arithmetic above was already right when the score's zoom was broken;
  /// what was wrong there was the plumbing between an event and the arithmetic,
  /// and no test of `View` could have seen it. So this one presses and moves a
  /// synthetic pointer over the real handles and reads `ranges` afterwards.
  #[test]
  fn a_drag_on_a_handle_moves_that_bound() {
    use egui::{Event, Modifiers, PointerButton, Pos2};

    let ctx = egui::Context::default();
    crate::glyph::install(&ctx);
    let screen = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(320.0, 700.0));
    let mut ranges = vec![(33, 45), (28, 40), (21, 33)];

    // Where the widget lands, so the test can aim at a handle. Recorded from
    // inside the frame rather than assumed, because a layout change here would
    // otherwise turn this test into one that drags empty space and passes.
    let origin = std::cell::Cell::new(Pos2::ZERO);
    let clock = std::cell::Cell::new(1.0f64);
    let frame = |ranges: &mut Vec<(i16, i16)>, events: Vec<Event>| {
      clock.set(clock.get() + 1.0 / 60.0);
      let input =
        egui::RawInput { screen_rect: Some(screen), time: Some(clock.get()), events, ..Default::default() };
      ctx
        .run_ui(input, |ui| {
          origin.set(ui.next_widget_position());
          Compass { ranges, key: [0; 7], least: 7 }.show(ui);
        })
        .drop_without_applying_deltas();
    };

    frame(&mut ranges, vec![]);
    let o = origin.get();
    let top = o.y + 5.0;
    // The same column arithmetic `show` uses, so the pointer lands on voice 1.
    let (x0, x1) = (o.x + 30.0, o.x + 320.0 - 4.0);
    let w = (((x1 - x0 + 6.0) / 3.0) - 6.0).clamp(9.0, 38.0);
    let cx = x0 + w / 2.0;

    let press = |at: Pos2| {
      vec![
        Event::PointerMoved(at),
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
      ]
    };
    let release = |at: Pos2| {
      vec![Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
      }]
    };

    // Take the top voice's upper bound and pull it down five steps.
    let grab = Pos2::new(cx, y_of(45, top));
    frame(&mut ranges, press(grab));
    let to = Pos2::new(cx, y_of(40, top));
    frame(&mut ranges, vec![Event::PointerMoved(to)]);
    assert_eq!(ranges[0], (33, 40), "dragging the top handle did not move the top bound");
    frame(&mut ranges, release(to));

    // The lower bound moves on its own, and the partner stays put.
    let grab = Pos2::new(cx, y_of(33, top));
    frame(&mut ranges, press(grab));
    let to = Pos2::new(cx, y_of(26, top));
    frame(&mut ranges, vec![Event::PointerMoved(to)]);
    assert_eq!(ranges[0], (26, 40), "dragging the bottom handle did not move the bottom bound");
    frame(&mut ranges, release(to));

    // And it stops where the minimum span says, rather than passing its partner.
    let grab = Pos2::new(cx, y_of(26, top));
    frame(&mut ranges, press(grab));
    let to = Pos2::new(cx, y_of(44, top));
    frame(&mut ranges, vec![Event::PointerMoved(to)]);
    assert_eq!(ranges[0], (33, 40), "the low end passed the minimum span, or took the high end with it");
    frame(&mut ranges, release(to));

    // The body transposes: span kept, both ends moved, and the neighbours alone.
    let (lo, hi) = ranges[1];
    let cx1 = x0 + (w + 6.0) + w / 2.0;
    let grab = Pos2::new(cx1, (y_of(hi, top) + y_of(lo, top)) / 2.0);
    frame(&mut ranges, press(grab));
    let to = Pos2::new(cx1, grab.y + 4.0 * STEP);
    frame(&mut ranges, vec![Event::PointerMoved(to)]);
    assert_eq!(ranges[1], (lo - 4, hi - 4), "dragging the body did not transpose the compass");
    assert_eq!(ranges[0], (33, 40), "it moved another voice too");
    frame(&mut ranges, release(to));

    // And a drag that begins on nothing changes nothing.
    let before = ranges.clone();
    let empty = Pos2::new(o.x + 4.0, top + 2.0);
    frame(&mut ranges, press(empty));
    frame(&mut ranges, vec![Event::PointerMoved(Pos2::new(empty.x, empty.y + 60.0))]);
    assert_eq!(ranges, before, "a drag that started off the bars moved one");
  }

  /// The floor is the subject's own span, and an octave under that.
  #[test]
  fn the_minimum_comes_from_the_subject() {
    use contrapunctus::kern::{Note, Voice};
    use contrapunctus::pitch::Pitch;
    let voice = |steps: &[i16]| Voice {
      notes: steps
        .iter()
        .enumerate()
        .map(|(i, s)| Note { onset: i as i64, dur: 1, pitch: Pitch::new(*s, 0), attack: true })
        .collect(),
    };
    assert_eq!(least_for(&voice(&[28, 29, 30])), 7, "a subject spanning a third still needs an octave");
    assert_eq!(least_for(&voice(&[28, 40])), 12, "and a wide one needs its own span");
    assert_eq!(least_for(&voice(&[])), 7, "a subject with no notes is not a reason to divide by zero");
  }
}
