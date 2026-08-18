//! Marpurg's tonal answer — the *Gefährte* of Hauptstück 3.
//!
//! The first thing this project has transcribed from a treatise of **Bach's own
//! circle** rather than from Fux, and [§9](../readme.md)'s standing question
//! about which rulebook fits the WTC is why. Marpurg, *Abhandlung von der Fuge*
//! (1753), Erster Theil, drittes Hauptstück, "Vom Gefährten".
//!
//! # What the chapter says
//!
//! The answer rests on **two Grundsätze**, and Marpurg numbers them:
//!
//! 1. *"Der Gesang des Gefährten muß dem Gesange des Führers ähnlich gemacht
//!    werden."* The answer keeps the subject's figure and note values, its time
//!    signature, its rests, its mode, and — chiefly — its intervals *in the same
//!    proportion*: where the subject moves by a third the answer moves by a
//!    third, and a major third stays major.
//! 2. *"Es muß recht moduliert werden."* The answer must not carry the music
//!    into a key foreign to the home one.
//!
//! The two conflict, and the conflict is the whole subject of the chapter. It
//! arises because *"die Octave aus zwey ungleichen Hälften besteht"*: from tonic
//! up to dominant is **five** notes and from dominant up to tonic is **four**, so
//! a subject that crosses between the halves cannot both keep every interval and
//! stay in the key. One interval has to give.
//!
//! # The mutation
//!
//! Marpurg calls it a *Vertauschung* and says exactly what it does:
//!
//! - *"durch Überschlagung einer Stufe"* — skipping a degree, in the **larger**
//!   half. This *"heißt den Gesang erweitern"*.
//! - *"durch Verdoppelung einer Stufe"* — striking a note twice, in the
//!   **smaller** half, which *"abkürzen oder einschränken"* — contracts it.
//!
//! and states the consequence as a table of interval substitutions: a unison
//! becomes a second, a second a third, and so on to a seventh becoming an octave;
//! and the reverse. **One melodic interval changes by exactly one scale degree.**
//!
//! That is what [`Leg`] models. Transposing one note up a fifth and the next up a
//! fourth widens or narrows the interval between them by one degree and by
//! nothing else, so a choice of `Leg` per note *is* Marpurg's mutation, and a
//! single change of `Leg` along the subject is a single *Vertauschung*.
//!
//! Where the change falls, Marpurg settles by a rule of thumb — *"daß man
//! allezeit eher auf das folgende als auf das vorhergehende sehen müsse"*, look
//! forward rather than back — and then by thirty worked examples on his plates.
//! **A rule of thumb is not transcribable and worked examples are not a rule**,
//! so this module does not pick a point. It enumerates every point the stated
//! rules leave open, which is what [`admissible`] returns and why the thing it
//! returns is a set. §8.7's instrument then applies unchanged: a whitelist is
//! worth having only if the music stays inside it, and the set's size is what
//! says whether staying inside it means anything.
//!
//! # What the two named rules fix
//!
//! §4 pins the ends of the subject, and these are quoted rather than inferred:
//!
//! **I. Über die erste Note des Führers.** Beginning on the tonic, the answer
//! follows on the fifth; beginning on the fifth, the answer follows on the tonic
//! — *"die Haupttonsnote und die Dominante müssen allezeit einander antworten
//! auf der ersten Note des Fugensatzes."* Marpurg calls these the *ordentliche*
//! openings and defers the others to later sections, so [`first_leg`] leaves
//! every other degree free.
//!
//! **II. Über die letzte Note des Führers.** Tonic answers dominant and dominant
//! tonic; the third of the tonic answers the third of the dominant and back
//! again. A subject closing instead on the second, fourth or sixth of the tonic
//! is answered on the second, fourth or sixth of the dominant. Marpurg adds that
//! this rule *"öfters nach Beschaffenheit der Umstände ihre Ausnahmen leiden
//! kann"*, so it is a default here and not a filter.
//!
//! # What is deliberately left out
//!
//! - The **half** a mutation belongs in — skip in the larger, double in the
//!   smaller. It further constrains where the change of `Leg` may fall, and
//!   reading it off the OCR of a Fraktur setting closely enough to encode would
//!   be guessing at the author's meaning rather than transcribing it.
//! - Answers **in the old modes**, which the chapter treats separately and which
//!   need the mode reduction Marpurg describes for a *verstellte Tonart*.
//! - Openings on the third, fourth, sixth, second or seventh, which the chapter
//!   defers to *besondere Abschnitte*.

use crate::{
  kern::{Note, Voice},
  pitch::Pitch,
};

/// The two transpositions a fifth-fugue answer may use at any one note.
///
/// Both are diatonic and within the key, so `Fifth` is four scale steps and
/// `Fourth` is three whatever the degree — the point of §2.1's lattice. The
/// difference between them is one degree, which is Marpurg's *Vertauschung*
/// exactly.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Leg {
  Fifth,
  Fourth,
}

impl Leg {
  pub fn steps(self) -> i16 {
    match self {
      Leg::Fifth => 4,
      Leg::Fourth => 3,
    }
  }
  pub fn other(self) -> Leg {
    match self {
      Leg::Fifth => Leg::Fourth,
      Leg::Fourth => Leg::Fifth,
    }
  }
}

/// A pitch's scale degree, `0` for the tonic and `4` for the dominant.
pub fn degree(p: Pitch, tonic: usize) -> usize {
  (p.step - tonic as i16).rem_euclid(7) as usize
}

/// Which diatonic letter the key's tonic is, given its pitch class.
///
/// The pitch class alone does not say: `E flat` and `D sharp` are one class and
/// two letters, and a fugue in E flat whose tonic came back as `D sharp` would
/// have every degree in this module off by one. The key signature settles it,
/// which is why it is an argument.
pub fn tonic_letter(pc: u8, key: &[i8; 7]) -> Option<usize> {
  (0..7).find(|&l| Pitch::new(l as i16, key[l]).chroma().rem_euclid(12) == pc as i16)
}

/// Marpurg's Rule I, on the subject's first note. `None` where he defers the
/// case rather than states it.
pub fn first_leg(deg: usize) -> Option<Leg> {
  match deg {
    0 => Some(Leg::Fifth),  // tonic answered by the dominant
    4 => Some(Leg::Fourth), // dominant answered by the tonic
    _ => None,
  }
}

/// Marpurg's Rule II, on the subject's last note. Every degree is covered:
/// tonic and dominant answer each other, the two thirds answer each other, and
/// the second, fourth and sixth of the tonic are answered by those of the
/// dominant — which is the fifth in each case.
pub fn last_leg(deg: usize) -> Option<Leg> {
  match deg {
    4 | 6 => Some(Leg::Fourth),
    _ => Some(Leg::Fifth),
  }
}

/// One pitch moved `n` diatonic steps **within the key**, keeping whatever
/// chromatic inflection it had relative to the scale — so an accidental foreign
/// to the key stays foreign, which is Grundsatz 1 applied to spelling.
pub fn step_in_key(p: Pitch, n: i16, key: &[i8; 7]) -> Pitch {
  let from = p.step.rem_euclid(7) as usize;
  let inflection = p.alter - key[from];
  let step = p.step + n;
  Pitch::new(step, key[step.rem_euclid(7) as usize] + inflection)
}

/// Every assignment of a `Leg` per attack that Marpurg's stated rules admit:
/// constant, or constant and then constant with **one** change of `Leg`.
///
/// Returned in order of the mutation point, earliest first, with the unmutated
/// transpositions before them. A subject of `n` attacks yields at most `2n`
/// before Rules I and II are applied, and in practice a handful after.
pub fn admissible_legs(degrees: &[usize]) -> Vec<Vec<Leg>> {
  admissible_legs_opt(degrees, true)
}

/// The same, with Rule II optionally dropped.
///
/// Marpurg states Rule I flatly and hedges Rule II — it *"öfters nach
/// Beschaffenheit der Umstände ihre Ausnahmen leiden kann"* — so applying it as
/// a filter is stricter than the text. Both readings are run and reported, and
/// the difference between them is the hedge's worth.
pub fn admissible_legs_opt(degrees: &[usize], pin_last: bool) -> Vec<Vec<Leg>> {
  let n = degrees.len();
  if n == 0 {
    return vec![];
  }
  let want_first = first_leg(degrees[0]);
  let want_last = if pin_last { last_leg(degrees[n - 1]) } else { None };
  let ok = |legs: &[Leg]| {
    want_first.map_or(true, |w| legs[0] == w) && want_last.map_or(true, |w| legs[n - 1] == w)
  };
  let mut out = vec![];
  // no mutation at all: the answer is a plain transposition
  for a in [Leg::Fifth, Leg::Fourth] {
    let legs = vec![a; n];
    if ok(&legs) {
      out.push(legs);
    }
  }
  // one mutation, between attack `k-1` and attack `k`
  for k in 1..n {
    for a in [Leg::Fifth, Leg::Fourth] {
      let mut legs = vec![a; n];
      legs[k..].fill(a.other());
      if ok(&legs) {
        out.push(legs);
      }
    }
  }
  out
}

/// Apply one `Leg` assignment to a subject. Onsets, durations and ties are
/// untouched: Grundsatz 1 says the answer keeps the subject's figure and note
/// values, and [§2.6](../readme.md) says rhythm is not a variable here anyway.
pub fn render(subject: &Voice, legs: &[Leg], key: &[i8; 7]) -> Voice {
  if legs.is_empty() {
    return subject.clone();
  }
  let mut k = 0usize;
  Voice {
    notes: subject
      .notes
      .iter()
      .map(|n| {
        // a tied continuation keeps whichever leg its own attack was given
        let j = if n.attack {
          let j = k.min(legs.len() - 1);
          k += 1;
          j
        } else {
          k.saturating_sub(1).min(legs.len() - 1)
        };
        Note { pitch: step_in_key(n.pitch, legs[j].steps(), key), ..*n }
      })
      .collect(),
  }
}

/// Every answer Marpurg's rules admit for one subject.
pub fn admissible(subject: &Voice, key: &[i8; 7], tonic: usize) -> Vec<Voice> {
  admissible_opt(subject, key, tonic, true)
}

/// Every answer admitted, with Rule II optionally dropped.
pub fn admissible_opt(subject: &Voice, key: &[i8; 7], tonic: usize, pin_last: bool) -> Vec<Voice> {
  let degs = degrees(subject, tonic);
  admissible_legs_opt(&degs, pin_last).iter().map(|l| render(subject, l, key)).collect()
}

/// What a `Leg` does to one degree — the answer's degree for a subject's.
pub fn answered(deg: usize, leg: Leg) -> usize {
  (deg + leg.steps() as usize) % 7
}

/// A plain transposition of the whole subject, which is the *real* answer and
/// the control every figure here is read against. `Fifth` is the answer at the
/// fifth above, `Fourth` the same pitches an octave down.
pub fn real(subject: &Voice, leg: Leg, key: &[i8; 7]) -> Voice {
  let n = subject.notes.iter().filter(|x| x.attack).count().max(1);
  render(subject, &vec![leg; n], key)
}

/// A voice as scale degrees, one per attack — the form two entries are compared
/// in, since an answer sits in another voice at another octave and only its
/// degrees are the claim.
pub fn degrees(v: &Voice, tonic: usize) -> Vec<usize> {
  v.notes.iter().filter(|n| n.attack).map(|n| degree(n.pitch, tonic)).collect()
}

/// The same, with the chromatic inflection kept: `(degree, alteration relative
/// to the key)`. Strict, and the one that says whether the *spelling* agrees and
/// not merely the degree.
pub fn degrees_exact(v: &Voice, tonic: usize, key: &[i8; 7]) -> Vec<(usize, i8)> {
  v.notes
    .iter()
    .filter(|n| n.attack)
    .map(|n| {
      let l = n.pitch.step.rem_euclid(7) as usize;
      (degree(n.pitch, tonic), n.pitch.alter - key[l])
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::kern::TICKS_PER_QUARTER as Q;

  const C: [i8; 7] = [0; 7]; // C major, no signature

  /// A voice of quarter notes from diatonic step numbers.
  fn line(steps: &[i16]) -> Voice {
    Voice {
      notes: steps
        .iter()
        .enumerate()
        .map(|(i, &s)| Note { onset: i as i64 * Q, dur: Q, pitch: Pitch::new(s, 0), attack: true })
        .collect(),
    }
  }

  fn degs(v: &Voice) -> Vec<usize> {
    degrees(v, 0)
  }

  /// Rule I, quoted: the tonic and the dominant answer each other on the first
  /// note. Every admissible answer must obey it, not merely one of them.
  #[test]
  fn the_first_note_answers_tonic_with_dominant_and_back() {
    // C D E F G — begins on the tonic
    for a in admissible(&line(&[28, 29, 30, 31, 32]), &C, 0) {
      assert_eq!(degs(&a)[0], 4, "a subject opening on the tonic must be answered on the fifth");
    }
    // G A B C — begins on the dominant
    for a in admissible(&line(&[32, 33, 34, 35]), &C, 0) {
      assert_eq!(degs(&a)[0], 0, "a subject opening on the fifth must be answered on the tonic");
    }
  }

  /// Rule II, quoted: tonic answers dominant, and the third of the tonic answers
  /// the third of the dominant.
  #[test]
  fn the_last_note_follows_marpurgs_second_rule() {
    let end = |steps: &[i16]| -> Vec<usize> {
      admissible(&line(steps), &C, 0).iter().map(|a| *degs(a).last().unwrap()).collect()
    };
    for d in end(&[28, 29, 30, 31, 28]) {
      assert_eq!(d, 4, "ending on the tonic must be answered on the fifth");
    }
    for d in end(&[28, 29, 30, 31, 32]) {
      assert_eq!(d, 0, "ending on the fifth must be answered on the tonic");
    }
    for d in end(&[28, 29, 31, 32, 30]) {
      assert_eq!(d, 6, "ending on the third of the tonic must be answered on the third of the fifth");
    }
    for d in end(&[28, 29, 31, 32, 34]) {
      assert_eq!(d, 2, "ending on the third of the fifth must be answered on the third of the tonic");
    }
  }

  /// The *Vertauschung* changes **one** melodic interval and by **one** degree —
  /// Marpurg's own table of substitutions, a unison for a second and so on. This
  /// is the structural claim the whole module rests on.
  #[test]
  fn a_mutation_changes_one_interval_by_one_degree() {
    // opens on the tonic and closes on the dominant, so Rules I and II want
    // different legs and every admissible answer must mutate exactly once
    let sub = line(&[28, 30, 29, 31, 33, 32]);
    let want: Vec<i16> = sub.notes.windows(2).map(|w| w[1].pitch.step - w[0].pitch.step).collect();
    for a in admissible(&sub, &C, 0) {
      let got: Vec<i16> = a.notes.windows(2).map(|w| w[1].pitch.step - w[0].pitch.step).collect();
      let diffs: Vec<i16> =
        want.iter().zip(&got).map(|(x, y)| y - x).filter(|d| *d != 0).collect();
      assert_eq!(diffs.len(), 1, "a mutation must change exactly one interval, not {}", diffs.len());
      assert_eq!(diffs[0].abs(), 1, "an interval changed by {} degrees, not one", diffs[0]);
    }
    assert_eq!(admissible(&sub, &C, 0).len(), 5, "five places for the one mutation");
  }

  /// Grundsatz 1 on everything that is not pitch: the answer keeps the subject's
  /// figure and note values exactly.
  #[test]
  fn the_answer_keeps_the_subjects_rhythm_exactly() {
    let sub = Voice {
      notes: vec![
        Note { onset: 0, dur: 2 * Q, pitch: Pitch::new(28, 0), attack: true },
        Note { onset: 2 * Q, dur: Q / 2, pitch: Pitch::new(30, 0), attack: true },
        Note { onset: 2 * Q + Q / 2, dur: Q, pitch: Pitch::new(32, 0), attack: true },
      ],
    };
    for a in admissible(&sub, &C, 0) {
      let f = |v: &Voice| -> Vec<(i64, i64, bool)> {
        v.notes.iter().map(|n| (n.onset, n.dur, n.attack)).collect()
      };
      assert_eq!(f(&a), f(&sub));
    }
  }

  /// The set is small. §8.6 counts `10¹²` fills of three bars; the point of a
  /// whitelist is that it does not.
  #[test]
  fn the_admissible_set_is_small() {
    for len in 2..16usize {
      let steps: Vec<i16> = (0..len as i16).map(|i| 28 + i % 7).collect();
      let n = admissible(&line(&steps), &C, 0).len();
      assert!(n <= 2 * len, "{len} attacks admitted {n} answers");
    }
  }

  /// A subject whose ends Marpurg does not pin admits both plain transpositions,
  /// and a subject he does pin at both ends to the same `Leg` admits exactly one
  /// answer — the real one.
  #[test]
  fn the_rules_bind_only_where_marpurg_states_them() {
    // opens on the second, closes on the second: Rule I free, Rule II says fifth
    let free = admissible(&line(&[29, 30, 31, 29]), &C, 0);
    assert!(free.len() > 1, "an unpinned opening should leave a choice");
    // opens and closes on the tonic: both ends want the fifth, so no mutation fits
    let pinned = admissible(&line(&[28, 30, 32, 28]), &C, 0);
    assert_eq!(pinned.len(), 1);
    assert_eq!(degs(&pinned[0]), vec![4, 6, 1, 4]);
  }

  /// The tonic's letter comes from the key signature, not from the pitch class:
  /// three flats and a tonic class of 3 is E flat, and calling it D sharp would
  /// put every degree in this module out by one.
  #[test]
  fn the_tonic_letter_is_read_from_the_key_signature() {
    let e_flat: [i8; 7] = [0, -1, -1, 0, 0, -1, 0]; // B, E, A flat
    assert_eq!(tonic_letter(3, &e_flat), Some(2)); // E
    assert_eq!(tonic_letter(0, &C), Some(0)); // C
    assert_eq!(tonic_letter(7, &C), Some(4)); // G
  }
}
