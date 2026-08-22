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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
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

/// A block **as somebody authored it** — `docs/ui-spec.md` section 4.5's palette.
///
/// Not a [`Block`], and the difference is the point: **`at` is not here.** Where
/// a block sits is derived by laying the list end to end from tick zero, so a
/// gap and an overlap are not things a palette can express — the same guarantee
/// the parameter path gets for free from accumulating, kept rather than replaced
/// by a validator that would have to be remembered.
///
/// Nor is an entry's length, which is the subject's and is not a choice. Every
/// field here is one somebody can mean.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Built {
  /// A statement of the subject, as long as the subject is.
  Entry { voice: usize, shift: i16, tonal: bool, key_of: i16 },
  /// An episode of `bars` bars, its motive in `voice`.
  Episode { voice: usize, shift: i16, key_of: i16, bars: i64 },
}

impl Built {
  /// What this becomes at `at`, given the design that says how long a subject is.
  fn block(&self, d: &Design, at: i64) -> Block {
    match *self {
      Built::Entry { voice, shift, tonal, key_of } => {
        Block { at, len: subject_bars(d), kind: Kind::Entry { voice, shift, tonal }, key_of }
      }
      Built::Episode { voice, shift, key_of, bars } => {
        Block { at, len: bars.max(1) * d.measure.max(1), kind: Kind::Episode { voice, shift }, key_of }
      }
    }
  }

  /// The lane it is in, which a palette authors directly.
  pub fn voice(&self) -> usize {
    match *self {
      Built::Entry { voice, .. } | Built::Episode { voice, .. } => voice,
    }
  }
}

/// One block of the derivation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
  /// A subject entry in one voice, `shift` diatonic steps from the subject as
  /// given. `tonal` marks the comes, whose first note follows §8.11's Rule I
  /// rather than the shift.
  Entry { voice: usize, shift: i16, tonal: bool },
  /// A stretch with no subject in it, whose motive is placed in `voice`.
  Episode { voice: usize, shift: i16 },
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

/// A block's **identity**, for seeding — what it is, not where it sits.
///
/// The seed used to be the block's index, which is fine for one generate and
/// wrong for editing. Inserting a middle shifts every later index, so every
/// later block would reseed and the whole piece would change underneath an edit
/// that should have been local. Keyed on the block's own description instead,
/// an untouched block keeps its notes when something before it grows.
///
/// `nth` distinguishes blocks that are otherwise identical — two episodes in
/// the same key — and counts only among *those*, so it does not move when an
/// unrelated block is inserted.
///
/// **Taken from the chain, before [`Layout::turns`] is applied** — which is what
/// [`identities_of`] is for, and why it is the only way to ask.
///
/// A turn rotates the lanes of every block from one point on. The lane is part
/// of a block's description, so if identity were read off the *derived* blocks a
/// turn would move every identity behind it: the tail would reseed, and moving a
/// block to another lane would rewrite its notes twice over — once because the
/// lane really did change, and once for no reason anybody asked for. Reading it
/// off the chain instead leaves the lane in the hash and takes the turn out of
/// it, which keeps both properties and changes no seed that existed before.
fn ident(b: &Block, nth: usize) -> u64 {
  let (tag, voice, shift, tonal) = match &b.kind {
    Kind::Entry { voice, shift, tonal } => (1u64, *voice as u64, *shift as i64 as u64, *tonal as u64),
    Kind::Episode { voice, shift } => (2u64, *voice as u64, *shift as i64 as u64, 0),
  };
  let mut h = 0xcbf2_9ce4_8422_2325u64; // FNV-1a, which is enough to spread a seed
  for x in [tag, voice, shift, tonal, b.key_of as i64 as u64, b.len as u64, nth as u64] {
    h ^= x;
    h = h.wrapping_mul(0x0000_0100_0000_01b3);
  }
  h
}

/// Each block's identity, one per block, in order. Private, because the blocks a
/// caller holds are the *derived* ones and asking with those would be wrong the
/// moment a turn is set — [`identities_of`] is the way to ask.
fn identities(blocks: &[Block]) -> Vec<u64> {
  let mut seen: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
  blocks
    .iter()
    .map(|b| {
      let bare = ident(b, 0);
      let nth = seen.entry(bare).or_insert(0);
      let id = ident(b, *nth);
      *nth += 1;
      id
    })
    .collect()
}

/// Per-block seeds, stable under insertion elsewhere in the piece.
///
/// `rerolls` nudges individual blocks — [`Layout::rerolls`], keyed on the same
/// identity. A block that has been asked for again draws differently and every
/// other block draws exactly as before, which is what makes *this bar, again* a
/// local operation rather than a new fugue.
/// Each block's identity, one per block, in order.
///
/// The key a caller names a block by when it wants *that* block written again.
/// An index would not do: it moves when anything before it is inserted, and an
/// editor's whole business is inserting things.
///
/// **Taken from a design and a layout rather than from blocks**, so that it is
/// the chain's identities and not the turned ones — see [`ident`]. There is no
/// way to ask the other question, which is the point: a caller holding an
/// `Outcome` would naturally pass its blocks, and would get answers that moved
/// whenever a lane did.
pub fn identities_of(d: &Design, l: &Layout) -> Vec<u64> {
  identities(&chain(d, l))
}

/// Each block's seed: the base, its identity, and how many times it has been
/// asked for again.
pub fn seeds(d: &Design, l: &Layout, base: u64) -> Vec<u64> {
  seeds_from(&identities_of(d, l), base, &l.rerolls)
}

fn seeds_from(ids: &[u64], base: u64, rerolls: &[(u64, u32)]) -> Vec<u64> {
  ids
    .iter()
    .copied()
    .map(|id| {
      let n = rerolls.iter().find(|(k, _)| *k == id).map_or(0, |(_, n)| *n);
      // The golden-ratio odd constant is only there to move the nudge into the
      // high bits before it meets the identity; the sampler's SplitMix does the
      // spreading.
      base ^ id ^ (n as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
    })
    .collect()
}

/// Which voice a block places its line in.
fn held_of(b: Option<&Block>) -> usize {
  match b.map(|x| &x.kind) {
    Some(Kind::Entry { voice, .. }) | Some(Kind::Episode { voice, .. }) => *voice,
    None => 0,
  }
}

/// Which voice a block places its line in, by reference.
fn held_of_ref(b: &Block) -> usize {
  held_of(Some(b))
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
  /// Blocks that have been asked for again, and how many times.
  ///
  /// Keyed on [`identities`] rather than on an index, so a reroll survives an
  /// edit that inserts something before it — and so a block that is removed and
  /// comes back the same block comes back with the same notes, which is the
  /// behaviour anybody would expect and an index could not give.
  ///
  /// It lives in the layout rather than beside the seed because it *is* a
  /// parameter of the piece: without it in the settings file, a rerolled block
  /// would not survive a save, and `docs/ui-spec.md` section 8 promises that the
  /// same file gives the same fugue. Entries whose block no longer exists are
  /// kept, cost nothing, and are what makes coming back work.
  #[cfg_attr(feature = "serde", serde(default))]
  pub rerolls: Vec<(u64, u32)>,
  /// Rotations of the voice chain, each applied **from one block to the end**.
  ///
  /// The one thing about a block that `derive` decides and nothing could ask it
  /// to decide differently, until this. `ui-spec.md` section 4.3 wanted a block
  /// draggable to another lane and found there was no parameter for it — a
  /// block's voice comes from its predecessor's, so it is not independently
  /// settable and never will be without changing what the generator writes.
  ///
  /// A rotation is the honest thing that *is* settable. Dragging a block down a
  /// lane rotates it and everything after it by the same amount, which is
  /// exactly the knock-on 4.3 says to show rather than hide, and it leaves the
  /// chain rule the derivation is built on entirely intact — every step after
  /// the turn is still one lane on from the last.
  ///
  /// Keyed on [`identities`] like [`Layout::rerolls`], and for the same reason:
  /// a turn belongs to a place in the journey, not to an index that moves when
  /// something is inserted before it. Identity does not include the lane, so a
  /// turn does not change the key it is looked up by — see [`ident`].
  ///
  /// Legal on the first block, where it rotates the whole piece, and on anything
  /// after the exposition. Not inside the exposition, where the entries are
  /// one per voice by construction and a rotation of part of them would state
  /// the subject twice in one voice and never in another — [`turnable`].
  #[cfg_attr(feature = "serde", serde(default))]
  pub turns: Vec<(u64, i16)>,
  /// Voices that say nothing in a block, **beyond the ones the grammar already
  /// rests** — one entry per block that has any, naming the voices by index.
  ///
  /// [`resting`] rests a voice until it has entered, which costs no parameter
  /// because the derivation already says who enters when. That rule empties
  /// itself at the end of the exposition, and this is everything after: a voice
  /// silent again once it has been heard, which is a *choice* and so is a field.
  ///
  /// It is what reaches four voices. readme §8.17 measured the exact search's
  /// wall against the number of voices it must choose rather than the number
  /// sounding — four voices with one resting costs what three voices costs, to
  /// the state — and the grammar's rule cannot supply that, because by the fourth
  /// entry every voice has entered and none may rest again.
  ///
  /// Keyed on [`identities_of`] like [`Layout::rerolls`] and [`Layout::turns`],
  /// so a rest belongs to a place in the journey and not to an index that moves
  /// when something is inserted before it. The block's own held voice may not be
  /// named: a block is a line placed in a voice, and silencing that line is not a
  /// texture but an empty block.
  #[cfg_attr(feature = "serde", serde(default))]
  pub rests: Vec<(u64, Vec<usize>)>,
  /// Whether the **search** decides who else rests, on top of whatever
  /// [`Layout::rests`] pins. **Off**, and the reason it is a flag rather than the
  /// behaviour is that nothing has measured whether a drawn texture is one.
  ///
  /// What it does invents no rule, which is the whole of why this shape and not
  /// another. Every rest pattern a block could legally take is a search of its
  /// own, `realise::fill` counts its legal set exactly, and a pattern drawn with
  /// probability proportional to that count — then a fill drawn uniformly inside
  /// it — is a fill drawn uniformly from the union of all of them. So the texture
  /// comes out of the same mechanism the notes do, and
  /// [§8.10](../readme.md)'s finding that drawing beats optimising covers it.
  ///
  /// It replaces [`rests_that_fit`], which rests whichever voice has gone longest
  /// since holding anything and makes no claim to be musical. What comes out here
  /// instead is *whichever absence leaves the counterpoint the most room*, which
  /// is a consequence of the arithmetic rather than a preference anybody typed.
  ///
  /// A `Layout` field and not interface state, because it changes what is
  /// generated and `docs/ui-spec.md` section 8 requires a saved fugue to reproduce.
  #[cfg_attr(feature = "serde", serde(default))]
  pub texture: Texture,
  /// A plan **authored block by block**, instead of derived from the fields
  /// above — section 4.5.
  ///
  /// When this is set, [`middles`](Layout::middles), `episode_bars`, `link` and
  /// `close_at_home` say nothing: they are the parameters of a *generator* of
  /// plans, and this is a plan. `derive` returns these, laid end to end.
  ///
  /// **Nothing validates that it is a fugue, and that is deliberate.**
  /// `form::parse` already judges any plan against §2.4's grammar and returns
  /// five independent verdicts, and those are what the generated pieces are
  /// scored against. So a palette lets somebody build whatever they like and the
  /// verdict says which of the five things they built — a palette that permitted
  /// only legal plans would teach nothing, because everything it allowed would
  /// already be legal.
  #[cfg_attr(feature = "serde", serde(default))]
  pub built: Option<Vec<Built>>,
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
      rerolls: vec![],
      turns: vec![],
      rests: vec![],
      texture: Texture::Given,
      built: None,
    }
  }
}

/// Derive a plan from a design and a layout.
///
/// The chain [`chain`] builds, with [`Layout::turns`] applied to it. Splitting
/// the two is what lets a turn move a block's lane without moving its identity:
/// [`identities`] is the same either side of this, because the lane is not part
/// of what a block is.
pub fn derive(d: &Design, l: &Layout) -> Vec<Block> {
  let mut out = chain(d, l);
  if l.turns.is_empty() {
    return out;
  }
  let ids = identities(&out);
  let mut by = 0i16;
  for (i, b) in out.iter_mut().enumerate() {
    // A turn applies from its own block onward, so the rotations accumulate as
    // the walk goes forward and two turns at the same place add up.
    by += l.turns.iter().filter(|(k, _)| *k == ids[i]).map(|(_, t)| *t).sum::<i16>();
    if by.rem_euclid(d.voices.max(1) as i16) == 0 {
      continue;
    }
    let n = d.voices.max(1) as i16;
    let moved = |v: &usize| (*v as i16 + by).rem_euclid(n) as usize;
    b.kind = match &b.kind {
      Kind::Entry { voice, shift, tonal } => Kind::Entry { voice: moved(voice), shift: *shift, tonal: *tonal },
      Kind::Episode { voice, shift } => Kind::Episode { voice: moved(voice), shift: *shift },
    };
  }
  out
}

/// Whether block `bi` may carry a turn, and why not when it may not.
///
/// The exposition states the subject **once in each voice**, top down, and those
/// lanes are written by [`chain`] rather than chained from a predecessor. A
/// rotation of part of that run is therefore not a rearrangement of it but a
/// destruction: some voice gets two entries and some gets none. Turning at the
/// very first block rotates the whole run together and is fine.
///
/// Everything after the exposition is chained, so a rotation of a tail of it is
/// still a chain, one lane on at every step.
pub fn turnable(d: &Design, l: &Layout, bi: usize) -> Result<(), String> {
  if bi == 0 {
    return Ok(());
  }
  match origins(d, l).get(bi) {
    None => Err(format!("no block {bi}")),
    Some(Origin::Built) => Err(
      "a built plan sets its own lanes; move the block rather than rotating the chain into it".into(),
    ),
    Some(Origin::Exposition(_)) | Some(Origin::Link) => Err(
      "the exposition states the subject once in each voice; turning part of it would state it        twice in one and never in another"
        .into(),
    ),
    Some(_) => Ok(()),
  }
}

/// Refuse a layout whose turns are attached where [`turnable`] says they cannot
/// be. Checked once, where the plan is derived, rather than at every use.
fn check_turns(d: &Design, l: &Layout) -> Result<(), String> {
  if l.turns.is_empty() {
    return Ok(());
  }
  let ids = identities_of(d, l);
  for (k, by) in &l.turns {
    let Some(bi) = ids.iter().position(|id| id == k) else {
      // A turn whose block is gone is kept and costs nothing, exactly as a
      // reroll's is — it is what makes an undone edit come back the same.
      continue;
    };
    if *by != 0 {
      turnable(d, l, bi).map_err(|e| format!("a turn on block {bi}: {e}"))?;
    }
  }
  Ok(())
}

fn chain(d: &Design, l: &Layout) -> Vec<Block> {
  // An authored plan is not derived from anything: it is laid end to end from
  // tick zero, which is where the tiling guarantee comes from rather than from
  // a check somebody has to remember to run.
  if let Some(built) = &l.built {
    let mut at = 0i64;
    return built
      .iter()
      .map(|b| {
        let block = b.block(d, at);
        at += block.len;
        block
      })
      .collect();
  }
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

/// One block, filled — **the only place a block is written**.
///
/// It exists because there were two such places and they had drifted apart in
/// four ways: the relaxation ladder, the terminal pin, how the prior for the
/// next block is taken, and whether a voice about to enter is given one at all.
/// Generating a piece and refilling part of one then disagreed about what a
/// block *is*, which is how a saved fugue stopped reproducing.
///
/// The block's notes at tick zero, the prior for whatever follows, which attempt
/// succeeded — nought if nothing had to be relaxed — and the peak live states
/// the search reached, which is what §8.17 measures the voice count against.
type Written = (Vec<Voice>, Vec<Option<Pitch>>, usize, usize);

/// Returns [`Written`].
#[allow(clippy::too_many_arguments)]
fn fill_block(
  d: &Design,
  blocks: &[Block],
  bi: usize,
  plan: &[harmony::Segment],
  tier: &[Rule],
  seed: u64,
  prior: &[Option<Pitch>],
  terminal: &[Option<Pitch>],
  silent: &[bool],
  texture: Texture,
) -> Result<Written, String> {
  let b = &blocks[bi];
  let held = held_of(Some(b));
  // Which voice takes the *next* block's placed line. A voice about to state
  // the subject drops out before it does — which is what Bach writes, and which
  // is also the only way to stop it entering by a leap of an eleventh from
  // wherever its accompanying line happened to end. The entry's first note is
  // placed by the derivation, so no amount of care in the fill can reach it; the
  // fix is to have nothing there to reach from. A test found this, at bar 3, in
  // the first fugue this generator produced.
  let next = blocks.get(bi + 1).map(held_of_ref);
  // a bar of rest where the block can spare one, half the block where it cannot
  // — a one-bar subject has no bar to give
  let quiet = d.measure.min(b.len / 2);
  let line = match &b.kind {
    Kind::Entry { shift, tonal, .. } => state(d, 0, *shift, *tonal),
    Kind::Episode { shift, .. } => sequence(d, 0, b.len, *shift),
  };
  // every voice gets notes: the held one its placed line, the rest the
  // subject's rhythm, whose pitches the search discards
  // A voice `silent` names says nothing for the whole block. It is not free and
  // it is not fixed-with-notes: it is absent, and `realise::fill` returns it
  // unchanged, which for an empty voice is empty.
  //
  // The held voice can never be one of them — a block whose subject nobody
  // states is not a block.
  let quiet_voice = |v: usize| v != held && silent.get(v).copied().unwrap_or(false);
  let voices: Vec<Voice> = (0..d.voices)
    .map(|v| {
      if v == held {
        line.clone()
      } else if quiet_voice(v) {
        Voice { notes: vec![] }
      } else if Some(v) == next && quiet > 0 {
        Voice { notes: rhythm(d, 0, b.len - quiet, v) }
      } else {
        Voice { notes: rhythm(d, 0, b.len, v) }
      }
    })
    .collect();
  if voices.iter().enumerate().any(|(v, x)| x.notes.is_empty() && !quiet_voice(v)) {
    return Err(format!("block {bi} leaves a voice with no notes to place"));
  }
  let free: Vec<bool> = (0..d.voices).map(|v| v != held && !quiet_voice(v)).collect();

  // The prior for whatever follows: each voice's last pitch, except where the
  // voice is meant to arrive cold. Written once because both the ordinary path
  // and the one below need it and they must not drift.
  let joins = |written: &[Voice]| -> Vec<Option<Pitch>> {
    (0..d.voices)
      .map(|v| {
        // a voice that rested before entering enters cold, which is the point —
        // and one that said nothing at all has nothing to join to
        if (Some(v) == next && quiet > 0) || quiet_voice(v) {
          return None;
        }
        written.get(v).and_then(|f: &Voice| f.notes.iter().max_by_key(|n| n.onset)).map(|n| n.pitch)
      })
      .collect()
  };

  // **A block with nothing free is already written.** The exposition's first
  // block is one voice alone, and a single line needs no counterpoint —
  // `realise::fill` refuses a problem with no free voice in it, and is right to.
  if !free.iter().any(|f| *f) {
    let after = joins(&voices);
    return Ok((voices, after, 0, 0));
  }
  let here: Vec<harmony::Segment> = plan
    .iter()
    .filter(|s| s.start >= b.at && s.start < b.at + b.len)
    .map(|s| harmony::Segment { start: s.start - b.at, end: s.end - b.at, ..s.clone() })
    .collect();

  // **Which rest patterns this block may take.** One, ordinarily: whatever the
  // grammar and the layout already said. With `Layout::drawn_texture` it is every
  // subset of the voices still free that leaves no more than [`FREE_WALL`] of
  // them — including the empty one, so the fullest texture is a candidate like
  // any other and is not privileged by being the default.
  let loose: Vec<usize> = (0..d.voices).filter(|v| free[*v]).collect();
  let patterns: Vec<Vec<bool>> = if texture == Texture::Given {
    vec![free.clone()]
  } else {
    (0..1u32 << loose.len())
      .filter_map(|mask| {
        let resting: Vec<usize> =
          loose.iter().enumerate().filter(|(i, _)| mask >> i & 1 == 1).map(|(_, v)| *v).collect();
        (loose.len() - resting.len() <= FREE_WALL).then(|| {
          (0..d.voices).map(|v| free[v] && !resting.contains(&v)).collect()
        })
      })
      .collect()
  };

  let mut got: Option<(realise::Solution, usize)> = None;
  let mut why = String::new();
  for attempt in 0..3 {
    // Every pattern at this rung before relaxing further, so that a texture is
    // never bought with a constraint that did not have to be dropped.
    let mut round: Vec<(realise::Solution, Vec<bool>)> = vec![];
    for pattern in &patterns {
      if !pattern.iter().any(|f| *f) {
        continue; // nothing free: `fill` refuses that, and the caller above handles it
      }
      let pr = Problem {
        voices: voices
          .iter()
          .enumerate()
          .map(|(v, line)| if v != held && free[v] && !pattern[v] { Voice { notes: vec![] } } else { line.clone() })
          .collect(),
        free: pattern.clone(),
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
        prior: if attempt == 0 { prior.to_vec() } else { vec![] },
        terminal: terminal.to_vec(),
        samples: 0,
        seed,
        beta: 0.0,
      }
      .drawing();
      match realise::fill(&pr) {
        Ok(s) => round.push((s, pattern.clone())),
        Err(e) => why = e,
      }
    }
    // The chosen pattern needs no carrying: `fill` returns a rested voice
    // unchanged, and unchanged for a voice with no notes is no notes — so the
    // solution already says who said nothing, and `joins` reads it off that.
    if let Some((sol, _)) = pick_pattern(round, seed, texture) {
      got = Some((sol, attempt));
      break;
    }
  }
  let Some((sol, attempt)) = got else {
    // say what was being asked for, not only that it failed: a refusal whose
    // context is invisible is a refusal nobody can act on
    let lo = line.notes.iter().map(|n| n.pitch.step).min().unwrap_or(0);
    let hi = line.notes.iter().map(|n| n.pitch.step).max().unwrap_or(0);
    let ranges: Vec<String> = (0..d.voices)
      .filter(|v| free[*v])
      .map(|v| format!("v{v} {}..{}", d.compass[v].0, d.compass[v].1))
      .collect();
    // And when the reason is the free count, say the remedy. §8.17 measured the
    // wall against the voices the search must choose, so a block with more than
    // `FREE_WALL` of them has a fix that is not "use fewer voices" — it is to
    // rest one, which is what `Layout::rests` is for. A refusal that names its
    // own cure is the difference between a limit and a dead end.
    let too_many = ranges.len() > FREE_WALL;
    return Err(format!(
      "block {bi} at bar {}: {why}
     held voice {held} spans {lo}..{hi}; free voices {}{}",
      b.at / d.measure + 1,
      ranges.join(", "),
      if too_many {
        format!(
          "\n     {} voices are free here and the exact search can choose {FREE_WALL}. \
           Rest one in this block — `Layout::rests`, or `compose::rests_that_fit` for a pattern \
           that fits the whole piece.",
          ranges.len()
        )
      } else {
        String::new()
      }
    ));
  };

  let chosen = sol.chosen();
  let after = joins(chosen);
  Ok((chosen.to_vec(), after, attempt, sol.peak_states))
}

/// Pick one of the block's rest patterns, **weighted by how much music each
/// admits** — and the weighting is what makes this a uniform draw rather than a
/// preference.
///
/// `realise::fill` returns an exact count of the fills a pattern allows and a
/// sample drawn uniformly from among them. Take pattern `i` with probability
/// `n_i / Σn` and then its own sample, and every fill in the union comes up with
/// probability `(n_i/Σn)·(1/n_i) = 1/Σn`. So the texture is drawn from the same
/// legal set the notes are, by the same argument
/// [§8.10](../readme.md) makes for drawing over optimising, and **nothing here
/// prefers one texture to another**.
///
/// What emerges is not nothing, though, and it is worth naming because it looks
/// like a choice and is not: the fullest texture wins most of the time, because a
/// pattern with one more free voice admits orders of magnitude more fills. When
/// something has to rest, the voice that goes is whichever one's absence leaves
/// the counterpoint the most room. Both fall out of the arithmetic.
fn pick_pattern(
  round: Vec<(realise::Solution, Vec<bool>)>,
  seed: u64,
  texture: Texture,
) -> Option<(realise::Solution, Vec<bool>)> {
  if round.len() <= 1 {
    return round.into_iter().next();
  }
  // **The thinnest that fills**, which is a preference and behaves like one:
  // fewest free voices wins, and fewest is none but the subject. §8.18 measures
  // where that ends up rather than arguing about it.
  if texture == Texture::Thinnest {
    return round.into_iter().min_by_key(|(_, p)| p.iter().filter(|f| **f).count());
  }
  let total: u128 = round.iter().map(|(s, _)| s.legal_fills.max(1)).sum();
  // SplitMix64 on the block's own seed, so the choice is part of what the seed
  // reproduces and a reroll moves it along with everything else.
  let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
  z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
  z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
  z ^= z >> 31;
  let mut want = (z as u128) % total.max(1);
  for (sol, pattern) in round {
    let n = sol.legal_fills.max(1);
    if want < n {
      return Some((sol, pattern));
    }
    want -= n;
  }
  None
}

/// Which voices say nothing in each block: **a voice is silent until it has
/// stated the subject.**
///
/// The grammar's own rule, and the reason it is a rule rather than a parameter is
/// that the derivation already says who enters when — there is nothing here for
/// anybody to choose. Without it a three-voice fugue has three voices from bar
/// one, and an exposition, whose entire identity is voices arriving one at a
/// time, has no arrival in it.
///
/// **The held voice is never silent**, whatever else is true of it, which is not
/// a special case so much as the definition: a block is a line placed in a voice.
/// It matters at the link, whose motive `derive` hands to the voice that is
/// *about* to enter — so that voice sounds there before it has stated anything,
/// and the rule must not silence the one line the block exists to carry.
///
/// One entry per voice in the exposition means every voice has entered by the end
/// of it, so this empties itself: after the exposition nobody is resting and the
/// texture is as full as it ever was. Which is exactly as far as a rule can get,
/// and readme §8.17 measures where that leaves the voice count.
pub fn resting(d: &Design, l: &Layout) -> Vec<Vec<bool>> {
  let blocks = derive(d, l);
  let ids = identities_of(d, l);
  let voices = d.voices;
  let mut entered = vec![false; voices];
  blocks
    .iter()
    .enumerate()
    .map(|(bi, b)| {
      let held = held_of(Some(b));
      if matches!(b.kind, Kind::Entry { .. }) && held < voices {
        entered[held] = true;
      }
      // the grammar's rule, and then whatever the layout adds to it
      let named = ids.get(bi).and_then(|id| l.rests.iter().find(|(k, _)| k == id)).map(|(_, vs)| vs);
      (0..voices)
        .map(|v| v != held && (!entered[v] || named.is_some_and(|vs| vs.contains(&v))))
        .collect()
    })
    .collect()
}

/// How a block decides which voices rest.
///
/// Three, and **two of them are degenerate in opposite directions**, which is
/// [§8.18](../readme.md) and is the reason the third is a control rather than a
/// default.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Texture {
  /// Whatever the grammar rests and [`Layout::rests`] names, and nothing else.
  /// The default, and the only one that does what somebody asked for.
  #[default]
  Given,
  /// Drawn from the legal set: every pattern a block could take, weighted by how
  /// much music each admits. Invents no rule — [§8.10](../readme.md)'s argument
  /// for drawing over optimising, one level up — and
  /// [§8.17](../readme.md) measured what it returns, which is the **densest**
  /// legal texture essentially always.
  Drawn,
  /// The fewest voices that will still fill. [§8.18](../readme.md)'s control, and
  /// it collapses to the subject alone: a criterion that prefers thinness has a
  /// cheapest way to be satisfied and the search finds it, which is exactly what
  /// §8.10 found for `move by step`, `move against` and `state the harmony`.
  ///
  /// Kept runnable rather than deleted, because the figure it produces is the
  /// evidence for the claim above and a claim whose experiment has been removed
  /// is an opinion.
  Thinnest,
}

/// How many free voices the exact search can afford at once.
///
/// [§2.7](../readme.md) predicted four and [§8.6](../readme.md) measured **two**,
/// on the obligation set rather than the pitch product.
/// [§8.17](../readme.md) found the same number wherever it looked and found that
/// it is the *free* count it attaches to, not the voice count.
pub const FREE_WALL: usize = 2;

/// A rest pattern that keeps every block under [`FREE_WALL`], and **not a musical
/// choice**.
///
/// This is a feasibility helper, not a rule, and the distinction is the whole of
/// §1's constraint: nothing here is transcribed from anybody, and it does not
/// claim to be the texture a composer would write. What it claims is narrower —
/// that a voice count is *reachable*, and that somebody who asks for four voices
/// gets a piece instead of a refusal they cannot act on.
///
/// The voice it rests is the one that has gone longest since holding anything,
/// which is the plainest available answer and is chosen for being plain rather
/// than for being right. A caller who cares should edit the pattern; that is what
/// [`Layout::rests`] is a field for.
pub fn rests_that_fit(d: &Design, l: &Layout) -> Vec<(u64, Vec<usize>)> {
  let blocks = derive(d, l);
  let ids = identities_of(d, l);
  let already = resting(d, l);
  let mut out: Vec<(u64, Vec<usize>)> = vec![];
  // least recently held first, which is the order a rest is taken in
  let mut order: Vec<usize> = (0..d.voices).collect();
  for (bi, b) in blocks.iter().enumerate() {
    let held = held_of(Some(b));
    order.retain(|v| *v != held);
    order.push(held);
    let mut free: Vec<usize> = (0..d.voices).filter(|v| *v != held && !already[bi][*v]).collect();
    let mut rest = l.rests.iter().find(|(k, _)| Some(k) == ids.get(bi)).map_or(vec![], |(_, v)| v.clone());
    while free.len() > FREE_WALL {
      let Some(&oldest) = order.iter().find(|v| free.contains(v)) else { break };
      free.retain(|v| *v != oldest);
      rest.push(oldest);
    }
    if !rest.is_empty() {
      out.push((ids[bi], rest));
    }
  }
  out
}

/// Take a derived plan apart into an authored one — section 4.5's way in.
///
/// A palette that starts empty is a palette nobody uses: the interesting thing
/// to do to a fugue's plan is change it, and the interesting thing to change is
/// one this program wrote. So the parameters produce a plan and this turns that
/// plan into blocks, after which the parameters have nothing more to say.
///
/// It is exact. `derive(d, &Layout { built: Some(taken_apart(d, l)), ..l })`
/// gives back the blocks `derive(d, l)` gave, which
/// `taking_a_plan_apart_changes_nothing` asserts — because the moment it is not
/// exact, "take it apart" becomes an edit somebody did not ask for.
pub fn taken_apart(d: &Design, l: &Layout) -> Vec<Built> {
  derive(d, l)
    .into_iter()
    .map(|b| match b.kind {
      Kind::Entry { voice, shift, tonal } => Built::Entry { voice, shift, tonal, key_of: b.key_of },
      Kind::Episode { voice, shift } => {
        Built::Episode { voice, shift, key_of: b.key_of, bars: (b.len / d.measure.max(1)).max(1) }
      }
    })
    .collect()
}

/// Refuse a layout whose blocks name a voice that is not there, or no blocks at
/// all. **Nothing here asks whether it is a fugue** — `form::parse` answers that
/// afterwards, and answering it here would be a palette that refuses to build
/// what somebody wanted to look at.
fn check_built(d: &Design, l: &Layout) -> Result<(), String> {
  let Some(built) = &l.built else { return Ok(()) };
  if built.is_empty() {
    return Err("a built plan with no blocks in it is not a piece".into());
  }
  for (i, b) in built.iter().enumerate() {
    if b.voice() >= d.voices {
      return Err(format!("block {i} is in voice {}, and there are {} voices", b.voice(), d.voices));
    }
    if let Built::Episode { bars, .. } = b {
      if *bars < 1 {
        return Err(format!("block {i} is an episode of {bars} bars"));
      }
    }
  }
  Ok(())
}

/// Refuse a layout whose rests name a voice that is not there, or the one voice
/// a block exists to carry. Checked where the plan is derived, beside the turns.
fn check_rests(d: &Design, l: &Layout) -> Result<(), String> {
  if l.rests.is_empty() {
    return Ok(());
  }
  let blocks = derive(d, l);
  let ids = identities_of(d, l);
  for (k, voices) in &l.rests {
    // a rest whose block is gone is kept and costs nothing, exactly as a
    // reroll's is — it is what makes an undone edit come back the same
    let Some(bi) = ids.iter().position(|id| id == k) else { continue };
    let held = held_of(blocks.get(bi));
    for v in voices {
      if *v >= d.voices {
        return Err(format!("block {bi} rests voice {v}, and there are {} voices", d.voices));
      }
      if *v == held {
        return Err(format!("block {bi} rests voice {v}, which is the voice holding it"));
      }
    }
  }
  Ok(())
}

/// **What one block costs**, at this voice count, with `silent` voices absent —
/// readme §8.17's measurement.
///
/// The wall §2.7 argues about and §8.6 measured is on the number of voices the
/// search must *choose*, not on the number sounding: one voice holds the subject
/// and the rest are free, so a piece in `V` voices asks the search for `V − 1` at
/// once. A voice that rests is neither held nor free, and takes its whole domain
/// out of the product.
///
/// This exists so that the question can be asked through [`fill_block`] — the one
/// place a block is written — rather than by an experiment building its own
/// problem beside it. Two such places is how this file's history begins.
///
/// Returns the peak live states and the slice count, or the refusal.
pub fn block_cost(d: &Design, tier: &[Rule], silent: &[bool], with_plan: bool) -> Result<(usize, usize), String> {
  let blocks =
    vec![Block { at: 0, len: subject_bars(d), kind: Kind::Entry { voice: 0, shift: 0, tonal: false }, key_of: 0 }];
  let segments = if with_plan { plan(d, &blocks) } else { vec![] };
  let (_, _, attempt, peak) = fill_block(d, &blocks, 0, &segments, tier, 0x5EED, &[], &[], silent, Texture::Given)?;
  Ok((peak, attempt))
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
  let mut run = Run::new(d, l, tier, seed)?;
  while run.step()? {}
  Ok(run.parts())
}

/// A generate **in progress**: one block per [`Run::step`].
///
/// `ui-spec.md` section 7.3 wants a block per frame rather than a frame per
/// piece — six tenths of a second is rude on a desktop and freezes a browser
/// tab. The blockwise fill already had the right shape for it; what it did not
/// have was a way to stop between two blocks and come back.
///
/// **[`generate`] is this, run to completion, and that is the code rather than a
/// description of it.** The same reason `fill_block` exists: there were once two
/// places that wrote a block and they drifted apart in four ways, and a piece
/// generated all at once stopped agreeing with one refilled in part. A resumable
/// path that duplicated the loop would be the third such place, and the bug it
/// eventually produced would be a piece that came out differently depending on
/// how fast the machine drawing it was.
///
/// What a caller gets between steps is [`Run::voices`] — every block written so
/// far, in order, sorted. The plan strip fills in block by block from it, which
/// is a better progress indicator than a bar because it is the thing itself.
///
/// **On the web this is necessary and not sufficient.** `cpal`'s WebAudio
/// backend queues about 85 ms and one block is tens of milliseconds, so a block
/// per frame still crowds the audio budget; 7.3 says a worker is the honest
/// answer there and 6.4 is what stands in until there is one.
pub struct Run<'a> {
  /// Owned, not borrowed, so that a caller can keep a run across frames without
  /// also having to keep the design still. It is a subject and six numbers; the
  /// clone costs nothing beside one block of search.
  d: Design,
  tier: &'a [Rule],
  blocks: Vec<Block>,
  plan: Vec<harmony::Segment>,
  seeds: Vec<u64>,
  /// Who says nothing in each block — [`resting`], computed once with the plan
  /// because it is a property of the derivation and not of the fill.
  resting: Vec<Vec<bool>>,
  /// [`Layout::texture`], carried because `Run` does not keep the layout.
  texture: Texture,
  voices: Vec<Voice>,
  prior: Vec<Option<Pitch>>,
  relaxed: Relaxed,
  /// The block [`Run::step`] will fill next, so `next == blocks.len()` is done.
  next: usize,
  /// Seconds spent **inside** `new` and `step`, and not the wall clock between
  /// them. A run spread over twenty frames took the same work as one that
  /// blocked, and reporting the frames as part of it would make `Outcome::seconds`
  /// mean two different things depending on which path produced it.
  spent: f64,
}

impl<'a> Run<'a> {
  /// Derive the plan and prepare the fill. No block is written yet.
  ///
  /// Refuses here rather than part-way through, because a layout with a turn in
  /// the exposition is not a piece that fills badly — it is not a piece.
  pub fn new(d: &Design, l: &Layout, tier: &'a [Rule], seed: u64) -> Result<Run<'a>, String> {
    let t0 = crate::clock::Instant::now();
    check_built(d, l)?;
    check_turns(d, l)?;
    check_rests(d, l)?;
    let blocks = derive(d, l);
    let plan = plan(d, &blocks);
    let seeds = seeds(d, l, seed);
    let resting = resting(d, l);
    Ok(Run {
      texture: l.texture,
      d: d.clone(),
      tier,
      blocks,
      plan,
      seeds,
      resting,
      voices: vec![Voice { notes: vec![] }; d.voices],
      prior: vec![None; d.voices],
      relaxed: Relaxed::default(),
      next: 0,
      spent: t0.elapsed().as_secs_f64(),
    })
  }

  /// The derivation, in full and from the start — a plan view can draw the whole
  /// shape before a note of it exists.
  pub fn blocks(&self) -> &[Block] {
    &self.blocks
  }

  /// How many blocks are written, and how many there are.
  pub fn progress(&self) -> (usize, usize) {
    (self.next, self.blocks.len())
  }

  pub fn done(&self) -> bool {
    self.next >= self.blocks.len()
  }

  /// Every note written so far, sorted, one voice per part.
  ///
  /// Sorted **per block as it lands** rather than once at the end, which is the
  /// same sequence: a block's onsets all fall inside its own span, the spans
  /// ascend and do not overlap, so sorted runs concatenated are already in
  /// order. Doing it this way means what a caller draws mid-run is in the same
  /// order as what it draws at the end, and a beam grouping — which reads the
  /// notes in sequence — does not need to know whether the piece is finished.
  pub fn voices(&self) -> &[Voice] {
    &self.voices
  }

  /// Fill the next block. `Ok(true)` while there is more to do.
  pub fn step(&mut self) -> Result<bool, String> {
    if self.done() {
      return Ok(false);
    }
    let t0 = crate::clock::Instant::now();
    let bi = self.next;
    let (filled, after, attempt, _) =
      fill_block(&self.d, &self.blocks, bi, &self.plan, self.tier, self.seeds[bi], &self.prior, &[], &self.resting[bi], self.texture)?;
    self.relaxed.blocks += (attempt > 0) as usize;
    self.relaxed.without_prior += (attempt >= 1) as usize;
    self.relaxed.without_plan += (attempt >= 2) as usize;
    if attempt >= 1 {
      self.relaxed.cold.push(bi);
    }
    let at = self.blocks[bi].at;
    for (v, notes) in filled.iter().enumerate() {
      let from = self.voices[v].notes.len();
      self.voices[v].notes.extend(notes.notes.iter().map(|n| Note { onset: n.onset + at, ..*n }));
      self.voices[v].notes[from..].sort_by_key(|n| n.onset);
    }
    self.prior = after;
    self.next += 1;
    self.spent += t0.elapsed().as_secs_f64();
    Ok(!self.done())
  }

  /// The three things [`generate`] returns. Whatever is written so far, so a run
  /// abandoned part-way gives back the part that exists rather than nothing.
  pub fn parts(self) -> (Vec<Block>, Vec<Voice>, Relaxed) {
    (self.blocks, self.voices, self.relaxed)
  }

  /// Judge it, as [`fugue`] does. Refuses while blocks remain, because every
  /// figure in an [`Outcome`] is about a whole piece and a verdict on four
  /// blocks of twelve is not a smaller truth but a wrong one.
  pub fn finish(self) -> Result<Outcome, String> {
    if !self.done() {
      let (done, all) = self.progress();
      return Err(format!("{done} of {all} blocks written; a piece cannot be judged in part"));
    }
    let (d, spent) = (self.d.clone(), self.spent);
    let (blocks, voices, relaxed) = self.parts();
    Ok(judge(&d, blocks, voices, relaxed, spent))
  }
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
///
/// `Clone` because an editor keeps the previous one: `refill` splices into a
/// copy, and an edit that any block refuses must leave nothing changed.
#[derive(Clone)]
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
  // Through [`Run`], so that the blocking path and the resumable one are the
  // same code and not merely meant to agree.
  let mut run = Run::new(d, l, tier, seed)?;
  while run.step()? {}
  run.finish()
}

/// Judge notes that were filled **somewhere else** — the interface's worker,
/// `docs/ui-spec.md` section 7.3.
///
/// A browser has no threads, so generating without stalling the page means
/// generating in a worker, and a worker can send back notes but not an
/// [`Outcome`]: the verdict, the tally and the block list are not things to
/// serialise and post across a boundary when two of the three are cheap to
/// recompute and the third is a pure function of the design.
///
/// So the worker returns what only it has — the voices, and the relaxation log
/// that says what it had to give up — and this puts an `Outcome` back together
/// on the other side. It is [`fugue`]'s second half, and it is the same second
/// half: a piece judged here and a piece judged there cannot disagree, because
/// there is one `judge` and both go through it.
pub fn judged(d: &Design, l: &Layout, voices: Vec<Voice>, relaxed: Relaxed, seconds: f64) -> Outcome {
  judge(d, derive(d, l), voices, relaxed, seconds)
}

/// Every check §8 can make, over a finished piece.
///
/// Shared by [`fugue`] and [`refill`] rather than written twice: a result that
/// two paths judge differently is worse than one nobody judges.
fn judge(
  d: &Design,
  blocks: Vec<Block>,
  voices: Vec<Voice>,
  relaxed: Relaxed,
  seconds: f64,
) -> Outcome {
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

  Outcome {
    bars: length(&blocks) / d.measure.max(1),
    blocks,
    voices,
    relaxed,
    verdict,
    tally,
    seconds,
  }
}

/// Refill **one block**, leaving every other note exactly where it was.
///
/// [`refill_span`] with a span of one. See there for what this costs and when it
/// refuses.
pub fn refill(
  d: &Design,
  l: &Layout,
  tier: &[Rule],
  seed: u64,
  prev: &Outcome,
  bi: usize,
) -> Result<Outcome, String> {
  refill_span(d, l, tier, seed, prev, bi, bi)
}

/// Refill blocks `from..=to`, leaving every note outside them where it was.
///
/// The operation a plan editor needs. §8.16 fills a piece block by block and the
/// only thing crossing a block boundary is the pitch each voice ends on, so a
/// span refilled to the *same* ending is a drop-in replacement: every later
/// block keeps the notes it already had, and none of them is searched again. On
/// a twelve-block fugue a single block is a twelfth of the work, and at five
/// voices — where one block costs far more than a whole three-voice piece does —
/// it is the difference between an editor that responds and one that recomputes.
///
/// **A span, and not only a block, because one edit is not always one block.**
/// Changing where a return goes changes the key of the episode that travels to
/// it *and* the entry that arrives. Refilling those one at a time pins the seam
/// between them to notes chosen for the key being edited away, which is an
/// arbitrary constraint on the inside of an edit; here the interior seams are
/// free and only the **outer** ones are pinned.
///
/// **The measurement does not support that argument, and is recorded anyway.**
/// Over 144 key changes on 8 subjects, a span refill succeeded 110 times and a
/// block-at-a-time refill 108, disagreeing on 2 cases — two discordant pairs is
/// no evidence at all. What can be said is the principle, and that a span of two
/// is the smallest case there is: every interior seam is one more arbitrary pin,
/// so whatever the effect is, it grows with the span while this measurement
/// cannot see it. Roughly a quarter of key changes refuse under either, which is
/// the number a caller should plan around.
///
/// **Span-preserving edits only.** Changing which voice takes a block, or what
/// key it is in, leaves the piece the same length and this applies. Changing an
/// episode's length, or adding a middle, moves everything after it in time;
/// there is no sense in which those later bars are unchanged, and this refuses
/// rather than pretending. The caller falls back to [`fugue`].
///
/// It also refuses when the pinned ending is unreachable — the span's new
/// contents may not be able to arrive where the old ones did. That is a real
/// answer and not an error: the edit is possible, it just is not local, and the
/// caller regenerates from `from` onwards.
pub fn refill_span(
  d: &Design,
  l: &Layout,
  tier: &[Rule],
  seed: u64,
  prev: &Outcome,
  from: usize,
  to: usize,
) -> Result<Outcome, String> {
  check_built(d, l)?;
  check_turns(d, l)?;
  check_rests(d, l)?;
  let blocks = derive(d, l);
  if blocks.len() != prev.blocks.len() {
    return Err("the layout changed the number of blocks; refill is span-preserving".into());
  }
  if blocks.iter().zip(&prev.blocks).any(|(a, b)| a.at != b.at || a.len != b.len) {
    return Err("the layout moved a block in time; refill is span-preserving".into());
  }
  if to < from || to >= blocks.len() {
    return Err(format!("no such span: {from}..={to} of {} blocks", blocks.len()));
  }

  let full = plan(d, &blocks);
  let seeds = seeds(d, l, seed);
  let quiet = resting(d, l);
  // The running result. Each block's prior comes off *this*, so a block sees
  // what the one before it in the span actually wrote.
  let mut out = prev.voices.clone();
  let mut prior: Vec<Option<Pitch>> = (0..d.voices)
    .map(|v| {
      let at = blocks[from].at;
      out[v].notes.iter().filter(|n| n.onset < at).max_by_key(|n| n.onset).map(|n| n.pitch)
    })
    .collect();

  for bi in from..=to {
    let b = &blocks[bi];
    // **Pinned only where something follows the span**, and read off `prev`
    // because that is what the untouched tail still expects. A span that runs to
    // the end of the piece has nothing to protect, so it takes no pin — and that
    // is the case in which this is *exactly* what `generate` would have written,
    // which is what makes an edited fugue reproducible from its settings.
    let terminal: Vec<Option<Pitch>> = if bi == to && to + 1 < blocks.len() {
      (0..d.voices)
        .map(|v| crate::kern::sounding(&prev.voices[v], b.at + b.len - 1).map(|(p, _)| p))
        .collect()
    } else {
      vec![]
    };

    let (filled, after, _, _) =
      fill_block(d, &blocks, bi, &full, tier, seeds[bi], &prior, &terminal, &quiet[bi], l.texture)
        .map_err(|e| e.to_string())?;
    prior = after;

    // splice: the block's own bars replaced, everything else untouched
    for (v, notes) in filled.iter().enumerate() {
      out[v].notes.retain(|n| n.onset < b.at || n.onset >= b.at + b.len);
      out[v].notes.extend(notes.notes.iter().map(|n| Note { onset: n.onset + b.at, ..*n }));
      out[v].notes.sort_by_key(|n| n.onset);
    }
  }

  Ok(judge(d, blocks, out, prev.relaxed.clone(), 0.0))
}


/// Write an outcome as MIDI, tracks top voice first and named by their role.
pub fn write(o: &Outcome, d: &Design, path: &std::path::Path, qpm: u32) -> std::io::Result<()> {
  std::fs::write(path, encode(o, d, qpm))
}

/// The same as bytes, for a caller with no filesystem.
pub fn encode(o: &Outcome, d: &Design, qpm: u32) -> Vec<u8> {
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
  crate::midi::encode_score(&o.voices, &roles, qpm, crate::kern::meter_of(d.measure, d.beat))
}

/// Which part of the [`Layout`] a block came from.
///
/// [`derive`] is the only thing that knows this, and an editor has to: a gesture
/// on a block is meaningless until it can be turned back into the field that
/// produced it. Deriving the mapping a second time somewhere else is how the two
/// come apart, so it is computed here, from the same walk, and checked against
/// `derive` by a test.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
  /// A block somebody authored — [`Layout::built`]. It came from nothing but
  /// itself, which is why every edit phrased over the parameters below simply
  /// does not offer itself on one.
  Built,
  /// One of the exposition's entries.
  Exposition(usize),
  /// The episode [`Layout::link`] inserts into the exposition.
  Link,
  /// The episode or the entry of `middles[k]`.
  Middle(usize),
  /// The closing episode and entry of [`Layout::close_at_home`].
  Close,
}

/// One [`Origin`] per block of `derive(d, l)`, in the same order.
pub fn origins(d: &Design, l: &Layout) -> Vec<Origin> {
  if let Some(built) = &l.built {
    return vec![Origin::Built; built.len()];
  }
  let mut out = vec![];
  for i in 0..d.voices {
    out.push(Origin::Exposition(i));
    if let Some((after, bars)) = l.link {
      if i == after && bars > 0 && i + 1 < d.voices {
        out.push(Origin::Link);
      }
    }
  }
  for k in 0..l.middles.len() {
    out.push(Origin::Middle(k));
    out.push(Origin::Middle(k));
  }
  if l.close_at_home {
    out.push(Origin::Close);
    out.push(Origin::Close);
  }
  out
}

/// The blocks `middles[k]` produced — its episode and its entry.
///
/// Both, because changing a middle's degree changes the key of the episode that
/// travels to it as well as the entry that arrives, and an editor that refilled
/// only one of the two would leave the piece saying two different things about
/// where it is.
pub fn blocks_of_middle(d: &Design, l: &Layout, k: usize) -> Vec<usize> {
  origins(d, l)
    .iter()
    .enumerate()
    .filter(|(_, o)| **o == Origin::Middle(k))
    .map(|(i, _)| i)
    .collect()
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

  fn lanes(bs: &[Block]) -> Vec<usize> {
    bs.iter()
      .map(|b| match &b.kind {
        Kind::Entry { voice, .. } | Kind::Episode { voice, .. } => *voice,
      })
      .collect()
  }

  /// **A turn moves its block and everything after it, and nothing before it.**
  ///
  /// Which is the whole of what `ui-spec.md` section 4.3 calls the knock-on: a
  /// block's lane is not independently settable, so a drag rotates the tail, and
  /// the interface's job is to show that rather than pretend otherwise.
  #[test]
  fn a_turn_rotates_its_block_and_the_tail_behind_it() {
    let d = design();
    let plain = Layout { middles: vec![4, 5], episode_bars: 2, ..Layout::default() };
    let before = lanes(&derive(&d, &plain));
    let ids = identities_of(&d, &plain);

    // the first block after the exposition
    let at = origins(&d, &plain).iter().position(|o| matches!(o, Origin::Middle(_))).expect("a middle");
    for by in [1i16, 2, -1] {
      let turned = Layout { turns: vec![(ids[at], by)], ..plain.clone() };
      let after = lanes(&derive(&d, &turned));
      assert_eq!(after.len(), before.len(), "a turn changed the number of blocks");
      assert_eq!(&after[..at], &before[..at], "a turn by {by} moved something before it");
      for i in at..after.len() {
        let want = (before[i] as i16 + by).rem_euclid(d.voices as i16) as usize;
        assert_eq!(after[i], want, "block {i} under a turn of {by}");
      }
      // the chain rule the derivation is built on survives it: after the turn
      // point every step is still one lane on from the last
      for i in at + 1..after.len() {
        assert_eq!(after[i], (after[i - 1] + 1) % d.voices, "the chain broke at block {i}");
      }
    }
    // a whole rotation is no rotation
    let round = Layout { turns: vec![(ids[at], d.voices as i16)], ..plain.clone() };
    assert_eq!(lanes(&derive(&d, &round)), before, "turning by the voice count moved something");
  }

  /// **A turn does not change what a block is**, so a reroll and the turn itself
  /// are still looked up by the same key afterwards.
  ///
  /// This is why `ident` does not hash the lane. If it did, rotating a tail would
  /// reseed every block in it — the notes would change because the lane changed
  /// *and* because the seed did, and a turn undone would not come back.
  #[test]
  fn a_turn_leaves_every_identity_where_it_was() {
    let d = design();
    let plain = Layout { middles: vec![4, 5, 3], episode_bars: 2, ..Layout::default() };
    let ids = identities_of(&d, &plain);
    let seeds_before = seeds(&d, &plain, 0x5EED);
    let at = origins(&d, &plain).iter().position(|o| matches!(o, Origin::Middle(_))).expect("a middle");

    let turned = Layout { turns: vec![(ids[at], 1)], ..plain.clone() };
    assert_eq!(identities_of(&d, &turned), ids, "a turn moved the identities");
    assert_eq!(seeds(&d, &turned, 0x5EED), seeds_before, "a turn reseeded a block");

    // and the turn is still found by its own key after it has been applied
    assert!(lanes(&derive(&d, &turned)) != lanes(&derive(&d, &plain)), "the turn did nothing");
  }

  /// **The exposition cannot be turned in part.** Its entries are one per voice
  /// by construction rather than chained, so rotating a tail of them states the
  /// subject twice in one voice and never in another. Turning at the very first
  /// block rotates the whole run together and is allowed.
  #[test]
  fn the_exposition_refuses_a_turn_inside_it() {
    let d = design();
    let l = Layout { middles: vec![4], episode_bars: 2, ..Layout::default() };
    let ids = identities_of(&d, &l);
    let os = origins(&d, &l);

    for (bi, o) in os.iter().enumerate() {
      let inside = bi > 0 && matches!(o, Origin::Exposition(_) | Origin::Link);
      assert_eq!(turnable(&d, &l, bi).is_err(), inside, "block {bi} is {o:?}");
      let attempt = Layout { turns: vec![(ids[bi], 1)], ..l.clone() };
      match (inside, fugue(&d, &attempt, CONFIRMED, 0x5EED)) {
        (true, Err(e)) => assert!(e.contains("exposition"), "refused for another reason: {e}"),
        (true, Ok(_)) => panic!("block {bi} took a turn inside the exposition"),
        (false, Err(e)) => panic!("a legal turn at block {bi} was refused: {e}"),
        (false, Ok(_)) => {}
      }
    }

    // turning at block 0 rotates the whole piece, and the exposition is still
    // one entry per voice — a permutation of the lanes, not a subset of them
    let all = Layout { turns: vec![(ids[0], 1)], ..l.clone() };
    let mut expo: Vec<usize> = lanes(&derive(&d, &all)).into_iter().take(d.voices).collect();
    expo.sort();
    assert_eq!(expo, (0..d.voices).collect::<Vec<_>>(), "the rotated exposition is not one entry per voice");
  }

  /// A turned layout composes, and to something different from the untuned one.
  /// The lane a line is in decides its compass and its neighbours, so the notes
  /// must move; if they did not, the parameter would not be one.
  #[test]
  fn a_turned_plan_still_composes_and_to_something_else() {
    let d = design();
    let plain = Layout { middles: vec![4, 5], episode_bars: 2, ..Layout::default() };
    let ids = identities_of(&d, &plain);
    let at = origins(&d, &plain).iter().position(|o| matches!(o, Origin::Middle(_))).expect("a middle");
    let turned = Layout { turns: vec![(ids[at], 1)], ..plain.clone() };

    let a = fugue(&d, &plain, CONFIRMED, 0x5EED).expect("the plain one composed");
    let b = fugue(&d, &turned, CONFIRMED, 0x5EED).expect("the turned one composed");
    assert_eq!(a.bars, b.bars, "a turn changed the length of the piece");
    assert_ne!(notes(&a.voices), notes(&b.voices), "a turn left every note where it was");
    // **A turn reaches one block further back than it looks.** `fill_block` asks
    // which voice holds the *next* block, and gives that voice a bar of rest at
    // the end of this one so it does not enter by a leap from wherever its
    // accompanying line happened to end. Turn a block and its predecessor is
    // told a different voice is coming, so the predecessor changes too.
    //
    // Which is a fact the interface needs: the ghost in section 4.3 must fade
    // from the block *before* the drop, or it will fade less than really moves.
    let notes_before = |o: &Outcome, t: i64| {
      o.voices.iter().map(|v| v.notes.iter().filter(|n| n.onset < t).count()).collect::<Vec<_>>()
    };
    assert_eq!(
      notes_before(&a, a.blocks[at - 1].at),
      notes_before(&b, b.blocks[at - 1].at),
      "a turn changed the piece more than one block before it"
    );
    assert_ne!(
      notes_before(&a, a.blocks[at].at),
      notes_before(&b, b.blocks[at].at),
      "the block before the turn did not change, so the look-ahead has gone"
    );
  }

  /// **Four voices, with one resting wherever three would otherwise be free.**
  ///
  /// readme §8.17 measured one block and said the search could afford it. This is
  /// the whole piece, which is the part that measurement explicitly did not cover:
  /// a resting voice has no `Problem::prior`, so every re-entry is a cold start,
  /// and nothing had asked whether a piece survives being cold that often.
  #[test]
  fn four_voices_compose_when_one_of_them_rests() {
    let mut d = design();
    d.voices = 4;
    d.compass = (0..4).map(|v| { let top = 45 - 5 * v as i16; (top - 12, top) }).collect();
    let brief = Layout { middles: vec![4], episode_bars: 2, ..Layout::default() };

    // Without a rest it is three free voices from the fourth entry on, and the
    // search refuses — which is the state §8.17 leaves four voices in.
    match fugue(&d, &brief, CONFIRMED, 0x5EED) {
      Ok(_) => panic!("four voices composed with nobody resting, so this test proves nothing"),
      Err(e) => assert!(e.contains("state explosion"), "refused for another reason: {e}"),
    }

    // Now rest one voice in every block that has three free. Which one is a
    // choice and this makes the plainest available: the voice that entered
    // longest ago, which is the one furthest from having anything to say.
    let rests = rests_that_fit(&d, &brief);
    assert!(!rests.is_empty(), "no block needed a rest, so four voices was never the hard case");

    let l = Layout { rests, ..brief.clone() };
    let quiet = resting(&d, &l);
    for (bi, row) in quiet.iter().enumerate() {
      let free = d.voices - 1 - row.iter().filter(|q| **q).count();
      assert!(free <= FREE_WALL, "block {bi} still has {free} free voices");
    }

    let o = fugue(&d, &l, CONFIRMED, 0x5EED).expect("four voices with one resting");
    assert_eq!(o.voices.len(), 4);
    // and every voice really is in the piece rather than resting through all of it
    for (v, voice) in o.voices.iter().enumerate() {
      assert!(!voice.notes.is_empty(), "voice {v} never sounds");
    }
    // the subject is stated in all four, which is what makes it a four-voice fugue
    assert!(o.verdict.exposition_covers_the_voices, "the exposition does not cover four voices");
  }

  /// **The search can choose the rests itself, and reaches four voices doing it.**
  ///
  /// `Layout::texture` set to `Drawn`, with no `rests` at all: every legal rest pattern is a
  /// search of its own, drawn in proportion to the fills it admits. What that has
  /// to deliver is a piece where `rests_that_fit`'s heuristic delivered one, and
  /// without the heuristic.
  #[test]
  fn the_search_can_choose_who_rests() {
    let mut d = design();
    d.voices = 4;
    d.compass = (0..4).map(|v| { let top = 45 - 5 * v as i16; (top - 12, top) }).collect();
    let brief = Layout { middles: vec![4], episode_bars: 2, ..Layout::default() };

    // off, it is the same refusal as ever
    match fugue(&d, &brief, CONFIRMED, 0x5EED) {
      Ok(_) => panic!("four voices composed with nobody resting and the draw off"),
      Err(e) => assert!(e.contains("state explosion"), "refused for another reason: {e}"),
    }

    // on, with no pattern given, it finds one
    let l = Layout { texture: Texture::Drawn, ..brief.clone() };
    assert!(l.rests.is_empty(), "this must prove the draw, not a pattern");
    let o = fugue(&d, &l, CONFIRMED, 0x5EED).expect("the draw reached four voices");
    for (v, voice) in o.voices.iter().enumerate() {
      assert!(!voice.notes.is_empty(), "voice {v} never sounds");
    }
    assert!(o.verdict.exposition_covers_the_voices, "the exposition does not cover four voices");

    // and somebody really is resting where the wall would otherwise bite
    let quiet_somewhere = o.blocks.iter().any(|b| {
      let (lo, hi) = (b.at, b.at + b.len);
      (0..d.voices).filter(|v| !o.voices[*v].notes.iter().any(|n| n.onset < hi && n.onset + n.dur > lo)).count() > 0
    });
    assert!(quiet_somewhere, "nothing rests anywhere, so the draw did not do what it claims");
  }

  /// **The draw is off by default, and at three voices it does nothing at all.**
  ///
  /// The second half is the finding, not a disappointment to be worked around. A
  /// rest pattern with one more free voice admits thousands of times more fills —
  /// at three voices the full texture takes 99.94% of the draw and resting anyone
  /// takes 0.06% between them — so drawing uniformly over textures returns the
  /// densest legal one, which is the texture the generator already had. Note for
  /// note the same piece.
  ///
  /// That is what `readme` §8.17 concluded and it is why this ships off: §8.10's
  /// finding that drawing beats optimising is right about notes, and applied to
  /// texture the same argument returns the one thing texture is supposed to vary.
  /// A test that asserted the flag *changes* the music would be asserting the
  /// opposite of what was measured.
  #[test]
  fn drawing_the_texture_is_off_and_at_three_voices_is_the_same_piece() {
    assert_eq!(Layout::default().texture, Texture::Given, "the draw must be off by default");
    let d = design();
    let brief = Layout { middles: vec![4, 5], episode_bars: 2, ..Layout::default() };
    let a = fugue(&d, &brief, CONFIRMED, 0x5EED).expect("plain");
    let b = fugue(&d, &Layout { texture: Texture::Drawn, ..brief.clone() }, CONFIRMED, 0x5EED).expect("drawn");
    assert_eq!(a.bars, b.bars);
    let same = notes(&a.voices) == notes(&b.voices);
    assert!(same, "at three voices the draw took a thinner texture than the full one, which 99.94% says it should not");
  }

  /// **Both ways of choosing a texture collapse onto a constant, in opposite
  /// directions** — readme §8.18, and §8.10's finding arriving at texture.
  ///
  /// Drawing weights a pattern by how much music it admits and another free voice
  /// *is* more music, so it takes the fullest. Minimising takes the thinnest that
  /// will fill. Neither is answering a question about the bar it is in.
  #[test]
  fn choosing_a_texture_collapses_whichever_way_it_is_asked() {
    let d = design();
    let l = |t: Texture| Layout { texture: t, middles: vec![4, 5], episode_bars: 2, ..Layout::default() };
    let density = |o: &Outcome| -> f64 {
      let m = d.measure;
      let bars = (length(&o.blocks) + m - 1) / m;
      let total: usize = (0..bars)
        .map(|b| {
          let (lo, hi) = (b * m, (b + 1) * m);
          o.voices.iter().filter(|v| v.notes.iter().any(|n| n.onset < hi && n.onset + n.dur > lo)).count()
        })
        .sum();
      total as f64 / bars.max(1) as f64
    };

    let given = fugue(&d, &l(Texture::Given), CONFIRMED, 0x5EED).expect("given");
    let drawn = fugue(&d, &l(Texture::Drawn), CONFIRMED, 0x5EED).expect("drawn");
    let thin = fugue(&d, &l(Texture::Thinnest), CONFIRMED, 0x5EED).expect("thinnest");

    assert!(density(&thin) < density(&given), "the thinnest criterion did not thin anything");
    assert!(density(&thin) < density(&drawn), "the two ends came out the same way round");

    // **The thinnest does not reach silence**, and the reason is not the
    // criterion: the all-resting pattern is never a candidate, because
    // `realise::fill` refuses a problem with no free voice in it. A degenerate
    // optimum stopped by an unrelated detail is still degenerate.
    assert!(density(&thin) >= 1.0, "something is sounding in every bar");
    for (v, voice) in thin.voices.iter().enumerate() {
      assert!(!voice.notes.is_empty(), "voice {v} was thinned out of the piece entirely");
    }

    // and every one of them is still a fugue, which is what makes this a finding
    // about texture rather than about breaking the generator
    for (what, o) in [("given", &given), ("drawn", &drawn), ("thinnest", &thin)] {
      assert!(o.verdict.exposition_covers_the_voices, "{what} stopped being a fugue");
    }
  }

  /// **Taking a plan apart changes nothing.**
  ///
  /// The way into section 4.5's palette is a plan this program wrote, and the
  /// moment "take it apart" is not exact it becomes an edit nobody asked for.
  /// Block for block, over every shape the parameters can make.
  #[test]
  fn taking_a_plan_apart_changes_nothing() {
    let d = design();
    for middles in [vec![], vec![4], vec![4, 5, 3]] {
      for link in [None, Some((1, 1)), Some((0, 2))] {
        for close in [true, false] {
          let l = Layout { middles: middles.clone(), episode_bars: 2, link, close_at_home: close, ..Layout::default() };
          let was = derive(&d, &l);
          let apart = Layout { built: Some(taken_apart(&d, &l)), ..l.clone() };
          assert_eq!(shape(&derive(&d, &apart)), shape(&was), "taken apart and put back differs: {l:?}");
          // and the identities with them, so a reroll survives being taken apart
          assert_eq!(identities_of(&d, &apart), identities_of(&d, &l), "the blocks changed name");
        }
      }
    }
  }

  /// **An authored plan tiles time, and cannot be made not to.**
  ///
  /// `Built` has no `at` in it: where a block sits is derived by laying the list
  /// end to end. So a gap and an overlap are not things a palette can express,
  /// which is the guarantee the parameter path gets from accumulating and the
  /// one 4.5 was most worried about losing.
  #[test]
  fn an_authored_plan_leaves_no_gaps_and_no_overlaps() {
    let d = design();
    let built = vec![
      Built::Entry { voice: 0, shift: 0, tonal: false, key_of: 0 },
      Built::Episode { voice: 2, shift: 0, key_of: 4, bars: 3 },
      Built::Entry { voice: 1, shift: 4, tonal: true, key_of: 4 },
      Built::Episode { voice: 0, shift: 0, key_of: 0, bars: 1 },
    ];
    let l = Layout { built: Some(built.clone()), ..Layout::default() };
    let blocks = derive(&d, &l);
    assert_eq!(blocks.len(), built.len());
    assert_eq!(blocks[0].at, 0, "the piece does not start at the beginning");
    for w in blocks.windows(2) {
      assert_eq!(w[0].at + w[0].len, w[1].at, "a gap or an overlap between two authored blocks");
    }
    // an episode's length is authored in bars; an entry's is the subject's
    assert_eq!(blocks[1].len, 3 * d.measure);
    assert_eq!(blocks[0].len, subject_bars(&d));

    // the parameters say nothing once blocks are authored
    let noisy = Layout { middles: vec![1, 2, 3], episode_bars: 6, close_at_home: true, ..l.clone() };
    assert_eq!(shape(&derive(&d, &noisy)), shape(&blocks), "a parameter moved an authored plan");
  }

  /// **An authored plan composes, and the grammar judges it rather than gating
  /// it** — which is 4.5's whole argument. A plan that is not a fugue is written
  /// and then told what it is missing.
  #[test]
  fn an_authored_plan_composes_and_is_judged_not_gated() {
    let d = design();
    // deliberately not a fugue: one entry, then episodes, and no exposition
    let l = Layout {
      built: Some(vec![
        Built::Entry { voice: 0, shift: 0, tonal: false, key_of: 0 },
        Built::Episode { voice: 1, shift: 0, key_of: 0, bars: 2 },
        Built::Entry { voice: 1, shift: 0, tonal: false, key_of: 0 },
      ]),
      ..Layout::default()
    };
    let o = fugue(&d, &l, CONFIRMED, 0x5EED).expect("an authored plan composes");
    assert_eq!(o.blocks.len(), 3);
    // and the verdict says what it is not, rather than the layout having refused
    assert!(!o.verdict.exposition_covers_the_voices, "three voices were covered by two entries");

    // what *is* refused is a plan that is not a plan
    let empty = Layout { built: Some(vec![]), ..Layout::default() };
    match fugue(&d, &empty, CONFIRMED, 0x5EED) {
      Ok(_) => panic!("a plan with no blocks composed"),
      Err(e) => assert!(e.contains("no blocks"), "refused for another reason: {e}"),
    }
    let nowhere = Layout {
      built: Some(vec![Built::Entry { voice: 9, shift: 0, tonal: false, key_of: 0 }]),
      ..Layout::default()
    };
    match fugue(&d, &nowhere, CONFIRMED, 0x5EED) {
      Ok(_) => panic!("a block in a voice that does not exist composed"),
      Err(e) => assert!(e.contains("voice 9"), "refused for another reason: {e}"),
    }
  }

  /// `Block` and `Voice` carry no `PartialEq` and this file is not the place to
  /// give them one, so the two comparisons below are on projections that hold
  /// everything either type is: what a block is and where, and every field of
  /// every note including its spelling — §2.1's step *and* alteration, so two
  /// pieces that sound alike and are written differently do not compare equal.
  fn shape(bs: &[Block]) -> Vec<(i64, i64, i16, &Kind)> {
    bs.iter().map(|b| (b.at, b.len, b.key_of, &b.kind)).collect()
  }
  /// Onset, duration, step, alteration, attack — every field a note has.
  type Written = (i64, i64, i16, i8, bool);
  fn notes(vs: &[Voice]) -> Vec<Vec<Written>> {
    vs.iter()
      .map(|v| v.notes.iter().map(|n| (n.onset, n.dur, n.pitch.step, n.pitch.alter, n.attack)).collect())
      .collect()
  }

  /// **A run stepped block by block is the piece generated in one call.**
  ///
  /// The whole point of `Run`: the interface stops between blocks so the frame
  /// keeps up, and what it ends with must be the piece a driver would have got
  /// by blocking. Note for note, and the same relaxation log, on layouts of
  /// three blocks and of eleven.
  ///
  /// `generate` and `fugue` both run this loop rather than owning a copy of it,
  /// which is what makes the assertion cheap to keep true — but a test still
  /// says so, because the reason `fill_block` exists is that two places wrote a
  /// block and drifted, and the fix for that is not to trust the fix.
  #[test]
  fn stepping_a_run_writes_what_generating_it_would_have() {
    let d = design();
    for middles in [vec![], vec![4], vec![4, 5, 3, 6, 1]] {
      let l = Layout { middles, episode_bars: 2, ..Layout::default() };
      let (blocks, voices, relaxed) = generate(&d, &l, CONFIRMED, 0x5EED).expect("generated");

      let mut run = Run::new(&d, &l, CONFIRMED, 0x5EED).expect("a plan");
      assert_eq!(shape(run.blocks()), shape(&blocks), "the plan differs before a note is written");
      assert_eq!(run.progress(), (0, blocks.len()));
      let mut steps = 0;
      while run.step().expect("stepped") {
        steps += 1;
        assert!(steps <= blocks.len(), "step never said it was done");
      }
      assert!(run.done());
      assert_eq!(run.progress(), (blocks.len(), blocks.len()));

      let (b2, v2, r2) = run.parts();
      assert_eq!(shape(&b2), shape(&blocks), "the blocks differ");
      assert_eq!(notes(&v2), notes(&voices), "the notes differ");
      assert_eq!(r2.blocks, relaxed.blocks, "the relaxation count differs");
      assert_eq!(r2.cold, relaxed.cold, "a different set of blocks came out cold");
    }
  }

  /// What a caller draws mid-run: every block written so far and nothing else,
  /// each voice in onset order.
  ///
  /// The order is the half worth asserting. `generate` sorted once at the end
  /// and `Run` sorts each block as it lands; they agree because a block's onsets
  /// all fall inside its own span and the spans ascend. If that ever stopped
  /// being true the notes would still all be present, and only a beam grouping —
  /// which reads them in sequence — would show it.
  #[test]
  fn a_run_in_progress_holds_the_blocks_it_has_finished() {
    let d = design();
    let l = Layout { middles: vec![4, 5], episode_bars: 2, ..Layout::default() };
    let mut run = Run::new(&d, &l, CONFIRMED, 0x5EED).expect("a plan");
    let blocks = run.blocks().to_vec();

    let mut last = 0usize;
    for (bi, block) in blocks.iter().enumerate() {
      run.step().expect("stepped");
      let end = block.at + block.len;
      let here: usize = run.voices().iter().map(|v| v.notes.len()).sum();
      assert!(here > last, "block {bi} wrote nothing");
      last = here;
      for (v, voice) in run.voices().iter().enumerate() {
        assert!(voice.notes.windows(2).all(|w| w[0].onset <= w[1].onset), "voice {v} is out of order after block {bi}");
        assert!(voice.notes.iter().all(|n| n.onset < end), "voice {v} has a note past block {bi}");
      }
    }
  }

  /// **A piece is judged whole or not at all.** Every figure in an `Outcome` is
  /// about a complete piece — the grammar verdict most of all, since a parse of
  /// four blocks out of twelve is not a partial verdict but a wrong one.
  #[test]
  fn a_half_written_run_refuses_to_be_judged() {
    let d = design();
    let l = Layout { middles: vec![4, 5], episode_bars: 2, ..Layout::default() };
    let mut run = Run::new(&d, &l, CONFIRMED, 0x5EED).expect("a plan");
    run.step().expect("stepped");
    run.step().expect("stepped");
    match run.finish() {
      Ok(_) => panic!("a run judged itself half-written"),
      Err(e) => assert!(e.contains("cannot be judged in part"), "refused for another reason: {e}"),
    }
  }

  /// `origins` walks the same shape `derive` does, and nothing keeps them in
  /// step but this. One entry per block, in order, over every layout an
  /// interface can produce — because the failure this prevents is silent: an
  /// editor would refill the wrong block and the piece would simply be wrong
  /// somewhere else.
  #[test]
  fn origins_line_up_with_the_blocks_derive_makes() {
    let d = design();
    for middles in [vec![4], vec![4, 5, 3], vec![4, 5, 1, 3, 6]] {
      for link in [None, Some((0, 1)), Some((1, 2)), Some((5, 1))] {
        for close in [true, false] {
          let l = Layout { middles: middles.clone(), episode_bars: 2, link, close_at_home: close, ..Layout::default() };
          let blocks = derive(&d, &l);
          let os = origins(&d, &l);
          assert_eq!(blocks.len(), os.len(), "{l:?}");

          // every middle owns exactly two blocks, and they carry its degree
          for (k, &deg) in l.middles.iter().enumerate() {
            let owned = blocks_of_middle(&d, &l, k);
            assert_eq!(owned.len(), 2, "middle {k} of {l:?}");
            for bi in owned {
              assert_eq!(blocks[bi].key_of, deg, "block {bi} of {l:?}");
            }
          }
          // and a link exists in the origins exactly when derive made one
          let linked = os.iter().filter(|o| **o == Origin::Link).count();
          assert!(linked <= 1, "{l:?}");
          assert_eq!(linked == 1, l.link.is_some_and(|(a, b)| b > 0 && a + 1 < d.voices), "{l:?}");
        }
      }
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


  /// **The point of the terminal pin**: a refilled block changes its own bars
  /// and nothing else, note for note.
  ///
  /// Asserted over the whole piece rather than over the boundary, because a
  /// splice that tore one bar later would still pass a boundary check and would
  /// still be wrong on screen.
  #[test]
  fn refilling_one_block_leaves_every_other_note_alone() {
    let d = design();
    let l = Layout::default();
    let first = fugue(&d, &l, CONF_MEL, 0x5EED).expect("a fugue");
    // a middle entry, so there is music on both sides of it
    let bi = 6;
    let b = first.blocks[bi].clone();
    let again = refill(&d, &l, CONF_MEL, 0x5EED, &first, bi).expect("a refill");

    let outside = |o: &Outcome| -> Vec<(usize, i64, i16, i8)> {
      o.voices
        .iter()
        .enumerate()
        .flat_map(|(v, x)| {
          x.notes
            .iter()
            .filter(|n| n.onset < b.at || n.onset >= b.at + b.len)
            .map(move |n| (v, n.onset, n.pitch.step, n.pitch.alter))
        })
        .collect()
    };
    assert_eq!(outside(&first), outside(&again), "the splice disturbed music outside the block");
    assert!(!outside(&first).is_empty(), "the block must not be the whole piece");
  }

  /// And the pin is what does it: the refilled block ends on the same pitches
  /// it did before, which is why nothing after it had to move.
  #[test]
  fn a_refilled_block_ends_where_it_ended() {
    let d = design();
    let l = Layout::default();
    let first = fugue(&d, &l, CONF_MEL, 0x5EED).expect("a fugue");
    let bi = 6;
    let b = first.blocks[bi].clone();
    let again = refill(&d, &l, CONF_MEL, 0x5EED, &first, bi).expect("a refill");
    for v in 0..d.voices {
      let end = |o: &Outcome| crate::kern::sounding(&o.voices[v], b.at + b.len - 1).map(|(p, _)| p);
      assert_eq!(end(&first), end(&again), "voice {v} ends the block somewhere else");
    }
  }

  /// A span-changing edit is refused rather than half-applied. Lengthening an
  /// episode moves every later bar, and no pin can make that local.
  #[test]
  fn a_span_changing_edit_is_refused() {
    let d = design();
    let first = fugue(&d, &Layout::default(), CONF_MEL, 0x5EED).expect("a fugue");
    let longer = Layout { episode_bars: 5, ..Layout::default() };
    assert!(refill(&d, &longer, CONF_MEL, 0x5EED, &first, 6).is_err());
    let more = Layout { middles: vec![4, 5, 3, 1], ..Layout::default() };
    assert!(refill(&d, &more, CONF_MEL, 0x5EED, &first, 6).is_err());
  }

  /// The seed is keyed on what a block **is**, so an edit reseeds only the
  /// blocks it actually described differently.
  ///
  /// Changing the last middle's key leaves every earlier block's seed alone —
  /// which is the property an editor needs, and the one the old index-keyed
  /// seed did not have: under that scheme any change to the block *list* moved
  /// every index and redrew the piece.
  ///
  /// Note what this does **not** claim. `derive` gives each block the voice
  /// after its predecessor's, so *inserting* a middle rotates every later
  /// block into a different voice — those blocks really are different blocks
  /// and really do reseed. That is a property of the derivation, not of the
  /// seeding, and it is why [`refill`] accepts span-preserving edits only.
  #[test]
  fn changing_a_late_block_does_not_reseed_the_early_ones() {
    let d = design();
    let before = seeds(&d, &Layout::default(), 0x5EED);
    let mut edited = Layout::default();
    *edited.middles.last_mut().unwrap() = 1; // the last middle goes elsewhere
    let after = seeds(&d, &edited, 0x5EED);

    assert_eq!(before.len(), after.len(), "a key change must not change the block count");
    let same = before.iter().zip(&after).take_while(|(a, b)| a == b).count();
    assert!(same >= 8, "only {same} of {} blocks kept their seed", before.len());
    assert_ne!(before, after, "the edited block must reseed");
  }

  /// Two blocks that are identical in every respect still get different seeds,
  /// or the piece would repeat itself exactly.
  #[test]
  fn identical_blocks_still_differ() {
    let d = design();
    // three middles in the same key, so the blocks are otherwise identical
    let l = Layout { middles: vec![4, 4, 4], ..Layout::default() };
    let blocks = derive(&d, &l);
    let s = seeds_from(&identities(&blocks), 0x5EED, &[]);
    let mut uniq = s.clone();
    uniq.sort_unstable();
    uniq.dedup();
    assert_eq!(s.len(), uniq.len(), "two blocks share a seed: {s:?}");
  }

  /// **A refill that runs to the end of the piece is exactly what `generate`
  /// writes.**
  ///
  /// This is the guarantee `docs/ui-spec.md` section 8 rests on, and it was
  /// false until it was tested. An edited fugue is saved as its settings, and
  /// loading regenerates from them — so if editing produced something the
  /// generator would not, a saved piece came back as a different piece and the
  /// fingerprint said so. Which it did: the report was a mismatch between engine
  /// 0.1.0 and engine 0.1.0.
  ///
  /// The cause was that a refilled span pins its last block's ending to what the
  /// piece sounded *before* the edit, and that pin is information the settings do
  /// not carry. A span reaching the end of the piece has nothing following to
  /// protect, so it takes no pin, and then the two agree note for note.
  #[test]
  fn a_refill_to_the_end_is_what_the_generator_would_have_written() {
    let d = design();
    let l0 = Layout::default();
    let base = fugue(&d, &l0, crate::automaton::HARD, 0x5EED).expect("a fugue");
    let ids = identities(&base.blocks);
    let last = base.blocks.len() - 1;

    for bi in [1usize, 5, 8, last] {
      let mut l = l0.clone();
      l.rerolls.push((ids[bi], 1));

      let spliced = refill_span(&d, &l, crate::automaton::HARD, 0x5EED, &base, bi, last).expect("refill");
      let fresh = fugue(&d, &l, crate::automaton::HARD, 0x5EED).expect("fugue");
      assert_eq!(
        crate::settings::fingerprint(&spliced.voices),
        crate::settings::fingerprint(&fresh.voices),
        "refilling from block {bi} to the end differs from generating the whole piece"
      );

      // and it really did change something, or the comparison proves nothing
      assert_ne!(crate::settings::fingerprint(&spliced.voices), crate::settings::fingerprint(&base.voices));

      // everything *before* the edit is untouched, which is the locality that
      // survives — and the one worth having, since it is the part already heard
      let at = base.blocks[bi].at;
      for v in 0..d.voices {
        let before = |o: &Outcome| -> Vec<(i64, i16, i8)> {
          o.voices[v].notes.iter().filter(|n| n.onset < at).map(|n| (n.onset, n.pitch.step, n.pitch.alter)).collect()
        };
        assert_eq!(before(&base), before(&spliced), "block {bi}: the bars before the edit moved");
      }
    }
  }

  /// **Asking for one block again changes that block's seed and no other.**
  ///
  /// The point of keying the nudge on identity rather than on an index: an
  /// editor's business is inserting things, and a reroll that moved when
  /// something before it grew would be a reroll of whatever happened to be there
  /// instead.
  #[test]
  fn a_reroll_moves_one_seed_and_leaves_the_rest() {
    let d = design();
    let l = Layout::default();
    let blocks = derive(&d, &l);
    let ids = identities(&blocks);
    let plain = seeds_from(&ids, 0x5EED, &[]);

    let target = 5usize;
    let once = seeds_from(&ids, 0x5EED, &[(ids[target], 1)]);
    for i in 0..blocks.len() {
      if i == target {
        assert_ne!(plain[i], once[i], "the rerolled block kept its seed");
      } else {
        assert_eq!(plain[i], once[i], "block {i} moved when block {target} was rerolled");
      }
    }
    // and asking again is a third thing, not a toggle back to the first
    let twice = seeds_from(&ids, 0x5EED, &[(ids[target], 2)]);
    assert_ne!(twice[target], once[target]);
    assert_ne!(twice[target], plain[target]);
  }

  /// And the nudge stays with its block across an edit elsewhere, which is the
  /// whole reason it is keyed on identity.
  ///
  /// Note the claim, which is the same one [`seeds`] makes and no larger.
  /// Sending a *different* return somewhere else leaves this block the same
  /// block, and it keeps both its seed and the reroll applied to it. Inserting a
  /// middle would not: that rotates every later block into another voice, so
  /// they are different blocks and reseed by design. An editor that promised
  /// otherwise would be promising something the derivation does not do.
  #[test]
  fn a_reroll_stays_with_its_block_across_an_edit_elsewhere() {
    let d = design();
    let l = Layout::default();
    let blocks = derive(&d, &l);
    let ids = identities(&blocks);

    // the last block, and a change to the *first* middle — a different return
    let target = blocks.len() - 1;
    let mine = vec![(ids[target], 1)];
    let before = seeds_from(&ids, 0x5EED, &mine)[target];

    let mut edited = l.clone();
    edited.middles[0] = 1;
    let after_blocks = derive(&d, &edited);
    let after_ids = identities(&after_blocks);
    assert_eq!(after_ids[target], ids[target], "the last block changed identity under an edit to the first middle");
    assert_eq!(before, seeds_from(&after_ids, 0x5EED, &mine)[target], "the reroll did not follow its block");

    // and it is still a reroll: without the nudge that block draws differently
    assert_ne!(before, seeds_from(&after_ids, 0x5EED, &[])[target]);
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
    // **From the first note, not from tick zero.** This subject begins on an
    // upbeat, so a fugue that states it alone — which is what an exposition is,
    // and what `resting` now writes — has nothing sounding before it. That is an
    // anacrusis and not a gap; Bach's own fugues on upbeat subjects would fail a
    // check that counted it. What the listening report behind this test heard was
    // the whole texture dropping out *inside* the piece, every three to six
    // seconds, and that is what is measured here.
    let mut t = voices.iter().filter_map(|v| v.notes.iter().map(|n| n.onset).min()).min().unwrap_or(0);
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
    let l = Layout::default();
    let (blocks, voices, _) = generate(&d, &l, CONFIRMED, 0x5EED).expect("a fugue");
    let quiet = resting(&d, &l);
    for (bi, b) in blocks.iter().enumerate() {
      for (i, v) in voices.iter().enumerate() {
        let sounds = v.notes.iter().any(|n| n.onset >= b.at && n.onset < b.at + b.len);
        assert_eq!(
          sounds,
          !quiet[bi][i],
          "voice {i} at bar {}: sounding = {sounds}, and `resting` says it should be {}",
          b.at / d.measure + 1,
          !quiet[bi][i]
        );
      }
    }
  }

  /// **A voice says nothing until it has entered, and everything after.**
  ///
  /// The exposition of a fugue is voices arriving one at a time, and until this
  /// rule there were no arrivals: `fill_block` gave every voice that was not
  /// holding the subject a tiled rhythm, so a three-voice fugue had three voices
  /// from bar one. readme §8.17 is the measurement and this is the property.
  #[test]
  fn a_voice_is_silent_until_it_has_entered() {
    let d = design();
    let l = Layout::default();
    let blocks = derive(&d, &l);
    let quiet = resting(&d, &l);

    // the first block is one voice alone, which is what an exposition opens with
    assert_eq!(quiet[0].iter().filter(|q| **q).count(), d.voices - 1, "the piece does not open with one voice");

    let mut entered = vec![None; d.voices];
    for (bi, b) in blocks.iter().enumerate() {
      let held = match &b.kind {
        Kind::Entry { voice, .. } | Kind::Episode { voice, .. } => *voice,
      };
      assert!(!quiet[bi][held], "block {bi} silences the voice holding it");
      if matches!(b.kind, Kind::Entry { .. }) && entered[held].is_none() {
        entered[held] = Some(bi);
      }
      // and nobody who has entered is ever silenced again
      for v in 0..d.voices {
        if let Some(at) = entered[v] {
          assert!(!quiet[bi][v], "voice {v} entered at block {at} and is silent again at {bi}");
        }
      }
    }
    assert!(entered.iter().all(|e| e.is_some()), "some voice never entered: {entered:?}");

    // the rule empties itself: once every voice has entered nobody rests, so the
    // texture after the exposition is exactly as full as it was before
    let last_entry = entered.iter().flatten().max().copied().expect("an entry");
    for (bi, row) in quiet.iter().enumerate().skip(last_entry) {
      assert!(row.iter().all(|q| !q), "block {bi} still rests somebody after the exposition");
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
