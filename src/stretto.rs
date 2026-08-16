//! Step 2: the compatibility table, and the clique test — readme §3.
//!
//! An entry is `(τ, d, k)`: a transformation, an offset in ticks, a
//! transposition. Because every transformation acts relative to its own entry
//! point, the interval sequence between two entries depends only on the pair of
//! transformation types and the *differences* `Δd` and `Δk`. Compatibility is
//! therefore a **table**, filled once per subject by running the §2.2 automaton
//! over each overlap, and a legal stretto is a **clique** in the graph that
//! table defines.
//!
//! The verdict test is §3.1's, and it is integer equality rather than a
//! threshold: BWV 867's five final entries stand at quarters
//! `{266, 268, 270, 272, 274}`, which is `{0, 2, 4, 6, 8}` from the first.
//! Does that set come out as a clique?

use crate::{
  corpus,
  kern::{Note, Voice, TICKS_PER_QUARTER},
  pitch::{Interval, Pitch},
};

/// A subject, held as offsets and durations from its own first note, with
/// pitches as written. Placing it is then a shift in time and a transposition.
#[derive(Clone, Debug)]
pub struct Subject {
  pub notes: Vec<Note>,
  pub len: i64,
}

impl Subject {
  /// Cut a window out of a voice. `len` is the subject's length, which for
  /// BWV 867 is an editorial decision rather than a fact — see §3.3.
  ///
  /// The end is **inclusive**, which is the corpus's own convention and not a
  /// detail: `syntax.ref` says a length is measured "between the offsets of the
  /// start and the end", where "the end of the pattern denotes the impact of
  /// the last note". An exclusive bound drops that last note — and dropping it
  /// gave a five-note subject where ricercar's independent hand transcription
  /// of the same span has six, which is how the off-by-one was caught.
  pub fn cut(v: &Voice, start: i64, len: i64) -> Subject {
    let notes: Vec<Note> = v
      .notes
      .iter()
      .filter(|n| n.onset >= start && n.onset <= start + len)
      .map(|n| Note { onset: n.onset - start, ..*n })
      .collect();
    Subject { notes, len }
  }

  /// Place at offset `d`, transposed by a named interval.
  pub fn place(&self, d: i64, dsteps: i16, dsemis: i16) -> Voice {
    Voice {
      notes: self
        .notes
        .iter()
        .map(|n| Note { onset: n.onset + d, pitch: n.pitch.transpose(dsteps, dsemis), ..*n })
        .collect(),
    }
  }

  pub fn head(&self) -> Option<Pitch> {
    self.notes.first().map(|n| n.pitch)
  }
}

/// Two entries, and what the rulebook says about the pair.
#[derive(Clone, Debug)]
pub struct Verdict {
  pub hard: usize,
  pub soft: usize,
  pub worst: Vec<(&'static str, usize)>,
}

impl Verdict {
  pub fn legal(&self) -> bool {
    self.hard == 0
  }
}

/// Run the automaton over the overlap of two placed entries, judged against a
/// given tier of hard rules.
pub fn compatible(a: &Voice, b: &Voice, measure: i64, tier: &[crate::automaton::Rule]) -> Verdict {
  let t = corpus::check_voices(a, b, measure);
  let mut worst: Vec<(&'static str, usize)> =
    t.by_rule.iter().filter(|(k, _)| tier.iter().any(|r| r.name() == **k))
      .map(|(k, v)| (*k, *v)).collect();
  worst.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
  Verdict {
    hard: tier.iter().filter_map(|r| t.by_rule.get(r.name())).sum(),
    soft: t.by_rule.iter().filter(|(k, _)| !is_hard_name(k)).map(|(_, v)| *v).sum(),
    worst,
  }
}

fn is_hard_name(n: &str) -> bool {
  crate::automaton::HARD.iter().any(|r| r.name() == n)
}

/// A placement under consideration: offset in ticks, transposition as a named
/// interval, and the voice it would occupy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
  pub d: i64,
  pub dsteps: i16,
  pub dsemis: i16,
}

impl Entry {
  pub fn label(&self) -> String {
    format!("+{:>3}q {:+}st", self.d / TICKS_PER_QUARTER, self.dsteps)
  }
}

/// The compatibility graph over a candidate set, as an adjacency matrix.
pub struct Table {
  pub entries: Vec<Entry>,
  pub ok: Vec<Vec<bool>>,
}

pub fn build(sub: &Subject, entries: &[Entry], measure: i64,
  tier: &[crate::automaton::Rule]) -> Table {
  let voices: Vec<Voice> =
    entries.iter().map(|e| sub.place(e.d, e.dsteps, e.dsemis)).collect();
  let n = entries.len();
  let mut ok = vec![vec![true; n]; n];
  for i in 0..n {
    for j in (i + 1)..n {
      let legal = compatible(&voices[i], &voices[j], measure, tier).legal();
      ok[i][j] = legal;
      ok[j][i] = legal;
    }
  }
  Table { entries: entries.to_vec(), ok }
}

impl Table {
  /// The largest set of mutually compatible entries. Clique size is bounded by
  /// the voice count, so this is a depth-`V` search rather than a general
  /// maximum-clique instance — readme §3.
  pub fn max_clique(&self, limit: usize) -> Vec<usize> {
    let n = self.entries.len();
    let mut best: Vec<usize> = vec![];
    let mut cur: Vec<usize> = vec![];
    fn go(
      t: &Table,
      n: usize,
      start: usize,
      cur: &mut Vec<usize>,
      best: &mut Vec<usize>,
      limit: usize,
    ) {
      if cur.len() > best.len() {
        *best = cur.clone();
      }
      if cur.len() == limit {
        return;
      }
      for v in start..n {
        if cur.iter().all(|&u| t.ok[u][v]) {
          cur.push(v);
          go(t, n, v + 1, cur, best, limit);
          cur.pop();
        }
      }
    }
    go(self, n, 0, &mut cur, &mut best, limit);
    best
  }

  /// Is this specific set mutually compatible? The verdict test.
  pub fn is_clique(&self, ix: &[usize]) -> bool {
    ix.iter().all(|&i| ix.iter().all(|&j| i == j || self.ok[i][j]))
  }
}

/// Recover what transposition an entry in the score actually uses, by
/// comparing its first note against the subject's.
pub fn interval_from(sub: &Subject, first: Pitch) -> (i16, i16) {
  let head = sub.head().expect("subject has notes");
  let iv = Interval::between(head, first);
  (iv.steps, iv.semis)
}

pub fn rule_is_hard(name: &str) -> bool {
  is_hard_name(name)
}
