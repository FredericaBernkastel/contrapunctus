//! Step 7: a whole fugue, from a subject — readme §8.16.
//!
//! Everything before this filled voices **against music that already existed**.
//! §8.6 held one of Bach's entries and reconstructed the others; §8.3 placed
//! entries into a span Bach had written. This emits the span too.
//!
//! # What it builds, and why not §2.4's grammar
//!
//! [§8.15](../readme.md) parsed §2.4's ten productions against the book and they
//! derive three fugues in twenty-two. What survives is the shape — exposition,
//! middles, a cadence at home — and what fails is `Exposition`, which forbids the
//! link that 82% of real expositions contain. So the grammar here is §8.15's
//! corrected one, and its numbers are §8.15's and §8.13's rather than §2.4's:
//! a median of **three** middle groups, about **1.35** entries each, episodes of
//! about **three bars**, and a close at home.
//!
//! # Three constraints this had to be built around
//!
//! **Two free voices.** [§2.7](../readme.md) predicted the wall at four and
//! [§8.6](../readme.md) measured it at two, so a three-voice fugue is what an
//! exact search can do and that is the scope. Half the book is four voices or
//! more and out of reach until a solver replaces the DP.
//!
//! **Episodes have nothing held in them.** [§8.13](../readme.md) found episodes
//! are **54% of the book by duration**, and in an episode no subject sounds — so
//! all three voices would be free, one more than the search can do. The way out
//! is §2.4's own `Sequence(motive, …)`: a motive taken from the subject is
//! *placed* in one voice and sequenced, leaving two free. That is a real
//! commitment and §8.13 measured its cost — only 13.3% of Bach's episodes are
//! strictly sequential, so this generator writes a kind of episode that is a
//! minority of the book's.
//!
//! **Rhythm is data.** [§2.6](../readme.md) makes rhythm an input, not a
//! variable, which is what keeps the search a shortest path. A reconstruction
//! gets rhythm from the piece it is reconstructing; a generator has to invent it.
//! Here every free voice takes **the subject's own rhythm**, tiled — which is
//! both the cheapest defensible choice and a real limitation, since a fugue
//! whose accompanying voices all move in the subject's rhythm is a stiffer thing
//! than Bach writes.

use crate::{
  answer,
  automaton::Rule,
  harmony,
  kern::{Note, Piece, Voice},
  pitch::Pitch,
  realise::{self, Problem},
};

/// What the generator is given.
pub struct Design {
  /// The subject, starting at tick 0.
  pub subject: Voice,
  pub voices: usize,
  pub key: [i8; 7],
  /// The tonic's diatonic letter, `0` for C.
  pub tonic: usize,
  pub measure: i64,
  pub beat: i64,
  /// Each voice's range, top voice first.
  pub compass: Vec<(i16, i16)>,
}

/// One block of the derivation.
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
  /// A subject entry in one voice, `shift` diatonic steps from the subject as
  /// given. `tonal` marks the comes, whose first note follows §8.11's Rule I
  /// rather than the shift.
  Entry { voice: usize, shift: i16, tonal: bool },
  /// A stretch with no subject in it, whose motive is placed in `voice`.
  Episode { voice: usize, shift: i16 },
}

#[derive(Clone, Debug)]
pub struct Block {
  pub at: i64,
  pub len: i64,
  pub kind: Kind,
  /// The local key, as diatonic steps above the home tonic.
  pub key_of: i16,
}

/// Indices into [`harmony::QUALITIES`], named so the plan below reads as music
/// rather than as subscripts.
const TRIAD: usize = 0;
const DOMINANT_SEVENTH: usize = 4;

/// Which voice a block places its line in.
fn held_of(b: Option<&Block>) -> usize {
  match b.map(|x| &x.kind) {
    Some(Kind::Entry { voice, .. }) | Some(Kind::Episode { voice, .. }) => *voice,
    None => 0,
  }
}

/// The subject's length in ticks, rounded up to a whole bar so that entries and
/// episodes fall on bar lines — which is where §8.15 measured them.
fn subject_bars(d: &Design) -> i64 {
  let end = d.subject.notes.iter().map(|n| n.onset + n.dur).max().unwrap_or(0);
  ((end + d.measure - 1) / d.measure).max(1) * d.measure
}

/// The shape of the piece, as a caller may vary it.
///
/// Separate from [`Design`] because the two answer different questions.
/// `Design` is *what the music is made of* — a subject, a key, a number of
/// voices — and this is *what is done with it*. A user interface wants a
/// control for each of these; a measurement wants the defaults, which are
/// §8.15's and §8.13's readings of the book and not anybody's taste.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
  /// One middle group per entry, giving its key as **diatonic steps above the
  /// home tonic** — so `4` is the dominant, `3` the subdominant, `5` the
  /// submediant, and the mode of each follows from the key signature rather
  /// than being chosen. §2.4 calls this a bounded walk on the circle of fifths
  /// and it is the only part of that production this kept.
  ///
  /// §8.15 measured a median of **three** middle groups across the book, with a
  /// range of nought to nine.
  pub middles: Vec<i16>,
  /// Bars per episode. §8.13 measured a median of **three**.
  pub episode_bars: i64,
  /// Bars of link inside the exposition, and after which entry it falls.
  ///
  /// §2.4's `Exposition` forbids this and §8.15 found **82%** of Bach's contain
  /// one, which is the single largest correction that section made. `None`
  /// writes the exposition §2.4 describes and 18% of the book has.
  pub link: Option<(usize, i64)>,
  /// Whether to close with an episode and a final entry at home. §8.15 found
  /// every fugue in the book ending at home, 22 of 22.
  pub close_at_home: bool,
}

impl Default for Layout {
  /// The book's own shape, as [§8.15](../readme.md) and [§8.13](../readme.md)
  /// measured it. Every published figure uses this.
  fn default() -> Self {
    Layout {
      middles: vec![4, 5, 3],
      episode_bars: 3,
      // after the second entry, which is where a three-voice exposition takes
      // one; `voices - 2` in the general case
      link: Some((1, 1)),
      close_at_home: true,
    }
  }
}

/// Derive a plan from a design and a layout.
pub fn derive(d: &Design, l: &Layout) -> Vec<Block> {
  let sl = subject_bars(d);
  let ep = l.episode_bars.max(1) * d.measure;
  let mut out = vec![];
  let mut t = 0i64;

  // Exposition: one entry per voice, alternating dux and comes, top voice down,
  // with the link §2.4 forbids and §8.15 found in 82% of the book.
  for i in 0..d.voices {
    let tonal = i % 2 == 1;
    out.push(Block {
      at: t,
      len: sl,
      kind: Kind::Entry { voice: i, shift: if tonal { 4 } else { 0 }, tonal },
      key_of: if tonal { 4 } else { 0 },
    });
    t += sl;
    if let Some((after, bars)) = l.link {
      if i == after && bars > 0 && i + 1 < d.voices {
        let held = held_of(out.last());
        out.push(Block {
          at: t,
          len: bars * d.measure,
          kind: Kind::Episode { voice: (held + 1) % d.voices, shift: 0 },
          key_of: 0,
        });
        t += bars * d.measure;
      }
    }
  }

  // Each middle is an episode and then an entry.
  //
  // The episode's motive never goes to the voice that just finished an entry.
  // Both lines are *placed*, so nothing in the search stands between the end of
  // one and the start of the other, and a voice asked to do both in succession
  // jumps whatever distance separates them — eight steps, in the run that found
  // this. Handing the motive to another voice costs nothing and is what a fugue
  // does anyway.
  for &key_of in l.middles.iter() {
    let after = held_of(out.last());
    out.push(Block {
      at: t,
      len: ep,
      kind: Kind::Episode { voice: (after + 1) % d.voices, shift: 0 },
      key_of,
    });
    t += ep;
    let after = held_of(out.last());
    out.push(Block {
      at: t,
      len: sl,
      kind: Kind::Entry { voice: (after + 1) % d.voices, shift: key_of, tonal: false },
      key_of,
    });
    t += sl;
  }

  if l.close_at_home {
    let after = held_of(out.last());
    out.push(Block {
      at: t,
      len: ep,
      kind: Kind::Episode { voice: (after + 1) % d.voices, shift: 0 },
      key_of: 0,
    });
    t += ep;
    let after = held_of(out.last());
    out.push(Block {
      at: t,
      len: sl,
      kind: Kind::Entry { voice: (after + 1) % d.voices, shift: 0, tonal: false },
      key_of: 0,
    });
  }
  out
}

/// The whole piece's length.
pub fn length(blocks: &[Block]) -> i64 {
  blocks.iter().map(|b| b.at + b.len).max().unwrap_or(0)
}

/// Transpose the subject `shift` diatonic steps within the key, and move it to
/// `at`. A `tonal` entry takes §8.11's answer instead, whose first note obeys
/// Marpurg's Rule I — the one transcribed rule the WTC does not break.
fn state(d: &Design, at: i64, shift: i16, tonal: bool) -> Voice {
  let base = if tonal {
    answer::admissible(&d.subject, &d.key, d.tonic).into_iter().next().unwrap_or_else(|| d.subject.clone())
  } else {
    Voice {
      notes: d
        .subject
        .notes
        .iter()
        .map(|n| Note { pitch: answer::step_in_key(n.pitch, shift, &d.key), ..*n })
        .collect(),
    }
  };
  Voice { notes: base.notes.iter().map(|n| Note { onset: n.onset + at, ..*n }).collect() }
}

/// An episode's placed voice: the subject's **head** — its first bar — stated
/// and then sequenced down by step, which is §2.4's `Sequence(motive, …)` with
/// the motive taken from the subject rather than invented.
fn sequence(d: &Design, at: i64, len: i64, shift: i16) -> Voice {
  let head: Vec<Note> = d.subject.notes.iter().filter(|n| n.onset < d.measure).cloned().collect();
  if head.is_empty() {
    return Voice { notes: vec![] };
  }
  let mut notes = vec![];
  let mut k = 0i64;
  while k * d.measure < len {
    for n in &head {
      if n.onset >= d.measure {
        continue;
      }
      notes.push(Note {
        onset: at + k * d.measure + n.onset,
        pitch: answer::step_in_key(n.pitch, shift - k as i16, &d.key),
        ..*n
      });
    }
    k += 1;
  }
  notes.retain(|n| n.onset + n.dur <= at + len);
  Voice { notes }
}

/// The subject's rhythm, tiled to fill `[at, at+len)` — §2.6's cost, paid where
/// it falls. A free voice needs onsets before it can be given pitches, and a
/// generator has nowhere else to get them.
fn rhythm(d: &Design, at: i64, len: i64, phase: usize) -> Vec<Note> {
  // The subject's note *values*, laid end to end from the first tick rather than
  // copied with their onsets.
  //
  // Copying the onsets was the first version and it had an audible fault. A
  // subject with an upbeat — BWV 847's begins at tick 120 — leaves that gap at
  // the head of every tile; every free voice carries the same tiled rhythm, so
  // every voice fell silent in the same 120 ticks, together, once per tile. At
  // 76 to the minute that is four tenths of a second of nothing, every six
  // seconds, and a listener reported it before any test did.
  //
  // Laid end to end there is no gap to share. `phase` rotates the sequence per
  // voice so that even the note *boundaries* do not line up, which is a
  // separate fault of the same kind and cheaper to prevent than to hear.
  let durs: Vec<(i64, Pitch)> =
    d.subject.notes.iter().filter(|n| n.attack).map(|n| (n.dur.max(1), n.pitch)).collect();
  if durs.is_empty() {
    return vec![];
  }
  let mut out = vec![];
  let (mut t, mut i) = (0i64, phase);
  while t < len {
    let (dur, pitch) = durs[i % durs.len()];
    let dur = dur.min(len - t);
    out.push(Note { onset: at + t, dur, pitch, attack: true });
    t += dur;
    i += 1;
  }
  out
}

/// The harmonic plan, **per beat** — [§8.9](../readme.md)'s requirement, which
/// measured a chord per bar as losing more than half of what a correct plan is
/// worth.
///
/// The chords are the subject's own, analysed once by §8.5's analyser and
/// transposed to each block's key. An episode takes a descending-fifths
/// sequence, which is what a sequence is harmonically.
pub fn plan(d: &Design, blocks: &[Block]) -> Vec<harmony::Segment> {
  let mut out = vec![];
  let heard = harmony::analyse_viterbi(std::slice::from_ref(&d.subject), d.beat, 1.0);
  for b in blocks {
    let mut t = b.at;
    let mut step = 0i64;
    while t < b.at + b.len {
      let chord = match &b.kind {
        Kind::Entry { .. } => {
          let u = t - b.at;
          heard.iter().find(|s| s.start <= u && u < s.end).or(heard.first()).and_then(|s| s.chord)
        }
        // a descending-fifths sequence: the root falls a fifth each bar
        Kind::Episode { .. } => None,
      };
      let root = |deg: i16| -> u8 {
        let l = (d.tonic as i16 + deg).rem_euclid(7) as usize;
        Pitch::new(l as i16, d.key[l]).chroma().rem_euclid(12) as u8
      };
      let c = match chord {
        Some(c) => {
          // transpose the subject's own chord by the block's key
          let shift = root(b.key_of) as i16 - root(0) as i16;
          harmony::Chord { root: (c.root as i16 + shift).rem_euclid(12) as u8, quality: c.quality }
        }
        None => {
          // The dominant of the local key takes a seventh, which is what makes a
          // descending-fifths sequence pull rather than merely step. Both arms of
          // this returned a plain triad until clippy pointed out they were the
          // same expression — an intention that had been written down and not
          // finished, in the one part of this plan §8.16 already calls the
          // weakest.
          let deg = (b.key_of + 4 - (step % 4) as i16 * 3).rem_euclid(7);
          let dominant = (deg - b.key_of).rem_euclid(7) == 4;
          harmony::Chord { root: root(deg), quality: if dominant { DOMINANT_SEVENTH } else { TRIAD } }
        }
      };
      out.push(harmony::Segment { start: t, end: t + d.beat, chord: Some(c), fit: 1.0 });
      t += d.beat;
      step += 1;
    }
  }
  out
}

/// How many blocks needed a constraint dropped before they could be filled.
///
/// A generator that dies on one block of twelve is not a generator, and one that
/// silently relaxes until it succeeds is not a measurement. So the relaxation is
/// ordered, counted and printed: the harmonic plan first, since
/// the melodic continuity across the join first, since that is a convenience
/// this generator invented rather than anything the rulebook asks for; and only
/// then the harmonic plan, which is [§2.3](../readme.md)'s obligation system and
/// is *also* what keeps the search tractable — dropping it first turns a dead
/// block into an exploded one, which is worse.
#[derive(Default, Debug, Clone)]
pub struct Relaxed {
  pub blocks: usize,
  /// Filled only after the join to the previous block was cut.
  pub without_prior: usize,
  /// Filled only after the harmonic plan was dropped as well.
  pub without_plan: usize,
  /// **Which** blocks lost the join, by index.
  ///
  /// A count is enough to report and not enough to check against: a voice that
  /// enters a block cold may legally leap a tenth into it, because there is
  /// nothing for the melodic rule to measure the leap from. That is the join's
  /// cost made visible, and a test that wants to hold the *other* blocks to a
  /// standard has to know which these are.
  pub cold: Vec<usize>,
}

/// Generate: derive a plan, then fill it **block by block**.
///
/// One call per block rather than one for the piece, because
/// [`Problem::free`] marks a voice free for a whole span and a fugue is exactly
/// the case where that is not constant — the subject moves between voices, so
/// which voice is held changes every block. Filling the whole thing at once
/// would mean calling every voice free, which is three, and §8.6 measured the
/// wall at two.
///
/// Block by block, the held voice is whichever the derivation placed there and
/// the other two are free — two, which is the wall exactly. The join is what
/// [`Problem::prior`] is for.
///
/// The search runs in [`Problem::drawing`]'s configuration, which is what
/// [§8.10](../readme.md) endorses — no objective, and the answer taken from a
/// uniform draw rather than from a tie-break it measured at 1.3%.
pub fn generate(
  d: &Design,
  l: &Layout,
  tier: &[Rule],
  seed: u64,
) -> Result<(Vec<Block>, Vec<Voice>, Relaxed), String> {
  let blocks = derive(d, l);
  let plan = plan(d, &blocks);
  let mut out: Vec<Voice> = vec![Voice { notes: vec![] }; d.voices];
  let mut prior: Vec<Option<Pitch>> = vec![None; d.voices];
  let mut relaxed = Relaxed::default();

  for (bi, b) in blocks.iter().enumerate() {
    let held = match &b.kind {
      Kind::Entry { voice, .. } => *voice,
      Kind::Episode { voice, .. } => *voice,
    };
    // Which voice takes the *next* block's placed line. A voice about to state
    // the subject drops out before it does — which is what Bach writes, and
    // which is also the only way to stop it entering by a leap of an eleventh
    // from wherever its accompanying line happened to end. The entry's first
    // note is placed by the derivation, so no amount of care in the fill can
    // reach it; the fix is to have nothing there to reach from. A test found
    // this, at bar 3, in the first fugue this generator produced.
    let next = blocks.get(bi + 1).map(|nb| match &nb.kind {
      Kind::Entry { voice, .. } => *voice,
      Kind::Episode { voice, .. } => *voice,
    });
    // a bar of rest where the block can spare one, half the block where it
    // cannot — a one-bar subject has no bar to give
    let quiet = d.measure.min(b.len / 2);
    let line = match &b.kind {
      Kind::Entry { shift, tonal, .. } => state(d, 0, *shift, *tonal),
      Kind::Episode { shift, .. } => sequence(d, 0, b.len, *shift),
    };
    // every voice gets notes: the held one its placed line, the rest the
    // subject's rhythm, whose pitches the search discards
    let voices: Vec<Voice> = (0..d.voices)
      .map(|v| {
        if v == held {
          line.clone()
        } else if Some(v) == next && quiet > 0 {
          Voice { notes: rhythm(d, 0, b.len - quiet, v) }
        } else {
          Voice { notes: rhythm(d, 0, b.len, v) }
        }
      })
      .collect();
    if voices.iter().any(|v| v.notes.is_empty()) {
      return Err(format!("block {bi} leaves a voice with no notes to place"));
    }
    let free: Vec<bool> = (0..d.voices).map(|v| v != held).collect();
    let here: Vec<harmony::Segment> = plan
      .iter()
      .filter(|s| s.start >= b.at && s.start < b.at + b.len)
      .map(|s| harmony::Segment { start: s.start - b.at, end: s.end - b.at, ..s.clone() })
      .collect();
    let joined: Vec<Option<Pitch>> = prior.clone();

    let mut sol = None;
    let mut why = String::new();
    for attempt in 0..3 {
      let pr = Problem {
        voices: voices.clone(),
        free: free.clone(),
        compass: d.compass.clone(),
        key: d.key,
        measure: d.measure,
        // the join goes before the plan does. The plan is §2.3's obligation
        // system and it is also what keeps the search tractable — dropping it
        // first turns a dead block into an exploded one, which is worse. The
        // prior is this generator's own convenience and is dropped first.
        plan: if attempt < 2 { here.clone() } else { vec![] },
        tier,
        weights: [1.0; 6],
        prescribe: [0.0; 3],
        prior: if attempt == 0 { joined.clone() } else { vec![] },
        samples: 0,
        seed: seed ^ (bi as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
        beta: 0.0,
      }
      .drawing();
      match realise::fill(&pr) {
        Ok(s) => {
          relaxed.blocks += (attempt > 0) as usize;
          relaxed.without_prior += (attempt >= 1) as usize;
          relaxed.without_plan += (attempt >= 2) as usize;
          if attempt >= 1 {
            relaxed.cold.push(bi);
          }
          sol = Some(s);
          break;
        }
        Err(e) => why = e,
      }
    }
    let Some(sol) = sol else {
      // say what was being asked for, not only that it failed: a refusal whose
      // context is invisible is a refusal nobody can act on
      let lo = line.notes.iter().map(|n| n.pitch.step).min().unwrap_or(0);
      let hi = line.notes.iter().map(|n| n.pitch.step).max().unwrap_or(0);
      let ranges: Vec<String> = (0..d.voices)
        .filter(|v| free[*v])
        .map(|v| format!("v{v} {}..{}", d.compass[v].0, d.compass[v].1))
        .collect();
      return Err(format!(
        "block {bi} at bar {}: {why}
     held voice {held} spans {lo}..{hi}; free voices {}",
        b.at / d.measure + 1,
        ranges.join(", ")
      ));
    };
    for (v, filled) in sol.chosen().iter().enumerate() {
      if let Some(last) = filled.notes.iter().max_by_key(|n| n.onset) {
        prior[v] = Some(last.pitch);
      }
      // a voice that rested before entering enters cold, which is the point
      if Some(v) == next && quiet > 0 {
        prior[v] = None;
      }
      out[v].notes.extend(filled.notes.iter().map(|n| Note { onset: n.onset + b.at, ..*n }));
    }
  }
  for v in out.iter_mut() {
    v.notes.sort_by_key(|n| n.onset);
  }
  Ok((blocks, out, relaxed))
}

/// The derivation as §8.15's parser sees it, so that a generated fugue can be
/// held to the same grammar the book was.
pub fn as_plan(d: &Design, blocks: &[Block], p: &Piece) -> crate::form::Plan {
  let entries: Vec<(usize, i64, usize)> = blocks
    .iter()
    .filter_map(|b| match &b.kind {
      Kind::Entry { voice, .. } => {
        // the first *attack* inside the block, not whatever sounds at its first
        // tick. A subject with an upbeat — BWV 847's, for one — has nothing
        // sounding at the bar line it is annotated to, and reading the pitch
        // there silently drops every entry, which is what this did.
        let first = p.voices[*voice]
          .notes
          .iter()
          .find(|n| n.attack && n.onset >= b.at && n.onset < b.at + b.len)?;
        Some((*voice, b.at, answer::degree(first.pitch, d.tonic)))
      }
      _ => None,
    })
    .collect();
  let last = length(blocks);
  crate::form::plan_of(p, &entries, &[(last, "I:PAC".into())], subject_bars(d))
}


// ------------------------------------------------- everything, in one call ---

/// A finished fugue and every judgement this repository can pass on it.
///
/// One struct rather than five return values, because the thing a caller does
/// after generating is always the same: show the plan, show the notes, and say
/// what is wrong with them. A user interface wants all of that to repaint one
/// panel; a driver wants it to print one table. Neither wants to remember to run
/// the checker afterwards, and a result that can be displayed without being
/// checked is one that will be.
pub struct Outcome {
  /// The derivation, block by block — what a plan view draws.
  pub blocks: Vec<Block>,
  /// The notes, one voice per part, in the order [`Design::compass`] gives.
  pub voices: Vec<Voice>,
  /// Where the fill had to give something up, and which blocks.
  pub relaxed: Relaxed,
  /// Whether the result parses under the grammar it was derived from — §8.15's
  /// own parser, turned on the generator. It caught this generator dropping
  /// every entry when the subject had an upbeat.
  pub verdict: crate::form::Verdict,
  /// Rule firings over the whole piece, by §8.2's checker, which does not know
  /// it is looking at generated music.
  pub tally: crate::corpus::Tally,
  pub bars: i64,
  /// Wall clock for the fill, which a caller iterating wants to see.
  pub seconds: f64,
}

impl Outcome {
  /// Firings of the rules in `tier`, per thousand slices — the figure §8.16
  /// compares against Bach's 112.3 on the full five.
  pub fn per_thousand(&self, tier: &[Rule]) -> f64 {
    let n: usize = tier.iter().map(|r| self.tally.by_rule.get(r.name()).copied().unwrap_or(0)).sum();
    1000.0 * n as f64 / self.tally.slices.max(1) as f64
  }
  /// Whether the piece breaks any rule of `tier`.
  pub fn clean_on(&self, tier: &[Rule]) -> bool {
    tier.iter().all(|r| self.tally.by_rule.get(r.name()).copied().unwrap_or(0) == 0)
  }
}

/// Compose, and judge what was composed.
///
/// The call a caller outside this crate should reach for. [`generate`] returns
/// the notes; this adds every check §8 can make and the timings, so that a
/// result cannot be shown without also being able to say what is wrong with it.
///
/// `tier` is what the **search** must obey. [§8.16](../readme.md) is the section
/// arguing that this is not the tier [§8.2](../readme.md) endorses for
/// *describing* the repertoire: a generator on `CONFIRMED + melodic` writes
/// dissonance at 366 per thousand and a listener calls it cacophony, and one on
/// the full five writes 69, below Bach's own 112. A rule can be wrong as a
/// description and right as a constraint.
pub fn fugue(d: &Design, l: &Layout, tier: &[Rule], seed: u64) -> Result<Outcome, String> {
  let t0 = std::time::Instant::now();
  let (blocks, voices, relaxed) = generate(d, l, tier, seed)?;
  let seconds = t0.elapsed().as_secs_f64();

  let piece = Piece {
    id: "generated".into(),
    voices: voices.clone(),
    measure: d.measure,
    beat: d.beat,
    key: d.key,
    tonic: None,
    polyphonic_instants: 0,
  };
  let verdict = crate::form::parse(&as_plan(d, &blocks, &piece));

  let mut tally = crate::corpus::Tally::default();
  for a in 0..voices.len() {
    for b in a + 1..voices.len() {
      tally.merge(&crate::corpus::check_voices(&voices[a], &voices[b], d.measure));
    }
  }
  for v in &voices {
    crate::corpus::check_melody(v, &mut tally);
  }

  Ok(Outcome {
    bars: length(&blocks) / d.measure.max(1),
    blocks,
    voices,
    relaxed,
    verdict,
    tally,
    seconds,
  })
}

/// Write an outcome as MIDI, tracks top voice first and named by their role.
pub fn write(o: &Outcome, d: &Design, path: &std::path::Path, qpm: u32) -> std::io::Result<()> {
  let roles: Vec<String> = (0..o.voices.len())
    .map(|v| {
      let entries = o
        .blocks
        .iter()
        .filter(|b| matches!(&b.kind, Kind::Entry { voice, .. } if *voice == v))
        .count();
      format!("- {entries} subject entries")
    })
    .collect();
  crate::midi::write_score(path, &o.voices, &roles, qpm, crate::kern::meter_of(d.measure, d.beat))
}
#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    automaton::{CONFIRMED, CONF_MEL},
    corpus,
    kern::TICKS_PER_QUARTER as Q,
  };

  /// A subject of one bar in C major, three voices well apart — and beginning
  /// on an **upbeat**, like BWV 847's, because the upbeat is what made the whole
  /// texture fall silent together and a fixture without one cannot catch it.
  fn design() -> Design {
    Design {
      subject: Voice {
        notes: (0..3)
          .map(|i| Note {
            onset: Q + i * Q,
            dur: Q,
            pitch: Pitch::new(28 + [0, 2, 4][i as usize], 0),
            attack: true,
          })
          .collect(),
      },
      voices: 3,
      key: [0; 7],
      tonic: 0,
      measure: 4 * Q,
      beat: Q,
      compass: vec![(33, 42), (28, 37), (21, 30)],
    }
  }

  fn piece_of(d: &Design, voices: &[Voice]) -> Piece {
    Piece {
      id: "t".into(),
      voices: voices.to_vec(),
      measure: d.measure,
      beat: d.beat,
      key: d.key,
      tonic: Some((0, false)),
      polyphonic_instants: 0,
    }
  }

  /// The derivation has the shape §8.15 measured: one entry per voice with a
  /// link in the exposition, three middle groups, and a close at home.
  #[test]
  fn the_derivation_has_the_shape_the_book_has() {
    let d = design();
    let b = derive(&d, &Layout::default());
    let entries = b.iter().filter(|x| matches!(x.kind, Kind::Entry { .. })).count();
    let episodes = b.iter().filter(|x| matches!(x.kind, Kind::Episode { .. })).count();
    assert_eq!(entries, 3 + 3 + 1, "three in the exposition, three middles, one final");
    assert_eq!(episodes, 1 + 3 + 1, "the expositional link, three middles, the final one");
    assert_eq!(b.last().unwrap().key_of, 0, "it must end at home");
    for w in b.windows(2) {
      assert_eq!(w[0].at + w[0].len, w[1].at, "blocks must abut, with no gap and no overlap");
    }
  }

  /// An episode's placed voice must actually be a sequence — a restatement at a
  /// different pitch — or §2.4's `Sequence(motive, …)` is not what was written.
  #[test]
  fn an_episode_is_a_sequence() {
    let d = design();
    let v = sequence(&d, 0, 3 * d.measure, 0);
    let steps: Vec<i16> = v.notes.iter().map(|n| n.pitch.step).collect();
    // however many notes the subject puts in its first bar
    let bar = d.subject.notes.iter().filter(|n| n.onset < d.measure).count();
    assert!(bar >= 2 && steps.len() >= 2 * bar, "{steps:?}");
    for i in 0..bar {
      assert_eq!(steps[i + bar], steps[i] - 1, "bar two must be bar one a step lower");
    }
  }

  /// The generated fugue must satisfy the grammar it was derived from, read back
  /// by §8.15's own parser — the check that caught this generator dropping every
  /// entry when the subject had an upbeat.
  #[test]
  fn what_is_generated_parses_under_the_grammar_it_came_from() {
    let d = design();
    let (blocks, voices, _) = generate(&d, &Layout::default(), CONFIRMED, 0x5EED).expect("a fugue");
    let piece = piece_of(&d, &voices);
    let v = crate::form::parse(&as_plan(&d, &blocks, &piece));
    assert!(v.exposition_covers_the_voices, "{v:?}");
    assert!(v.exposition_alternates, "{v:?}");
    assert!(v.has_a_middle, "{v:?}");
    assert!(v.ends_at_home, "{v:?}");
    assert!(!v.exposition_is_unbroken, "the link is written on purpose");
  }

  /// One voice's notes in `[t0, t1)`, for checking a block on its own.
  fn window(v: &Voice, t0: i64, t1: i64) -> Voice {
    Voice {
      notes: v
        .notes
        .iter()
        .filter(|n| n.onset >= t0 && n.onset < t1)
        .map(|n| Note { onset: n.onset - t0, ..*n })
        .collect(),
    }
  }

  /// Count what the checker flags across a set of voices.
  fn flagged(voices: &[Voice], measure: i64, rules: &[Rule]) -> usize {
    let mut t = corpus::Tally::default();
    for a in 0..voices.len() {
      for b in a + 1..voices.len() {
        t.merge(&corpus::check_voices(&voices[a], &voices[b], measure));
      }
    }
    rules.iter().map(|r| t.by_rule.get(r.name()).copied().unwrap_or(0)).sum()
  }

  /// **The search must not emit counterpoint its own checker flags — inside a
  /// block.** §8.6's first test, at the scale this generator works at.
  ///
  /// Inside a block, and not across the whole piece, because those are different
  /// claims and only the first is one the search can make. The fill runs one
  /// block at a time, so the automaton's state resets at every seam:
  /// [`Problem::prior`] carries the previous slice's pitches across, which is
  /// enough for the melodic rules and for motion, and the *obligation* state
  /// does not survive. A violation that straddles a seam is therefore a known
  /// cost of filling block by block rather than a fault in the search, and the
  /// test that would hide the distinction is the one that checks the whole piece
  /// and asserts zero.
  ///
  /// What the seam costs is measured in §8.16 rather than asserted away here.
  #[test]
  fn no_block_contains_counterpoint_the_checker_flags() {
    let d = design();
    let (blocks, voices, _) = generate(&d, &Layout::default(), CONFIRMED, 0x5EED).expect("a fugue");
    assert!(
      flagged(&voices, d.measure, CONFIRMED) < 5,
      "the seams should cost a violation or two, not a piece full of them"
    );
    for b in &blocks {
      let inside: Vec<Voice> =
        voices.iter().map(|v| window(v, b.at, b.at + b.len)).collect();
      assert_eq!(
        flagged(&inside, d.measure, CONFIRMED),
        0,
        "the block at bar {} contains counterpoint the search itself would refuse",
        b.at / d.measure + 1
      );
    }
  }

  /// **The whole texture must never fall silent.** A listener heard this before
  /// any test did: four tenths of a second of nothing, every six seconds.
  ///
  /// The cause was that every free voice carried the subject's rhythm *with its
  /// onsets*, and BWV 847's subject begins on an upbeat — so every voice
  /// inherited the same gap at the head of every tile and they all rested
  /// together. Laying the note values end to end removes the gap; a per-voice
  /// phase keeps the boundaries from lining up either. This is the test that
  /// says so, and it is stated as a property of the piece rather than of the
  /// rhythm function, so any future way of inventing rhythm has to satisfy it
  /// too.
  #[test]
  fn the_texture_never_falls_silent() {
    let d = design();
    let (blocks, voices, _) = generate(&d, &Layout::default(), CONF_MEL, 0x5EED).expect("a fugue");
    let end = length(&blocks);
    let step = d.beat / 4;
    let mut worst = 0i64;
    let mut run = 0i64;
    let mut t = 0;
    while t < end {
      let sounding = voices
        .iter()
        .any(|v| v.notes.iter().any(|n| n.onset <= t && t < n.onset + n.dur));
      run = if sounding { 0 } else { run + step };
      worst = worst.max(run);
      t += step;
    }
    assert_eq!(
      worst, 0,
      "the whole texture rests for {worst} ticks — {:.2} of a beat",
      worst as f64 / d.beat as f64
    );
  }

  /// Every voice must sound through every block, or what was written is not in
  /// three parts however many staves it has.
  #[test]
  fn every_voice_sounds_in_every_block() {
    let d = design();
    let (blocks, voices, _) = generate(&d, &Layout::default(), CONFIRMED, 0x5EED).expect("a fugue");
    for b in &blocks {
      for (i, v) in voices.iter().enumerate() {
        assert!(
          v.notes.iter().any(|n| n.onset >= b.at && n.onset < b.at + b.len),
          "voice {i} is silent through the block at bar {}",
          b.at / d.measure + 1
        );
      }
    }
  }

  /// The whole point of `prior`: **where the join is kept**, no voice leaps more
  /// than an octave across a block boundary.
  ///
  /// Where it is not kept the voice enters cold and may legally leap a tenth,
  /// because there is nothing for the melodic rule to measure the leap from —
  /// which is why [`Relaxed::cold`] records which blocks those are rather than
  /// only how many. A test that skipped the distinction would either fail on a
  /// cost the generator already reports, or pass by not looking.
  #[test]
  fn the_join_keeps_the_free_voices_from_leaping_between_blocks() {
    let d = design();
    let (blocks, voices, relaxed) = generate(&d, &Layout::default(), CONF_MEL, 0x5EED).expect("a fugue");
    assert!(relaxed.without_plan == 0, "the plan should not have needed dropping here");
    assert!(relaxed.cold.len() < blocks.len() / 2, "most joins should hold");
    for (bi, b) in blocks.iter().enumerate().skip(1) {
      if relaxed.cold.contains(&bi) {
        continue;
      }
      for v in voices.iter() {
        let before = v.notes.iter().filter(|n| n.onset < b.at).max_by_key(|n| n.onset);
        let after = v.notes.iter().find(|n| n.onset >= b.at);
        if let (Some(x), Some(y)) = (before, after) {
          // a voice that rested into the block entered cold and had nothing to
          // leap from, which is the whole point of resting it
          if y.onset - (x.onset + x.dur) >= d.measure / 2 {
            continue;
          }
          assert!(
            (y.pitch.step - x.pitch.step).abs() <= 7,
            "voice {} leaps {} steps into bar {} ({} -> {}), block {:?}, gap {}",
            voices.iter().position(|z| std::ptr::eq(z, v)).unwrap_or(9),
            (y.pitch.step - x.pitch.step).abs(),
            b.at / d.measure + 1,
            x.pitch.name(),
            y.pitch.name(),
            b.kind,
            y.onset - (x.onset + x.dur)
          );
        }
      }
    }
  }
}
