//! A reader for the algomus `.ref` ground truth — readme §8 step 0.
//!
//! Step 2 hand-coded one subject's position from one fugue. A ranking needs 24,
//! and guessing them would make the ranking a measurement of my guesses. The
//! annotations give the subject's length and every entry's offset, checked
//! against four musicological sources, so they are the right input and the
//! parser is the price of using them.
//!
//! Offsets are in **measures**, with the fractional grammar `syntax.ref`
//! defines: `29` starts measure 29, `29.5` is its midpoint, `29+1/4` is a
//! quarter *note* after it, `29-1/16` a sixteenth before. Measure 1 is the
//! first complete measure, so an offset of 1 is tick 0.

use crate::kern::TICKS_PER_WHOLE;

#[derive(Clone, Debug)]
pub struct SubjectSpec {
  pub id: String,
  /// Length in ticks, from `[length N]` — N is in measures.
  pub len: i64,
  /// Every annotated entry: the voice letter, and the offset in ticks.
  pub entries: Vec<(char, i64)>,
  /// Alternative lengths other sources give, from `== S alternative` — §3.3.
  pub alternatives: Vec<i64>,
  /// Annotated cadences: tick, and the label in Hepokoski-Darcy notation such
  /// as `III:PAC`. Present in the ground truth from the beginning and unused
  /// until §16 — the only external check available on the harmonic analyser.
  pub cadences: Vec<(i64, String)>,
}

/// Parse one offset in the `.ref` grammar into ticks.
fn offset(tok: &str, measure: i64) -> Option<i64> {
  let tok = tok.trim();
  if tok.is_empty() {
    return None;
  }
  // split off a trailing +a/b or -a/b (but not the leading sign of the bar)
  let (head, extra) = match tok[1..].find(['+', '-']) {
    Some(i) => {
      let i = i + 1;
      (&tok[..i], Some(&tok[i..]))
    }
    None => (tok, None),
  };
  let bars: f64 = head.parse().ok()?;
  // measure 1 is tick 0; a fractional bar such as `.5` is a fraction of a bar
  let whole = (bars - 1.0) * measure as f64;
  let mut t = whole.round() as i64;
  if let Some(e) = extra {
    let sign = if e.starts_with('-') { -1 } else { 1 };
    let (a, b) = e[1..].split_once('/')?;
    let (a, b): (i64, i64) = (a.trim().parse().ok()?, b.trim().parse().ok()?);
    if b == 0 || TICKS_PER_WHOLE * a % b != 0 {
      return None;
    }
    t += sign * TICKS_PER_WHOLE * a / b;
  }
  Some(t)
}

/// Read every Bach subject spec from `fugues.ref`.
///
/// `measures` supplies the ticks-per-measure of each piece, which the
/// annotations do not carry — they are in bars, and a bar is only a duration
/// once the score says what the time signature is.
pub fn read(
  path: &std::path::Path,
  measures: &dyn Fn(&str) -> Option<i64>,
) -> Result<Vec<SubjectSpec>, String> {
  let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
  let mut out: Vec<SubjectSpec> = vec![];
  let mut cur: Option<SubjectSpec> = None;
  let mut measure = 0i64;
  // which label block we are inside: the primary S, an alternative, or other
  let mut in_s = false;
  let mut in_cad = false;

  for raw in text.lines() {
    let line = raw.split('#').next().unwrap_or("");
    let t = line.trim();
    if t.is_empty() {
      continue;
    }
    if let Some(rest) = t.strip_prefix("==== ") {
      if let Some(c) = cur.take() {
        out.push(c);
      }
      let id = rest.split_whitespace().next().unwrap_or("").to_string();
      measure = measures(&id).unwrap_or(0);
      cur = if measure > 0 {
        Some(SubjectSpec { id, len: 0, entries: vec![], alternatives: vec![], cadences: vec![] })
      } else {
        None
      };
      in_s = false;
      in_cad = false;
      continue;
    }
    let Some(c) = cur.as_mut() else { continue };

    if let Some(rest) = t.strip_prefix("== ") {
      let head = rest.split('[').next().unwrap_or("").trim();
      let len = rest
        .split_once("length")
        .and_then(|(_, r)| r.trim().split([']', ' ']).next())
        .and_then(|n| n.trim().parse::<f64>().ok())
        .map(|n| (n * measure as f64).round() as i64);
      // `S alternative` records a dissenting source's reading — §3.3
      if head == "S alternative" {
        if let Some(l) = len {
          c.alternatives.push(l);
        }
        in_s = false;
      } else if head == "S" {
        if let Some(l) = len {
          c.len = l;
        }
        in_s = true;
      } else {
        in_s = false;
        in_cad = head == "cadences";
      }
      continue;
    }

    if in_cad {
      // `*  25 (III:PAC), 37 (VII:PAC), ...`
      let Some((voices, rest)) = t.split_once(char::is_whitespace) else { continue };
      if !voices.chars().all(|ch| ch.is_ascii_uppercase() || ch == '*') {
        continue;
      }
      for field in rest.split(',') {
        let bare = field.split('(').next().unwrap_or("").trim();
        let label = field
          .split_once('(')
          .and_then(|(_, r)| r.split(')').next())
          .unwrap_or("")
          .to_string();
        if let Some(tick) = offset(bare, measure) {
          c.cadences.push((tick, label));
        }
      }
      continue;
    }

    if in_s {
      // an occurrence line: voice letters, then comma-separated offsets
      let Some((voices, rest)) = t.split_once(char::is_whitespace) else { continue };
      if !voices.chars().all(|ch| ch.is_ascii_uppercase() || ch == '*') {
        continue;
      }
      for field in rest.split(',') {
        // strip any per-occurrence [keywords]
        let bare = field.split('[').next().unwrap_or("").trim();
        if let Some(tick) = offset(bare, measure) {
          for v in voices.chars() {
            c.entries.push((v, tick));
          }
        }
      }
    }
  }
  if let Some(c) = cur.take() {
    out.push(c);
  }
  Ok(out)
}
