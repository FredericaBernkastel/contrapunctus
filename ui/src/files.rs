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
  settings::{Fidelity, Settings},
};

use crate::task::{spawn, Slot};

/// What came back from a load, once the dialog and the search are both done.
pub struct Loaded {
  pub settings: Settings,
  pub outcome: Outcome,
  pub how: Fidelity,
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
    match settings.reproduce() {
      Ok((outcome, how)) => into.put(Ok(Loaded { settings, outcome, how })),
      Err(e) => into.put(Err(Note::Failed(format!("the settings loaded and the search refused: {e}")))),
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
