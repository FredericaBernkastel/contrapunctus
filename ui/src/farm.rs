//! Generating somewhere else — spec 7.3.
//!
//! On the web that is a worker; on the desktop it is nothing at all, and the two
//! are the same three calls so that `app.rs` has one path.
//!
//! # Why only the web
//!
//! `App::grind` fills a block at a time and spends six milliseconds of each frame
//! on it, which keeps the window answerable. On the desktop that is the whole
//! problem solved, because `cpal` runs the audio on a thread of its own and a
//! busy main thread cannot starve it.
//!
//! In a browser it is not, because `cpal`'s WebAudio backend schedules on the
//! same thread the page draws on. It queues about **85 ms**, and a block cannot
//! be stopped part-way: the longest measured here is **84 ms** natively and wasm
//! is slower than native. So every long block is a hole in the sound, and 6.4's
//! stream rebuild is a cure for the symptom.
//!
//! # It always falls back, and always says which
//!
//! A worker can fail to start for reasons no build check reaches — a browser
//! without them, a page served from `file://`, a policy that forbids them. So
//! [`Farm::start`] never fails: it returns something that generates either way,
//! and [`Farm::where_it_runs`] is a sentence for the panel. Silence about which
//! one is running would make the difference between working and not working
//! invisible, which is the failure mode this repository is most against.

use contrapunctus::compose::{Design, Layout, Outcome};
use contrapunctus::settings::Settings;

/// What comes back, and it is compiled into **both** binaries — the worker
/// includes this file by `#[path]`. One definition, so the two ends of a message
/// cannot drift; two definitions of a wire format is the oldest way there is to
/// build something that works until it is rebuilt.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Answer {
  pub voices: Vec<contrapunctus::kern::Voice>,
  pub relaxed: contrapunctus::compose::Relaxed,
  pub seconds: f64,
}

/// The answer, or the reason there is none. Named arms rather than `Result`'s,
/// so that the words on the wire are chosen rather than inherited.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Reply {
  Wrote(Answer),
  Refused(String),
}

/// Generate what a request asks for. **The worker's whole job**, and it is here
/// rather than in the worker so that the desktop compiles and tests it: every
/// web fault in this project so far has been in a path the desktop never took.
#[allow(dead_code)] // the worker's half; the page never calls it
pub fn answer(request: &str) -> Result<Answer, String> {
  let s = Settings::from_json(request)?;
  let t0 = web_time::Instant::now();
  let (_, voices, relaxed) = contrapunctus::compose::generate(&s.design, &s.layout, s.tier.rules(), s.seed)?;
  Ok(Answer { voices, relaxed, seconds: t0.elapsed().as_secs_f64() })
}

/// A request in, a reply out, as the strings that cross the boundary.
#[allow(dead_code)] // likewise — see the note on `answer`
pub fn reply(request: &str) -> String {
  let r = match answer(request) {
    Ok(a) => Reply::Wrote(a),
    Err(e) => Reply::Refused(e),
  };
  // A reply that will not serialise is still a reply. Silence is the one outcome
  // the page cannot act on, because it is indistinguishable from still working.
  serde_json::to_string(&r).unwrap_or_else(|e| format!("{{\"Refused\":\"the reply would not serialise: {e}\"}}"))
}

/// What a page asks a worker, as the string that goes over.
#[allow(dead_code)] // the page's half; the worker never calls it
pub fn request(d: &Design, l: &Layout, tier: contrapunctus::automaton::Tier, seed: u64) -> Result<String, String> {
  Settings {
    format: 1,
    engine: env!("CARGO_PKG_VERSION").into(),
    design: d.clone(),
    layout: l.clone(),
    tier,
    seed,
    fingerprint: 0,
  }
  .to_json()
}

/// Where a generate happens, and what it is doing.
pub enum Farm {
  /// A worker, and whether it has ever answered. Until it has, nothing is known
  /// about whether it will.
  #[cfg(target_arch = "wasm32")]
  Worker(web::Hand),
  /// This thread, a block per frame — `App::grind`. Everything the desktop does,
  /// and what a browser without workers falls back to.
  Here(String),
}

impl Farm {
  /// Try for somewhere else, and settle for here. Never fails.
  pub fn start() -> Farm {
    // Braced, so each arm is the tail expression of its own block: written as
    // two `return`s the wasm build lints them as needless, because clippy sees
    // only the arm its target compiled.
    #[cfg(target_arch = "wasm32")]
    {
      match web::Hand::new() {
        Ok(h) => Farm::Worker(h),
        Err(why) => Farm::Here(why),
      }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
      Farm::Here("the desktop needs none: `cpal` has an audio thread of its own".into())
    }
  }

  /// One sentence for the panel. Spec 6.3's rule about absence, applied to a
  /// thing whose absence is otherwise completely invisible.
  pub fn where_it_runs(&self) -> String {
    match self {
      #[cfg(target_arch = "wasm32")]
      Farm::Worker(h) => h.state(),
      Farm::Here(why) => format!("in the page, a block a frame — {why}"),
    }
  }

  /// Whether generating here would stall the sound. False on the desktop and
  /// wherever a worker took the work.
  pub fn stalls_the_sound(&self) -> bool {
    match self {
      #[cfg(target_arch = "wasm32")]
      Farm::Worker(_) => false,
      #[cfg(target_arch = "wasm32")]
      Farm::Here(_) => true,
      #[cfg(not(target_arch = "wasm32"))]
      Farm::Here(_) => false,
    }
  }

  /// Ask for a piece. `false` when there is nowhere else and the caller should
  /// generate in the frame itself.
  pub fn ask(&mut self, d: &Design, l: &Layout, tier: contrapunctus::automaton::Tier, seed: u64) -> bool {
    let _ = (d, l, tier, seed);
    match self {
      #[cfg(target_arch = "wasm32")]
      Farm::Worker(h) => {
        match request(d, l, tier, seed) {
          Ok(json) => h.send(&json),
          Err(_) => false,
        }
      }
      Farm::Here(_) => false,
    }
  }

  /// Anything that has come back since last asked.
  pub fn take(&mut self, d: &Design, l: &Layout) -> Option<Result<Outcome, String>> {
    let _ = (d, l);
    match self {
      #[cfg(target_arch = "wasm32")]
      Farm::Worker(h) => h.take(d, l),
      Farm::Here(_) => None,
    }
  }
}

#[cfg(target_arch = "wasm32")]
pub mod web {
  use super::*;
  use std::cell::RefCell;
  use std::rc::Rc;
  use wasm_bindgen::prelude::*;
  use wasm_bindgen::JsCast;

  /// A worker and the last thing it said.
  ///
  /// `Rc<RefCell<_>>` because the message handler is a closure the browser owns
  /// and calls whenever it likes, and there is exactly one thread for it to race
  /// against — which is none.
  pub struct Hand {
    worker: web_sys::Worker,
    heard: Rc<RefCell<Option<String>>>,
    /// Set once the worker has answered anything at all. Until then it may yet
    /// turn out not to work, and the panel says so rather than promising.
    answered: Rc<RefCell<bool>>,
    _on: Closure<dyn FnMut(web_sys::MessageEvent)>,
  }

  impl Hand {
    pub fn new() -> Result<Hand, String> {
      // `worker-boot.js`, not `worker.js`: trunk emits wasm-bindgen's
      // `no-modules` shim, which defines `wasm_bindgen` and never calls it.
      let worker = web_sys::Worker::new("./worker-boot.js")
        .map_err(|e| format!("this browser would not start a worker: {}", brief(&e)))?;
      let heard = Rc::new(RefCell::new(None));
      let answered = Rc::new(RefCell::new(false));
      let (h, a) = (heard.clone(), answered.clone());
      let on = Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |e: web_sys::MessageEvent| {
        *a.borrow_mut() = true;
        *h.borrow_mut() = e.data().as_string();
      });
      worker.set_onmessage(Some(on.as_ref().unchecked_ref()));
      Ok(Hand { worker, heard, answered, _on: on })
    }

    pub fn state(&self) -> String {
      if *self.answered.borrow() {
        "in a worker, so the sound runs through it".into()
      } else {
        "in a worker — it has not answered yet, and until it does this is a hope rather than a fact".into()
      }
    }

    pub fn send(&self, json: &str) -> bool {
      self.worker.post_message(&JsValue::from_str(json)).is_ok()
    }

    pub fn take(&self, d: &Design, l: &Layout) -> Option<Result<Outcome, String>> {
      let said = self.heard.borrow_mut().take()?;
      Some(read(&said, d, l))
    }
  }

  fn brief(e: &JsValue) -> String {
    e.as_string().unwrap_or_else(|| format!("{e:?}"))
  }
}

/// Turn what the worker said into a piece, or into a reason.
///
/// Out here rather than in the wasm half so that it is compiled — and tested —
/// on the desktop too. Every previous web fault in this project has been in a
/// path the desktop never ran.
#[allow(dead_code)] // the page's half; the worker never calls it
pub fn read(said: &str, d: &Design, l: &Layout) -> Result<Outcome, String> {
  match serde_json::from_str::<Reply>(said) {
    Ok(Reply::Wrote(a)) => Ok(contrapunctus::compose::judged(d, l, a.voices, a.relaxed, a.seconds)),
    Ok(Reply::Refused(why)) => Err(why),
    Err(e) => Err(format!("the worker said something this build cannot read: {e}")),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Built here rather than taken from `catalog`, because the worker binary
  /// includes this file and has no `crate::catalog` in it.
  fn design() -> (Design, Layout) {
    use contrapunctus::kern::{Note, Voice};
    use contrapunctus::pitch::Pitch;
    let d = Design {
      subject: Voice {
        notes: (0..3)
          .map(|i| Note { onset: 240 + i * 240, dur: 240, pitch: Pitch::new(28 + [0, 2, 4][i as usize], 0), attack: true })
          .collect(),
      },
      voices: 3,
      key: [0; 7],
      tonic: 0,
      measure: 960,
      beat: 240,
      compass: vec![(33, 42), (28, 37), (21, 30)],
    };
    (d, Layout { middles: vec![4], episode_bars: 2, ..Default::default() })
  }

  /// **What the worker says becomes a piece, or a reason, and never a panic.**
  ///
  /// The reading half runs on the desktop as well as in a browser, which is the
  /// point of it being here: every web fault in this project so far has been in
  /// a path the desktop never took, so the parts that *can* be brought back over
  /// the line are brought back.
  #[test]
  fn a_reply_becomes_a_piece_or_a_reason() {
    let (d, l) = design();
    // the whole round trip, both halves, as strings
    let asked = request(&d, &l, contrapunctus::automaton::Tier::Confirmed, 0x5EED).expect("a request");
    let said = reply(&asked);
    let out = read(&said, &d, &l).expect("the reply became a piece");
    assert_eq!(out.voices.len(), 3);
    assert!(out.bars > 0);
    // and the verdict is there, which is what `judged` is for: the worker sent
    // notes and this side did the judging
    assert!(out.verdict.exposition_covers_the_voices);

    for nonsense in ["", "null", "{}", "{\"Wrote\":{}}", "not json at all"] {
      match read(nonsense, &d, &l) {
        Ok(_) => panic!("{nonsense:?} became a piece"),
        Err(e) => assert!(!e.trim().is_empty(), "{nonsense:?} refused with nothing to say"),
      }
    }
    let refused = "{\"Refused\":\"the wall\"}";
    match read(refused, &d, &l) {
      Ok(_) => panic!("a refusal became a piece"),
      Err(e) => assert_eq!(e, "the wall", "the reason did not survive the wire"),
    }
  }

  /// A farm always exists and always says where it is running. On the desktop
  /// that is here, and saying so is not a placeholder — it is the answer.
  #[test]
  fn there_is_always_somewhere_to_generate_and_it_is_named() {
    let farm = Farm::start();
    assert!(!farm.where_it_runs().trim().is_empty());
    assert!(!farm.stalls_the_sound(), "the desktop has an audio thread; nothing here stalls it");
  }
}
