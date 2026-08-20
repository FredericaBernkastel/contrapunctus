//! Everything that determines a fugue, in one file — `docs/ui-spec.md` §8.
//!
//! The requirement is exact: **loading a settings file produces the same fugue,
//! note for note.** That is achievable here because nothing about generation is
//! hidden — there is no clock, no global state, and the only randomness is a
//! seed. But it has to be written down completely, and it has to be *checked*,
//! because one of the four things that determine the output cannot go in a file.
//!
//! ```text
//! Design   subject notes · voices · key · tonic · measure · beat · compass
//! Layout   middles · episode_bars · link · close_at_home
//! Search   tier · seed
//! Engine   the code that turns those into notes
//! ```
//!
//! The first three are saved. The fourth is **recorded and verified**: a
//! [`Settings::fingerprint`] over the generated notes, written on save and
//! recomputed on load. A match is silent; a mismatch says so, and says which
//! engine wrote the file.
//!
//! That is not defensive programming, it is this project's own history. Readme
//! §8.16's dissonance rate moved four times as the code beneath it changed, and
//! every one of those changes was correct. A format that assumed the engine
//! stable would promise something the repository's record denies.
//!
//! JSON, via `serde`, behind the `serde` feature — so a build that only wants
//! the model still pulls in no crate at all, which is what readme §10.5 claims
//! about the figures in §8.

use crate::{automaton::Tier, compose};

/// The format's own version. Bumped when a field's *meaning* changes, not when
/// one is added — unknown fields are ignored on read, so an older build opens a
/// newer file with the settings it understands.
pub const FORMAT: u32 = 1;

/// One fugue's worth of settings.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Settings {
  pub format: u32,
  /// The crate version that wrote the file, for the mismatch message.
  pub engine: String,
  pub design: compose::Design,
  pub layout: compose::Layout,
  pub tier: Tier,
  pub seed: u64,
  /// A hash over the notes this produced when it was saved. `0` means the file
  /// was written without generating, and no check is possible.
  pub fingerprint: u64,
}

impl Settings {
  /// Capture what produced an outcome.
  pub fn of(
    design: &compose::Design,
    layout: &compose::Layout,
    tier: Tier,
    seed: u64,
    out: &compose::Outcome,
  ) -> Settings {
    Settings {
      format: FORMAT,
      engine: env!("CARGO_PKG_VERSION").to_string(),
      design: design.clone(),
      layout: layout.clone(),
      tier,
      seed,
      fingerprint: fingerprint(&out.voices),
    }
  }

  /// Regenerate, and say whether the result is the one the file recorded.
  ///
  /// The whole point of the format. A caller shows the music either way and
  /// tells the truth about which it is.
  pub fn reproduce(&self) -> Result<(compose::Outcome, Fidelity), String> {
    let out = compose::fugue(&self.design, &self.layout, self.tier.rules(), self.seed)?;
    let got = fingerprint(&out.voices);
    let how = if self.fingerprint == 0 {
      Fidelity::Unchecked
    } else if got == self.fingerprint {
      Fidelity::Exact
    } else {
      Fidelity::Differs { wrote: self.engine.clone(), now: env!("CARGO_PKG_VERSION").to_string() }
    };
    Ok((out, how))
  }
}

/// What came back, against what the file said would.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fidelity {
  /// Note for note, the fugue the file recorded.
  Exact,
  /// The settings loaded and the music is not the same. Both engine versions
  /// are given, because that is almost always the reason.
  Differs { wrote: String, now: String },
  /// The file carried no fingerprint, so nothing could be checked.
  Unchecked,
}

impl Fidelity {
  pub fn message(&self) -> Option<String> {
    match self {
      Fidelity::Exact => None,
      Fidelity::Unchecked => Some("this file records no fingerprint, so the music could not be checked".into()),
      Fidelity::Differs { wrote, now } => Some(format!(
        "this file was written by engine {wrote} and this is {now}; the settings loaded, and the music is not the same"
      )),
    }
  }
}

/// A hash over every note of every voice.
///
/// Order-independent per voice it is not — the voices are ordered and so are
/// their notes, and both orders are part of what a fugue *is*. Every field that
/// reaches the ear is in it: which voice, when, how long, and which pitch,
/// spelled. Two pieces that differ by an enharmonic respelling hash
/// differently, which is correct here and would not be in a program that
/// thought in semitones — readme §2.1.
pub fn fingerprint(voices: &[crate::kern::Voice]) -> u64 {
  let mut h = 0xcbf2_9ce4_8422_2325u64;
  let mut eat = |x: u64| {
    h ^= x;
    h = h.wrapping_mul(0x0000_0100_0000_01b3);
  };
  for (v, voice) in voices.iter().enumerate() {
    eat(v as u64);
    for n in &voice.notes {
      eat(n.onset as u64);
      eat(n.dur as u64);
      eat(n.pitch.step as i64 as u64);
      eat(n.pitch.alter as i64 as u64);
      eat(n.attack as u64);
    }
  }
  h
}

#[cfg(feature = "serde")]
impl Settings {
  /// JSON, indented, which is the default format and the one a person can read
  /// in a diff.
  pub fn to_json(&self) -> Result<String, String> {
    serde_json::to_string_pretty(self).map_err(|e| e.to_string())
  }
  pub fn from_json(text: &str) -> Result<Settings, String> {
    let s: Settings = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if s.format > FORMAT {
      return Err(format!("this file is format {} and this build reads {FORMAT}", s.format));
    }
    Ok(s)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::kern::{Note, Voice, TICKS_PER_QUARTER as Q};
  use crate::pitch::Pitch;

  fn design() -> compose::Design {
    compose::Design {
      subject: Voice {
        notes: (0..3)
          .map(|i| Note {
            onset: Q + i * Q,
            dur: Q,
            pitch: Pitch::new(28 + [0, 2, 4][i as usize], 0),
            attack: true,
          })
          .collect(),
      },
      voices: 3,
      key: [0; 7],
      tonic: 0,
      measure: 4 * Q,
      beat: Q,
      compass: vec![(33, 42), (28, 37), (21, 30)],
    }
  }

  /// The guarantee the format exists for.
  #[test]
  fn the_same_settings_give_the_same_fugue() {
    let (d, l) = (design(), compose::Layout::default());
    let first = compose::fugue(&d, &l, Tier::Full.rules(), 0x5EED).expect("a fugue");
    let s = Settings::of(&d, &l, Tier::Full, 0x5EED, &first);
    let (again, how) = s.reproduce().expect("a fugue");
    assert_eq!(how, Fidelity::Exact, "{:?}", how.message());
    assert_eq!(fingerprint(&first.voices), fingerprint(&again.voices));
  }

  /// And it is a real check, not a tautology: a different seed is a different
  /// fugue and the fingerprint says so.
  #[test]
  fn a_changed_setting_changes_the_fingerprint() {
    let (d, l) = (design(), compose::Layout::default());
    let a = compose::fugue(&d, &l, Tier::Full.rules(), 1).expect("a fugue");
    let b = compose::fugue(&d, &l, Tier::Full.rules(), 2).expect("a fugue");
    assert_ne!(fingerprint(&a.voices), fingerprint(&b.voices));

    // a file that claims one and produces the other is caught
    let mut lying = Settings::of(&d, &l, Tier::Full, 1, &a);
    lying.seed = 2;
    let (_, how) = lying.reproduce().expect("a fugue");
    assert!(matches!(how, Fidelity::Differs { .. }), "{how:?}");
  }

  /// Spelling reaches the ear and so reaches the hash. A program that thought
  /// in semitones could not tell these apart — readme §2.1 is the argument.
  #[test]
  fn an_enharmonic_respelling_is_a_different_fugue() {
    let one = vec![Voice { notes: vec![Note { onset: 0, dur: Q, pitch: Pitch::new(30, -1), attack: true }] }];
    let other = vec![Voice { notes: vec![Note { onset: 0, dur: Q, pitch: Pitch::new(29, 1), attack: true }] }];
    assert_eq!(one[0].notes[0].pitch.chroma(), other[0].notes[0].pitch.chroma(), "same sound");
    assert_ne!(fingerprint(&one), fingerprint(&other), "different spelling, same hash");
  }

  #[cfg(feature = "serde")]
  #[test]
  fn settings_round_trip_through_json() {
    let (d, l) = (design(), compose::Layout::default());
    let out = compose::fugue(&d, &l, Tier::Full.rules(), 0x5EED).expect("a fugue");
    let s = Settings::of(&d, &l, Tier::Full, 0x5EED, &out);
    let text = s.to_json().expect("json");
    let back = Settings::from_json(&text).expect("parse");
    assert_eq!(back.fingerprint, s.fingerprint);
    assert_eq!(back.seed, s.seed);
    assert_eq!(back.tier, s.tier);
    assert_eq!(back.layout, s.layout);
    let (_, how) = back.reproduce().expect("a fugue");
    assert_eq!(how, Fidelity::Exact, "a round trip must still reproduce");
  }

  /// **A fugue with a block asked for again survives being saved.**
  ///
  /// This is why the per-block nudge lives in [`compose::Layout`] and not beside
  /// the seed. A reroll that was not written down would come back as a different
  /// block on load, and the promise this whole module exists for — the same file
  /// gives the same fugue — would be false for exactly the pieces somebody had
  /// worked on hardest.
  #[cfg(feature = "serde")]
  #[test]
  fn a_rerolled_block_survives_a_round_trip() {
    let d = design();
    let plain = compose::Layout::default();
    let blocks = compose::derive(&d, &plain);
    let id = compose::identities(&blocks)[3];

    let mut l = plain.clone();
    l.rerolls.push((id, 1));
    let out = compose::fugue(&d, &l, Tier::Full.rules(), 0x5EED).expect("a fugue");

    // it is a different fugue from the un-rerolled one, or there is nothing to save
    let flat = compose::fugue(&d, &plain, Tier::Full.rules(), 0x5EED).expect("a fugue");
    assert_ne!(fingerprint(&out.voices), fingerprint(&flat.voices), "the reroll changed nothing");

    let text = Settings::of(&d, &l, Tier::Full, 0x5EED, &out).to_json().expect("json");
    assert!(text.contains("rerolls"), "the layout did not carry the reroll into the file");
    let back = Settings::from_json(&text).expect("parse");
    assert_eq!(back.layout.rerolls, l.rerolls);
    let (again, how) = back.reproduce().expect("a fugue");
    assert_eq!(how, Fidelity::Exact, "{:?}", how.message());
    assert_eq!(fingerprint(&again.voices), fingerprint(&out.voices));
  }

  /// And a file written before rerolls existed still opens, because the field
  /// was added rather than given a new meaning — which is what `FORMAT` counts
  /// and why it did not move.
  #[cfg(feature = "serde")]
  #[test]
  fn a_file_without_rerolls_still_opens() {
    let (d, l) = (design(), compose::Layout::default());
    let out = compose::fugue(&d, &l, Tier::Full.rules(), 1).expect("a fugue");
    let text = Settings::of(&d, &l, Tier::Full, 1, &out).to_json().expect("json");
    // strip the field, as a file written by the build before it would not have
    let mut doc: serde_json::Value = serde_json::from_str(&text).expect("json");
    doc.get_mut("layout").and_then(|l| l.as_object_mut()).expect("a layout").remove("rerolls");
    let older = doc.to_string();
    assert!(!older.contains("rerolls"), "the field was not actually removed");
    let back = Settings::from_json(&older).expect("an older file must still open");
    assert!(back.layout.rerolls.is_empty());
    assert_eq!(back.reproduce().expect("a fugue").1, Fidelity::Exact);
  }

  /// A file from a newer format is refused rather than half-read.
  #[cfg(feature = "serde")]
  #[test]
  fn a_newer_format_is_refused() {
    let (d, l) = (design(), compose::Layout::default());
    let out = compose::fugue(&d, &l, Tier::Full.rules(), 1).expect("a fugue");
    let mut s = Settings::of(&d, &l, Tier::Full, 1, &out);
    s.format = FORMAT + 1;
    let text = s.to_json().expect("json");
    assert!(Settings::from_json(&text).is_err());
  }

  /// An unknown key is ignored, so an older build opens a newer file with the
  /// settings it understands rather than not at all.
  #[cfg(feature = "serde")]
  #[test]
  fn an_unknown_field_is_ignored() {
    let (d, l) = (design(), compose::Layout::default());
    let out = compose::fugue(&d, &l, Tier::Full.rules(), 1).expect("a fugue");
    let text = Settings::of(&d, &l, Tier::Full, 1, &out).to_json().expect("json");
    let widened = text.replacen('{', "{\n  \"tempo_from_a_later_version\": 92,", 1);
    let back = Settings::from_json(&widened).expect("an unknown key must not stop a load");
    assert_eq!(back.seed, 1);
  }
}
