//! §2.4's grammar, parsed against the book — readme §8.15.
//!
//! [§2.4](../readme.md) asserts that form is a grammar and writes one:
//!
//! ```text
//! Fugue       → Exposition Middle+ Final
//! Exposition  → Entry (Countersubject Entry){V−1}
//! Middle      → Episode Entry+
//! Final       → Stretto? Pedal? Cadence
//! ```
//!
//! Ten lines, and until now none of them has been checked. §8.13 checked the
//! fifth production and found it a tendency rather than a rule; this checks the
//! other four, in the only way a grammar can be checked — **does it derive the
//! sentences it claims to be the grammar of?** A grammar that cannot parse the
//! Well-Tempered Clavier is not a grammar of fugue, whatever else it is.
//!
//! # What is parsed
//!
//! Not the notes: the **plan**. The ground truth annotates every subject entry
//! and every cadence, so a fugue arrives here as a sequence of entries with
//! voices and pitch levels, a set of typed cadences, and a length. That is
//! exactly what §2.4's non-terminals range over, and it is what a form grammar
//! would have to emit.
//!
//! # Each production as a separate verdict
//!
//! Reporting one number for "does the grammar parse" would hide which
//! production fails, and §8.2's whole method is that a rulebook is not one
//! thing. So each is a predicate with its own rate over the 24 fugues, and the
//! conjunction is reported beside them rather than instead of them.

use crate::{answer, kern::Piece};

/// One annotated subject entry.
#[derive(Clone, Copy, Debug)]
pub struct Entry {
  pub voice: usize,
  pub start: i64,
  /// The scale degree its first note takes, `0` for the tonic and `4` for the
  /// dominant. This rather than the whole entry's transposition, because
  /// §8.11 showed a comes is often *not* a transposition of anything — but its
  /// first note is a fact either way, and it is the fact `Exposition` is about.
  pub degree: usize,
}

/// A fugue as §2.4's non-terminals see it.
pub struct Plan {
  pub voices: usize,
  /// Sorted by start.
  pub entries: Vec<Entry>,
  pub measure: i64,
  /// The subject's length, which is what makes a gap measurable: an episode is
  /// time in which no entry is *sounding*, so it runs from where one entry ends
  /// to where the next begins, not from one start to the next.
  pub subject: i64,
  pub end: i64,
  pub cadences: Vec<(i64, String)>,
}

impl Plan {
  /// Entries split into groups, a group being a run with no episode inside it.
  ///
  /// `Middle → Episode Entry+` says entries come in groups separated by
  /// episodes, so the grouping *is* the parse — which is also why there is no
  /// verdict for that production. Once a group is defined as a run with no
  /// episode inside it, "every middle is reached across an episode" is true by
  /// construction, and a check that cannot fail is not a check. What the
  /// production actually claims and [`entries_per_middle`] measures is the
  /// **`+`**: that entries cluster after the exposition rather than arriving one
  /// at a time.
  ///
  /// An episode is a bar or more in which no entry sounds, measured from where
  /// the last one **ends**, which is §8.13's definition used unchanged so that
  /// the two sections cannot disagree about what an episode is.
  pub fn groups(&self) -> Vec<Vec<Entry>> {
    let mut out: Vec<Vec<Entry>> = vec![];
    for e in &self.entries {
      // the gap since the last entry *finished*. Measuring it from the last
      // entry's start instead puts every exposition into as many groups as it
      // has voices, since entries follow each other a subject-length apart —
      // which is exactly the reading that made this report 0% on all 22.
      match out.last_mut() {
        Some(g) if e.start - (g.iter().map(|x| x.start).max().unwrap() + self.subject) < self.measure => {
          g.push(*e)
        }
        _ => out.push(vec![*e]),
      }
    }
    out
  }
}

/// One verdict per production, and the conjunction.
#[derive(Debug, Default, Clone, Copy)]
pub struct Verdict {
  /// `Exposition → Entry (Countersubject Entry){V−1}`: the first `V` entries
  /// state the subject once in each of the `V` voices.
  ///
  /// Judged on the first `V` entries rather than on the first *group*, so that
  /// it does not depend on where an episode is held to begin. That threshold is
  /// a separate claim and gets [`Verdict::exposition_is_unbroken`] to itself.
  pub exposition_covers_the_voices: bool,
  /// And alternates tonic with dominant, which is what makes it an exposition
  /// rather than `V` entries in a row. Also on the first `V` entries.
  pub exposition_alternates: bool,
  /// And the production contains no `Episode`, so those `V` entries should run
  /// unbroken. This is the one exposition claim that depends on the episode
  /// threshold, which is why it is stated apart from the two that do not.
  pub exposition_is_unbroken: bool,
  /// `Fugue → Exposition Middle+`: there is at least one middle group, so the
  /// piece does not end when the exposition does.
  pub has_a_middle: bool,
  /// `Final → … Cadence`: the last annotated cadence is in the home key.
  pub ends_at_home: bool,
}

impl Verdict {
  pub fn all(&self) -> bool {
    self.exposition_covers_the_voices
      && self.exposition_alternates
      && self.exposition_is_unbroken
      && self.has_a_middle
      && self.ends_at_home
  }
}

/// Whether an entry is at the **dux's** level or the **comes'**, judged by the
/// degree its first note takes.
///
/// Not by whether that degree is the tonic. Seven of the WTC's subjects begin on
/// the dominant ([§8.11](../readme.md)), and their dux therefore opens on degree
/// 4 while still being a home-key entry; reading degree 4 as *dominant level*
/// calls those expositions non-alternating when they alternate perfectly. What
/// makes an entry a comes is that it answers the dux, and §8.11's Rule I — exact
/// on all seven of the cases where it says anything — is what that means.
///
/// A subject opening on neither tonic nor dominant is one Marpurg defers to a
/// later chapter; the ordinary answer at the fifth is assumed for it, and that
/// assumption is visible here rather than buried.
fn levels(dux_degree: usize) -> (usize, usize) {
  let leg = answer::first_leg(dux_degree).unwrap_or(answer::Leg::Fifth);
  (dux_degree, answer::answered(dux_degree, leg))
}

/// Parse one fugue's plan against §2.4.
pub fn parse(p: &Plan) -> Verdict {
  let groups = p.groups();
  let Some(first) = groups.first() else { return Verdict::default() };
  let mut v = Verdict::default();

  // Exposition: the first V entries, one per voice, every voice used. Taken by
  // count rather than by group, so the figure does not move with the threshold.
  let open = &p.entries[..p.voices.min(p.entries.len())];
  let mut seen: Vec<usize> = open.iter().map(|e| e.voice).collect();
  seen.sort_unstable();
  seen.dedup();
  v.exposition_covers_the_voices = open.len() == p.voices && seen.len() == p.voices;

  // and alternating dux level, comes level, dux level, …
  let (dux, comes) = levels(p.entries[0].degree);
  v.exposition_alternates = open.len() == p.voices
    && open.iter().enumerate().all(|(i, e)| e.degree == if i % 2 == 0 { dux } else { comes });

  // and unbroken: the production has no `Episode` in it, so all V should be in
  // the opening group. This is the claim that moves with the threshold.
  v.exposition_is_unbroken = first.len() >= p.voices;

  v.has_a_middle = groups.len() > 1;

  // Final: the last cadence names the home key. `I:` and `i:` are the home key
  // by construction, the labels being relative to it.
  v.ends_at_home = p
    .cadences
    .iter()
    .max_by_key(|(t, _)| *t)
    .and_then(|(_, l)| l.split(':').next().map(|r| r.trim().eq_ignore_ascii_case("i")))
    .unwrap_or(false);
  v
}

/// Does any pair of entries overlap — a stretto, in `Final`'s sense.
pub fn has_stretto(p: &Plan) -> bool {
  p.entries.windows(2).any(|w| w[1].start - w[0].start < p.subject && w[1].voice != w[0].voice)
}

/// The number of `Middle` groups a fugue actually has, which is the `+` in
/// `Middle+` and the one quantity §2.4 leaves unbounded.
pub fn middles(p: &Plan) -> usize {
  p.groups().len().saturating_sub(1)
}

/// Mean entries per middle group — the `+` in `Entry+`.
///
/// If this is one, the production is `Middle → Episode Entry` and entries do not
/// cluster after the exposition at all, which would be a fact about fugue worth
/// knowing and is not what §2.4 says.
pub fn entries_per_middle(p: &Plan) -> f64 {
  let g = p.groups();
  if g.len() < 2 {
    return 0.0;
  }
  g[1..].iter().map(|x| x.len() as f64).sum::<f64>() / (g.len() - 1) as f64
}

/// The gaps between entry groups, in bars — the episodes `Middle` names.
pub fn gaps(p: &Plan) -> Vec<f64> {
  let g = p.groups();
  g.windows(2)
    .map(|w| {
      let ends = w[0].iter().map(|x| x.start).max().unwrap() + p.subject;
      (w[1][0].start - ends) as f64 / p.measure.max(1) as f64
    })
    .collect()
}

/// Build a plan from a piece and its annotation.
pub fn plan_of(
  p: &Piece,
  entries: &[(usize, i64, usize)],
  cadences: &[(i64, String)],
  subject: i64,
) -> Plan {
  let mut es: Vec<Entry> =
    entries.iter().map(|&(voice, start, degree)| Entry { voice, start, degree }).collect();
  es.sort_by_key(|e| e.start);
  Plan {
    voices: p.voices.len(),
    entries: es,
    measure: p.measure,
    subject,
    end: p.voices.iter().flat_map(|v| v.notes.iter().map(|n| n.onset + n.dur)).max().unwrap_or(0),
    cadences: cadences.to_vec(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Bars of 960 ticks and a subject two bars long, so that entries a
  /// subject-length apart are one exposition and not four groups.
  fn plan(voices: usize, es: &[(usize, i64, usize)], cad: &[(i64, &str)]) -> Plan {
    Plan {
      voices,
      entries: es.iter().map(|&(v, s, d)| Entry { voice: v, start: s, degree: d }).collect(),
      measure: 960,
      subject: 1920,
      end: 100_000,
      cadences: cad.iter().map(|(t, l)| (*t, l.to_string())).collect(),
    }
  }

  /// A textbook three-voice exposition — tonic, dominant, tonic, in three
  /// voices, then an episode and a middle group, then a cadence at home.
  #[test]
  fn a_textbook_fugue_parses() {
    let p = plan(
      3,
      &[(0, 0, 0), (1, 1920, 4), (2, 3840, 0), (1, 12000, 4), (0, 13920, 0)],
      &[(9000, "I:PAC")],
    );
    let v = parse(&p);
    assert!(v.exposition_covers_the_voices, "{v:?}");
    assert!(v.exposition_alternates, "{v:?}");
    assert!(v.exposition_is_unbroken, "{v:?}");
    assert!(v.has_a_middle, "{v:?}");
    assert!(v.ends_at_home, "{v:?}");
    assert!(v.all());
    assert_eq!(middles(&p), 1);
  }

  /// The two exposition claims come apart, and must: entries in all four voices
  /// with an episode between the third and the fourth cover the voices and are
  /// not unbroken, which is the ordinary shape of a Bach exposition.
  #[test]
  fn covering_the_voices_and_running_unbroken_are_different_claims() {
    let p = plan(
      3,
      &[(0, 0, 0), (1, 1920, 4), (2, 8000, 0), (0, 20000, 0)],
      &[(30000, "I:PAC")],
    );
    let v = parse(&p);
    assert!(v.exposition_covers_the_voices, "three entries in three voices");
    assert!(v.exposition_alternates);
    assert!(!v.exposition_is_unbroken, "an episode falls before the third");
  }

  /// An exposition that states the subject twice in one voice and never in
  /// another is not the production `Exposition` names, however it sounds.
  #[test]
  fn an_exposition_must_use_every_voice_once() {
    let p = plan(3, &[(0, 0, 0), (1, 1920, 4), (0, 3840, 0)], &[(9000, "I:PAC")]);
    assert!(!parse(&p).exposition_covers_the_voices);
  }

  /// Tonic, tonic, dominant is not an alternation.
  #[test]
  fn an_exposition_must_alternate_tonic_and_dominant() {
    let p = plan(3, &[(0, 0, 0), (1, 1920, 0), (2, 3840, 4)], &[(9000, "I:PAC")]);
    assert!(parse(&p).exposition_covers_the_voices);
    assert!(!parse(&p).exposition_alternates);
  }

  /// A subject opening on the **dominant** still alternates: its dux takes
  /// degree 4 and its comes takes the tonic, which is §8.11's Rule I. Reading
  /// degree 4 as `dominant level` regardless would fail seven of the WTC's
  /// expositions for alternating exactly as they should.
  #[test]
  fn an_exposition_on_a_dominant_subject_alternates_the_other_way() {
    let p = plan(3, &[(0, 0, 4), (1, 1920, 0), (2, 3840, 4)], &[(9000, "I:PAC")]);
    let v = parse(&p);
    assert!(v.exposition_covers_the_voices);
    assert!(v.exposition_alternates, "dominant subject, tonic answer, dominant again");
  }

  /// An entry arriving half a bar after the last one finished is part of the
  /// same group, not a new one across an episode — which is why there is no
  /// verdict for `Middle → Episode Entry+` and a measurement instead.
  #[test]
  fn an_entry_arriving_before_an_episode_joins_the_group() {
    let close = plan(2, &[(0, 0, 0), (1, 1920, 4), (0, 4320, 0)], &[(9000, "I:PAC")]);
    assert_eq!(close.groups().len(), 1, "half a bar is not an episode");
    let apart = plan(2, &[(0, 0, 0), (1, 1920, 4), (0, 6000, 0)], &[(9000, "I:PAC")]);
    assert_eq!(apart.groups().len(), 2);
    assert_eq!(entries_per_middle(&apart), 1.0);
    assert!((gaps(&apart)[0] - 2.25).abs() < 0.01, "{:?}", gaps(&apart));
  }

  /// A fugue ending away from home does not match `Final`.
  #[test]
  fn the_last_cadence_must_be_at_home() {
    let p = plan(2, &[(0, 0, 0), (1, 1920, 4)], &[(500, "I:PAC"), (9000, "V:PAC")]);
    assert!(!parse(&p).ends_at_home);
  }

  /// Overlapping entries in different voices are a stretto; the same voice
  /// re-entering is not.
  #[test]
  fn a_stretto_is_two_voices_at_once() {
    let mut p = plan(2, &[(0, 0, 0), (1, 400, 4)], &[]);
    assert!(has_stretto(&p));
    p.subject = 300;
    assert!(!has_stretto(&p));
  }
}
