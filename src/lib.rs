//! **Counterpoint is a regular language** — the model, and a fugue generator
//! built on it.
//!
//! This is the library half of the repository. The other half is a binary that
//! reproduces every figure in [`readme.md`](../readme.md) §8, and the split is
//! not cosmetic: what lives here is the *model*, and what lives there is the
//! *measurement of it*. A caller who wants to compose needs none of the second.
//!
//! # Where to start
//!
//! [`compose`] is the whole of it for most callers:
//!
//! ```no_run
//! use contrapunctus::{automaton::HARD, compose, kern};
//!
//! let piece = kern::read(std::path::Path::new("corpus/bach-wtc-fugues/kern/wtc1f02.krn"))?;
//! let design = compose::Design {
//!   subject: kern::clip(&piece.voices[1], 0, 2 * piece.measure),
//!   voices: 3,
//!   key: piece.key,
//!   tonic: 0,
//!   measure: piece.measure,
//!   beat: piece.beat,
//!   compass: vec![(33, 45), (28, 40), (21, 33)],
//! };
//!
//! let outcome = compose::fugue(&design, &compose::Layout::default(), HARD, 0x5EED)?;
//! println!("{} bars, {:.1} violations per thousand", outcome.bars, outcome.per_thousand(HARD));
//! compose::write(&outcome, &design, std::path::Path::new("fugue.mid"), 76)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! [`compose::Design`] is *what the music is made of* and [`compose::Layout`] is
//! *what is done with it*; the split is there because a user interface wants a
//! control for each field of the second and rarely touches the first.
//! [`compose::Outcome`] carries the notes **and** every judgement this
//! repository can pass on them, so that a result cannot be displayed without
//! also being able to say what is wrong with it.
//!
//! # What each module is
//!
//! **The lattice and the score.** [`pitch`] is a diatonic step plus an
//! alteration, never a semitone integer — readme §2.1, and the reason a
//! diminished fifth and an augmented fourth stay different things. [`kern`]
//! reads Humdrum `**kern`, which spells its pitches, and [`midi`] writes MIDI,
//! which does not: §8.6 measured 13 notes in 20 coming back respelled, so MIDI
//! is an output format here and never an interchange one.
//!
//! **The rulebook.** [`automaton`] is §2.2's finite automaton — 513 reachable
//! states, an alphabet of vertical interval plus the two voices' motion, and a
//! stratified rulebook. [`corpus`] checks a texture against it. [`species`] is
//! Fux's four figures as a whitelist (§8.7), [`shape`] three criteria over a
//! whole line (§8.8).
//!
//! **Harmony.** [`harmony`] infers a chord path by Viterbi (§8.5) and [`key`] a
//! local key the same way (§8.14); [`plan`] builds the variants §8.9 prices.
//!
//! **The search.** [`realise`] is §2.5's shortest path over a layered DAG,
//! which fills free voices against fixed ones, counts the legal set exactly, and
//! can draw from it uniformly. It refuses rather than beams.
//!
//! **Fugue.** [`answer`] is Marpurg's tonal answer (§8.11), [`stretto`] the
//! entry-placement search (§8.3), [`episode`] the sequence detector (§8.13),
//! [`form`] the grammar parser (§8.15), and [`compose`] the generator (§8.16).
//! [`refdata`] reads the annotations those sections are measured against.
//!
//! # What this library will not do
//!
//! It is not fitted to a corpus and does not contain a model trained on one:
//! every rule in it was transcribed from a treatise by hand, and readme §8.2
//! and §8.11 are measurements of how well those transcriptions describe two
//! repertoires rather than fits to either. §5 is the argument for why, and §8.6
//! is the honest account of what it costs — agreement with Bach around seven per
//! cent, against a legal set of `10¹²` fills per three bars.

pub mod answer;
pub mod automaton;
pub mod compose;
pub mod corpus;
#[cfg(feature = "embedded-corpus")]
pub mod embedded;
pub mod episode;
pub mod form;
pub mod harmony;
pub mod kern;
pub mod key;
pub mod midi;
pub mod pitch;
pub mod plan;
pub mod realise;
pub mod refdata;
pub mod settings;
pub mod shape;
pub mod species;
pub mod stretto;
