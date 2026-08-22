//! Saving and loading — spec 8, and spec 6.4's export.
//!
//! Three operations, all through the same async dialog so that there is one code
//! path and not one per platform. Nothing here touches a `Path`: a file is a
//! name and some bytes, which is what spec 7.4 means by keeping paths out of
//! application state.
//!
//! The interesting one is loading. **The requirement is that a settings file
//! produces the same fugue, note for note**, and that is checkable rather than
//! merely intended, because `settings::Settings` carries a fingerprint over the
//! notes and recomputes it on load. A match is silent; a mismatch says which
//! engine wrote the file. §8.16's own rate moved four times as the code beneath
//! it changed and every one of those changes was correct, so a format that
//! assumed the engine stable would be promising what this project's record
//! denies.

use contrapunctus::{
  compose::{self, Design, Layout, Outcome},
  settings::Settings,
};

use crate::task::{spawn, Slot};

/// What came back from a load, once the dialog and the search are both done.
/// A settings file, read and parsed and **not yet generated**.
///
/// Reading is instant and generating is half a second, and putting the second
/// inside the first froze the window until it was over — reported, and the plain
/// difference between Open and Compose, which had a progress line and a worker.
/// So this carries what was in the file and the interface generates it the way
/// it generates anything else.
pub struct Loaded {
  pub settings: Settings,
}

/// A message for the status line — the interface says what happened either way,
/// because a save that silently did nothing is indistinguishable from one that
/// worked.
pub enum Note {
  Saved(String),
  Cancelled,
  Failed(String),
}

/// Write the settings that produced `out` as JSON.
pub fn save_settings(
  design: &Design,
  layout: &Layout,
  tier: contrapunctus::automaton::Tier,
  seed: u64,
  out: &Outcome,
  into: Slot<Note>,
) {
  let s = Settings::of(design, layout, tier, seed, out);
  let text = match s.to_json() {
    Ok(t) => t,
    Err(e) => return into.put(Note::Failed(e)),
  };
  spawn(async move {
    let Some(handle) = rfd::AsyncFileDialog::new()
      .set_file_name("fugue.json")
      .add_filter("settings", &["json"])
      .save_file()
      .await
    else {
      return into.put(Note::Cancelled);
    };
    let name = handle.file_name();
    match handle.write(text.as_bytes()).await {
      Ok(()) => into.put(Note::Saved(name)),
      Err(e) => into.put(Note::Failed(e.to_string())),
    }
  });
}

/// Read a settings file and regenerate from it.
///
/// The regeneration happens here rather than on the frame because it is the
/// slow part and this is already off the frame. What comes back is the music
/// *and* the verdict on whether it is the music the file recorded.
pub fn load_settings(into: Slot<Result<Loaded, Note>>) {
  spawn(async move {
    let Some(handle) = rfd::AsyncFileDialog::new().add_filter("settings", &["json"]).pick_file().await
    else {
      return into.put(Err(Note::Cancelled));
    };
    let bytes = handle.read().await;
    let text = match String::from_utf8(bytes) {
      Ok(t) => t,
      Err(_) => return into.put(Err(Note::Failed("that file is not text".into()))),
    };
    let settings = match Settings::from_json(&text) {
      Ok(s) => s,
      Err(e) => return into.put(Err(Note::Failed(e))),
    };
    // Parsed and handed over. **Not generated here**: this is an async task with
    // no thread under it on the web, so a search inside it is a search on the
    // one thread the page has.
    into.put(Ok(Loaded { settings }));
  });
}

/// What an imported file turned out to hold.
pub struct Imported {
  pub name: String,
  pub piece: contrapunctus::kern::Piece,
  /// Which voice of it was taken, and how many there were.
  pub took: usize,
  pub of: usize,
}

/// Import a subject from a `**kern` file — spec 3.2.
///
/// **`**kern` and not MIDI, and that is not a preference.** Humdrum spells its
/// pitches; MIDI does not, and §8.6 measured 13 notes in 20 coming back
/// respelled through a round trip. A subject imported as semitones would arrive
/// with its augmented fourths turned into diminished fifths, and every rule in
/// this program that distinguishes them would then be answering a question about
/// a different piece of music.
///
/// **The file is the subject.** There is no annotation in a bare `**kern` file
/// saying where a subject ends, so nothing here guesses: what the file holds is
/// what is taken, and if it holds a whole fugue the caller is told the length so
/// they can see that is what happened.
pub fn import_subject(into: Slot<Result<Imported, Note>>) {
  spawn(async move {
    let Some(handle) = rfd::AsyncFileDialog::new()
      .add_filter("Humdrum **kern", &["krn", "kern"])
      .pick_file()
      .await
    else {
      return into.put(Err(Note::Cancelled));
    };
    let name = handle.file_name();
    let bytes = handle.read().await;
    let text = match String::from_utf8(bytes) {
      Ok(t) => t,
      Err(_) => return into.put(Err(Note::Failed("that file is not text".into()))),
    };
    match contrapunctus::kern::parse(&text, &name) {
      Err(e) => into.put(Err(Note::Failed(format!("{name} did not parse: {e}")))),
      Ok(piece) if piece.voices.iter().all(|v| v.notes.is_empty()) => {
        into.put(Err(Note::Failed(format!("{name} holds no notes"))))
      }
      Ok(piece) => {
        // The first voice that has anything in it. Which voice is not a guess
        // when there is only one, and the interface says so when there is not.
        let took = piece.voices.iter().position(|v| !v.notes.is_empty()).unwrap_or(0);
        let of = piece.voices.len();
        into.put(Ok(Imported { name, piece, took, of }))
      }
    }
  });
}

/// Export the music as a MIDI file — spec 6.4.
///
/// Through `compose::encode`, which orders the tracks by mean sounding pitch and
/// names each with how many entries it carries. MIDI is an output format here
/// and never an interchange one: §8.6 measured 13 notes in 20 coming back
/// respelled, so what leaves this way cannot be read back in without loss.
pub fn export_midi(out: &Outcome, design: &Design, qpm: u32, into: Slot<Note>) {
  let bytes = compose::encode(out, design, qpm);
  spawn(async move {
    let Some(handle) = rfd::AsyncFileDialog::new()
      .set_file_name("fugue.mid")
      .add_filter("MIDI", &["mid"])
      .save_file()
      .await
    else {
      return into.put(Note::Cancelled);
    };
    let name = handle.file_name();
    match handle.write(&bytes).await {
      Ok(()) => into.put(Note::Saved(name)),
      Err(e) => into.put(Note::Failed(e.to_string())),
    }
  });
}
