//! The two clef glyphs, and the one rule that places them.
//!
//! `ui/assets/clefs.otf` is a subset of Bravura, the reference font for
//! [SMuFL](https://w3c.github.io/smufl/) — two characters out of 3715, 15 kB out
//! of 868. `ui/assets/README.md` says where it came from and why it is renamed;
//! the licence requires the renaming and travels with the file.
//!
//! **The reason to use a font here rather than draw the outlines** is that SMuFL
//! specifies the geometry exactly, and the specification does the work a
//! hand-tuned constant would otherwise do badly:
//!
//! - **One em is four staff spaces.** So a clef drawn at a font size of four
//!   staff spaces is the right size for the staff, at any zoom, without a scale
//!   factor anywhere.
//! - **Each clef's origin is on the line it names.** The G clef's origin sits on
//!   the G line and the F clef's on the F line — so placing the glyph's
//!   *baseline* on that staff line is not an approximation of correct placement,
//!   it is the definition of it.
//!
//! Both halves of that are used below, which is why there is no constant in this
//! file that anybody had to tune by eye.

use egui::{Color32, Context, FontData, FontDefinitions, FontFamily, FontId, Painter, Pos2};

/// SMuFL's codepoints for the two clefs this program draws.
pub const G_CLEF: &str = "\u{E050}";
pub const F_CLEF: &str = "\u{E062}";

/// The family the subset is registered under. Not `Bravura`: the licence
/// reserves that name for the unmodified font.
const FAMILY: &str = "clefs";

pub fn family() -> FontFamily {
  FontFamily::Name(FAMILY.into())
}

/// The font size at which one em is four staff spaces, which is SMuFL's own
/// definition and therefore the size a clef is meant to be drawn at.
pub fn size(staff_space: f32) -> f32 {
  staff_space * 4.0
}

/// Make the clefs available on a context.
///
/// A family of its own rather than a fallback on the body font: these are
/// private-use codepoints, and a fallback chain that reached them by accident
/// would be a worse kind of luck than not having them.
pub fn install(ctx: &Context) {
  let mut fonts = FontDefinitions::default();
  fonts
    .font_data
    .insert(FAMILY.to_owned(), std::sync::Arc::new(FontData::from_static(include_bytes!("../assets/clefs.otf"))));
  fonts.families.insert(family(), vec![FAMILY.to_owned()]);
  ctx.set_fonts(fonts);
}

/// Draw a clef with its **baseline on `line`**, which is the staff line it
/// names.
///
/// egui anchors a galley by its box and not by its baseline, and the two differ
/// by the font's ascent — so the baseline is read off the laid-out glyph rather
/// than guessed at. `Glyph::pos` is documented as the baseline position, which
/// makes this exact rather than close.
pub fn clef(p: &Painter, which: &str, x: f32, line: f32, staff_space: f32, colour: Color32) {
  let font = FontId::new(size(staff_space), family());
  let galley = p.layout_no_wrap(which.to_owned(), font, colour);
  let baseline = galley.rows.first().and_then(|r| r.glyphs.first()).map_or(0.0, |g| g.pos.y);
  p.galley(Pos2::new(x, line - baseline), galley, colour);
}

#[cfg(test)]
mod tests {
  use super::*;

  /// A frame with the clefs installed, since laying text out needs a painter
  /// and a painter needs a frame. `egui::__run_test_ui` cannot be used here: it
  /// installs an empty font set to save time, which is exactly the thing under
  /// test.
  fn with_painter<R>(f: impl FnOnce(&Painter) -> R) -> R {
    let ctx = Context::default();
    install(&ctx);
    let mut f = Some(f);
    let mut out = None;
    ctx
      .run_ui(Default::default(), |ui| {
        if let Some(f) = f.take() {
          out = Some(f(ui.painter()));
        }
      })
      .drop_without_applying_deltas();
    out.expect("the frame ran")
  }

  /// **Both clefs rasterise to something.**
  ///
  /// The check that matters for an embedded font, and the one a missing or
  /// unreadable glyph fails: egui would lay out a replacement box and the page
  /// would show a tofu where a clef belongs. It also catches the subset having
  /// dropped the wrong characters, which is the mistake a subsetting command
  /// makes silently.
  #[test]
  fn both_clefs_have_ink_in_them() {
    with_painter(|p| {
      for (what, ch) in [("G clef", G_CLEF), ("F clef", F_CLEF)] {
        let g = p.layout_no_wrap(ch.to_owned(), FontId::new(32.0, family()), Color32::WHITE);
        let row = g.rows.first().expect(what);
        assert!(!row.glyphs.is_empty(), "{what} laid out to nothing");
        assert!(row.glyphs[0].advance_width > 1.0, "{what} has no width: {}", row.glyphs[0].advance_width);
        assert!(!row.visuals.mesh.vertices.is_empty(), "{what} rasterised to an empty mesh");
      }
    });
  }

  /// **One em is four staff spaces**, so a clef asked for at the staff's height
  /// comes back about as wide as SMuFL says it is — 2.7 spaces for either of
  /// these. A font swapped for one with different metrics would fail here rather
  /// than by looking wrong to somebody.
  #[test]
  fn a_clef_is_the_size_smufl_says() {
    let space = 8.0;
    with_painter(|p| {
      for (what, ch) in [("G clef", G_CLEF), ("F clef", F_CLEF)] {
        let g = p.layout_no_wrap(ch.to_owned(), FontId::new(size(space), family()), Color32::WHITE);
        let w = g.rows[0].glyphs[0].advance_width / space;
        assert!((2.4..3.1).contains(&w), "{what} is {w:.2} staff spaces wide, which is not what SMuFL specifies");
      }
    });
  }

  /// The baseline is inside the laid-out box and below its top, which is what
  /// makes the placement above a subtraction rather than a guess.
  #[test]
  fn the_baseline_is_where_the_glyph_says() {
    with_painter(|p| {
      let g = p.layout_no_wrap(G_CLEF.to_owned(), FontId::new(32.0, family()), Color32::WHITE);
      let baseline = g.rows[0].glyphs[0].pos.y;
      assert!(baseline > 0.0, "the baseline is at the very top of the box");
      assert!(baseline <= g.rect.height() + 1.0, "the baseline is below the box");
    });
  }
}
