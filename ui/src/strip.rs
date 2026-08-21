//! The plan strip — spec 4, and the centrepiece of the interface.
//!
//! One lane per voice, `x` proportional to tick, the subject drawn solid where it
//! sounds. The reason this and not the score is the centrepiece: it teaches what
//! a fugue *is* by being looked at. You watch the subject move from voice to voice,
//! and that is the one judgement someone with no theory can make and be right
//! about.
//!
//! # The two classes of edit
//!
//! Spec 4.2 says the interface must not blur them, so which class an edit is in
//! is [`Edit::touches`] — a method the code dispatches on, not a comment — and a
//! test checks its answer against what `compose::derive` actually does.
//!
//! *Span-preserving* — a change of key, to one return or to several. The piece
//! keeps its length, `compose::refill_span` rewrites the blocks those returns
//! own, and every other note stays exactly where it was. **Reordering the
//! returns belongs here**, which was not obvious: `derive` gives every return an
//! episode and an entry of the same lengths whatever degree it carries, so
//! shuffling the order moves not one bar.
//!
//! *Span-changing* — an episode's length, the link's length. Everything after
//! moves in time; there is no sense in which those later bars are unchanged, and
//! the piece is recomposed.
//!
//! # Showing the change before making it
//!
//! `compose::derive` is pure and costs nothing: it produces the *plan* without
//! searching for a single note. So a drag can show the exact plan it would
//! commit to, live, and the blocks that are about to move are drawn faded while
//! it does. That is the honest version of what 4.2 asks for — not a hint that
//! something will change, but the thing it will change into.
//!
//! The horizontal scale stays pinned to the committed piece for the length of a
//! drag. Rescaling to the preview would move the edge out from under the pointer
//! that is dragging it, which is a feedback loop rather than an interface.

use contrapunctus::compose::{self, Block, Design, Kind, Layout, Origin, Outcome};
use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::catalog::degree_name;
use crate::theme;

const LANE: f32 = 38.0;
const GAP: f32 = 4.0;
const RIBBON: f32 = 18.0;
const RULER: f32 = 16.0;
/// The row of handles that add a return, remove one, and close the piece.
const HANDLES: f32 = 17.0;
/// How near an episode's right edge the pointer must be to take hold of it.
const GRIP: f32 = 7.0;

pub fn height(voices: usize) -> f32 {
  RULER + voices as f32 * (LANE + GAP) + RIBBON + HANDLES + 6.0
}

/// What a gesture asks for. Every variant is a `Layout` field, because nothing
/// else is a parameter — spec 4.2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edit {
  /// `middles[k]` becomes this degree.
  Key(usize, i16),
  /// Every episode becomes this many bars.
  EpisodeBars(i64),
  /// The exposition's link becomes this many bars; zero removes it.
  LinkBars(i64),
  /// The return at `from` moves to `to`.
  MoveMiddle(usize, usize),
  /// A return is inserted at this position, carrying this degree.
  AddMiddle { at: usize, degree: i16 },
  /// The return at this position goes.
  RemoveMiddle(usize),
  /// Whether the piece closes with an episode and an entry at home.
  CloseAtHome(bool),
  /// This block is written again — 4.2's per-block reroll. `id` is what
  /// `Layout::rerolls` is keyed on and `block` is where it is now.
  Reroll { block: usize, id: u64 },
  /// This block moves `by` lanes, **and every block after it moves with it** —
  /// 4.3's voice drag, and the only shape it can honestly take.
  ///
  /// `derive` gives each block the lane after its predecessor's, so a lane is
  /// not a per-block parameter and never was; what is settable is where the
  /// chain is rotated. `id` keys `Layout::turns` and `block` is where it is now.
  Turn { block: usize, id: u64, by: i16 },
}

impl Edit {
  /// The **blocks** this edit changes, when all it changes is their contents.
  ///
  /// `Key` moves one return, which owns two blocks. **Reordering moves several
  /// and is still span-preserving**, which is not obvious: `derive` gives every
  /// return an episode and an entry of the same lengths whatever degree it
  /// carries, so shuffling the order changes `key_of` and `shift` and moves not
  /// one bar. A reroll is one block on its own.
  ///
  /// `None` for the edits that move bars, and those are recomposed.
  pub fn touches(self, d: &Design, l: &Layout) -> Option<std::ops::RangeInclusive<usize>> {
    let owns = |k: usize| {
      let own = compose::blocks_of_middle(d, l, k);
      own.first().copied().zip(own.last().copied())
    };
    match self {
      Edit::Key(k, _) => owns(k).map(|(a, b)| a..=b),
      Edit::MoveMiddle(f, t) => {
        let (lo, hi) = (f.min(t), f.max(t));
        owns(lo).zip(owns(hi)).map(|((a, _), (_, b))| a..=b)
      }
      Edit::Reroll { block, .. } => Some(block..=block),
      // Span-preserving: a turn changes which lane a block is in and moves not
      // one bar, so `refill_span` can rewrite it in place.
      //
      // **From the block before it**, which is not an off-by-one. `fill_block`
      // asks which voice holds the next block and rests that voice for a bar at
      // the end of this one, so it does not enter by a leap. Turn a block and
      // its predecessor is told a different voice is coming, and really does
      // change — `a_turned_plan_still_composes_and_to_something_else` asserts
      // both halves. Refilling from the turn itself would leave one block
      // stale, and fading from the turn itself would show less than moves.
      Edit::Turn { block, .. } => {
        let last = compose::origins(d, l).len().checked_sub(1)?;
        Some(block.saturating_sub(1)..=last)
      }
      // All of these change how many blocks there are, so every bar after them
      // moves and there is nothing local about it.
      Edit::EpisodeBars(_)
      | Edit::LinkBars(_)
      | Edit::AddMiddle { .. }
      | Edit::RemoveMiddle(_)
      | Edit::CloseAtHome(_) => None,
    }
  }

  /// The layout this edit would produce. One function, used both to preview the
  /// edit and to commit it, so the picture cannot promise something the commit
  /// does not do.
  pub fn applied(self, l: &Layout) -> Layout {
    let mut out = l.clone();
    match self {
      Edit::Key(k, deg) => {
        if let Some(slot) = out.middles.get_mut(k) {
          *slot = deg;
        }
      }
      Edit::EpisodeBars(n) => out.episode_bars = n.clamp(1, 8),
      Edit::LinkBars(n) => out.link = if n <= 0 { None } else { Some((l.link.map_or(1, |(a, _)| a), n.min(6))) },
      Edit::MoveMiddle(from, to) => {
        if from < out.middles.len() && to < out.middles.len() {
          let m = out.middles.remove(from);
          out.middles.insert(to, m);
        }
      }
      Edit::Reroll { id, .. } => match out.rerolls.iter_mut().find(|(k, _)| *k == id) {
        Some((_, n)) => *n = n.wrapping_add(1),
        None => out.rerolls.push((id, 1)),
      },
      // Turns at one place add up, and one that adds up to nothing is taken out
      // rather than left as a rotation of zero. Otherwise dragging a block down
      // and back up would leave a settings file that says something happened.
      Edit::Turn { id, by, .. } => {
        match out.turns.iter_mut().find(|(k, _)| *k == id) {
          Some((_, n)) => *n += by,
          None => out.turns.push((id, by)),
        }
        out.turns.retain(|(_, n)| *n != 0);
      }
      Edit::AddMiddle { at, degree } => out.middles.insert(at.min(out.middles.len()), degree),
      Edit::RemoveMiddle(k) => {
        if k < out.middles.len() {
          out.middles.remove(k);
        }
      }
      // §8.15 found a range of nought to nine returns across the book, so an
      // empty journey is a shape the book has and not a degenerate case.
      Edit::CloseAtHome(on) => out.close_at_home = on,
    }
    out
  }
}

/// The turn that would put block `bi` in lane `to`, if it may have one.
///
/// `None` where the library refuses — inside the exposition, whose entries are
/// one per voice by construction, so rotating part of that run would state the
/// subject twice in one voice and never in another. `compose::turnable` is the
/// authority and this asks it rather than reimplementing the reason, so a drag
/// can never propose a layout `compose::fugue` would then refuse.
fn turn_to(d: &Design, l: &Layout, bi: usize, from: usize, to: usize, ids: &[u64]) -> Option<Edit> {
  if to == from || compose::turnable(d, l, bi).is_err() {
    return None;
  }
  Some(Edit::Turn { block: bi, id: *ids.get(bi)?, by: to as i16 - from as i16 })
}

/// Everything one frame of the strip asks the application to do.
///
/// A seek is not an [`Edit`], and keeping them apart is not fussiness: an edit
/// changes the music and a seek changes only where you are listening from.
#[derive(Default)]
pub struct Asked {
  pub edit: Option<Edit>,
  pub seek: Option<i64>,
}

/// The strip's inputs, as one value, because there were becoming a lot of them.
pub struct Strip<'a> {
  pub out: &'a Outcome,
  pub design: &'a Design,
  pub layout: &'a Layout,
  pub playhead: Option<i64>,
  /// A generate in progress: the plan being written, and how many blocks of it
  /// are done — spec 7.3.
  ///
  /// The strip draws that plan with the unwritten tail faded, which is the
  /// progress indicator 7.3 asks for and a better one than a bar because it is
  /// the thing itself: you watch the fugue arrive block by block. It also takes
  /// no gestures while this is set, because there is nothing to edit until the
  /// piece exists.
  pub writing: Option<(&'a [Block], usize)>,
  /// What part of the piece the score is showing, as fractions of the whole.
  ///
  /// The strip fits the whole piece and the score does not, so once the score is
  /// zoomed in the two views are looking at different things — and the strip is
  /// the one that can say so. An overview that shows where the detail view is
  /// looking is the reason to have both.
  pub window: Option<(f32, f32)>,
}

impl Strip<'_> {
  pub fn show(&self, ui: &mut Ui) -> Asked {
    let voices = self.design.voices;
    let measure = self.design.measure.max(1);
    let want = Vec2::new(ui.available_width(), height(voices));
    let (resp, p) = ui.allocate_painter(want, Sense::click());
    let area = resp.rect;
    let dark = ui.visuals().dark_mode;
    let faint = ui.visuals().weak_text_color();

    // Pinned to the committed piece for the whole of a drag — see the module
    // note. Everything below measures against this and nothing else, except
    // while a generate is running, when the plan being written is the only one
    // worth showing and no drag is in flight to be kept still.
    let base: &[Block] = match self.writing {
      Some((bs, _)) => bs,
      None => &self.out.blocks,
    };
    let total = compose::length(base).max(1);
    let x_of = |t: i64| area.left() + area.width() * (t as f32 / total as f32);
    let per_bar = area.width() * measure as f32 / total as f32;
    let lane_top = |v: usize| area.top() + RULER + v as f32 * (LANE + GAP);
    // Which lane a height is in — `lane_top`'s inverse, and what makes a drag
    // out of a block's own lane mean the lane it was dragged into.
    let lane_at = |y: f32| {
      (((y - area.top() - RULER) / (LANE + GAP)).floor().max(0.0) as usize).min(voices.saturating_sub(1))
    };

    let origins = compose::origins(self.design, self.layout);
    // What each block is, for naming one in a reroll. An index would move under
    // the next insertion; this does not.
    let ids = compose::identities_of(self.design, self.layout);
    let mut asked = Asked::default();

    // ---- the gestures, before drawing, because what is drawn depends on them
    let mut proposed: Option<(Edit, Layout)> = None;
    for (i, b) in self.out.blocks.iter().enumerate().take_while(|_| self.writing.is_none()) {
      let Some(v) = voice_of(&b.kind).filter(|v| *v < voices) else { continue };
      let r = block_rect(b, v, &x_of, &lane_top);
      let of = origins.get(i).copied();

      // An episode's right edge sets how long episodes are; the link's sets its
      // own length, and dragging it to nothing removes it — 4.2's table.
      if matches!(b.kind, Kind::Episode { .. }) {
        let grip = Rect::from_min_max(Pos2::new(r.right() - GRIP, r.top()), Pos2::new(r.right() + 2.0, r.bottom()));
        let h = ui.interact(grip, ui.id().with(("edge", i)), Sense::drag());
        if h.hovered() || h.dragged() {
          ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
        if h.dragged() || h.drag_stopped() {
          if let Some(pos) = h.interact_pointer_pos() {
            // Absolute, not accumulated: the length is wherever the pointer is,
            // which needs no state to survive between frames.
            let bars = (((pos.x - x_of(b.at)) / per_bar).round() as i64).max(0);
            let e = if of == Some(Origin::Link) { Edit::LinkBars(bars) } else { Edit::EpisodeBars(bars.max(1)) };
            if h.drag_stopped() {
              asked.edit = Some(e);
            } else {
              proposed = Some((e, e.applied(self.layout)));
            }
          }
        }
      }

      // A return: click to choose where it goes, drag to move it in the order.
      if let Some(Origin::Middle(k)) = of {
        let h = ui.interact(r, ui.id().with(("block", i)), Sense::click_and_drag());
        let key_of = b.key_of;
        let cold = self.out.relaxed.cold.contains(&i);
        let what = describe(b);
        let (at, len) = (b.at, b.len);
        h.clone().on_hover_ui(|ui| {
          ui.label(egui::RichText::new(what).strong());
          ui.label(format!("bar {} to {}, in {}", at / measure + 1, (at + len) / measure + 1, degree_name(key_of)));
          ui.label(egui::RichText::new("Click to send it somewhere else, or to have it written again; drag to move it in the order.").weak().small());
          if cold {
            ui.colored_label(
              theme::warn(ui.visuals().dark_mode),
              "This block would not fill under every constraint, so one was dropped. The report says which.",
            );
          }
        });

        if h.dragged() || h.drag_stopped() {
          if let Some(pos) = h.interact_pointer_pos() {
            // **Which gesture this is, decided by where the pointer went.** Out
            // of the block's own lane is a voice drag; along it is a reorder.
            // No modifier, no mode: the two edits move in different directions
            // and the pointer has already said which one it meant.
            let e = if lane_at(pos.y) != v {
              turn_to(self.design, self.layout, i, v, lane_at(pos.y), &ids)
            } else {
              middle_under(pos.x, self.design, self.layout, &x_of)
                .filter(|to| *to != k)
                .map(|to| Edit::MoveMiddle(k, to))
            };
            if let Some(e) = e {
              if h.drag_stopped() {
                asked.edit = Some(e);
              } else {
                proposed = Some((e, e.applied(self.layout)));
              }
            }
          }
        } else {
          opened(&h).show(|ui| {
            ui.label(egui::RichText::new("send this return to").weak().small());
            for deg in 0..7i16 {
              if ui.selectable_label(deg == key_of, degree_name(deg)).clicked() {
                asked.edit = Some(Edit::Key(k, deg));
                ui.close();
              }
            }
            ui.separator();
            if ui.button("write these bars again").clicked() {
              asked.edit = Some(Edit::Reroll { block: i, id: ids[i] });
              ui.close();
            }
            if ui.button("take this return out").clicked() {
              asked.edit = Some(Edit::RemoveMiddle(k));
              ui.close();
            }
          });
        }
      } else {
        // Every other block: nothing about its *plan* is a parameter, but the
        // notes in it are still a draw from the legal set, and asking for
        // another draw is 4.2's reroll.
        //
        // A menu rather than 4.2's double-click. A double click that means
        // something no single click does is a gesture nobody discovers, and the
        // menu it would replace has to exist anyway for the returns.
        let h = ui.interact(r, ui.id().with(("block", i)), Sense::click_and_drag());
        if h.dragged() || h.drag_stopped() {
          if let Some(pos) = h.interact_pointer_pos().filter(|pos| lane_at(pos.y) != v) {
            if let Some(e) = turn_to(self.design, self.layout, i, v, lane_at(pos.y), &ids) {
              if h.drag_stopped() {
                asked.edit = Some(e);
              } else {
                proposed = Some((e, e.applied(self.layout)));
              }
            }
          }
        }
        let (at, len, key_of, what) = (b.at, b.len, b.key_of, describe(b));
        let cold = self.out.relaxed.cold.contains(&i);
        h.clone().on_hover_ui(|ui| {
          ui.label(egui::RichText::new(what).strong());
          ui.label(format!("bar {} to {}, in {}", at / measure + 1, (at + len) / measure + 1, degree_name(key_of)));
          // Otherwise the reroll is a menu nobody knows is there. The plan of
          // these bars is fixed; the notes in them are one draw of many.
          ui.label(egui::RichText::new("Click to have these bars written again.").weak().small());
          if cold {
            ui.colored_label(
              theme::warn(ui.visuals().dark_mode),
              "This block would not fill under every constraint, so one was dropped. The report says which.",
            );
          }
        });
        let closing = of == Some(Origin::Close);
        opened(&h).show(|ui| {
          if ui.button("write these bars again").clicked() {
            asked.edit = Some(Edit::Reroll { block: i, id: ids[i] });
            ui.close();
          }
          // The other half of the handle at the end of the row: whichever state
          // the close is in, the way out of it is on screen.
          if closing && ui.button("stop after the last return").clicked() {
            asked.edit = Some(Edit::CloseAtHome(false));
            ui.close();
          }
        });
      }
    }

    // ---- what to draw: the proposal if one is being dragged, else the piece
    let showing: Vec<Block> = match (&proposed, self.writing) {
      (Some((_, l)), _) => compose::derive(self.design, l),
      (None, Some((bs, _))) => bs.to_vec(),
      (None, None) => self.out.blocks.clone(),
    };
    // Which blocks are about to move, so they can be faded. Everything from the
    // first block whose position or identity differs.
    let settled = match (&proposed, self.writing) {
      (Some(_), _) => showing
        .iter()
        .zip(&self.out.blocks)
        .position(|(a, b)| a.at != b.at || a.len != b.len || a.key_of != b.key_of || a.kind != b.kind)
        .unwrap_or(showing.len()),
      // Written is solid, not yet written is faded. The same fade a drag uses,
      // and it means the same thing in both: this part is not settled yet.
      (None, Some((_, done))) => done,
      (None, None) => showing.len(),
    };

    // **A turn fades one block further back than its plan changed.** The lanes
    // move from the turn onward, but the block before it changes too: it is the
    // one that rests the voice about to enter, and after a turn a different
    // voice is entering. `Edit::touches` refills from there for the same reason,
    // and the library asserts that it really does change.
    let settled = match &proposed {
      Some((Edit::Turn { .. }, _)) => settled.saturating_sub(1),
      _ => settled,
    };

    // The ruler: a bar number every four bars, dense enough to locate a block
    // and sparse enough not to become a texture of its own.
    for bar in (0..(total / measure).max(1)).step_by(4) {
      let x = x_of(bar * measure);
      p.line_segment(
        [Pos2::new(x, area.top() + RULER - 4.0), Pos2::new(x, area.bottom() - RIBBON - 2.0)],
        Stroke::new(1.0, faint.gamma_multiply(0.25)),
      );
      p.text(Pos2::new(x + 3.0, area.top()), Align2::LEFT_TOP, format!("{}", bar + 1), FontId::monospace(9.0), faint);
    }

    for v in 0..voices {
      let r = Rect::from_min_size(Pos2::new(area.left(), lane_top(v)), Vec2::new(area.width(), LANE));
      p.rect_filled(r, CornerRadius::same(3), theme::wash(v, dark, if dark { 18 } else { 14 }));
    }

    let clip = p.with_clip_rect(area);
    for (i, b) in showing.iter().enumerate() {
      let Some(v) = voice_of(&b.kind).filter(|v| *v < voices) else { continue };
      let r = block_rect(b, v, &x_of, &lane_top);
      let fading = i >= settled;
      let cold = self.out.relaxed.cold.contains(&i) && proposed.is_none() && self.writing.is_none();
      draw_block(&clip, ui, b, v, r, dark, fading, cold);
    }

    // The key ribbon, adjacent equal degrees merged — otherwise a plan that
    // stays in one key for four blocks reads as four decisions instead of one.
    let ribbon = area.bottom() - RIBBON - HANDLES;
    for (from, to, deg) in runs(&showing) {
      let r = Rect::from_min_max(
        Pos2::new(x_of(from) + 1.0, ribbon),
        Pos2::new(x_of(to) - 1.0, ribbon + RIBBON - 4.0),
      );
      let home = deg.rem_euclid(7) == 0;
      clip.rect_filled(
        r,
        CornerRadius::same(2),
        if home { theme::wash(0, dark, if dark { 55 } else { 34 }) } else { ui.visuals().widgets.inactive.bg_fill },
      );
      if r.width() > 26.0 {
        clip.text(r.center(), Align2::CENTER_CENTER, degree_name(deg), FontId::monospace(9.0), ui.visuals().text_color());
      }
    }

    // ---- the handles: a return added, a return removed, the close toggled
    //
    // These are the edits 4.2 wanted as gestures on the blocks themselves, and
    // as gestures they came out **one-way**: clicking the final block to stop
    // closing at home removes the block that was clicked, leaving nothing to
    // click to bring it back. A row of insertion points has both directions
    // present at once, which is the property that was missing rather than the
    // gesture.
    let handles = area.bottom() - HANDLES;
    let mid_y = handles + HANDLES / 2.0 - 1.0;
    let owned: Vec<(f32, f32)> = (0..self.layout.middles.len())
      .filter_map(|k| {
        let own = compose::blocks_of_middle(self.design, self.layout, k);
        let (f, t) = (own.first().copied()?, own.last().copied()?);
        let (a, b) = (showing.get(f)?, showing.get(t)?);
        Some((x_of(a.at), x_of(b.at + b.len)))
      })
      .collect();

    // One handle before each return and one after the last, so a journey of
    // three offers four places to put a fourth.
    let mut spots: Vec<(usize, f32)> = owned.iter().enumerate().map(|(k, (a, _))| (k, *a)).collect();
    if let Some((_, b)) = owned.last() {
      spots.push((owned.len(), *b));
    } else {
      // no returns at all: one handle, in the middle of what there is
      spots.push((0, area.center().x));
    }

    for (k, x) in spots {
      let r = Rect::from_center_size(Pos2::new(x, mid_y), Vec2::splat(HANDLES - 3.0));
      let h = ui.interact(r, ui.id().with(("add", k)), Sense::click());
      let lit = h.hovered();
      p.circle_filled(r.center(), r.width() / 2.0, if lit { theme::wash(0, dark, 200) } else { faint.gamma_multiply(0.25) });
      p.text(
        r.center(),
        Align2::CENTER_CENTER,
        "+",
        FontId::proportional(12.0),
        if lit { ui.visuals().panel_fill } else { ui.visuals().text_color() },
      );
      h.on_hover_text(format!("Add a return here — it would be the {} of {}", k + 1, self.layout.middles.len() + 1));
      if ui.ctx().read_response(ui.id().with(("add", k))).is_some_and(|r| r.clicked()) {
        // The dominant, because it is what the book returns to most and what
        // the returns slider already adds when it grows.
        asked.edit = Some(Edit::AddMiddle { at: k, degree: 4 });
      }
    }

    // And the close, which is the other direction of the same problem: when it
    // is on, its own blocks offer to remove it; when it is off, there is a
    // handle at the end offering to put it back.
    if !self.layout.close_at_home {
      let r = Rect::from_center_size(Pos2::new(area.right() - HANDLES, mid_y), Vec2::new(HANDLES * 3.4, HANDLES - 3.0));
      let h = ui.interact(r, ui.id().with("close"), Sense::click());
      let lit = h.hovered();
      p.rect_filled(r, CornerRadius::same(3), if lit { theme::wash(0, dark, 200) } else { faint.gamma_multiply(0.2) });
      p.text(
        r.center(),
        Align2::CENTER_CENTER,
        "+ close",
        FontId::proportional(10.0),
        if lit { ui.visuals().panel_fill } else { ui.visuals().text_color() },
      );
      if h.on_hover_text("End with an episode and a last entry at home, which §8.15 found in 22 fugues of 22.").clicked() {
        asked.edit = Some(Edit::CloseAtHome(true));
      }
    }

    // What the drag would do, said in words as well as drawn. A picture of a
    // plan is not much use to someone who does not yet know what a plan is.
    if let Some((e, l)) = &proposed {
      let bars = compose::length(&compose::derive(self.design, l)) / measure;
      let said = match e {
        Edit::EpisodeBars(_) => format!("episodes of {} bars — {bars} bars in all", l.episode_bars),
        Edit::LinkBars(_) => match l.link {
          Some((_, n)) => format!("a link of {n} bars — {bars} bars in all"),
          None => format!("no link — {bars} bars in all"),
        },
        Edit::MoveMiddle(..) => {
          format!("{} — {bars} bars in all", l.middles.iter().map(|d| degree_name(*d)).collect::<Vec<_>>().join(" · "))
        }
        Edit::Turn { block, .. } => {
          let lane = compose::derive(self.design, l)
            .get(*block)
            .and_then(|b| voice_of(&b.kind))
            .map_or(0, |v| v + 1);
          format!("into voice {lane}, and every block after it moves with it")
        }
        // None of these is ever dragged, so none is ever previewed.
        Edit::Key(..)
        | Edit::Reroll { .. }
        | Edit::AddMiddle { .. }
        | Edit::RemoveMiddle(_)
        | Edit::CloseAtHome(_) => String::new(),
      };
      let at = Pos2::new(area.left() + 6.0, area.bottom() - RIBBON - 6.0);
      let r = p.text(at, Align2::LEFT_BOTTOM, &said, FontId::proportional(11.0), ui.visuals().strong_text_color());
      p.rect_filled(r.expand(3.0), CornerRadius::same(3), ui.visuals().panel_fill.gamma_multiply(0.85));
      p.text(at, Align2::LEFT_BOTTOM, said, FontId::proportional(11.0), ui.visuals().strong_text_color());
    }

    // Where the score is looking, when that is less than everything. Drawn under
    // the playhead and over the blocks: it is context, not a mark.
    if let Some((a, b)) = self.window.filter(|(a, b)| *b - *a < 0.999) {
      let (l, r) = (area.left() + area.width() * a, area.left() + area.width() * b);
      let win = Rect::from_min_max(Pos2::new(l, area.top() + RULER - 4.0), Pos2::new(r, area.bottom() - HANDLES));
      // Shade what is *not* on the page, rather than what is: the eye goes to
      // the bright part, and the bright part should be the music being read.
      for side in [
        Rect::from_min_max(Pos2::new(area.left(), win.top()), Pos2::new(l, win.bottom())),
        Rect::from_min_max(Pos2::new(r, win.top()), Pos2::new(area.right(), win.bottom())),
      ] {
        if side.width() > 0.5 {
          p.rect_filled(side, CornerRadius::ZERO, ui.visuals().panel_fill.gamma_multiply(0.55));
        }
      }
      p.rect_stroke(win, CornerRadius::same(2), Stroke::new(1.0, faint.gamma_multiply(0.6)), StrokeKind::Inside);
    }

    // The playhead, over everything, because it is the one mark that says *now*
    // and it has to be findable against a block of any colour.
    if let Some(t) = self.playhead {
      let x = x_of(t.clamp(0, total));
      p.line_segment(
        [Pos2::new(x, area.top() + RULER - 4.0), Pos2::new(x, area.bottom() - 2.0)],
        Stroke::new(1.5, ui.visuals().strong_text_color()),
      );
    }

    // A click on nothing in particular moves the playhead there. The blocks
    // register their own interaction after this response does, so egui's
    // layering should already keep their clicks — but "should" is doing work in
    // that sentence, and one gesture doing two things is the kind of bug that
    // only appears under a pointer nobody is watching.
    if resp.clicked() && proposed.is_none() {
      if let Some(pos) = resp.interact_pointer_pos() {
        let on_block = self.out.blocks.iter().enumerate().any(|(i, b)| {
          voice_of(&b.kind).filter(|v| *v < voices).is_some_and(|v| block_rect(b, v, &x_of, &lane_top).contains(pos))
            && matches!(origins.get(i), Some(Origin::Middle(_)))
        });
        if !on_block {
          let f = ((pos.x - area.left()) / area.width().max(1.0)).clamp(0.0, 1.0);
          asked.seek = Some((f as f64 * total as f64) as i64);
        }
      }
    }

    asked
  }
}

/// A block's menu, opened by **either** mouse button.
///
/// `Popup::menu` opens on the primary button and `Popup::context_menu` on the
/// secondary, and a block that answers only one of them has a dead spot under
/// the other. Which button a person reaches for is a habit, not a decision, and
/// there is nothing here for the two to mean differently — so both open it.
fn opened(on: &egui::Response) -> egui::Popup<'static> {
  egui::Popup::menu(on)
    .open_memory((on.clicked() || on.secondary_clicked()).then_some(egui::SetOpenCommand::Toggle))
}

fn block_rect(b: &Block, v: usize, x_of: &impl Fn(i64) -> f32, lane_top: &impl Fn(usize) -> f32) -> Rect {
  Rect::from_min_max(
    Pos2::new(x_of(b.at) + 1.0, lane_top(v) + 3.0),
    Pos2::new(x_of(b.at + b.len) - 1.0, lane_top(v) + LANE - 3.0),
  )
}

#[allow(clippy::too_many_arguments)]
fn draw_block(p: &egui::Painter, ui: &Ui, b: &Block, v: usize, r: Rect, dark: bool, fading: bool, cold: bool) {
  let (label, solid) = match &b.kind {
    Kind::Entry { tonal, .. } => (if *tonal { "answer" } else { "subject" }, true),
    Kind::Episode { .. } => ("episode", false),
  };
  let c = theme::voice(v, dark);
  // Faded means *about to move*, and it is drawn as such rather than merely
  // dimmed: a block that is going somewhere else should not look settled.
  let a = if fading { 0.45 } else { 1.0 };

  if solid {
    p.rect_filled(r, CornerRadius::same(4), c.gamma_multiply(a));
  } else {
    p.rect_filled(r, CornerRadius::same(4), theme::wash(v, dark, 40).gamma_multiply(a));
    let hatch = p.with_clip_rect(r);
    let mut x = r.left() - LANE;
    while x < r.right() {
      hatch.line_segment(
        [Pos2::new(x, r.bottom()), Pos2::new(x + LANE, r.top())],
        Stroke::new(1.0, theme::wash(v, dark, 90).gamma_multiply(a)),
      );
      x += 7.0;
    }
    p.rect_stroke(r, CornerRadius::same(4), Stroke::new(1.0, c.gamma_multiply(a)), StrokeKind::Inside);
  }

  // A block that lost a constraint says so on its face. §8.16 reports these per
  // block and not merely as a count, which is the only reason this can.
  if cold {
    p.rect_stroke(r, CornerRadius::same(4), Stroke::new(2.0, theme::warn(dark)), StrokeKind::Outside);
  }

  if r.width() > 34.0 {
    let ink = if solid { text_on(c) } else { ui.visuals().text_color() };
    p.text(r.center(), Align2::CENTER_CENTER, label, FontId::proportional(10.0), ink.gamma_multiply(a));
  }
}

fn voice_of(k: &Kind) -> Option<usize> {
  match k {
    Kind::Entry { voice, .. } | Kind::Episode { voice, .. } => Some(*voice),
  }
}

/// Which return the pointer is over, by the horizontal span its blocks occupy.
fn middle_under(x: f32, d: &Design, l: &Layout, x_of: &impl Fn(i64) -> f32) -> Option<usize> {
  let blocks = compose::derive(d, l);
  let origins = compose::origins(d, l);
  let mut best: Option<(usize, f32)> = None;
  for k in 0..l.middles.len() {
    let own: Vec<usize> = origins
      .iter()
      .enumerate()
      .filter(|(_, o)| **o == Origin::Middle(k))
      .map(|(i, _)| i)
      .collect();
    let (Some(&f), Some(&t)) = (own.first(), own.last()) else { continue };
    let (a, b) = (x_of(blocks[f].at), x_of(blocks[t].at + blocks[t].len));
    let mid = (a + b) / 2.0;
    let dist = (x - mid).abs();
    if best.is_none_or(|(_, d)| dist < d) {
      best = Some((k, dist));
    }
  }
  best.map(|(k, _)| k)
}

fn describe(b: &Block) -> String {
  match &b.kind {
    Kind::Entry { tonal: true, .. } => "The answer — the subject, adjusted to fit the new key".into(),
    Kind::Entry { .. } => "The subject".into(),
    Kind::Episode { .. } => "An episode — the subject is away, and a fragment of it travels".into(),
  }
}

/// Adjacent blocks sharing a key, merged into one span.
fn runs(blocks: &[Block]) -> Vec<(i64, i64, i16)> {
  let mut out: Vec<(i64, i64, i16)> = vec![];
  for b in blocks {
    match out.last_mut() {
      Some(last) if last.2 == b.key_of => last.1 = b.at + b.len,
      _ => out.push((b.at, b.at + b.len, b.key_of)),
    }
  }
  out
}

/// Black or white, whichever the fill can carry. Cheap luminance; the voice
/// colours are few and none of them is borderline.
fn text_on(c: Color32) -> Color32 {
  let l = 0.299 * c.r() as f32 + 0.587 * c.g() as f32 + 0.114 * c.b() as f32;
  if l > 140.0 { Color32::from_gray(20) } else { Color32::from_gray(245) }
}

#[cfg(test)]
mod tests {
  use super::*;
  use contrapunctus::compose;

  fn design() -> Design {
    crate::catalog::load().subjects[1].design(3)
  }


  /// **The classification is checked against `derive`, not against the claim.**
  ///
  /// [`Edit::touches`] decides whether an edit goes through
  /// `refill_span` — which promises that every note outside the edit stays where
  /// it was — or through a recompose. Getting that wrong in the permissive
  /// direction would make the interface promise locality it cannot deliver,
  /// which spec 4.2 names as the thing not to do. So the test asks the only
  /// authority there is: it derives the plan before and after and compares the
  /// bars.
  ///
  /// It also found something. `MoveMiddle` was written as span-changing and is
  /// not.
  #[test]
  fn an_edit_preserves_the_span_exactly_when_derive_says_it_does() {
    let d = design();
    let base_layout = Layout::default();
    let base = compose::derive(&d, &base_layout);

    let cases = [
      Edit::Key(0, 3),
      Edit::Key(2, 6),
      Edit::MoveMiddle(0, 2),
      Edit::MoveMiddle(2, 0),
      Edit::MoveMiddle(1, 2),
      Edit::EpisodeBars(4),
      Edit::EpisodeBars(1),
      Edit::LinkBars(3),
      Edit::LinkBars(0),
      Edit::Reroll { block: 4, id: 12345 },
      Edit::AddMiddle { at: 0, degree: 4 },
      Edit::AddMiddle { at: 3, degree: 1 },
      Edit::RemoveMiddle(0),
      Edit::RemoveMiddle(2),
      Edit::CloseAtHome(false),
      // A turn moves not one bar, so it belongs on the span-preserving side —
      // which this test will say for itself, against `derive`, rather than on
      // my word. `MoveMiddle` was on the wrong side once for exactly this
      // reason and this is what found it.
      Edit::Turn { block: 5, id: compose::identities_of(&d, &base_layout)[5], by: 1 },
      Edit::Turn { block: 9, id: compose::identities_of(&d, &base_layout)[9], by: -1 },
    ];
    for e in cases {
      let l = e.applied(&base_layout);
      let after = compose::derive(&d, &l);
      let same = after.len() == base.len()
        && after.iter().zip(&base).all(|(a, b)| a.at == b.at && a.len == b.len);
      assert_eq!(e.touches(&d, &l).is_some(), same, "{e:?} is classified wrongly: derive says same-span = {same}");
    }
  }

  /// And the **blocks** a span-preserving edit touches are the ones that
  /// actually changed — no fewer, so nothing stale is left behind, and no more,
  /// so the fast path stays fast.
  #[test]
  fn the_touched_blocks_are_the_ones_that_changed() {
    let d = design();
    let l0 = Layout::default();
    let before = compose::derive(&d, &l0);

    for e in [Edit::Key(1, 6), Edit::MoveMiddle(0, 2), Edit::MoveMiddle(2, 1)] {
      let l = e.applied(&l0);
      let Some(range) = e.touches(&d, &l) else { continue };
      let after = compose::derive(&d, &l);
      let changed: Vec<usize> =
        (0..before.len()).filter(|i| before[*i].key_of != after[*i].key_of || before[*i].kind != after[*i].kind).collect();
      assert!(!changed.is_empty(), "{e:?} changed nothing at all");
      for i in &changed {
        assert!(range.contains(i), "{e:?} changed block {i} and does not claim to touch it");
      }
      assert_eq!(*range.start(), changed[0], "{e:?} claims more than it changed");
      assert_eq!(*range.end(), *changed.last().unwrap(), "{e:?} claims more than it changed");
    }

    // A reroll changes no block's *plan* at all — only the notes drawn into one
    // — so it claims exactly its own block and derive sees nothing.
    let e = Edit::Reroll { block: 6, id: compose::identities_of(&d, &l0)[6] };
    let l = e.applied(&l0);
    assert_eq!(e.touches(&d, &l), Some(6..=6));
    assert_eq!(compose::derive(&d, &l).len(), before.len(), "a reroll must not change the plan");
  }

  /// **A turn claims the block before it, and it is right to.**
  ///
  /// The lanes move from the turn onward, so `derive` sees the change starting
  /// there — but `fill_block` rests the voice that holds the *next* block, so
  /// the predecessor is told a different voice is coming and its notes change
  /// too. The range therefore starts one earlier than the plan difference, which
  /// looks like an off-by-one and is the opposite of one: refilling from the
  /// turn itself would leave a stale block, and fading from it would show less
  /// than moves.
  #[test]
  fn a_turn_reaches_the_block_before_it() {
    let d = design();
    let l0 = Layout::default();
    let before = compose::derive(&d, &l0);
    let ids = compose::identities_of(&d, &l0);
    let at = compose::origins(&d, &l0)
      .iter()
      .position(|o| matches!(o, compose::Origin::Middle(_)))
      .expect("a middle");

    let e = Edit::Turn { block: at, id: ids[at], by: 1 };
    let l = e.applied(&l0);
    let range = e.touches(&d, &l).expect("a turn is span-preserving");
    assert_eq!(*range.start(), at - 1, "a turn must claim the block before it");
    assert_eq!(*range.end(), before.len() - 1, "a turn reaches the end of the piece");

    // and the plan really does change from `at` on, and not before it
    let after = compose::derive(&d, &l);
    assert_eq!(after.len(), before.len(), "a turn changed the number of blocks");
    for i in 0..before.len() {
      assert_eq!(before[i].at, after[i].at, "block {i} moved in time");
      assert_eq!(before[i].kind != after[i].kind, i >= at, "block {i} changed lane when it should not have");
    }
  }

  /// A turn that adds up to nothing leaves nothing behind — dragging a block
  /// down a lane and back up must not write a rotation of zero into the
  /// settings file, or a saved fugue would record something that did not happen.
  #[test]
  fn a_turn_and_its_opposite_cancel() {
    let d = design();
    let l0 = Layout::default();
    let ids = compose::identities_of(&d, &l0);
    let at = compose::origins(&d, &l0)
      .iter()
      .position(|o| matches!(o, compose::Origin::Middle(_)))
      .expect("a middle");

    let down = Edit::Turn { block: at, id: ids[at], by: 1 }.applied(&l0);
    assert_eq!(down.turns, vec![(ids[at], 1)]);
    let up = Edit::Turn { block: at, id: ids[at], by: -1 }.applied(&down);
    assert!(up.turns.is_empty(), "a turn and its opposite left {:?} behind", up.turns);
    assert_eq!(up, l0, "a turn undone is not the layout it started from");
  }

  /// **A drag never proposes a layout the library rejects as a layout.**
  ///
  /// `turn_to` asks `compose::turnable` rather than reimplementing the reason
  /// the exposition cannot be turned in part, so the two cannot drift — which is
  /// the failure this rules out.
  ///
  /// What it deliberately does **not** assert is that every offered turn
  /// composes. It does not, and the first version of this test said it should:
  /// rotating this subject's whole piece by one lane hits §2.7's state wall at
  /// bar 26. That is not an illegal layout, it is a hard search, and the two are
  /// different in a way worth keeping separate — the interface's job with the
  /// first is to not offer it, and with the second to report it, which
  /// `recolour` does. The reason it is hard is the interesting part: a turn
  /// moves a *placed* subject into another lane, and a placed subject ignores
  /// the compass, so the entry can land far outside the compass of the voice now
  /// holding it and leave the free voices an awkward job.
  #[test]
  fn a_turn_is_never_offered_where_the_library_refuses_one() {
    let d = design();
    let l = Layout::default();
    let blocks = compose::derive(&d, &l);
    let ids = compose::identities_of(&d, &l);
    let mut offered = 0;
    for (bi, block) in blocks.iter().enumerate() {
      let from = voice_of(&block.kind).expect("a lane");
      for to in 0..d.voices {
        let Some(e) = turn_to(&d, &l, bi, from, to, &ids) else {
          // refused, and only ever for the one reason there is
          assert!(to == from || compose::turnable(&d, &l, bi).is_err(), "block {bi} refused lane {to} for no reason");
          continue;
        };
        offered += 1;
        let after = e.applied(&l);
        // The layout is one the library accepts *as a layout*. `Run::new` is
        // exactly that question — it derives the plan and validates the turns
        // and fills nothing — so this asks it rather than composing 24 fugues to
        // learn what a plan already knows.
        if let Err(why) = compose::Run::new(&d, &after, contrapunctus::automaton::CONFIRMED, 0x5EED) {
          panic!("block {bi} offered a turn to lane {to} the library calls illegal: {why}");
        }
      }
    }
    assert!(offered > 0, "no turn was offered anywhere, so this test checked nothing");
  }

  /// A drag says the same thing the commit does, because both go through
  /// `applied`. Worth a line: the preview promising one plan and the commit
  /// producing another is the failure mode a live preview invents.
  #[test]
  fn the_preview_and_the_commit_are_the_same_function() {
    let l0 = Layout::default();
    for e in [Edit::EpisodeBars(5), Edit::LinkBars(2), Edit::LinkBars(0), Edit::MoveMiddle(0, 1)] {
      assert_eq!(e.applied(&l0), e.applied(&l0));
    }
    // and asking for a block again counts, rather than toggling
    let once = Edit::Reroll { block: 0, id: 7 }.applied(&l0);
    assert_eq!(once.rerolls, vec![(7, 1)]);
    let twice = Edit::Reroll { block: 0, id: 7 }.applied(&once);
    assert_eq!(twice.rerolls, vec![(7, 2)], "asking twice must not undo asking once");

    assert_eq!(Edit::EpisodeBars(99).applied(&l0).episode_bars, 8, "clamped");
    assert_eq!(Edit::EpisodeBars(0).applied(&l0).episode_bars, 1, "clamped");
    assert!(Edit::LinkBars(0).applied(&l0).link.is_none(), "dragged to nothing removes it");
  }
}
