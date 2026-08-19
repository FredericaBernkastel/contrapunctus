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
//! What is not here yet is beaming and real clef glyphs; the roadmap says so.

use contrapunctus::kern::{Voice, TICKS_PER_WHOLE};
use egui::{Align2, FontId, Pos2, Response, Sense, Stroke, Ui, Vec2};

use crate::theme;

const HALF: f32 = 4.0; // half a staff space, so a step is one of these
const STAFF: f32 = HALF * 8.0; // five lines span eight half-spaces
const BETWEEN: f32 = 30.0; // room above and below for ledger lines

pub fn height(voices: usize) -> f32 {
  voices as f32 * (STAFF + BETWEEN) + 12.0
}

/// Draw every voice, one staff each, sharing a horizontal scale.
pub fn show(ui: &mut Ui, voices: &[Voice], key: &[i8; 7], measure: i64, width: f32) -> Response {
  let want = Vec2::new(width.max(ui.available_width()), height(voices.len()));
  let (resp, p) = ui.allocate_painter(want, Sense::hover());
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

    // No clef glyph yet — the name of the bottom line instead, which is at least
    // as useful to spec 1's beginner and needs no embedded font. The roadmap
    // has the SMuFL subset that replaces this.
    p.text(
      Pos2::new(area.left() + 2.0, mid + STAFF / 2.0),
      Align2::LEFT_CENTER,
      note_name(centre - 4),
      FontId::monospace(9.0),
      ui.visuals().weak_text_color(),
    );

    for n in v.notes.iter().filter(|n| n.attack) {
      let step = n.pitch.step;
      let y = y_of(step);
      let x = x_of(n.onset);
      let w = ((x_of(n.onset + n.dur) - x) * 0.55).clamp(3.0, 5.5);
      let hollow = n.dur * 4 >= TICKS_PER_WHOLE * 2; // a half note or longer

      // Ledger lines: every other step outside the five, which is what a line
      // *is* — an even offset from the middle line.
      let out_of = |s: i16| (s - centre).abs() > 4;
      if out_of(step) {
        let dir = if step > centre { 1 } else { -1 };
        let mut s = centre + dir * 6;
        while (s - centre).abs() <= (step - centre).abs() {
          if (s - centre) % 2 == 0 {
            let ly = y_of(s);
            p.line_segment([Pos2::new(x - w - 2.0, ly), Pos2::new(x + w + 2.0, ly)], Stroke::new(1.0, line));
          }
          s += dir;
        }
      }

      let c = theme::voice(vi, dark);
      if hollow {
        p.circle_stroke(Pos2::new(x, y), w, Stroke::new(1.4, c));
      } else {
        p.circle_filled(Pos2::new(x, y), w, c);
      }
      // Stem: up below the middle line, down above it, which is the ordinary
      // convention and keeps the staff from growing.
      let up = step < centre;
      let sx = if up { x + w - 0.5 } else { x - w + 0.5 };
      let sy = if up { y - HALF * 7.0 } else { y + HALF * 7.0 };
      if n.dur * 2 < TICKS_PER_WHOLE * 2 {
        p.line_segment([Pos2::new(sx, y), Pos2::new(sx, sy)], Stroke::new(1.2, c));
      }

      // An accidental where the note's own alteration differs from what the key
      // signature already says about that letter — the rule §2.1's lattice makes
      // a comparison rather than an inference.
      let sig = key[step.rem_euclid(7) as usize];
      if n.pitch.alter != sig {
        accidental(&p, Pos2::new(x - w - 5.0, y), n.pitch.alter, c);
      }
    }
  }

  resp
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

/// `E4` — the letter and the octave, from the diatonic step alone.
fn note_name(step: i16) -> String {
  const LETTER: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
  format!("{}{}", LETTER[step.rem_euclid(7) as usize], step.div_euclid(7))
}
