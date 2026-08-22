//! The block palette — spec 4.5, and a section of its own between the plan and
//! the score.
//!
//! # There is no mode to be in
//!
//! `Layout::built` is a plan authored rather than derived, and the first version
//! of this put a **Take it apart into blocks** button in the panel to get into
//! it. That was a mode, and a mode is a thing to be in before you can do
//! anything — which is exactly backwards for a palette, whose whole proposition
//! is *drag this and see*.
//!
//! So dragging a block into the plan is what converts it. `compose::taken_apart`
//! is exact, so the piece either side of that conversion is the same piece, and
//! `Edit::Insert` does it silently on the way past. Nobody has to know the mode
//! exists until they want to leave it.
//!
//! # The same blocks
//!
//! Drawn by `strip::draw_block`, the function the plan strip draws with, at the
//! same height and in the voice colours. Not "styled to match": the same code,
//! because a palette whose blocks merely resembled the ones in the plan would be
//! a picture of the feature rather than the feature.
//!
//! # Where a drop lands
//!
//! Two numbers, and both come off the pointer. Its `x` against the block
//! boundaries gives the position in the order — **between** blocks, so a block
//! can go in the middle and not only at the end. Its `y` gives the lane, which is
//! the voice. A drop outside the plan strip does nothing at all.

use contrapunctus::compose::{self, Built, Design, Kind, Layout};
use egui::{Rect, Sense, Ui, Vec2};

use crate::strip;
use crate::theme;

/// What is being dragged, while it is being dragged.
///
/// egui carries this between widgets, which is what makes the palette and the
/// plan two separate widgets that can still be one gesture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dragged(pub Built);

/// How tall the palette is. Shorter than a lane in the plan, because these are
/// specimens rather than music.
const HEIGHT: f32 = 34.0;

/// The specimens on offer, in the order they are drawn.
///
/// The subject, the answer and an episode — §2.4's three productions, which is
/// not a coincidence but is why there are three. `key_of` and `shift` are what
/// the plan's own menus already set after a block is in, so a palette that asked
/// for them up front would be asking twice.
fn specimens() -> Vec<(&'static str, Built)> {
  vec![
    ("subject", Built::Entry { voice: 0, shift: 0, tonal: false, key_of: 0 }),
    ("answer", Built::Entry { voice: 0, shift: 4, tonal: true, key_of: 4 }),
    ("episode", Built::Episode { voice: 0, shift: 0, key_of: 0, bars: 2 }),
  ]
}

/// Draw the palette. Full width, between the plan and the score.
pub fn show(ui: &mut Ui, d: &Design, l: &Layout) {
  let dark = ui.visuals().dark_mode;
  ui.horizontal(|ui| {
    ui.label(egui::RichText::new("BLOCKS").monospace().weak().small());
    ui.add_space(8.0);
    for (name, built) in specimens() {
      let block = specimen_block(d, &built);
      let width = if matches!(block.kind, Kind::Episode { .. }) { 96.0 } else { 74.0 };
      let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, HEIGHT), Sense::drag());
      let p = ui.painter_at(rect.expand(2.0));
      // The plan strip's own drawing, so these cannot drift out of step with it.
      strip::draw_block(&p, ui, &block, 0, rect, dark, resp.dragged(), false);
      let resp = resp.on_hover_text(hint(name));
      if resp.hovered() || resp.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
      }
      if resp.drag_started() {
        egui::DragAndDrop::set_payload(ui.ctx(), Dragged(built.clone()));
      }
      ui.add_space(6.0);
    }

    ui.add_space(6.0);
    if l.built.is_some() {
      ui.label(
        egui::RichText::new(format!("{} blocks, authored", l.built.as_ref().map_or(0, |v| v.len())))
          .weak()
          .small(),
      );
    } else {
      ui.label(egui::RichText::new("drag one into the plan above").weak().small());
    }
  });
  // What is being dragged, drawn under the pointer, because a drag with nothing
  // following the hand is a drag nobody believes is happening.
  if let Some(held) = egui::DragAndDrop::payload::<Dragged>(ui.ctx()) {
    if let Some(at) = ui.ctx().pointer_interact_pos() {
      let block = specimen_block(d, &held.0);
      let r = Rect::from_min_size(at + Vec2::new(8.0, -HEIGHT / 2.0), Vec2::new(74.0, HEIGHT));
      let p = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("held")));
      strip::draw_block(&p, ui, &block, 0, r, dark, true, false);
    }
  }
  let _ = theme::voice(0, dark);
}

/// A specimen as a `Block`, so the plan's own drawing can take it. `at` is
/// meaningless here and is zero; nothing reads it.
fn specimen_block(d: &Design, b: &Built) -> compose::Block {
  let l = Layout { built: Some(vec![b.clone()]), ..Layout::default() };
  compose::derive(d, &l).into_iter().next().unwrap_or(compose::Block {
    at: 0,
    len: d.measure,
    kind: Kind::Episode { voice: 0, shift: 0 },
    key_of: 0,
  })
}

fn hint(name: &str) -> &'static str {
  match name {
    "subject" => {
      "A statement of the subject, in the home key and as long as the subject is. Drag it into a \
       voice in the plan above — between two blocks, or at either end."
    }
    "answer" => {
      "The subject at the dominant, adjusted to fit — §8.11's tonal answer. An exposition \
       alternates it with the subject, which is one of the five things the verdict checks."
    }
    _ => {
      "Two bars in which the subject is away and a fragment of it travels. Click it once it is in \
       the plan to change its length."
    }
  }
}

/// Where a drop over the plan would land: which position in the order, and which
/// voice.
///
/// **Between blocks and not on one**, which is the whole of what the first
/// version could not do: `+ entry` appended, and a plan is not a thing you only
/// ever add to the end of. The boundary nearest the pointer wins, so dropping on
/// the left half of a block puts the new one before it.
pub fn landing(x: f32, lane: usize, edges: &[(f32, f32)]) -> (usize, usize) {
  if edges.is_empty() {
    return (0, lane);
  }
  let mut best = (f32::INFINITY, 0usize);
  for (i, (left, right)) in edges.iter().enumerate() {
    for (edge, at) in [(*left, i), (*right, i + 1)] {
      let d = (x - edge).abs();
      if d < best.0 {
        best = (d, at);
      }
    }
  }
  (best.1, lane)
}

#[cfg(test)]
mod tests {
  use super::*;

  /// **A drop lands between blocks, at the nearest boundary** — which is what
  /// makes this a palette rather than an append button.
  #[test]
  fn a_drop_lands_at_the_nearest_boundary() {
    // three blocks, 100 wide each, laid end to end from 0
    let edges = vec![(0.0, 100.0), (100.0, 200.0), (200.0, 300.0)];
    assert_eq!(landing(2.0, 0, &edges).0, 0, "before everything");
    assert_eq!(landing(40.0, 0, &edges).0, 0, "the left half of the first block goes before it");
    assert_eq!(landing(60.0, 0, &edges).0, 1, "the right half goes after it");
    assert_eq!(landing(140.0, 0, &edges).0, 1);
    assert_eq!(landing(170.0, 0, &edges).0, 2);
    assert_eq!(landing(298.0, 0, &edges).0, 3, "past the end appends");
    assert_eq!(landing(9999.0, 0, &edges).0, 3);

    // A point exactly between two boundaries is a tie, and the first wins — 150
    // is the middle of the second block and 50 from either side of it.
    assert_eq!(landing(150.0, 0, &edges).0, 1, "a tie should not wander");

    // the lane comes through untouched: it is the voice, and the caller reads it off y
    assert_eq!(landing(170.0, 2, &edges), (2, 2));

    // an empty plan takes a block at the beginning, which is the only place there is
    assert_eq!(landing(123.0, 1, &[]), (0, 1));
  }

  /// Every specimen is a block the library will accept, which is the one thing
  /// about them that a compiler does not already check.
  #[test]
  fn every_specimen_is_a_block_that_composes() {
    let d = crate::catalog::load().subjects[0].design(3);
    for (name, built) in specimens() {
      let b = specimen_block(&d, &built);
      assert!(b.len > 0, "{name} has no length");
      assert_eq!(compose::Built::voice(&built), 0, "{name} does not start in the top voice");
    }
  }
}
