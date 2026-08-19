//! Work that finishes later, and the one place this crate is not portable.
//!
//! Spec 7.2 asks for **one code path** for files: `rfd`'s async API, which is a
//! native dialog on the desktop and a file input in a browser. That holds for
//! the dialog. It cannot hold for what *drives* the future, because a browser
//! tab has no threads to block and a desktop has no `spawn_local`. So the
//! difference is confined to [`spawn`] — nine lines, in one module, rather than
//! spread through every call site.
//!
//! The result comes back through a [`Slot`], which the frame checks. That is the
//! immediate-mode shape: nothing is a callback, nothing mutates the application
//! from another thread, and the frame that reads the slot is the frame that
//! reacts to it.

use std::sync::{Arc, Mutex};

/// Somewhere for a finished task to leave its answer.
///
/// Cheap to clone, and cloning is how the task gets its end of it.
pub struct Slot<T>(Arc<Mutex<Option<T>>>);

impl<T> Clone for Slot<T> {
  fn clone(&self) -> Self {
    Slot(self.0.clone())
  }
}

impl<T> Default for Slot<T> {
  fn default() -> Self {
    Slot(Arc::new(Mutex::new(None)))
  }
}

impl<T> Slot<T> {
  pub fn put(&self, v: T) {
    if let Ok(mut g) = self.0.lock() {
      *g = Some(v);
    }
  }

  /// Take what is there, if anything. A frame calls this and acts on `Some`.
  pub fn take(&self) -> Option<T> {
    self.0.lock().ok().and_then(|mut g| g.take())
  }
}

/// Run a future to completion, somewhere that is not this frame.
///
/// Nothing may block here: on the desktop that would freeze the window while a
/// dialog is open, and in a browser it would freeze the tab hard enough that the
/// dialog never appears.
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn(f: impl std::future::Future<Output = ()> + Send + 'static) {
  std::thread::spawn(move || pollster::block_on(f));
}

#[cfg(target_arch = "wasm32")]
pub fn spawn(f: impl std::future::Future<Output = ()> + 'static) {
  wasm_bindgen_futures::spawn_local(f);
}
