//! Staff notation — spec 5, and it is cheap here for one reason.
//!
//! `Pitch { step, alter }` is a diatonic step and an accidental (§2.1), so
//! **staff position is `step` and nothing else**: no lookup, no key-signature
//! reasoning, no guessing which of two spellings was meant. A program that
//! stored semitones would have to decide whether 61 is a C sharp or a D flat
//! before it could place the notehead, and it would sometimes decide wrong. The
//! argument that section makes for the lattice turns out to be the argument for
//! it being drawable.
//!
//! Clefs come from an embedded SMuFL subset — `crate::glyph`, and
//! `ui/assets/README.md` for where it came from. Beams are geometry rather than
//! a font: a beam is a thick line between
//! two stem ends, and the rules for where one goes are about beats and
//! contiguity, both of which the tick lattice already answers exactly. Clef
//! glyphs are the one thing here that would want a font, and the roadmap has
//! them.

use contrapunctus::kern::{Voice, TICKS_PER_WHOLE};
use egui::{Pos2, Sense, Stroke, Ui, Vec2};

use crate::theme;

const HALF: f32 = 4.0; // half a staff space, so a step is one of these
const STAFF: f32 = HALF * 8.0; // five lines span eight half-spaces
// Room above and below the five lines. A G clef reaches 1.4 staff spaces above
// the top line and 1.6 below the bottom one — further than any ledger line this
// music needs — so it is the clef that sets this.
const BETWEEN: f32 = 34.0;

pub fn height(voices: usize) -> f32 {
  voices as f32 * (STAFF + BETWEEN) + 12.0
}

/// Where the score is looking, and how closely.
///
/// Kept apart from the drawing because it is the one part of zooming that can be
/// *wrong* rather than merely ugly, and a struct with two numbers can be tested
/// where a wheel event over a rectangle cannot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
  /// Pixels per bar.
  pub zoom: f32,
  /// How far along, in pixels.
  pub look: f32,
}

impl Default for View {
  fn default() -> View {
    View { zoom: 46.0, look: 0.0 }
  }
}

impl View {
  pub const CLOSEST: f32 = 400.0;
  pub const WIDEST: f32 = 8.0;

  /// How wide the whole piece is at this zoom, never narrower than the window
  /// it is shown in.
  pub fn content(&self, bars: f32, window: f32) -> f32 {
    (bars * self.zoom).max(window)
  }

  /// Move along by `by` pixels, staying inside the piece.
  pub fn pan(&mut self, by: f32, bars: f32, window: f32) {
    let content = self.content(bars, window);
    self.look = (self.look + by).clamp(0.0, (content - window).max(0.0));
  }

  /// Zoom by `factor`, **about the point `px` from the window's left edge**.
  ///
  /// Which is the whole difficulty: zooming about the left edge instead is what
  /// makes a reader lose their place, and keeping the bar under the pointer
  /// under the pointer costs one ratio. A test asserts that it does.
  pub fn zoom_about(&mut self, px: f32, factor: f32, bars: f32, window: f32) {
    let before = self.content(bars, window);
    let hold = (self.look + px) / before.max(1.0);
    self.zoom = (self.zoom * factor).clamp(View::WIDEST, View::CLOSEST);
    let after = self.content(bars, window);
    self.look = (hold * after - px).clamp(0.0, (after - window).max(0.0));
  }

  /// Keep `at` — a pixel into the content — inside the window, with a margin.
  /// What following the playhead means.
  pub fn keep_in_sight(&mut self, at: f32, bars: f32, window: f32) {
    let content = self.content(bars, window);
    let margin = window * 0.25;
    self.look = self.look.clamp(at - window + margin, at - margin).clamp(0.0, (content - window).max(0.0));
  }
}

/// The page, as one value.
pub struct Sheet<'a> {
  pub voices: &'a [Voice],
  pub key: &'a [i8; 7],
  pub measure: i64,
  /// What a beat is, which is what decides where a beam may run to.
  pub beat: i64,
  pub width: f32,
  pub playhead: Option<i64>,
}

impl Sheet<'_> {
/// Draw every voice, one staff each. Returns a tick to seek to, if the reader
/// clicked the page — spec 5.2, and the score is an output in every other way.
pub fn show(&self, ui: &mut Ui) -> Option<i64> {
  let (voices, key, measure, width, playhead) = (self.voices, self.key, self.measure, self.width, self.playhead);
  let want = Vec2::new(width.max(ui.available_width()), height(voices.len()));
  let (resp, p) = ui.allocate_painter(want, Sense::click());
  let area = resp.rect;
  let dark = ui.visuals().dark_mode;
  let line = ui.visuals().weak_text_color().gamma_multiply(0.55);

  let total = voices
    .iter()
    .flat_map(|v| v.notes.iter().map(|n| n.onset + n.dur))
    .max()
    .unwrap_or(1)
    .max(1);
  // A margin at the left for the staff's label, and one at the right so a final
  // note is not flush against the edge.
  let x0 = area.left() + 34.0;
  let x1 = area.right() - 8.0;
  let x_of = |t: i64| x0 + (x1 - x0) * (t as f32 / total as f32);

  for (vi, v) in voices.iter().enumerate() {
    let mid = area.top() + 6.0 + vi as f32 * (STAFF + BETWEEN) + BETWEEN / 2.0 + STAFF / 2.0;
    // The staff's middle line, in diatonic steps. Treble puts B4 (step 27+…)
    // there and bass D3; chosen per voice by where the voice actually sits,
    // which is the same mean-pitch ordering `midi::write_score` uses for tracks.
    let mean = mean_step(v);
    let treble = mean >= 27;
    let centre = if treble { 34 } else { 22 }; // B4 and D3 as diatonic steps
    let y_of = |step: i16| mid - (step - centre) as f32 * HALF;

    for k in -2..=2 {
      let y = mid + k as f32 * HALF * 2.0;
      p.line_segment([Pos2::new(x0 - 6.0, y), Pos2::new(x1, y)], Stroke::new(1.0, line));
    }
    // Bar lines, from the metre the piece already carries.
    let mut t = measure.max(1);
    while t < total {
      let x = x_of(t);
      p.line_segment([Pos2::new(x, mid - STAFF / 2.0), Pos2::new(x, mid + STAFF / 2.0)],
        Stroke::new(1.0, line.gamma_multiply(0.6)));
      t += measure.max(1);
    }

    // The clef, with its **baseline on the line it names** — the G line for
    // treble, second from the bottom, and the F line for bass, second from the
    // top. SMuFL puts each clef's origin there, so this is the definition of
    // correct placement rather than an approximation of it.
    let space = HALF * 2.0;
    let names = if treble { mid + space } else { mid - space };
    crate::glyph::clef(
      &p,
      if treble { crate::glyph::G_CLEF } else { crate::glyph::F_CLEF },
      area.left() + 3.0,
      names,
      space,
      ui.visuals().text_color(),
    );

    // Noteheads, ledger lines and accidentals: one note at a time, because none
    // of them depends on what any other note is doing.
    let heads: Vec<Head> = v
      .notes
      .iter()
      .filter(|n| n.attack)
      .map(|n| {
        let x = x_of(n.onset);
        let w = ((x_of(n.onset + n.dur) - x) * 0.55).clamp(3.0, 5.5);
        Head { onset: n.onset, dur: n.dur, step: n.pitch.step, alter: n.pitch.alter, x, y: y_of(n.pitch.step), w }
      })
      .collect();

    let c = theme::voice(vi, dark);
    for h in &heads {
      let hollow = h.dur * 4 >= TICKS_PER_WHOLE * 2; // a half note or longer

      // Ledger lines: every other step outside the five, which is what a line
      // *is* — an even offset from the middle line.
      if (h.step - centre).abs() > 4 {
        let dir = if h.step > centre { 1 } else { -1 };
        let mut st = centre + dir * 6;
        while (st - centre).abs() <= (h.step - centre).abs() {
          if (st - centre) % 2 == 0 {
            let ly = y_of(st);
            p.line_segment([Pos2::new(h.x - h.w - 2.0, ly), Pos2::new(h.x + h.w + 2.0, ly)], Stroke::new(1.0, line));
          }
          st += dir;
        }
      }

      if hollow {
        p.circle_stroke(Pos2::new(h.x, h.y), h.w, Stroke::new(1.4, c));
      } else {
        p.circle_filled(Pos2::new(h.x, h.y), h.w, c);
      }

      // An accidental where the note's own alteration differs from what the key
      // signature already says about that letter — the rule §2.1's lattice makes
      // a comparison rather than an inference.
      if h.alter != key[h.step.rem_euclid(7) as usize] {
        accidental(&p, Pos2::new(h.x - h.w - 5.0, h.y), h.alter, c);
      }
    }

    // Stems and beams, which do depend on the neighbours — a beam is the one
    // mark in notation that is about a *group*.
    for group in beam_groups(&heads, self.beat) {
      let run = &heads[group.clone()];
      // One direction for the whole group, decided by whichever note is
      // furthest from the middle line: the ordinary rule, and the one that
      // keeps a beam from crossing the staff.
      let far = run.iter().max_by_key(|h| (h.step - centre).abs()).map(|h| h.step).unwrap_or(centre);
      let up = far < centre;
      let dir = if up { -1.0 } else { 1.0 };

      // The beam sits a fixed reach from the *outermost* notehead, so it never
      // crosses one, and it tilts with the run rather than staying level.
      let reach = HALF * 7.0;
      let tip = |h: &Head| h.y + dir * reach;
      let (first, last) = (&run[0], &run[run.len() - 1]);
      let (mut y0, mut y1) = (tip(first), tip(last));
      if run.len() > 1 {
        // clamp the slope: a beam that follows a wide leap literally is a beam
        // nobody can read
        let slope = (y1 - y0).clamp(-HALF * 3.0, HALF * 3.0);
        y1 = y0 + slope;
        let along = |x: f32| y0 + (y1 - y0) * ((x - first.x) / (last.x - first.x).max(1.0));
        // and it must still clear every notehead in the run
        let push: f32 = run.iter().map(|h| (h.y + dir * (h.w + HALF * 2.0)) - along(h.x)).fold(0.0, |a: f32, b| if up { a.min(b) } else { a.max(b) });
        if (up && push < 0.0) || (!up && push > 0.0) {
          y0 += push;
          y1 += push;
        }
      }
      let at = |h: &Head| -> f32 {
        if run.len() < 2 {
          tip(h)
        } else {
          y0 + (y1 - y0) * ((h.x - first.x) / (last.x - first.x).max(1.0))
        }
      };
      let sx = |h: &Head| if up { h.x + h.w - 0.5 } else { h.x - h.w + 0.5 };

      for h in run {
        if h.dur * 2 >= TICKS_PER_WHOLE * 2 {
          continue; // a whole note has no stem
        }
        p.line_segment([Pos2::new(sx(h), h.y), Pos2::new(sx(h), at(h))], Stroke::new(1.2, c));
      }

      // One pass per beam level. A level is drawn over each maximal run of
      // notes that are short enough to need it, and a run of one gets a stub —
      // which is how a lone sixteenth among eighths is written.
      let deepest = run.iter().map(|h| beams_of(h.dur)).max().unwrap_or(0);
      for level in 1..=deepest {
        let mut k = 0;
        while k < run.len() {
          if beams_of(run[k].dur) < level {
            k += 1;
            continue;
          }
          let mut j = k;
          while j + 1 < run.len() && beams_of(run[j + 1].dur) >= level {
            j += 1;
          }
          let dy = dir * (level as f32 - 1.0) * (HALF * 1.6);
          let (mut ax, mut bx) = (sx(&run[k]), sx(&run[j]));
          let (mut ay, mut by) = (at(&run[k]) + dy, at(&run[j]) + dy);
          if k == j {
            // a stub, pointing back toward the note it belongs with — or
            // forward when there is nothing behind it
            let reach = (HALF * 2.2).min(run.len() as f32 * HALF * 2.2);
            if k > 0 {
              ax = bx - reach;
              ay = by;
            } else if run.len() > 1 {
              bx = ax + reach;
              by = ay;
            } else {
              // A single note on its own: a flag, drawn as a short stroke away
              // from the stem, which is what a flag is at this size. It leaves
              // the stem to the right whichever way the stem points — the *tilt*
              // carries the direction, and a first draft that branched on `up`
              // for the reach had both arms the same, which clippy said so.
              bx = ax + HALF * 2.0;
              by = ay + dir * -HALF * 1.6;
            }
          }
          p.line_segment([Pos2::new(ax, ay), Pos2::new(bx, by)], Stroke::new(2.2, c));
          k = j + 1;
        }
      }
    }
  }

  if let Some(t) = playhead {
    let x = x_of(t.clamp(0, total));
    p.line_segment(
      [Pos2::new(x, area.top() + 2.0), Pos2::new(x, area.bottom() - 2.0)],
      Stroke::new(1.5, ui.visuals().strong_text_color()),
    );
  }

  // Clicking the page listens from there. The same conversion the notes were
  // placed by, so the click lands where it looks like it lands.
  if resp.clicked() {
    if let Some(pos) = resp.interact_pointer_pos() {
      let f = ((pos.x - x0) / (x1 - x0).max(1.0)).clamp(0.0, 1.0);
      return Some((f as f64 * total as f64) as i64);
    }
  }
  None
}
}

/// One notehead, with everything the drawing needs already resolved.
struct Head {
  onset: i64,
  dur: i64,
  step: i16,
  alter: i8,
  x: f32,
  y: f32,
  /// Half the notehead's width, which is also how far a stem sits from centre.
  w: f32,
}

/// How many beams a duration wants. A quarter or longer wants none.
///
/// Counted **down from a quarter** rather than up from the duration, and the
/// difference is dots. A dotted eighth is three-quarters of a quarter, so
/// doubling it overshoots and the first version of this called it a quarter and
/// drew it with no beam at all — a note the corpus is full of, rendered wrong.
/// Halving from a quarter until the value is no longer than the note finds the
/// base value the dot was added to, which is what the beam count is about.
fn beams_of(dur: i64) -> usize {
  let mut n = 0;
  let mut d = contrapunctus::kern::TICKS_PER_QUARTER;
  while d > dur.max(1) && n < 8 {
    d /= 2;
    n += 1;
  }
  n
}

/// Which notes are beamed together.
///
/// Two rules, and the tick lattice answers both exactly: notes join when one
/// **ends where the next begins**, and when they fall in the **same beat**. A
/// beam that crossed a beat would hide the metre, which is the one thing a beam
/// is there to show. Notes long enough to want no beam are groups of one, so
/// that the stem drawing below has a single path through.
fn beam_groups(heads: &[Head], beat: i64) -> Vec<std::ops::Range<usize>> {
  let beat = beat.max(1);
  let mut out: Vec<std::ops::Range<usize>> = vec![];
  let mut start = 0usize;
  for i in 0..heads.len() {
    let joins = i > start
      && beams_of(heads[i].dur) > 0
      && beams_of(heads[i - 1].dur) > 0
      && heads[i - 1].onset + heads[i - 1].dur == heads[i].onset
      && heads[i - 1].onset / beat == heads[i].onset / beat;
    if !joins && i > start {
      out.push(start..i);
      start = i;
    }
  }
  if start < heads.len() {
    out.push(start..heads.len());
  }
  out
}

/// Draw a sharp, flat or natural from line segments. Three glyphs, no font, and
/// they are the only glyphs the score needs before beaming arrives.
fn accidental(p: &egui::Painter, at: Pos2, alter: i8, c: egui::Color32) {
  let s = Stroke::new(1.1, c);
  match alter {
    a if a > 0 => {
      for dx in [-1.5, 1.5] {
        p.line_segment([Pos2::new(at.x + dx, at.y - 5.0), Pos2::new(at.x + dx, at.y + 5.0)], s);
      }
      for dy in [-1.8, 1.8] {
        p.line_segment([Pos2::new(at.x - 3.4, at.y + dy + 0.8), Pos2::new(at.x + 3.4, at.y + dy - 0.8)], s);
      }
    }
    a if a < 0 => {
      p.line_segment([Pos2::new(at.x - 1.6, at.y - 6.0), Pos2::new(at.x - 1.6, at.y + 3.4)], s);
      p.line_segment([Pos2::new(at.x - 1.6, at.y + 3.4), Pos2::new(at.x + 2.2, at.y + 0.6)], s);
      p.line_segment([Pos2::new(at.x + 2.2, at.y + 0.6), Pos2::new(at.x - 1.6, at.y - 1.4)], s);
    }
    _ => {
      p.line_segment([Pos2::new(at.x - 1.8, at.y - 5.0), Pos2::new(at.x - 1.8, at.y + 2.6)], s);
      p.line_segment([Pos2::new(at.x + 1.8, at.y - 2.6), Pos2::new(at.x + 1.8, at.y + 5.0)], s);
      for dy in [-2.0, 2.0] {
        p.line_segment([Pos2::new(at.x - 1.8, at.y + dy + 0.6), Pos2::new(at.x + 1.8, at.y + dy - 0.6)], s);
      }
    }
  }
}

fn mean_step(v: &Voice) -> i16 {
  let n = v.notes.iter().filter(|n| n.attack).count().max(1);
  (v.notes.iter().filter(|n| n.attack).map(|n| n.pitch.step as i32).sum::<i32>() / n as i32) as i16
}

#[cfg(test)]
mod tests {
  use super::*;
  use contrapunctus::kern::TICKS_PER_QUARTER as Q;

  fn heads(spec: &[(i64, i64)]) -> Vec<Head> {
    spec
      .iter()
      .map(|&(onset, dur)| Head { onset, dur, step: 28, alter: 0, x: onset as f32, y: 0.0, w: 4.0 })
      .collect()
  }

  /// How many beams a duration wants, against the values that actually occur.
  #[test]
  fn a_duration_wants_the_beams_it_should() {
    assert_eq!(beams_of(Q * 4), 0, "a whole note");
    assert_eq!(beams_of(Q * 2), 0, "a half");
    assert_eq!(beams_of(Q), 0, "a quarter");
    assert_eq!(beams_of(Q / 2), 1, "an eighth");
    assert_eq!(beams_of(Q / 4), 2, "a sixteenth");
    assert_eq!(beams_of(Q / 8), 3, "a thirty-second");
    // **Dots.** A dotted eighth is three-quarters of a quarter, and it carries
    // the beams of the eighth it is a dot on. Counting up from the duration
    // overshoots and calls it a quarter, which is what the first version did.
    assert_eq!(beams_of(Q / 2 + Q / 4), 1, "a dotted eighth");
    assert_eq!(beams_of(Q / 4 + Q / 8), 2, "a dotted sixteenth");
    assert_eq!(beams_of(Q + Q / 2), 0, "a dotted quarter");
  }

  /// **A beam runs within a beat and across nothing else.**
  ///
  /// The two rules, and both are exact here because the tick lattice is: notes
  /// join when one ends where the next begins, and when they fall in the same
  /// beat. A beam that crossed a beat would hide the metre, which is the one
  /// thing a beam is there to show.
  #[test]
  fn a_beam_stays_inside_its_beat() {
    // four sixteenths filling one beat: one group
    let g = beam_groups(&heads(&[(0, Q / 4), (Q / 4, Q / 4), (Q / 2, Q / 4), (3 * Q / 4, Q / 4)]), Q);
    assert_eq!(g, vec![0..4]);

    // eight of them fill two beats: two groups, not one
    let spec: Vec<(i64, i64)> = (0..8).map(|i| (i * Q / 4, Q / 4)).collect();
    assert_eq!(beam_groups(&heads(&spec), Q), vec![0..4, 4..8]);

    // a gap between them breaks the beam even inside a beat
    let g = beam_groups(&heads(&[(0, Q / 4), (Q / 2, Q / 4)]), Q);
    assert_eq!(g, vec![0..1, 1..2], "a rest between two notes must break the beam");

    // and a quarter is never beamed to anything
    let g = beam_groups(&heads(&[(0, Q / 2), (Q / 2, Q / 2), (Q, Q)]), Q);
    assert_eq!(g, vec![0..2, 2..3]);
  }

  /// Every note belongs to exactly one group, in order — the property the
  /// drawing relies on and the one an off-by-one in the grouping would break.
  #[test]
  fn the_groups_cover_every_note_once() {
    let spec: Vec<(i64, i64)> = vec![
      (0, Q / 4),
      (Q / 4, Q / 4),
      (Q / 2, Q / 2),
      (Q, Q),
      (2 * Q, Q / 2),
      (2 * Q + Q / 2, Q / 4),
      (2 * Q + 3 * Q / 4, Q / 4),
    ];
    let hs = heads(&spec);
    let groups = beam_groups(&hs, Q);
    let mut seen = 0usize;
    for (i, g) in groups.iter().enumerate() {
      assert_eq!(g.start, seen, "group {i} does not start where the last one ended");
      assert!(g.end > g.start, "group {i} is empty");
      seen = g.end;
    }
    assert_eq!(seen, hs.len(), "the groups do not reach the last note");
  }

  /// The real thing: every voice of a generated fugue groups without panicking
  /// and covers all its notes.
  #[test]
  fn a_generated_fugue_beams_end_to_end() {
    let d = crate::catalog::load().subjects[1].design(3);
    let o = contrapunctus::compose::fugue(
      &d,
      &contrapunctus::compose::Layout::default(),
      contrapunctus::automaton::Tier::Full.rules(),
      0x5EED,
    )
    .expect("a fugue");

    let mut beamed = 0usize;
    for v in &o.voices {
      let hs: Vec<Head> = v
        .notes
        .iter()
        .filter(|n| n.attack)
        .map(|n| Head { onset: n.onset, dur: n.dur, step: n.pitch.step, alter: n.pitch.alter, x: 0.0, y: 0.0, w: 4.0 })
        .collect();
      let groups = beam_groups(&hs, d.beat);
      let mut seen = 0;
      for g in &groups {
        assert_eq!(g.start, seen);
        seen = g.end;
      }
      assert_eq!(seen, hs.len());
      beamed += groups.iter().filter(|g| g.len() > 1).count();
    }
    assert!(beamed > 0, "nothing in the whole fugue got beamed, so this checked nothing");
  }
}

#[cfg(test)]
mod view_tests {
  use super::View;

  const BARS: f32 = 27.0;
  const WINDOW: f32 = 800.0;

  /// **Zooming about the pointer leaves the bar under it under it.**
  ///
  /// The property the whole gesture is judged by, and the one an interface gets
  /// wrong by zooming about the left edge — which looks fine on the first notch
  /// and has thrown the reader across the page by the fourth.
  #[test]
  fn a_bar_under_the_pointer_stays_under_it() {
    for px in [10.0f32, 200.0, 400.0, 790.0] {
      for factor in [1.1f32, 0.9, 2.0, 0.5] {
        let mut v = View { zoom: 46.0, look: 300.0 };
        let before = (v.look + px) / v.content(BARS, WINDOW);
        v.zoom_about(px, factor, BARS, WINDOW);
        let after = (v.look + px) / v.content(BARS, WINDOW);
        // only where the clamp is not what decided it — at the ends of the
        // piece there is nowhere further to go, and staying put is correct
        let stuck = v.look <= 0.0 || v.look >= (v.content(BARS, WINDOW) - WINDOW).max(0.0) - 0.01;
        if !stuck {
          assert!((before - after).abs() < 1e-3, "px {px}, factor {factor}: {before:.4} became {after:.4}");
        }
      }
    }
  }

  /// Zoom stays between its limits however hard the wheel is turned, and the
  /// piece never scrolls off either end.
  #[test]
  fn the_view_stays_inside_the_piece() {
    let mut v = View::default();
    for _ in 0..200 {
      v.zoom_about(400.0, 1.3, BARS, WINDOW);
      assert!(v.zoom <= View::CLOSEST);
      assert!(v.look >= 0.0 && v.look <= (v.content(BARS, WINDOW) - WINDOW).max(0.0) + 0.01);
    }
    for _ in 0..200 {
      v.zoom_about(400.0, 0.7, BARS, WINDOW);
      assert!(v.zoom >= View::WIDEST);
      assert!(v.look >= 0.0 && v.look <= (v.content(BARS, WINDOW) - WINDOW).max(0.0) + 0.01);
    }
    for by in [-10_000.0f32, 10_000.0] {
      v.pan(by, BARS, WINDOW);
      assert!(v.look >= 0.0 && v.look <= (v.content(BARS, WINDOW) - WINDOW).max(0.0) + 0.01, "panned to {}", v.look);
    }
  }

  /// Zoomed all the way out, the piece fits and there is nowhere to pan to —
  /// which is the plan strip's permanent state and has to be a legal one here.
  #[test]
  fn a_piece_that_fits_does_not_scroll() {
    let mut v = View { zoom: View::WIDEST, look: 0.0 };
    assert_eq!(v.content(BARS, WINDOW), WINDOW, "27 bars at the widest zoom should fit in 800 px");
    v.pan(500.0, BARS, WINDOW);
    assert_eq!(v.look, 0.0, "there was nowhere to go and it went somewhere");
  }

  /// Following the playhead brings it inside the window and leaves it there.
  #[test]
  fn following_keeps_the_playhead_on_the_page() {
    let mut v = View { zoom: 120.0, look: 0.0 };
    let content = v.content(BARS, WINDOW);
    for at in [0.0f32, 500.0, 1500.0, content - 1.0] {
      v.keep_in_sight(at, BARS, WINDOW);
      let on = at >= v.look && at <= v.look + WINDOW;
      assert!(on, "the playhead at {at} is not on a page showing {}..{}", v.look, v.look + WINDOW);
    }
  }
}
