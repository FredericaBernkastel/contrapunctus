//! The generating worker — spec 7.3, and it exists only in a browser.
//!
//! A browser has no threads, and generating a fugue takes about half a second of
//! solid arithmetic. `App::grind` already breaks that into blocks and spends six
//! milliseconds of each frame on them, which keeps the *window* answerable — and
//! does nothing for the sound, because `cpal`'s WebAudio backend schedules on the
//! same thread. It queues about **85 ms**; the longest single block measured here
//! is **84 ms** natively and wasm is slower. A block cannot be stopped part-way,
//! so every long one is a hole in the audio.
//!
//! A worker is the only thing that fixes that rather than papering over it, and
//! 6.4's stream rebuild is the paper.
//!
//! # What crosses the boundary
//!
//! `settings::Settings` in, JSON, because it is already **exactly** the thing
//! that determines a fugue — section 8 exists to say so, and a second message
//! type for the same content would be a second place to forget a field.
//!
//! The voices and the relaxation log out. Not an `Outcome`: the block list is a
//! pure function of the design, and the verdict and the tally are cheap to
//! recompute, so posting them would be serialising three things to save
//! recomputing two of them. `compose::judged` puts the `Outcome` back together on
//! the far side through the same `judge` [`compose::fugue`] uses, so a piece
//! judged in the worker and one judged in the page cannot disagree.
//!
//! # Native
//!
//! Nothing. `main` is empty and this binary is never run: it is here so that
//! `trunk` can build a second wasm from it, and so that `cargo build` on the
//! desktop still type-checks the message handling.

fn main() {}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// What the page asks for and what comes back, as one type each.
///
/// `serde` derives on both, so the JSON is the struct and there is no hand-rolled
/// format to drift. Compiled on every target, so a change to either is a
/// compile error on the desktop and not a runtime surprise in a browser.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Answer {
  pub voices: Vec<contrapunctus::kern::Voice>,
  pub relaxed: contrapunctus::compose::Relaxed,
  pub seconds: f64,
}

/// Generate, and say what went wrong if it did not.
///
/// Shared by the worker below and by the page's own fallback, so that a browser
/// with no workers runs the same code a beat later rather than a different code
/// path — which is the arrangement `fill_block` exists to enforce one level down.
pub fn answer(request: &str) -> Result<Answer, String> {
  let s = contrapunctus::settings::Settings::from_json(request)?;
  let t0 = web_time::Instant::now();
  let (_, voices, relaxed) = contrapunctus::compose::generate(&s.design, &s.layout, s.tier.rules(), s.seed)?;
  Ok(Answer { voices, relaxed, seconds: t0.elapsed().as_secs_f64() })
}

/// The reply, as the page reads it: the answer, or the reason there is none.
///
/// A `Result` would serialise as `Ok`/`Err` and that is fine, but naming the
/// arms here means the page's parser and this one are looking at the same words.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Reply {
  Wrote(Answer),
  Refused(String),
}

pub fn reply(request: &str) -> String {
  let r = match answer(request) {
    Ok(a) => Reply::Wrote(a),
    Err(e) => Reply::Refused(e),
  };
  // A reply that will not serialise is still a reply, and silence would be the
  // one outcome the page cannot act on.
  serde_json::to_string(&r).unwrap_or_else(|e| format!("{{\"Refused\":\"the reply would not serialise: {e}\"}}"))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
  use wasm_bindgen::JsCast;
  let scope: web_sys::DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();
  let to = scope.clone();
  let on = Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |e: web_sys::MessageEvent| {
    let request = e.data().as_string().unwrap_or_default();
    let _ = to.post_message(&JsValue::from_str(&reply(&request)));
  });
  scope.set_onmessage(Some(on.as_ref().unchecked_ref()));
  // The closure outlives this function or the handler is freed the moment the
  // worker finishes starting, which is a worker that receives nothing forever.
  on.forget();
}

#[cfg(test)]
mod tests {
  /// **A request round-trips into notes, and a bad one into a reason.**
  ///
  /// The message handling, which is the half of a worker that can be tested
  /// without a browser — and the half that has been wrong every previous time
  /// something here first ran on the web.
  #[test]
  fn a_request_becomes_notes_or_a_reason() {
    let cat = contrapunctus::embedded::FUGUES;
    assert!(!cat.is_empty());
    let d = contrapunctus::compose::Design {
      subject: contrapunctus::kern::Voice {
        notes: (0..3)
          .map(|i| contrapunctus::kern::Note {
            onset: i * 240,
            dur: 240,
            pitch: contrapunctus::pitch::Pitch::new(28 + i as i16, 0),
            attack: true,
          })
          .collect(),
      },
      voices: 3,
      key: [0; 7],
      tonic: 0,
      measure: 960,
      beat: 240,
      compass: vec![(33, 45), (28, 40), (21, 33)],
    };
    let l = contrapunctus::compose::Layout { middles: vec![4], episode_bars: 2, ..Default::default() };
    let s = contrapunctus::settings::Settings {
      format: 1,
      engine: env!("CARGO_PKG_VERSION").into(),
      design: d.clone(),
      layout: l.clone(),
      tier: contrapunctus::automaton::Tier::Confirmed,
      seed: 0x5EED,
      fingerprint: 0,
    };
    let json = s.to_json().expect("settings serialise");

    match serde_json::from_str::<super::Reply>(&super::reply(&json)).expect("the reply parses") {
      super::Reply::Refused(e) => panic!("the worker refused a piece the library composes: {e}"),
      super::Reply::Wrote(a) => {
        assert_eq!(a.voices.len(), 3);
        // and it is judgeable on the far side, which is the whole point of
        // sending the voices rather than an outcome
        let o = contrapunctus::compose::judged(&d, &l, a.voices, a.relaxed, a.seconds);
        assert!(o.bars > 0, "the reassembled outcome has no bars in it");
        assert!(o.verdict.exposition_covers_the_voices, "the piece did not survive the round trip");
      }
    }

    // and nonsense comes back as a reason rather than as nothing
    match serde_json::from_str::<super::Reply>(&super::reply("not json")).expect("the reply parses") {
      super::Reply::Wrote(_) => panic!("the worker generated something from `not json`"),
      super::Reply::Refused(e) => assert!(!e.trim().is_empty(), "refused with nothing to say"),
    }
  }
}
