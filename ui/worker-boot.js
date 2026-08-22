// Starts the generating worker — spec 7.3.
//
// `trunk`'s `data-type="worker"` emits wasm-bindgen's `no-modules` shim, which
// *defines* `wasm_bindgen` and does not call it: a Worker pointed straight at
// that file would load a function and then sit there, and the
// `#[wasm_bindgen(start)]` entry would never run. This is the part that was
// missing, and it is two problems rather than one.
//
// **The second one loses the first message.** This script finishes evaluating in
// a millisecond; `wasm_bindgen(...)` then fetches and instantiates 1.1 MB of
// wasm, which does not. A worker's messages are queued only until its script has
// been evaluated — after that they dispatch immediately, and one that dispatches
// with no handler registered is simply dropped. The page asks for its first
// fugue on its first frame, which is inside that window every time. It was
// reported as "Writing — in a worker" stuck for ever, cured by pressing Compose:
// the second request arrives after the handler exists.
//
// So a handler goes on synchronously, before anything is awaited, and holds
// whatever arrives until the real one is there to take it.
const waiting = [];
self.onmessage = (e) => waiting.push(e.data);

importScripts('./worker.js');

wasm_bindgen('./worker_bg.wasm')
  .then(() => {
    // `start()` ran during instantiation and has replaced `onmessage` with the
    // Rust one, so re-dispatching hands these to it.
    for (const data of waiting) {
      self.dispatchEvent(new MessageEvent('message', { data }));
    }
    waiting.length = 0;
  })
  .catch((e) => {
    // A worker that fails to start must say so, or the page waits for a reply
    // that is never coming. `farm.rs` reads any reply it cannot parse as a
    // reason, and gives up on the worker after a while in any case.
    self.postMessage(JSON.stringify({ Refused: 'the worker would not start: ' + e }));
  });
