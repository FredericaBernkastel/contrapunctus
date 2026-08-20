//! The plan strip — spec 4, and the centrepiece of the interface.
//!
//! One lane per voice, `x` proportional to tick, the theme drawn solid where it
//! sounds. The reason this and not the score is the centrepiece: it teaches what
//! a fugue *is* by being looked at. You watch the tune move from voice to voice,
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
/// How near an episode's right edge the pointer must be to take hold of it.
const GRIP: f32 = 7.0;

pub fn height(voices: usize) -> f32 {
  RULER + voices as f32 * (LANE + GAP) + RIBBON + 6.0
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
}

impl Edit {
  /// The returns this edit changes, when all it changes is their keys.
  ///
  /// `Key` moves one return. **Reordering moves several and is still
  /// span-preserving**, which is not obvious and is the reason this is a range
  /// rather than an index: `derive` gives every return an episode and an entry
  /// of the same lengths whatever degree it carries, so shuffling the order
  /// changes `key_of` and `shift` and moves not one bar. Everything between the
  /// two ends of the move is affected and nothing outside it is.
  ///
  /// `None` for the edits that move bars, and those are recomposed.
  pub fn touches(self) -> Option<std::ops::RangeInclusive<usize>> {
    match self {
      Edit::Key(k, _) => Some(k..=k),
      Edit::MoveMiddle(f, t) => Some(f.min(t)..=f.max(t)),
      Edit::EpisodeBars(_) | Edit::LinkBars(_) => None,
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
    }
    out
  }
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
    // note. Everything below measures against this and nothing else.
    let total = compose::length(&self.out.blocks).max(1);
    let x_of = |t: i64| area.left() + area.width() * (t as f32 / total as f32);
    let per_bar = area.width() * measure as f32 / total as f32;
    let lane_top = |v: usize| area.top() + RULER + v as f32 * (LANE + GAP);

    let origins = compose::origins(self.design, self.layout);
    let mut asked = Asked::default();

    // ---- the gestures, before drawing, because what is drawn depends on them
    let mut proposed: Option<(Edit, Layout)> = None;
    for (i, b) in self.out.blocks.iter().enumerate() {
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
          ui.label(egui::RichText::new("Click to send it somewhere else; drag to move it in the order.").weak().small());
          if cold {
            ui.colored_label(
              theme::warn(ui.visuals().dark_mode),
              "This block would not fill under every constraint, so one was dropped. The report says which.",
            );
          }
        });

        if h.dragged() || h.drag_stopped() {
          if let Some(pos) = h.interact_pointer_pos() {
            if let Some(to) = middle_under(pos.x, self.design, self.layout, &x_of) {
              if to != k {
                let e = Edit::MoveMiddle(k, to);
                if h.drag_stopped() {
                  asked.edit = Some(e);
                } else {
                  proposed = Some((e, e.applied(self.layout)));
                }
              }
            }
          }
        } else {
          egui::Popup::menu(&h).show(|ui| {
            ui.label(egui::RichText::new("send this return to").weak().small());
            for deg in 0..7i16 {
              if ui.selectable_label(deg == key_of, degree_name(deg)).clicked() {
                asked.edit = Some(Edit::Key(k, deg));
                ui.close();
              }
            }
          });
        }
      } else if !matches!(b.kind, Kind::Episode { .. }) {
        // Entries that are not returns: nothing to edit, everything to explain.
        let h = ui.interact(r, ui.id().with(("block", i)), Sense::hover());
        let (at, len, key_of, what) = (b.at, b.len, b.key_of, describe(b));
        h.on_hover_ui(|ui| {
          ui.label(egui::RichText::new(what).strong());
          ui.label(format!("bar {} to {}, in {}", at / measure + 1, (at + len) / measure + 1, degree_name(key_of)));
        });
      }
    }

    // ---- what to draw: the proposal if one is being dragged, else the piece
    let showing: Vec<Block> = match &proposed {
      Some((_, l)) => compose::derive(self.design, l),
      None => self.out.blocks.clone(),
    };
    // Which blocks are about to move, so they can be faded. Everything from the
    // first block whose position or identity differs.
    let settled = match &proposed {
      Some(_) => showing
        .iter()
        .zip(&self.out.blocks)
        .position(|(a, b)| a.at != b.at || a.len != b.len || a.key_of != b.key_of || a.kind != b.kind)
        .unwrap_or(showing.len()),
      None => showing.len(),
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
      draw_block(&clip, ui, b, v, r, dark, fading, self.out.relaxed.cold.contains(&i) && proposed.is_none());
    }

    // The key ribbon, adjacent equal degrees merged — otherwise a plan that
    // stays in one key for four blocks reads as four decisions instead of one.
    let ribbon = area.bottom() - RIBBON;
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
        Edit::Key(..) => String::new(),
      };
      let at = Pos2::new(area.left() + 6.0, area.bottom() - RIBBON - 6.0);
      let r = p.text(at, Align2::LEFT_BOTTOM, &said, FontId::proportional(11.0), ui.visuals().strong_text_color());
      p.rect_filled(r.expand(3.0), CornerRadius::same(3), ui.visuals().panel_fill.gamma_multiply(0.85));
      p.text(at, Align2::LEFT_BOTTOM, said, FontId::proportional(11.0), ui.visuals().strong_text_color());
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

fn block_rect(b: &Block, v: usize, x_of: &impl Fn(i64) -> f32, lane_top: &impl Fn(usize) -> f32) -> Rect {
  Rect::from_min_max(
    Pos2::new(x_of(b.at) + 1.0, lane_top(v) + 3.0),
    Pos2::new(x_of(b.at + b.len) - 1.0, lane_top(v) + LANE - 3.0),
  )
}

#[allow(clippy::too_many_arguments)]
fn draw_block(p: &egui::Painter, ui: &Ui, b: &Block, v: usize, r: Rect, dark: bool, fading: bool, cold: bool) {
  let (label, solid) = match &b.kind {
    Kind::Entry { tonal, .. } => (if *tonal { "answer" } else { "theme" }, true),
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
    Kind::Entry { tonal: true, .. } => "The answer — the tune, adjusted to fit the new key".into(),
    Kind::Entry { .. } => "The theme".into(),
    Kind::Episode { .. } => "An episode — the tune is away, and a fragment of it travels".into(),
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
    ];
    for e in cases {
      let after = compose::derive(&d, &e.applied(&base_layout));
      let same = after.len() == base.len()
        && after.iter().zip(&base).all(|(a, b)| a.at == b.at && a.len == b.len);
      assert_eq!(e.touches().is_some(), same, "{e:?} is classified wrongly: derive says same-span = {same}");
    }
  }

  /// And the returns a span-preserving edit touches are the ones whose key
  /// actually changed — no fewer, so nothing stale is left behind, and no more,
  /// so the fast path stays fast.
  #[test]
  fn the_touched_returns_are_the_ones_that_changed() {
    let l0 = Layout::default();
    for e in [Edit::Key(1, 6), Edit::MoveMiddle(0, 2), Edit::MoveMiddle(2, 1)] {
      let l = e.applied(&l0);
      let Some(range) = e.touches() else { continue };
      let changed: Vec<usize> = (0..l0.middles.len()).filter(|k| l0.middles[*k] != l.middles[*k]).collect();
      for k in &changed {
        assert!(range.contains(k), "{e:?} changed return {k} and does not claim to touch it");
      }
      // and the claim is tight: its ends are returns that changed
      if !changed.is_empty() {
        assert_eq!(*range.start(), changed[0], "{e:?} claims more than it changed");
        assert_eq!(*range.end(), *changed.last().unwrap(), "{e:?} claims more than it changed");
      }
    }
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
    assert_eq!(Edit::EpisodeBars(99).applied(&l0).episode_bars, 8, "clamped");
    assert_eq!(Edit::EpisodeBars(0).applied(&l0).episode_bars, 1, "clamped");
    assert!(Edit::LinkBars(0).applied(&l0).link.is_none(), "dragged to nothing removes it");
  }
}
