//! Every `§` reference in the repository names the section it says it does.
//!
//! This exists because section references kept going stale and nothing noticed.
//! Anchors were checked — a link to `#86-realisation-and-the-first-notes` was
//! verified to resolve — but the **visible text** was not: `[§8.5](#86-…)` would
//! have passed, and after §11 was moved inside §7 a good deal of prose named
//! sections that had been renumbered under it. Doc comments in `src/` had no
//! protection at all, and there are 160 references in them.
//!
//! Three classes of check, all mechanical:
//!
//! 1. **Structure.** Numbered headings form a contiguous sequence, `Contents`
//!    lists each one exactly once, and each contents entry's anchor matches the
//!    heading it names.
//! 2. **Links.** Every `](#…)` resolves; every relative file path exists; and
//!    where a link reads `[§N](#anchor)` the heading at that anchor **is**
//!    section N. That last one is the check whose absence let the numbering
//!    drift silently.
//! 3. **Bare references.** Every `§N` in `readme.md`, `CHANGELOG.md` and
//!    `src/**.rs` names a section that exists.
//!
//! A reference to `ricercar`'s sections is resolved against *that* document
//! instead. Such a reference is identified without guesswork: either it is a
//! link whose target names the file, or the word immediately before it is
//! `ricercar`. Both forms are used, and nothing else is accepted — a bare
//! `§7.2` meaning ricercar's §7.2 will be reported against this document and
//! fail, which is the intended behaviour rather than a gap.

use std::collections::BTreeMap;

/// Every prose file whose `§` references are checked.
///
/// `docs/` is swept rather than listed, so a document added there is covered
/// the day it is written instead of the day somebody remembers this file. That
/// matters more than it sounds: `docs/ui-spec.md` cites §8.16, §2.7, §10.6 and
/// a dozen more, and a specification that names sections which have moved is
/// worse than one that names none.
///
/// `.html` is swept beside `.md` because `docs/ui-sketch.html` is prose too, and
/// was reviewed by eye for exactly as long as that worked: a settled decision
/// changed under it — settings became JSON through `serde` — and the sketch went
/// on describing a hand-written format until somebody read it. A picture of a
/// specification is a document that can go stale, so it is checked like one.
fn docs() -> Vec<String> {
  let mut out = vec!["readme.md".to_string(), "CHANGELOG.md".to_string()];
  if let Ok(rd) = std::fs::read_dir("docs") {
    let mut found: Vec<String> = rd
      .filter_map(|e| e.ok().map(|e| e.path()))
      .filter(|p| p.extension().is_some_and(|x| x == "md" || x == "html"))
      .map(|p| p.to_string_lossy().replace('\\', "/"))
      .collect();
    found.sort();
    out.extend(found);
  }
  out
}

fn read(p: &str) -> String {
  std::fs::read_to_string(p).unwrap_or_default().replace("\r\n", "\n")
}

/// GitHub's heading anchor: lowercase, drop punctuation, spaces to dashes.
fn slug(title: &str) -> String {
  title
    .trim()
    .to_lowercase()
    .chars()
    .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
    .map(|c| if c == ' ' { '-' } else { c })
    .collect()
}

struct Heading {
  num: Option<String>,
  title: String,
  slug: String,
  line: usize,
}

fn headings(text: &str) -> Vec<Heading> {
  let mut out = vec![];
  let mut fence = false;
  for (i, l) in text.lines().enumerate() {
    if l.starts_with("```") {
      fence = !fence;
      continue;
    }
    if fence {
      continue;
    }
    let hashes = l.chars().take_while(|&c| c == '#').count();
    if !(2..=4).contains(&hashes) || !l[hashes..].starts_with(' ') {
      continue;
    }
    let title = l[hashes + 1..].trim().to_string();
    let num = title.split_whitespace().next().and_then(|w| {
      let w = w.trim_end_matches('.');
      let ok = !w.is_empty() && w.chars().all(|c| c.is_ascii_digit() || c == '.');
      ok.then(|| w.to_string())
    });
    out.push(Heading { num, slug: slug(&title), title, line: i + 1 });
  }
  out
}

/// The numbered section each heading belongs to: its own number, or — for the
/// unnumbered `####` headings §8.6 and §7.1 use — the nearest numbered heading
/// above it. A link may therefore point at a subsection heading and still be
/// checked against the section number the prose names.
fn owners(hs: &[Heading]) -> Vec<Option<String>> {
  let mut out = Vec::with_capacity(hs.len());
  let mut last: Option<String> = None;
  for h in hs {
    if h.num.is_some() {
      last = h.num.clone();
    }
    out.push(if h.num.is_some() { h.num.clone() } else { last.clone() });
  }
  out
}

fn line_of(text: &str, byte: usize) -> usize {
  text[..byte].matches('\n').count() + 1
}

/// Every `[text](target)` in a document, with the byte range its text occupies.
fn links(text: &str) -> Vec<(String, String, usize, usize)> {
  let b = text.as_bytes();
  let mut out = vec![];
  let mut i = 0;
  while i < b.len() {
    if b[i] != b'[' {
      i += 1;
      continue;
    }
    let Some(close) = text[i..].find("](").map(|k| i + k) else { break };
    if text[i..close].contains('\n') {
      i += 1;
      continue;
    }
    // balanced, because DOIs contain parentheses: `10.1016/0004-3702(77)90007-8`
    let mut depth = 1usize;
    let mut j = close + 2;
    while j < b.len() {
      match b[j] {
        b'(' => depth += 1,
        b')' => {
          depth -= 1;
          if depth == 0 {
            break;
          }
        }
        _ => {}
      }
      j += 1;
    }
    if j >= b.len() {
      break;
    }
    out.push((text[i + 1..close].to_string(), text[close + 2..j].to_string(), i + 1, close));
    i = j + 1;
  }
  out
}

/// A `§N` occurrence: the number, whether it names `ricercar`'s document, and
/// the byte at which it starts.
struct Ref {
  num: String,
  foreign: bool,
  at: usize,
}

fn section_refs(text: &str) -> Vec<Ref> {
  let mut out = vec![];
  // by character, not by byte: stepping a byte at a time lands inside the
  // multi-byte dashes and section signs this document is full of
  let cs: Vec<(usize, char)> = text.char_indices().collect();
  let mut k = 0usize;
  while k < cs.len() {
    if cs[k].1 != '§' {
      k += 1;
      continue;
    }
    let start = cs[k].0;
    while k < cs.len() && cs[k].1 == '§' {
      k += 1;
    }
    let mut num = String::new();
    while k < cs.len() && (cs[k].1.is_ascii_digit() || cs[k].1 == '.') {
      num.push(cs[k].1);
      k += 1;
    }
    if num.is_empty() || !num.starts_with(|c: char| c.is_ascii_digit()) {
      continue;
    }
    // "ricercar §7.2" / "ricercar's §8" — the token immediately before decides,
    // rather than a window of context that could pick the word up by accident
    let before = text[..start].trim_end();
    let foreign = before
      .rsplit(|c: char| c.is_whitespace() || c == '(' || c == '[')
      .next()
      .map(|w| w.trim_end_matches("'s").to_lowercase() == "ricercar")
      .unwrap_or(false);
    let num = num.trim_end_matches('.').to_string();
    out.push(Ref { num, foreign, at: start });
    // a range, "§§1–7": check both ends
    if k < cs.len() && matches!(cs[k].1, '–' | '-' | '—') {
      let mut hi = String::new();
      let mut j = k + 1;
      while j < cs.len() && (cs[j].1.is_ascii_digit() || cs[j].1 == '.') {
        hi.push(cs[j].1);
        j += 1;
      }
      if !hi.is_empty() {
        out.push(Ref { num: hi.trim_end_matches('.').to_string(), foreign, at: start });
        k = j;
      }
    }
  }
  out
}

#[test]
fn the_readme_is_numbered_consistently() {
  let rd = read("readme.md");
  let hs = headings(&rd);
  let mut bad = vec![];

  let tops: Vec<&Heading> = hs.iter().filter(|h| h.num.as_deref().is_some_and(|n| !n.contains('.'))).collect();
  for (k, h) in tops.iter().enumerate() {
    let want = k.to_string();
    if h.num.as_deref() != Some(want.as_str()) {
      bad.push(format!("readme.md:{}: section numbered {:?}, expected {want}", h.line, h.num.as_deref().unwrap()));
    }
  }
  // subsections of each section run 1..n with no gaps
  let mut seen: BTreeMap<String, Vec<(String, usize)>> = BTreeMap::new();
  for h in hs.iter() {
    if let Some(n) = &h.num {
      if let Some((parent, sub)) = n.split_once('.') {
        seen.entry(parent.to_string()).or_default().push((sub.to_string(), h.line));
      }
    }
  }
  for (parent, subs) in &seen {
    for (k, (sub, line)) in subs.iter().enumerate() {
      let want = (k + 1).to_string();
      if *sub != want {
        bad.push(format!("readme.md:{line}: subsection {parent}.{sub}, expected {parent}.{want}"));
      }
    }
  }
  assert!(bad.is_empty(), "numbering is not contiguous:\n  {}", bad.join("\n  "));
}

#[test]
fn the_contents_lists_every_section_once() {
  let rd = read("readme.md");
  let hs = headings(&rd);
  let toc = match (rd.find("## Contents"), rd.find("\n---\n")) {
    (Some(a), _) => {
      let rest = &rd[a..];
      &rest[..rest.find("\n---\n").unwrap_or(rest.len())]
    }
    _ => panic!("readme.md has no Contents"),
  };
  let mut bad = vec![];
  for h in hs.iter().filter(|h| h.num.is_some()) {
    let entry = format!("[{}](#{})", h.title, h.slug);
    let n = toc.matches(&entry).count();
    if n != 1 {
      bad.push(format!("readme.md:{}: contents has {n} entries for {:?}, expected 1", h.line, h.title));
    }
  }
  assert!(bad.is_empty(), "contents out of step with the headings:\n  {}", bad.join("\n  "));
}

#[test]
fn every_link_resolves_and_names_the_right_section() {
  let rd = read("readme.md");
  let hs = headings(&rd);
  let own = owners(&hs);
  let by_slug: BTreeMap<&str, usize> = hs.iter().enumerate().map(|(i, h)| (h.slug.as_str(), i)).collect();

  let ric = read("ricercar/readme.md");
  let rh = headings(&ric);
  let rown = owners(&rh);
  let ric_by_slug: BTreeMap<&str, usize> = rh.iter().enumerate().map(|(i, h)| (h.slug.as_str(), i)).collect();

  let mut bad = vec![];
  let mut checked = 0usize;
  for (text, target, at, _) in links(&rd) {
    let line = line_of(&rd, at);
    if target.starts_with("http://") || target.starts_with("https://") || target.starts_with("mailto:") {
      continue; // an external URL is not this test's business
    }
    let (file, anchor) = match target.split_once('#') {
      Some((f, a)) => (f, Some(a)),
      None => (target.as_str(), None),
    };

    // the target must exist: a heading here, a heading there, or a file
    let foreign_target = file == "ricercar/readme.md";
    let table = if file.is_empty() {
      Some((&by_slug, &hs, &own))
    } else if foreign_target {
      if ric.is_empty() {
        None
      } else {
        Some((&ric_by_slug, &rh, &rown))
      }
    } else {
      if !std::path::Path::new(file).exists() {
        bad.push(format!("readme.md:{line}: link to {file:?}, which does not exist"));
      }
      None
    };
    let Some((table, list, own)) = table else { continue };
    let Some(anchor) = anchor else { continue };
    let Some(&i) = table.get(anchor) else {
      bad.push(format!("readme.md:{line}: anchor #{anchor} resolves to nothing"));
      continue;
    };

    // and where the text names a section, it must be *that* section
    for r in section_refs(&text) {
      // a link to our own document whose text names ricercar's section — the
      // heading "1. Diagnosis: ricercar's §8 …" is the case
      if r.foreign && !foreign_target {
        continue;
      }
      checked += 1;
      match &own[i] {
        Some(n) if *n == r.num => {}
        Some(n) => bad.push(format!(
          "readme.md:{line}: reads §{} but #{anchor} sits in section {n} ({:?})",
          r.num, list[i].title
        )),
        None => bad.push(format!("readme.md:{line}: reads §{} but #{anchor} is in no numbered section", r.num)),
      }
    }
  }
  assert!(checked > 100, "only {checked} link references compared; the scanner has stopped seeing them");
  assert!(bad.is_empty(), "links do not match what they claim:\n  {}", bad.join("\n  "));
}

#[test]
fn every_bare_reference_names_a_section_that_exists() {
  let rd = read("readme.md");
  let hs = headings(&rd);
  let nums: Vec<&str> = hs.iter().filter_map(|h| h.num.as_deref()).collect();

  let ric = read("ricercar/readme.md");
  let rnums: Vec<String> = headings(&ric).iter().filter_map(|h| h.num.clone()).collect();

  let mut files: Vec<String> = docs();
  let mut src: Vec<String> = std::fs::read_dir("src")
    .expect("src/")
    .filter_map(|e| e.ok().map(|e| e.path()))
    .filter(|p| p.extension().is_some_and(|x| x == "rs"))
    .map(|p| p.to_string_lossy().replace('\\', "/"))
    .collect();
  src.sort();
  files.append(&mut src);

  let mut bad = vec![];
  let mut checked = 0usize;
  for f in &files {
    let text = read(f);
    // byte ranges occupied by link *text*, which the link test already covers
    let spans: Vec<(usize, usize)> = links(&text).iter().map(|(_, _, a, b)| (*a, *b)).collect();
    for r in section_refs(&text) {
      if spans.iter().any(|&(a, b)| r.at >= a && r.at < b) {
        continue;
      }
      checked += 1;
      let known = if r.foreign { rnums.contains(&r.num) } else { nums.iter().any(|n| *n == r.num) };
      if !(known || r.foreign && ric.is_empty()) {
        let which = if r.foreign { "ricercar/readme.md" } else { "readme.md" };
        bad.push(format!("{f}:{}: §{} names no section of {which}", line_of(&text, r.at), r.num));
      }
    }
  }
  assert!(checked > 100, "only {checked} bare references scanned; the scanner has stopped seeing them");
  assert!(bad.is_empty(), "references to sections that do not exist:\n  {}", bad.join("\n  "));
}

/// §10.2's table names a command for every reported figure, and the program is
/// the authority on what commands there are. This runs `list` and compares.
///
/// Three ways they can disagree, and all three fail here: the table names a
/// command the binary does not have, a command's section in the table is not the
/// one the binary prints for it, or a command that produces a reported figure is
/// missing from the table. The old command line answered an unknown argument by
/// silently running something else, so a mistyped command in this table came
/// back as a measurement of something — which is why the check exists.
#[test]
fn the_command_table_matches_the_program() {
  let out = std::process::Command::new(env!("CARGO_BIN_EXE_contrapunctus"))
    .arg("list")
    .output()
    .expect("`list` runs");
  assert!(out.status.success(), "`list` failed: {}", String::from_utf8_lossy(&out.stderr));
  let listed = String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n");

  // `   §8.9     plan   harmonic plans, ...`, and the same without a section for
  // the superseded and batch blocks, which are indented past the section column.
  let mut prog: BTreeMap<String, String> = BTreeMap::new();
  for l in listed.lines() {
    if !l.starts_with("   ") || l.contains("--") {
      continue;
    }
    let t: Vec<&str> = l.split_whitespace().collect();
    match t.as_slice() {
      [s, name, ..] if s.starts_with('§') => {
        prog.insert((*name).to_string(), (*s).to_string());
      }
      [name, ..] if l.starts_with("            ") => {
        prog.insert((*name).to_string(), String::new());
      }
      _ => {}
    }
  }
  assert!(prog.len() > 30, "`list` parsed to only {} commands: {prog:?}", prog.len());

  let rd = read("readme.md");
  let table = {
    let a = rd.find("### 10.2").expect("§10.2 is there");
    let b = rd[a..].find("### 10.3").expect("§10.3 is there");
    &rd[a..a + b]
  };

  let mut bad = vec![];
  let mut named: Vec<String> = vec![];
  for l in table.lines().filter(|l| l.contains("cargo run --release -- ")) {
    let cmd = l
      .split("cargo run --release -- ")
      .nth(1)
      .and_then(|r| r.split('`').next())
      .unwrap_or("")
      .trim()
      .to_string();
    if cmd.is_empty() {
      continue;
    }
    named.push(cmd.clone());
    let Some(sec) = prog.get(&cmd) else {
      bad.push(format!("readme.md §10.2 names `{cmd}`, which the program does not have"));
      continue;
    };
    // rows of the table proper carry the section; the prose below it does not
    if l.trim_start().starts_with('|') {
      let want = l.split(']').next().unwrap_or("").rsplit('[').next().unwrap_or("");
      if want.starts_with('§') && want != sec {
        bad.push(format!("readme.md §10.2 files `{cmd}` under {want}; the program says {sec}"));
      }
    }
  }
  for (cmd, sec) in prog.iter().filter(|(_, s)| !s.is_empty()) {
    if !named.contains(cmd) {
      bad.push(format!("`{cmd}` produces {sec} and readme.md §10.2 does not list it"));
    }
  }
  assert!(bad.is_empty(), "§10.2 and the program disagree:\n  {}", bad.join("\n  "));
}
