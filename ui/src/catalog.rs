//! The subjects on offer — spec 3.2's *the subject*.
//!
//! Read from `embedded::FUGUES`, which is the corpus compiled into the library,
//! so this works in a browser with no network fetch and reports the same notes
//! the measurement binary reads off disk (spec 7.1).
//!
//! Nothing here is hard-coded that the corpus can answer. The key's name is
//! spelled from the piece's own key signature rather than from a table, which
//! is `Pitch { step, alter }` earning its keep again: the tonic of BWV 853 is
//! **E flat minor** and not D sharp minor, and only the spelling knows that.

use contrapunctus::{
  answer, compose,
  kern::{self, Piece, Voice},
};

/// One offered subject, with everything a [`compose::Design`] needs.
pub struct Subject {
  pub id: String,
  /// `No. 2 in C minor, BWV 847`.
  pub name: String,
  /// How many voices Bach wrote it in — not how many we will.
  pub scored_for: usize,
  pub notes: Voice,
  pub key: [i8; 7],
  pub tonic: usize,
  pub measure: i64,
  pub beat: i64,
}

impl Subject {
  /// A design for this subject in `voices` parts.
  pub fn design(&self, voices: usize) -> compose::Design {
    compose::Design {
      subject: self.notes.clone(),
      voices,
      key: self.key,
      tonic: self.tonic,
      measure: self.measure,
      beat: self.beat,
      compass: compass(voices),
    }
  }
}

/// Each voice's range, top first.
///
/// Three is the published layout — §8.16's own, and what every figure in that
/// section was measured with, so it is reproduced exactly rather than derived
/// from a formula that happens to agree.
///
/// Each box is **twelve diatonic steps**, a thirteenth, and the three overlap by
/// a ninth at the top pair and a sixth at the bottom. This comment used to call
/// them two-octave boxes overlapping by about a sixth, which was wrong twice and
/// went unnoticed for as long as the numbers were only numbers. `ui::compass`
/// now draws them on a staff, where anybody can count the lines.
pub fn compass(voices: usize) -> Vec<(i16, i16)> {
  match voices {
    2 => vec![(33, 45), (24, 36)],
    3 => vec![(33, 45), (28, 40), (21, 33)],
    _ => (0..voices).map(|v| { let top = 33 - 4 * v as i16; (top, top + 12) }).collect(),
  }
}

/// Every subject the corpus can supply one for, in the book's order.
///
/// A fugue whose subject cannot be located is **left out and counted**, not
/// silently skipped: [`Catalog::missing`] is shown in Advanced mode, because a
/// picker listing 22 of 24 with no explanation is a picker that lies.
pub struct Catalog {
  pub subjects: Vec<Subject>,
  pub missing: Vec<(String, &'static str)>,
}

pub fn load() -> Catalog {
  let pieces = contrapunctus::embedded::pieces();
  let specs = contrapunctus::embedded::specs(&pieces);

  let mut subjects = vec![];
  let mut missing = vec![];
  for (n, p) in pieces.iter().enumerate() {
    match subject_of(p, n, &specs) {
      Ok(s) => subjects.push(s),
      Err(why) => missing.push((p.id.clone(), why)),
    }
  }
  Catalog { subjects, missing }
}

fn subject_of(
  p: &Piece,
  n: usize,
  specs: &[contrapunctus::refdata::SubjectSpec],
) -> Result<Subject, &'static str> {
  let spec = specs.iter().find(|s| s.id == p.id).ok_or("not annotated")?;
  let (letter, at) = *spec.entries.first().ok_or("no annotated entry")?;
  let (pc, minor) = p.tonic.ok_or("no key interpretation")?;
  let tonic = answer::tonic_letter(pc, &p.key).ok_or("tonic is not a letter of the signature")?;

  let notes = kern::clip(&p.voices[voice_of(p, letter)], at, at + spec.len);
  if notes.notes.is_empty() {
    return Err("the annotated span holds no notes");
  }

  Ok(Subject {
    id: p.id.clone(),
    name: format!("No. {} in {}, BWV {}", n + 1, key_name(tonic, &p.key, minor), 845 + n + 1),
    scored_for: p.voices.len(),
    notes,
    key: p.key,
    tonic,
    measure: p.measure,
    beat: p.beat,
  })
}

/// The annotations name a voice by letter; the reader numbers voices top-first.
/// The same mapping `step5` uses, and the reason it is a mapping at all is that
/// `S A T B` is a count from the bottom and `voices[0]` is the top.
fn voice_of(p: &Piece, letter: char) -> usize {
  let order = ['S', 'A', 'T', 'B', 'C'];
  let i = order.iter().position(|&c| c == letter).unwrap_or(0);
  p.voices.len().saturating_sub(1 + i).min(p.voices.len() - 1)
}

/// `C minor`, `E flat minor`, `F sharp major` — spelled from the signature.
fn key_name(tonic: usize, key: &[i8; 7], minor: bool) -> String {
  const LETTER: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
  let alter = match key[tonic] {
    -2 => " double flat",
    -1 => " flat",
    0 => "",
    1 => " sharp",
    _ => " double sharp",
  };
  format!("{}{alter} {}", LETTER[tonic], if minor { "minor" } else { "major" })
}

/// The three journeys of spec 3.2, which is what *how far it travels* sets.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Journey {
  Home,
  Wander,
  Roam,
}

impl Journey {
  pub const ALL: [Journey; 3] = [Journey::Home, Journey::Wander, Journey::Roam];

  pub fn middles(self) -> Vec<i16> {
    match self {
      Journey::Home => vec![4],
      Journey::Wander => vec![4, 5, 3],
      Journey::Roam => vec![4, 5, 1, 3, 6],
    }
  }

  pub fn label(self) -> &'static str {
    match self {
      Journey::Home => "Stays home",
      Journey::Wander => "Wanders",
      Journey::Roam => "Roams",
    }
  }

  pub fn hint(self) -> &'static str {
    match self {
      Journey::Home => "One trip to the dominant and back.",
      Journey::Wander => "The default — the median of three returns, which is what §8.15 found in the book.",
      Journey::Roam => "Five returns, near the top of the range the book actually uses.",
    }
  }

  /// Which preset a set of middles is, if it is one. An edited plan is none of
  /// them, and the interface says so rather than showing a preset that is no
  /// longer what is on screen.
  pub fn of(middles: &[i16]) -> Option<Journey> {
    Journey::ALL.into_iter().find(|j| j.middles() == middles)
  }
}

/// The degree names a middle can carry, for the key chips of spec 4.2.
pub fn degree_name(deg: i16) -> &'static str {
  match deg.rem_euclid(7) {
    0 => "home",
    1 => "II",
    2 => "III",
    3 => "IV",
    4 => "V",
    5 => "VI",
    _ => "VII",
  }
}
