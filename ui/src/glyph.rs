//! The music glyphs: two clefs, two accidentals, and the rules that place them.
//!
//! `ui/assets/music.otf` is a subset of Bravura, the reference font for
//! [SMuFL](https://w3c.github.io/smufl/) — four characters out of 3715, 16 kB out
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
//!
//! # The accidentals are the other case, and they are shaped for text
//!
//! [`SHARP`] and [`FLAT`] are here because a label saying where a voice may go
//! has to be able to write `E♭4`, and **no font egui ships has those
//! characters** — `has_glyph` says no for U+266D and U+266F in both the
//! proportional and the monospace family, and what a reader gets instead is a
//! tofu box. That was reported, from the compass panel.
//!
//! There is no natural, because nothing asks for one: [`accidental`] is handed a
//! key signature's alteration, and a key signature has no naturals in it. Two
//! glyphs wider than the subset was, and not three.
//!
//! They come from the same subset, and they are the one thing in it that is
//! *not* SMuFL-placed. SMuFL centres an accidental on the notehead it applies
//! to, so the glyph straddles the baseline; dropped into a line of text it reads
//! as a subscript. The subsetting command raises each onto the baseline — the
//! sharp by 0.348 em, the flat by 0.175 — so that it sits like a letter. The
//! clefs are left exactly as Bravura draws them, because their origin is the
//! whole reason they come from a font at all.
//!
//! `score.rs` still draws the accidentals it puts on a staff from line segments,
//! which are placed against noteheads rather than against a baseline, and it
//! draws a natural among them. These two are for prose.

use egui::{Color32, Context, FontData, FontDefinitions, FontFamily, FontId, Painter, Pos2};

/// SMuFL's codepoints for the two clefs this program draws.
pub const G_CLEF: &str = "\u{E050}";
pub const F_CLEF: &str = "\u{E062}";

/// The two accidentals, at their **Unicode** codepoints rather than SMuFL's,
/// because these go in text and text is where Unicode's are the right ones. The
/// subset answers to both.
pub const SHARP: &str = "\u{266F}";
pub const FLAT: &str = "\u{266D}";

/// The family the subset is registered under. Not `Bravura`: the licence
/// reserves that name for the unmodified font.
const FAMILY: &str = "music";

pub fn family() -> FontFamily {
  FontFamily::Name(FAMILY.into())
}

/// The font size at which one em is four staff spaces, which is SMuFL's own
/// definition and therefore the size a clef is meant to be drawn at.
pub fn size(staff_space: f32) -> f32 {
  staff_space * 4.0
}

/// Make the glyphs available on a context.
///
/// **A family of its own, and a fallback as well**, which are two decisions and
/// not one.
///
/// The family of its own is for the clefs: those are private-use codepoints, and
/// a fallback chain that reached them by accident would be a worse kind of luck
/// than not having them. The fallback is for the accidentals, and the same
/// argument runs the other way — U+266D and its neighbours are ordinary Unicode
/// characters with settled meanings, so text reaching them is not an accident
/// but the point. Appended rather than prepended, so this font is consulted only
/// for characters the real text font does not have.
pub fn install(ctx: &Context) {
  let mut fonts = FontDefinitions::default();
  fonts
    .font_data
    .insert(FAMILY.to_owned(), std::sync::Arc::new(FontData::from_static(include_bytes!("../assets/music.otf"))));
  fonts.families.insert(family(), vec![FAMILY.to_owned()]);
  for text in [FontFamily::Proportional, FontFamily::Monospace] {
    fonts.families.entry(text).or_default().push(FAMILY.to_owned());
  }
  ctx.set_fonts(fonts);
}

/// The accidental for an alteration, in the spelling a label wants.
///
/// Empty for a natural, because a key signature's own notes are not marked in a
/// name: `F4` in G major *is* F sharp, and writing `F♮4` would say the opposite.
pub fn accidental(alter: i8) -> String {
  match alter {
    a if a > 0 => SHARP.repeat(a as usize),
    a if a < 0 => FLAT.repeat(-a as usize),
    _ => String::new(),
  }
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

  /// A frame with the glyphs installed, since laying text out needs a painter
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

  /// **The accidentals reach ordinary text**, which is the whole point of them
  /// and the bug that put them here: a compass label read `F□5`, because no font
  /// egui ships has U+266F in it and a missing glyph is drawn as a box.
  ///
  /// Asked of the *text* families rather than of this one, because a fallback
  /// that is registered and never consulted looks identical from inside this
  /// file to one that was never registered — and the second is what shows a box.
  #[test]
  fn the_accidentals_are_reachable_from_text() {
    let ctx = Context::default();
    install(&ctx);
    // One frame first: egui builds no fonts until it has run once, and asking
    // before that panics rather than answering.
    ctx.run_ui(Default::default(), |_| {}).drop_without_applying_deltas();
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
      for (what, s) in [("sharp", SHARP), ("flat", FLAT)] {
        let c = s.chars().next().expect(what);
        let id = FontId::new(13.0, family.clone());
        assert!(
          ctx.fonts_mut(|f| f.has_glyph(&id, c)),
          "no {what} in {family:?}, so a label using one shows a box"
        );
      }
    }
  }

  /// **An accidental sits on the baseline; a clef hangs below its own.**
  ///
  /// The one deliberate difference between the two halves of this subset, and
  /// the thing a regenerated font would quietly lose. SMuFL centres an
  /// accidental on the notehead it belongs to, so out of the box the sharp hangs
  /// 0.35 em under the baseline — in a line of text that reads as a subscript,
  /// not as a sharp. The subsetting command raises the three accidentals and
  /// leaves the clefs alone, because a clef hanging below its baseline is
  /// precisely what puts its origin on the line it names.
  ///
  /// Measured off a laid-out glyph, which is the same path a label takes.
  #[test]
  fn an_accidental_sits_on_the_baseline_and_a_clef_does_not() {
    let size = 40.0;
    // Ink below the baseline, in ems: `pos.y` is the baseline and `uv_rect` is
    // the ink box relative to it.
    let below = |g: &egui::Galley| {
      let gl = &g.rows[0].glyphs[0];
      (gl.uv_rect.offset.y + gl.uv_rect.size.y) / size
    };
    with_painter(|p| {
      let lay = |s: &str| p.layout_no_wrap(s.to_owned(), FontId::new(size, family()), Color32::WHITE);
      for (what, s) in [("sharp", SHARP), ("flat", FLAT)] {
        let d = below(&lay(s));
        assert!(d <= 0.02, "the {what} hangs {d:.3} em below the baseline and will read as a subscript");
      }
      for (what, s) in [("G clef", G_CLEF), ("F clef", F_CLEF)] {
        let d = below(&lay(s));
        assert!(d > 0.3, "the {what} hangs only {d:.3} em below its baseline, so its origin has moved off the line it names");
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

  /// An alteration becomes the accidental a reader expects, and a natural
  /// becomes nothing — a key signature's own notes are not marked in a name.
  #[test]
  fn an_alteration_names_itself() {
    assert_eq!(accidental(0), "");
    assert_eq!(accidental(1), SHARP);
    assert_eq!(accidental(-1), FLAT);
    assert_eq!(accidental(2), format!("{SHARP}{SHARP}"));
    assert_eq!(accidental(-2), format!("{FLAT}{FLAT}"));
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
