//! Wall-clock time, which not every target this library builds for has.
//!
//! `std::time::Instant::now()` **compiles** for `wasm32-unknown-unknown` and
//! panics when called — *time not implemented on this platform*. That is the
//! worst shape a portability fault can take: the compiler is happy, the linker
//! is happy, `cargo check --target wasm32-unknown-unknown` is happy, and the
//! page comes up as a stack trace. A path or a thread would have failed to
//! build; a clock does not.
//!
//! So the clock lives here, in one file, behind one name, and
//! `tests/portable.rs` fails the build if any other library module reaches for
//! `std::time` directly. On the desktop this is `std`'s own `Instant` and costs
//! nothing; only a `wasm32` build pulls in `web-time`, which reads
//! `performance.now()`. That is why readme §10.5's claim survives intact —
//! **the measurement binary is never built for `wasm32`**, so on the target that
//! produces §8's figures there is still no crate in the way.

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use std::time::Instant;

#[cfg(target_arch = "wasm32")]
pub(crate) use web_time::Instant;
