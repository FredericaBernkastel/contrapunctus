// Starts the generating worker — spec 7.3.
//
// `trunk`'s `data-type="worker"` emits wasm-bindgen's `no-modules` shim, which
// *defines* `wasm_bindgen` and does not call it: a Worker pointed straight at
// that file would load a function and then sit there, and the `#[wasm_bindgen(start)]`
// entry would never run. This is the two lines that were missing.
//
// A classic worker rather than a module one, because `no-modules` is what trunk
// emitted and `importScripts` is how a classic worker loads it. Both paths are
// relative to *this* file, which is where trunk puts all three.
importScripts('./worker.js');
wasm_bindgen('./worker_bg.wasm').catch((e) => {
  // A worker that fails to start must say so, or the page waits for a reply
  // that is never coming. `app.rs` treats any message that is not a reply as a
  // reason to fall back to generating in the frame.
  self.postMessage(JSON.stringify({ Refused: 'the worker would not start: ' + e }));
});
