//! The plan strip — spec 4, and the centrepiece of the interface.
//!
//! One lane per voice, `x` proportional to tick, the theme drawn solid where it
//! sounds. The reason this and not the score is the centrepiece: it teaches what
//! a fugue *is* by being looked at. You watch the tune move from voice to voice,
//! and that is the one judgement someone with no theory can make and be right
//! about.
//!
//! Editing is spec 4.2, and one gesture of it is wired: clicking a middle's
//! block chooses its key. That edit is **span-preserving** — the piece keeps its
//! length, and `compose::refill` rewrites the two blocks that middle owns while
//! every other note stays exactly where it was. The gestures that change the
//! span are a different class and the roadmap keeps them apart, because an
//! interface that blurred the two would promise locality it cannot deliver.

use contrapunctus::compose::{Block, Kind, Origin, Outcome};
use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::catalog::degree_name;
use crate::theme;

/// Lane height, and the ribbon under them.
const LANE: f32 = 38.0;
const GAP: f32 = 4.0;
const RIBBON: f32 = 18.0;
const RULER: f32 = 16.0;

pub fn height(voices: usize) -> f32 {
  RULER + voices as f32 * (LANE + GAP) + RIBBON + 6.0
}

/// What a gesture on the strip asks for. Every variant is a `Layout` field,
/// because nothing else is a parameter — spec 4.2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edit {
  /// `middles[k]` becomes this degree. Span-preserving.
  Key(usize, i16),
}

/// Draw the strip, and report any edit the user asked for.
pub fn show(
  ui: &mut Ui,
  out: &Outcome,
  voices: usize,
  measure: i64,
  origins: &[Origin],
) -> (Response, Option<Edit>) {
  let want = Vec2::new(ui.available_width(), height(voices));
  let (resp, p) = ui.allocate_painter(want, Sense::hover());
  let area = resp.rect;
  let dark = ui.visuals().dark_mode;
  let faint = ui.visuals().weak_text_color();

  let total = contrapunctus::compose::length(&out.blocks).max(1);
  let x_of = |t: i64| area.left() + area.width() * (t as f32 / total as f32);

  // The ruler: a bar number every four bars, which is dense enough to locate a
  // block and sparse enough not to become a texture of its own.
  let bars = (total / measure.max(1)).max(1);
  for b in (0..bars).step_by(4) {
    let x = x_of(b * measure);
    p.line_segment([Pos2::new(x, area.top() + RULER - 4.0), Pos2::new(x, area.bottom() - RIBBON - 2.0)],
      Stroke::new(1.0, faint.gamma_multiply(0.25)));
    p.text(Pos2::new(x + 3.0, area.top()), Align2::LEFT_TOP, format!("{}", b + 1), FontId::monospace(9.0), faint);
  }

  let lane_top = |v: usize| area.top() + RULER + v as f32 * (LANE + GAP);

  // Lane grounds first, so a voice with nothing in it still reads as a voice
  // rather than as absence.
  for v in 0..voices {
    let r = Rect::from_min_size(Pos2::new(area.left(), lane_top(v)), Vec2::new(area.width(), LANE));
    p.rect_filled(r, CornerRadius::same(3), theme::wash(v, dark, if dark { 18 } else { 14 }));
  }

  let mut rects: Vec<(usize, Rect)> = vec![];
  for (i, b) in out.blocks.iter().enumerate() {
    let (voice, label, solid) = match &b.kind {
      Kind::Entry { voice, tonal, .. } => (*voice, if *tonal { "answer" } else { "theme" }, true),
      Kind::Episode { voice, .. } => (*voice, "episode", false),
    };
    if voice >= voices {
      continue;
    }
    let r = Rect::from_min_max(
      Pos2::new(x_of(b.at) + 1.0, lane_top(voice) + 3.0),
      Pos2::new(x_of(b.at + b.len) - 1.0, lane_top(voice) + LANE - 3.0),
    );
    let c = theme::voice(voice, dark);
    rects.push((i, r));

    if solid {
      p.rect_filled(r, CornerRadius::same(4), c);
    } else {
      // Hatched and outlined: an episode is the same material seen through, and
      // the difference has to survive being small on screen.
      p.rect_filled(r, CornerRadius::same(4), theme::wash(voice, dark, 40));
      let clip = p.with_clip_rect(r);
      let mut x = r.left() - LANE;
      while x < r.right() {
        clip.line_segment(
          [Pos2::new(x, r.bottom()), Pos2::new(x + LANE, r.top())],
          Stroke::new(1.0, theme::wash(voice, dark, 90)),
        );
        x += 7.0;
      }
      p.rect_stroke(r, CornerRadius::same(4), Stroke::new(1.0, c), StrokeKind::Inside);
    }

    // A block that lost a constraint says so on its face. §8.16 reports these
    // per block and not merely as a count, which is the only reason this can.
    if out.relaxed.cold.contains(&i) {
      p.rect_stroke(r, CornerRadius::same(4), Stroke::new(2.0, theme::warn(dark)), StrokeKind::Outside);
    }

    if r.width() > 34.0 {
      let ink = if solid { text_on(c) } else { ui.visuals().text_color() };
      p.text(r.center(), Align2::CENTER_CENTER, label, FontId::proportional(10.0), ink);
    }
  }

  // The key ribbon, adjacent equal degrees merged — otherwise a plan that stays
  // in one key for four blocks reads as four decisions instead of one.
  let ribbon = area.bottom() - RIBBON;
  for (from, to, deg) in runs(&out.blocks) {
    let r = Rect::from_min_max(
      Pos2::new(x_of(from) + 1.0, ribbon),
      Pos2::new(x_of(to) - 1.0, ribbon + RIBBON - 4.0),
    );
    let home = deg.rem_euclid(7) == 0;
    p.rect_filled(r, CornerRadius::same(2), if home {
      theme::wash(0, dark, if dark { 55 } else { 34 })
    } else {
      ui.visuals().widgets.inactive.bg_fill
    });
    if r.width() > 26.0 {
      p.text(r.center(), Align2::CENTER_CENTER, degree_name(deg), FontId::monospace(9.0),
        ui.visuals().text_color());
    }
  }

  // One interactive region per block, rather than one hit test over the strip.
  // egui's own layering then decides what the pointer is on, and each block gets
  // a tooltip and a menu for free — which is also why the strip itself only
  // senses hover: two things competing for the same click is a bug that only
  // shows up under the pointer.
  let mut edit = None;
  for (i, r) in rects {
    let block = &out.blocks[i];
    let of = origins.get(i).copied();
    let middle = match of {
      Some(Origin::Middle(k)) => Some(k),
      _ => None,
    };
    let sense = if middle.is_some() { Sense::click() } else { Sense::hover() };
    let hit = ui.interact(r, ui.id().with(("block", i)), sense);

    let cold = out.relaxed.cold.contains(&i);
    let (at, len, key_of, what) = (block.at, block.len, block.key_of, describe(block));
    hit.clone().on_hover_ui(|ui| {
      ui.label(egui::RichText::new(what).strong());
      ui.label(format!(
        "bar {} to {}, in {}",
        at / measure.max(1) + 1,
        (at + len) / measure.max(1) + 1,
        degree_name(key_of),
      ));
      if middle.is_some() {
        ui.label(egui::RichText::new("Click to send it somewhere else.").weak().small());
      }
      if cold {
        ui.colored_label(
          theme::warn(ui.visuals().dark_mode),
          "This block would not fill under every constraint, so one was dropped. The report says which.",
        );
      }
    });

    if let Some(k) = middle {
      egui::Popup::menu(&hit).show(|ui| {
        ui.label(egui::RichText::new("send this return to").weak().small());
        for deg in 0..7i16 {
          if ui.selectable_label(deg == key_of, degree_name(deg)).clicked() {
            edit = Some(Edit::Key(k, deg));
            ui.close();
          }
        }
      });
    }
  }

  (resp, edit)
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
