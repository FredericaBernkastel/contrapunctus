//! The corpus, compiled in — for a build with no filesystem.
//!
//! Any build in a browser is such a build, and readme §10.2's whole command
//! table is unreachable without one. About 295 kB of `**kern` and annotations,
//! which is small enough not to argue about and large enough to be worth a
//! feature flag rather than an unconditional include.
//!
//! It needs the submodules present at compile time — `git submodule update
//! --init` — which is why it is off by default: a fresh clone without them
//! should build the library, not fail on a missing file.
//!
//! §10.4 records what this corpus turns out to be, and nothing here changes it:
//! these are the same 24 files `kern::read` would open, byte for byte.

use crate::{kern::Piece, refdata::SubjectSpec};

/// The 24 fugues of Book I, each as `(id, text)` — ids matching the ground
/// truth's, so the two can be joined without a filename in between.
pub const FUGUES: [(&str, &str); 24] = [
  ("wtc-i-01", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f01.krn")),
  ("wtc-i-02", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f02.krn")),
  ("wtc-i-03", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f03.krn")),
  ("wtc-i-04", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f04.krn")),
  ("wtc-i-05", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f05.krn")),
  ("wtc-i-06", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f06.krn")),
  ("wtc-i-07", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f07.krn")),
  ("wtc-i-08", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f08.krn")),
  ("wtc-i-09", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f09.krn")),
  ("wtc-i-10", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f10.krn")),
  ("wtc-i-11", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f11.krn")),
  ("wtc-i-12", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f12.krn")),
  ("wtc-i-13", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f13.krn")),
  ("wtc-i-14", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f14.krn")),
  ("wtc-i-15", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f15.krn")),
  ("wtc-i-16", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f16.krn")),
  ("wtc-i-17", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f17.krn")),
  ("wtc-i-18", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f18.krn")),
  ("wtc-i-19", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f19.krn")),
  ("wtc-i-20", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f20.krn")),
  ("wtc-i-21", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f21.krn")),
  ("wtc-i-22", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f22.krn")),
  ("wtc-i-23", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f23.krn")),
  ("wtc-i-24", include_str!("../corpus/bach-wtc-fugues/kern/wtc1f24.krn")),
];

/// The annotations readme §8.6 onwards is measured against.
pub const ANNOTATIONS: &str = include_str!("../corpus/algomus-data/fugues/fugues.ref");

/// Every fugue, parsed. Ids are preserved, so a caller can pair these with
/// [`specs`] without touching a path.
pub fn pieces() -> Vec<Piece> {
  FUGUES.iter().filter_map(|(id, text)| crate::kern::parse(text, id).ok()).collect()
}

/// The annotated subjects, resolved against the pieces' own bar lengths — the
/// same join `spans()` does from disk, which is why `measures` is a lookup
/// rather than a constant.
pub fn specs(pieces: &[Piece]) -> Vec<SubjectSpec> {
  crate::refdata::parse(ANNOTATIONS, &|id| {
    pieces.iter().find(|p| p.id == id).map(|p| p.measure)
  })
  .unwrap_or_default()
}

#[cfg(test)]
mod tests {
  use super::*;

  /// The embedded corpus must be the corpus, not a subset of it that happens to
  /// parse — a browser build reading 23 fugues would report different figures
  /// and say nothing.
  #[test]
  fn all_twenty_four_are_here_and_parse() {
    let ps = pieces();
    assert_eq!(ps.len(), 24, "only {} of 24 fugues parsed", ps.len());
    assert!(ps.iter().all(|p| p.voices.len() >= 2));
    let sp = specs(&ps);
    assert!(sp.len() >= 24, "only {} annotated subjects", sp.len());
  }

  /// And identical to what the filesystem gives, which is the only claim that
  /// makes a browser build's figures comparable with §8's.
  #[test]
  fn the_embedded_text_matches_the_files() {
    let dir = std::path::Path::new("corpus/bach-wtc-fugues/kern");
    if !dir.exists() {
      return; // submodules absent; the include would not have compiled either
    }
    for (n, (id, text)) in FUGUES.iter().enumerate() {
      let path = dir.join(format!("wtc1f{:02}.krn", n + 1));
      let disk = std::fs::read_to_string(&path).expect("a fugue on disk");
      // byte for byte, with no normalising: `include_str!` and `read_to_string`
      // both take the file as it is, so anything that differs here is a real
      // difference and not a line ending
      assert_eq!(disk, *text, "{id} differs from {path:?}");
    }
  }
}
