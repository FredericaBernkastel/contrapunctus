//! What has to hold for a build that runs in a browser.
//!
//! This exists because the check that was supposed to cover it did not.
//! `cargo check --target wasm32-unknown-unknown` catches a path or a thread,
//! because those do not compile there. It does not catch a **clock**:
//! `std::time::Instant::now()` compiles for `wasm32-unknown-unknown`, links,
//! survives `wasm-opt`, and panics when it is called —
//!
//! ```text
//! panicked at library/std/src/sys/time/unsupported.rs:13:9:
//! time not implemented on this platform
//! ```
//!
//! — which is what the page did, on the first attempt to open it. The compile
//! check had passed, so the sentence in `docs/ui-spec.md` claiming it would
//! catch anything of the kind was stronger than the check underneath it.
//!
//! > **A compile check catches what does not compile, and time compiles.**
//!
//! So the rules `docs/ui-spec.md` section 7 states in prose are enforced here in
//! text instead. This is a lint over source and not a proof: it reads what the
//! files say, and something reached indirectly through a dependency would slip
//! past it. It covers the way this fault actually arrived, which is by somebody
//! writing `std::time` in a file that a browser compiles.

use std::collections::BTreeSet;

fn read(p: &str) -> String {
  std::fs::read_to_string(p).unwrap_or_default().replace("\r\n", "\n")
}

/// The library's own modules, taken from `lib.rs` rather than from a list here,
/// so a module added tomorrow is covered the day it is written.
fn library_modules() -> Vec<String> {
  let lib = read("src/lib.rs");
  let mut out = BTreeSet::new();
  for l in lib.lines() {
    let l = l.trim();
    let name = l.strip_prefix("pub mod ").or_else(|| l.strip_prefix("mod "));
    if let Some(n) = name.and_then(|n| n.strip_suffix(';')) {
      out.insert(format!("src/{n}.rs"));
    }
  }
  assert!(out.len() > 10, "only {} modules parsed out of src/lib.rs", out.len());
  out.into_iter().collect()
}

fn interface_modules() -> Vec<String> {
  let mut out: Vec<String> = std::fs::read_dir("ui/src")
    .expect("ui/src")
    .filter_map(|e| e.ok().map(|e| e.path()))
    .filter(|p| p.extension().is_some_and(|x| x == "rs"))
    .map(|p| p.to_string_lossy().replace('\\', "/"))
    .collect();
  out.sort();
  assert!(out.len() > 5, "only {} interface modules found", out.len());
  out
}

/// Every line naming `what`, with its line number.
fn mentions(text: &str, what: &str) -> Vec<(usize, String)> {
  text
    .lines()
    .enumerate()
    .filter(|(_, l)| l.contains(what) && !l.trim_start().starts_with("//"))
    .map(|(i, l)| (i + 1, l.trim().to_string()))
    .collect()
}

/// **The clock lives in one file.**
///
/// `src/clock.rs` picks `std`'s `Instant` or `web-time`'s by target, and every
/// other module goes through it. The rule is worth having as a rule rather than
/// as care: the failure it prevents is invisible until the page is open.
#[test]
fn only_the_clock_module_names_a_clock() {
  let mut bad = vec![];
  for f in library_modules() {
    if f == "src/clock.rs" {
      continue;
    }
    for (line, text) in mentions(&read(&f), "std::time") {
      bad.push(format!("{f}:{line}: {text}"));
    }
  }
  for f in interface_modules() {
    for (line, text) in mentions(&read(&f), "std::time") {
      bad.push(format!("{f}:{line}: {text}"));
    }
  }
  assert!(
    bad.is_empty(),
    "`std::time` compiles for wasm32 and panics when called. Use `crate::clock::Instant` in the \
     library, or `web_time::Instant` in the interface:\n  {}",
    bad.join("\n  ")
  );
}

/// The interface reaches no filesystem, no environment and no process.
///
/// Spec 7.1's whole point: the library keeps its `&Path` wrappers, because a
/// desktop caller wants them, and the interface uses `parse` and `encode`
/// instead. That is a statement about `ui/src` and it is checkable there.
#[test]
fn the_interface_asks_for_nothing_an_operating_system_supplies() {
  let mut bad = vec![];
  for f in interface_modules() {
    let text = read(&f);
    for what in ["std::fs", "std::env", "std::process", "std::path"] {
      for (line, l) in mentions(&text, what) {
        bad.push(format!("{f}:{line}: {l}"));
      }
    }
  }
  assert!(
    bad.is_empty(),
    "the interface must run where there is no such thing — spec 7.1 and 7.5:\n  {}",
    bad.join("\n  ")
  );
}

/// A thread is allowed exactly where it is guarded.
///
/// `ui/src/task.rs` spawns one to drive a future on the desktop and calls
/// `spawn_local` in a browser. That difference is the one this crate could not
/// write once, and it is confined to that file by a `cfg` — which is only true
/// while something checks that it is.
#[test]
fn a_thread_is_spawned_only_behind_a_target_check() {
  let mut bad = vec![];
  for f in interface_modules() {
    let text = read(&f);
    let lines: Vec<&str> = text.lines().collect();
    for (line, l) in mentions(&text, "std::thread") {
      let guarded = lines[line.saturating_sub(4)..line]
        .iter()
        .any(|w| w.contains("cfg(not(target_arch = \"wasm32\"))"));
      if !guarded {
        bad.push(format!("{f}:{line}: {l}"));
      }
    }
  }
  assert!(
    bad.is_empty(),
    "a browser has no threads; guard it with `#[cfg(not(target_arch = \"wasm32\"))]`:\n  {}",
    bad.join("\n  ")
  );
}
