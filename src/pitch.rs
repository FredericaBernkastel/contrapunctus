//! Pitch on the lattice: a diatonic step and a chromatic alteration.
//!
//! Not a semitone integer, and the difference is the whole point. A diminished
//! fifth and an augmented fourth are the same six semitones and are *different
//! intervals* — one is a dissonance to be resolved inward, the other outward,
//! and a melodic augmented fourth is forbidden where a diminished fifth is
//! merely awkward. A model that cannot tell them apart cannot state the rules.
//!
//! This is also readme §2.1's correction arriving in the type: Giraud et al.
//! match on diatonic intervals rather than semitones because "a scale will
//! always match only a scale", and the tonal answer alters a subject's opening
//! interval without altering its scale degrees.

/// `step` counts diatonic degrees from C0 (C0 = 0, D0 = 1, … B0 = 6, C1 = 7);
/// `alter` is the accidental in semitones.
///
/// `Hash` and `Ord` are derived because §8.6's search keys its dynamic-
/// programming states on the tuple of sounding pitches.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct Pitch {
  pub step: i16,
  pub alter: i8,
}

/// Semitones above C for each diatonic degree within an octave.
const SEMIS: [i16; 7] = [0, 2, 4, 5, 7, 9, 11];

impl Pitch {
  pub fn new(step: i16, alter: i8) -> Self {
    Self { step, alter }
  }

  /// Semitones above C0 — the sounding pitch, with C4 at 48 diatonic steps.
  pub fn chroma(&self) -> i16 {
    let oct = self.step.div_euclid(7);
    let deg = self.step.rem_euclid(7) as usize;
    oct * 12 + SEMIS[deg] + self.alter as i16
  }

  /// MIDI note number, for reporting only. C4 (`step = 28`) is 60.
  pub fn midi(&self) -> i16 {
    self.chroma() + 12
  }

  /// Transpose by a **named** interval — so many diatonic steps and so many
  /// semitones — which is the only kind of transposition that preserves the
  /// spelling a fugue depends on. Adding semitones alone would turn a
  /// diminished fifth into an augmented fourth somewhere down the subject.
  pub fn transpose(&self, dsteps: i16, dsemis: i16) -> Pitch {
    let target = Pitch::new(self.step + dsteps, 0);
    let natural = target.chroma() - Pitch::new(self.step, 0).chroma();
    Pitch::new(self.step + dsteps, self.alter + (dsemis - natural) as i8)
  }

  /// Spelled name, for reporting: `B-4`, `F#5`.
  pub fn name(&self) -> String {
    let deg = "CDEFGAB".as_bytes()[self.step.rem_euclid(7) as usize] as char;
    let acc = match self.alter {
      -2 => "--",
      -1 => "-",
      0 => "",
      1 => "#",
      2 => "##",
      _ => "?",
    };
    format!("{deg}{acc}{}", self.step.div_euclid(7))
  }
}

/// A directed interval, kept as the pair that determines its quality: how many
/// diatonic degrees it spans, and how many semitones.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Interval {
  /// Diatonic size − 1: unison 0, second 1, … octave 7.
  pub steps: i16,
  pub semis: i16,
}

impl Interval {
  pub fn between(lo: Pitch, hi: Pitch) -> Self {
    Self { steps: hi.step - lo.step, semis: hi.chroma() - lo.chroma() }
  }

  /// Reduced to within an octave, keeping a true octave distinct from a unison
  /// only in that both are perfect — which is all the rules need.
  pub fn simple(&self) -> Self {
    let steps = self.steps.rem_euclid(7);
    let semis = self.semis.rem_euclid(12);
    Self { steps, semis }
  }

  /// Unused since the melodic rule was folded into `is_forbidden_melodic`,
  /// which tests the same thing more directly. Kept because an interval type
  /// that cannot say whether it is compound is a worse interval type.
  #[allow(dead_code)]
  pub fn is_compound(&self) -> bool {
    self.steps.abs() > 7
  }

  /// The vertical classification the rulebook is written in.
  ///
  /// The fourth is a **dissonance** here. That is the classical position and
  /// the one both Schottstaedt and Komosinski & Szachewicz adopt, the latter
  /// noting explicitly that modern practice disagrees. It matters: a texture
  /// judged by this classification will flag suspensions built on fourths.
  pub fn quality(&self) -> Quality {
    let s = self.simple();
    match (s.steps, s.semis) {
      (0, 0) => Quality::PerfectConsonance, // unison / octave
      (4, 7) => Quality::PerfectConsonance, // perfect fifth
      (2, 3) | (2, 4) => Quality::ImperfectConsonance, // minor / major third
      (5, 8) | (5, 9) => Quality::ImperfectConsonance, // minor / major sixth
      _ => Quality::Dissonance,
    }
  }

  /// A melodic step is a second; anything larger is a leap.
  pub fn is_step(&self) -> bool {
    self.steps.abs() == 1
  }

  #[allow(dead_code)] // the complement of `is_step`, and used by neither
  pub fn is_leap(&self) -> bool {
    self.steps.abs() >= 2
  }

  /// Melodic intervals Fux forbids outright as unsingable: any augmented or
  /// diminished interval, the seventh, and anything beyond an octave.
  ///
  /// Recognised by the mismatch between diatonic size and semitone size, which
  /// is exactly what a semitone-only representation cannot see.
  pub fn is_forbidden_melodic(&self) -> bool {
    let (st, se) = (self.steps.abs(), self.semis.abs());
    if st > 7 || se > 12 {
      return true;
    }
    if st == 6 {
      return true; // any seventh
    }
    // perfect/major/minor sizes for each diatonic span; anything else is
    // augmented or diminished
    let ok: &[i16] = match st {
      0 => &[0],
      1 => &[1, 2],
      2 => &[3, 4],
      3 => &[5],
      4 => &[7],
      5 => &[8, 9],
      7 => &[12],
      _ => &[],
    };
    !ok.contains(&se)
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Quality {
  PerfectConsonance,
  ImperfectConsonance,
  Dissonance,
}

impl Quality {
  pub fn is_consonant(&self) -> bool {
    !matches!(self, Quality::Dissonance)
  }
}

/// Parse a Humdrum `**kern` pitch: letters give degree and octave (lowercase
/// `c` is C4 and repeats ascend, uppercase `C` is C3 and repeats descend),
/// then `#`/`-`/`n` give the accidental.
pub fn parse_kern_pitch(tok: &str) -> Option<Pitch> {
  let mut letter = None;
  let mut count = 0i16;
  let mut upper = false;
  let mut alter = 0i8;

  for c in tok.chars() {
    match c {
      'a'..='g' | 'A'..='G' => {
        let l = c.to_ascii_lowercase();
        match letter {
          None => {
            letter = Some(l);
            upper = c.is_ascii_uppercase();
            count = 1;
          }
          Some(prev) if prev == l && c.is_ascii_uppercase() == upper => count += 1,
          Some(_) => return None, // a chord or a malformed token
        }
      }
      '#' => alter += 1,
      '-' => alter -= 1,
      'n' => {}
      _ => {}
    }
  }

  let l = letter?;
  let deg = "cdefgab".find(l)? as i16;
  // C4 is diatonic step 28; uppercase starts an octave lower and descends.
  let octave = if upper { 3 - (count - 1) } else { 4 + (count - 1) };
  Some(Pitch::new(octave * 7 + deg, alter))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn kern_octaves_and_accidentals() {
    assert_eq!(parse_kern_pitch("c"), Some(Pitch::new(28, 0))); // C4
    assert_eq!(parse_kern_pitch("cc"), Some(Pitch::new(35, 0))); // C5
    assert_eq!(parse_kern_pitch("C"), Some(Pitch::new(21, 0))); // C3
    assert_eq!(parse_kern_pitch("CC"), Some(Pitch::new(14, 0))); // C2
    assert_eq!(parse_kern_pitch("c").unwrap().midi(), 60);
    assert_eq!(parse_kern_pitch("b-").unwrap().midi(), 70); // B flat 4
    assert_eq!(parse_kern_pitch("f#").unwrap().midi(), 66);
  }

  #[test]
  fn the_tritone_is_two_different_intervals() {
    // F4 up to B4 is an augmented fourth; B4 up to F5 a diminished fifth.
    let f = parse_kern_pitch("f").unwrap();
    let b = parse_kern_pitch("b").unwrap();
    let ff = parse_kern_pitch("ff").unwrap();
    let aug4 = Interval::between(f, b);
    let dim5 = Interval::between(b, ff);
    assert_eq!((aug4.steps, aug4.semis), (3, 6));
    assert_eq!((dim5.steps, dim5.semis), (4, 6));
    assert_ne!(aug4.steps, dim5.steps); // same semitones, different intervals
    assert!(aug4.is_forbidden_melodic() && dim5.is_forbidden_melodic());
  }

  #[test]
  fn the_fifth_is_consonant_and_the_semitone_is_not() {
    let c = parse_kern_pitch("c").unwrap();
    let g = parse_kern_pitch("g").unwrap();
    let df = parse_kern_pitch("d-").unwrap();
    assert_eq!(Interval::between(c, g).quality(), Quality::PerfectConsonance);
    assert_eq!(Interval::between(c, df).quality(), Quality::Dissonance);
  }
}
