# Contrapunctus: Counterpoint is a regular language

### Fugue on the lattice: 513 states, a clique, and what Bach says about the rulebook

*Design document and current state. The counterpoint automaton, the stretto measure and the harmonic analyser
are built and measured — [§8](#8-what-is-built-and-what-it-measures). Realisation and form are not
— [§9](#9-roadmap). How this was arrived at, including the claims that did not survive, is in
[`CHANGELOG.md`](CHANGELOG.md).*

---

## Contents

- [Abstract](#abstract)
- [The name is the argument in miniature](#the-name-is-the-argument-in-miniature)
- [0. Where this comes from](#0-where-this-comes-from)
- [1. Diagnosis: ricercar's §8 is two causes, not seven items](#1-diagnosis-ricercars-8-is-two-causes-not-seven-items)
  - [1.1 The state is a point, not a transition](#11-the-state-is-a-point-not-a-transition)
  - [1.2 Ricercar's own evidence against the continuum](#12-ricercars-own-evidence-against-the-continuum)
- [2. The reformulation](#2-the-reformulation)
  - [2.1 Exact arithmetic, and therefore no certificates](#21-exact-arithmetic-and-therefore-no-certificates)
  - [2.2 Counterpoint is a finite automaton](#22-counterpoint-is-a-finite-automaton)
  - [2.3 Harmony is a second automaton](#23-harmony-is-a-second-automaton)
  - [2.4 Form is a grammar](#24-form-is-a-grammar)
  - [2.5 The search is a shortest path](#25-the-search-is-a-shortest-path)
  - [2.6 What is *not* a variable: rhythm](#26-what-is-not-a-variable-rhythm)
  - [2.7 Where a solver takes over from the DP](#27-where-a-solver-takes-over-from-the-dp)
- [3. Stretto, capacity, and the subject](#3-stretto-capacity-and-the-subject)
  - [3.1 The calibration disappears](#31-the-calibration-disappears)
  - [3.2 Capacity is a density, and it cannot be optimised](#32-capacity-is-a-density-and-it-cannot-be-optimised)
  - [3.3 The subject is input, and its boundary is contested](#33-the-subject-is-input-and-its-boundary-is-contested)
- [4. Space filling, in the right category](#4-space-filling-in-the-right-category)
- [5. What this will not do](#5-what-this-will-not-do)
- [6. What ricercar still owns](#6-what-ricercar-still-owns)
- [7. Prior art](#7-prior-art)
- [8. What is built, and what it measures](#8-what-is-built-and-what-it-measures)
  - [8.1 The automaton](#81-the-automaton)
  - [8.2 The rulebook, stratified by two corpora](#82-the-rulebook-stratified-by-two-corpora)
  - [8.3 The clique test](#83-the-clique-test)
  - [8.4 Capacity ranks subjects, and cannot design one](#84-capacity-ranks-subjects-and-cannot-design-one)
  - [8.5 The harmonic analyser](#85-the-harmonic-analyser)
- [9. Roadmap](#9-roadmap)
- [10. Reproducing the results](#10-reproducing-the-results)
  - [10.1 Environment and data](#101-environment-and-data)
  - [10.2 Which command produces which section](#102-which-command-produces-which-section)
  - [10.3 Parameters](#103-parameters)
  - [10.4 How the samples were taken](#104-how-the-samples-were-taken)
  - [10.5 What is not reproducible from this repository](#105-what-is-not-reproducible-from-this-repository)
---

## Abstract

Machine composition of fugue is usually attempted in one of two categories: fit a model to a corpus, or search a
continuous relaxation of the score. This document argues that both are the wrong category, and that fugue is a
**word problem over a finite alphabet subject to constraints of bounded memory** — so the natural instruments are
automata, dynamic programming and exact combinatorial search, none of which require training data and all of which
return proofs rather than samples.

The argument opens as a post-mortem. [`ricercar`](ricercar/readme.md) modelled counterpoint as a
Lipschitz-certifiable roughness field over a continuum of entry placements, and its own measurements refute it. The
legal region turned out **piecewise constant at the note grid**; and rounding a certified placement onto the
semitone grid costs about ten times the margin the certificate establishes, so the proof was being taken over the
wrong set. Everything expensive in that approach existed to bound a function whose answer is constant on a lattice.

On the lattice the reformulation is small, and most of it is classical. Counterpoint is a **finite automaton** over
`(interval, motion, articulation, metric weight)` whose state is *the interval plus what you owe* — a dissonance
owes a resolution, a leap owes a recovery — and it stays finite because strict counterpoint requires debts settled
on the next event. Harmony is a second automaton; form is a ten-line grammar; realising free voices against fixed
entries is a shortest path, escalating to a CDCL solver at four or more free voices, where layering is known to
fail. Densest stretto becomes **maximum clique in a Cayley graph** on the shift group, exactly computable, where
the continuous formulation of the same question was killed at thirty minutes without a single placement. Bach's own
five-voice hyperstretto in BWV 867 is such a clique, and it is an arithmetic progression — `{0, 2, 4, 6, 8}`
quarters — so the object is a clique in a Cayley graph and emphatically not a Sidon set.

The counterpoint automaton, the stretto measure and a harmonic analyser are built and measured against the 24
fugues of the Well-Tempered Clavier, Book I, and against 200 works of 15th-century polyphony, using published
ground-truth annotations and Huron's Humdrum encodings. The automaton has **513 reachable states**, and it
distinguishes a prepared suspension from the same interval struck on the same beat — the distinction a field over
instantaneous pitch is structurally unable to make, and the device most of the repertoire worth imitating is built
from. Run as a checker over two corpora it **stratifies its own rulebook into three**: parallel perfect consonances
and direct motion to a perfect consonance on a downbeat hold in both centuries at about one violation per thousand
slices; the melodic prohibition holds in Renaissance vocal writing and fails in Bach by a factor of thirty-eight,
so it is repertoire-specific rather than wrong; and the two dissonance rules fail in the very repertoire they were
written for. The surviving pair is precisely the pair a roughness field cannot express at all, since a perfect
fifth is among the smoothest intervals it knows.

The clique test selects that same two-rule tier a second time by a different route. **Under the full five-rule
tier Bach's hyperstretto is not a clique; under the two-rule tier it is**, on both contested readings of the
subject, and a control on the written notes rather than on idealised transpositions confirms the fault lies with
the rulebook rather than with the model of an entry. That two independent tests — one counting rule frequencies
across two centuries, one asking whether a single passage is mutually compatible — converge on the same two rules
is the strongest result here, because neither was designed to check the other.

Capacity, measured as the **edge density of the compatibility graph**, ranks **BWV 849 first of 24** — the fugue
musicians name when they name a stretto fugue — and correlates only −0.31 with note density, so it is not a proxy
for how busy a subject already is. It ranks and it **cannot design**: optimising a contour against it yields a
monotone, and Bach's own contours score below random on their own rhythms. The reason is structural. Both surviving
rules need a perfect consonance to fire, and a fugal answer is at the fifth, so the measure penalises the interval
the form is built on. A design objective has to be harmonic, and that remains open.

What the method does not do is decide whether the result is good, and its failure mode is the inverse of the usual
one: a complete search does not fail by finding nothing but by finding far too much — on the order of `10⁵`–`10⁶`
legal counterpoints to an eleven-note *cantus firmus*. Where taste enters is therefore the central design question
rather than an afterthought, and the position taken here is the **Pareto front** over soft criteria rather than a
weighted sum, on the ground that no weighting in the literature is defensible and Fux declines to supply one.

**Method.** Nothing here is fitted to a corpus. The rules are transcribed from treatises; every threshold is either
measured against an exhibited passage or swept and reported as a curve rather than chosen. The single parameter
that had to be picked is held out on half the corpus and cross-checked on the other. How this position was reached,
including the four claims it replaced, is in [`CHANGELOG.md`](CHANGELOG.md).

---

## The name is the argument in miniature

*Ricercare*, to search: that document named its method. *Contrapunctus* — **punctus contra punctum**, point against
point — names its objects. Note against note. The material is discrete, countable, and set against itself, and the
word says so in the first syllable. It is also Bach's own heading for each movement of the *Art of Fugue*, which is
the one place in the repertoire where the combinatorial content and the aesthetic content very nearly coincide.

---

## 0. Where this comes from

Ricercar reached step 5 and stopped, blocked twice: once on principle (the threshold `θ` was unpinned, so a corpus
comparison would measure the threshold rather than the subject) and once on cost (the calibrated run was killed at
thirty minutes without a single placement). [§8](ricercar/readme.md#8-what-this-will-not-do) of that document lists seven things the method will not do.

The question this answers: *starting over, without space-filling, is there an approach free of most of [§8](ricercar/readme.md#8-what-this-will-not-do) — and
elegant rather than fitted?*

Yes. And the first evidence for it is in ricercar's own results.

---

## 1. Diagnosis: ricercar's §8 is two causes, not seven items

| [§8](ricercar/readme.md#8-what-this-will-not-do) item | cause |
|---|---|
| the stylistic rules — parallel fifths, voice crossing | **the state is a point, not a transition** |
| resolution, suspension | same |
| harmony, "the difference between a cadence and a stop" | **the surrogate is not the thing** — roughness is psychoacoustics, not tonality |
| form | neither; it was simply never modelled |
| the 2-approximation guarantee | an artefact of the geometric framing |
| whether the result is good | irreducible — see [§5](#5-what-this-will-not-do) |

### 1.1 The state is a point, not a transition

**A parallel fifth is not a property of an instant.** Neither is contrary motion, voice crossing, a suspension, a
cadence, or voice independence in any form. They are properties of *consecutive configurations*. A field over
instantaneous pitch content is not underpowered here — it is structurally incapable of expressing any of them, and
that single fact generates most of [§8](ricercar/readme.md#8-what-this-will-not-do). Ricercar says as much: *"a field over instantaneous pitch cannot express any
of them."*

The repair is cheap once the diagnosis is stated:

> **Every rule of strict counterpoint is a condition on at most three consecutive events.**

That is not an approximation of the rulebook. It is what the rulebook says, and [§2.2](#22-counterpoint-is-a-finite-automaton) takes it literally.

### 1.2 Ricercar's own evidence against the continuum

Two measurements, neither of them arguments:

- **[§7.3](ricercar/readme.md#73-step-3-result-the-legal-region-and-what-its-shape-gives-away).** *"The legal placement region is piecewise constant in entry offset, at the note grid — measured, not
  argued, and it is why fugal onsets are quantized in practice."* That is the continuum announcing it does no work.
  Every certificate in the crate exists to bound a function over a domain whose answer is constant on a lattice.
- **[§7.4](ricercar/readme.md#74-step-4-result-the-pipeline-closes-on-two-answers) against [§7.1](ricercar/readme.md#71-step-1-result-go)**, which is arithmetic on two published numbers rather than a finding either section records.
  A certified placement has to be rounded onto the semitone grid before it can be notated. Worst-case rounding is
  25 cents; the pitch constant is `0.021` per cent; so rounding can move the roughness by `≈ 0.5`. The clearances
  actually certified were `0.0489` and `0.0354`. **The rounding is an order of magnitude larger than the margin it
  destroys.**

The second one is the tell. If the answer must be rounded onto a grid at the end, and the rounding is worth ten
times the margin the proof establishes, then the proof was over the wrong set.

---

## 2. The reformulation

### 2.1 Exact arithmetic, and therefore no certificates

Pitch is an integer — semitones, or a scale degree with a chromatic inflection. Time is a rational on a sixteenth
grid. The transformation group of ricercar [§1](ricercar/readme.md#1-the-dictionary) becomes exact integer and rational arithmetic:

| transformation | operation |
|---|---|
| **real** transposition by `k` | `x + k` |
| **tonal** answer | its own transformation type, not a value of `k` — see below |
| inversion about axis `a` | `a − x` |
| retrograde | reverse the word |
| augmentation by `r` | multiply durations by `r ∈ ℚ` |

This representation is not a choice so much as a convergence: **Schottstaedt (1984) uses semitones above low C with
onsets and durations counted in eighth notes; Giraud et al. (2015) use semitones with onsets and durations counted
in sixteenths.** Thirty-one years and opposite tasks — generation and analysis — and the same lattice. It is what
the problem is made of.

**No floating point anywhere.** Nothing to quantise, nothing to certify, no Lipschitz constant to measure, no
safety factor to assume. Roadmap steps 1 through 3 of ricercar — the roughness constant, the branch and bound over
time, the branch and bound over placement — do not have counterparts here. They were the cost of a continuum that
[§1.2](#12-ricercars-own-evidence-against-the-continuum) says was not there.

This also disposes of the register problem [§7.6](ricercar/readme.md#76-step-5-θ-calibrated-against-bach) found the hard way. There is no `L_R` to be register-dependent
about.

**A fugal answer is usually not an exact transposition.** It is *tonal*: transposed to the dominant, and altered
in its first few notes so that the dominant pitch maps back to the tonic rather than to the supertonic. BWV 867
shows it in its first two notes — the subject opens B♭4 → F4, a descending fourth, and the answer at measure 3
opens F4 → B♭3, a descending *fifth*. A real answer would have given F4 → C4. The ground truth marks the label
`(tonal_answer)`; Huron's encoding shows the mechanism.

So the tonal answer is **its own transformation type `τ`**, and [§3](#3-stretto-capacity-and-the-subject)'s
compatibility table is indexed by `(τᵢ, τⱼ, …)` for that reason. Pitch is held as `(scale degree, inflection)`
rather than as a semitone integer, which is also Giraud's argument for matching on diatonic intervals: "a scale
will always match only a scale."

### 2.2 Counterpoint is a finite automaton

Read a pair of voices tick by tick. The alphabet at each tick:

```
symbol = ( interval class, motion type, articulation, metric weight )

interval class   (p_upper − p_lower) mod 12, tagged perfect / imperfect / dissonant,
                 plus unison-vs-compound where it matters
motion type      parallel | similar | contrary | oblique — from the signs of the two melodic steps
articulation     which voices strike at this tick, and which are tied over
metric weight    strong | weak — a function of the tick, not of the music
```

Now the rulebook, transcribed:

| rule | as an automaton condition | order | tier |
|---|---|---|---|
| parallel fifths, octaves | forbidden edge `5 → 5`, `8 → 8` under parallel motion | 2 | **hard** |
| direct fifths on a downbeat | edge `* → 5` under similar motion with a leap above | 2 | **hard** |
| forbidden melodic interval | augmented, diminished, seventh, beyond an octave | 2 | hard for Renaissance repertoire only ([§8.2](#82-the-rulebook-stratified-by-two-corpora)) |
| passing dissonance | dissonant tick approached and left by step | 3 | soft |
| **suspension** | consonant-and-tied → dissonant-on-strong → step down to consonance | 3 | soft |
| neighbour tone | step away and back | 3 | soft |
| voice crossing, overlap | `p_upper(t) ≥ p_lower(t)`, `p_upper(t) ≥ p_lower(t−1)` | 2 | soft |
| leap recovery | a leap beyond a fourth is followed by a step against it | 2–3 | soft |

The tier column is measured rather than assumed, and [§8.2](#82-the-rulebook-stratified-by-two-corpora) is how.

The state is best described in one phrase: **the interval, plus what you owe.** A leap incurs an obligation to
recover; a dissonance incurs an obligation to resolve; a suspension is an obligation created and discharged. The
obligation set is finite and small precisely because counterpoint requires debts to be settled on the very next
event — that is what "strict" means.

Three things follow.

**The suspension is repaired.** Ricercar's [§8](ricercar/readme.md#8-what-this-will-not-do) calls preparation-and-resolution *"the device most of the repertoire
worth imitating is built from"*, and the model blind to it. In an automaton a prepared dissonance and an accidental
one are **different paths spelling the same instantaneous interval**. The distinction is free, because the state
remembers where it came from. This is the single largest gain, and it is not a patch — it falls out of using the
right category.

**The parallel fifth is repaired.** [§7.2](ricercar/readme.md#72-step-2-result-a-proof-not-a-sample) had to *substitute its own test* because the roughness field rates a
perfect fifth at `0.089`, among the least rough intervals there are, and would never flag a parallel one. Here it
is the canonical forbidden edge — the first thing the automaton knows.

**The rulebook is smaller than the model it replaces.** The crude product of the state components is 1280 and
**513 states are reachable** ([§8.1](#81-the-automaton)) — measured, not asserted, which is this project's habit
and the reason the figure moved once already when a rule was stated more carefully.

### 2.3 Harmony is a second automaton

A functional automaton over `(key, scale degree, inversion)`, with edges for the standard progressions and
modulation via pivot chords. The two automata compose by intersection — harmony as a regular language, voice
leading as a transduction over it — and it is worth saying that this reading is classical, because the components
being known-good is the point.

**What is built is the analytic half**, not the functional one. `src/harmony.rs` segments a texture at every onset
and chooses a chord path by Viterbi, charging a penalty to change chord so that the **harmonic rhythm emerges
rather than being imposed by a window**. It identifies the arrival chord of an annotated cadence 80% of the time
against a 14% baseline ([§8.5](#85-the-harmonic-analyser)).

The functional half — a cadence as a *labelled accepting path* rather than a coincidence — is **not** established.
The progression rule as written accepts nine of twelve root motions, so it admits 75% of everything before any
music is consulted, and it separates nothing. A real version needs degree successions relative to a **local** key,
and fugues modulate constantly, so it needs key-finding first ([§9](#9-roadmap)).

### 2.4 Form is a grammar

```
Fugue       → Exposition Middle+ Final
Exposition  → Entry (Countersubject Entry){V−1}
Middle      → Episode Entry+
Final       → Stretto? Pedal? Cadence
Episode     → Sequence(motive, transposition pattern, n)
```

with the key plan a bounded walk on the circle of fifths. Ten lines, and [§8](ricercar/readme.md#8-what-this-will-not-do)'s first item — *"a fugue is narrative;
a packing has none"* — is repaired by making the narrative the top-level object and letting counterpoint fill the
blocks in.

A packing cannot do this and never could, because a packing has no distinguished order. A grammar is nothing but
order. The mismatch was in the choice of formalism, not in the effort spent on it.

### 2.5 The search is a shortest path

Because the constraints have **bounded memory — order ≤ 3 ticks** — filling free voices against fixed entries is a
shortest path in a layered DAG. Plain Viterbi. Exact, no backtracking, no heuristics, no tuning, no restarts, and
no `LineSearch` that might have been climbing a lower bound.

State size at a tick is the tuple of sounding pitches with their obligations. For two or three voices this is
outright small; at four it wants pruning by the harmonic automaton, which cuts it hard because a chord constrains
every voice at once; at five it wants a beam or a constraint solver. In the solver direction the relevant tool is
Pesant's `regular` global constraint (CP 2004) — a domain-consistent propagator for *"this sequence is accepted by
this DFA"*, which is exactly this problem's shape, and it exists because this shape is common.

**And the cost profile is the right way round.** In a fugue most voices are not free; they are stating the subject.
If `e` entries sound, only `V − e` voices need filling, and in a dense stretto that is zero or one. **The method is
cheapest exactly where the counterpoint is densest** — the opposite of ricercar, where the calibrated stretto was
the run that had to be killed.

One boundary, and it is not where it first appears to be. Melodic *shape* rules — a single climax, tessitura over a
phrase, no repeating a figure — are long-range, and Schottstaedt's source shows how long: `TotalRange`,
`PitchRepeats` and `TooMuchOfInterval` each scan the entire melody so far. But every one of them is an
**accumulator** — a running min and max, a saturating count, a histogram — and an accumulator is finite-state
whenever its range is bounded. So the accurate statement is not about lookback:

> **Contrapuntal rules are order ≤ 3 in events. Melodic shape rules have unbounded lookback but bounded state,
> because they are accumulators.**

Everything stays finite-state; what changes is the state count, and the interval-mixture histogram is where it
stops being small. That is the honest reason to reach for a solver rather than a wider DP — and Schottstaedt,
having implemented all three, still concludes that his program *"makes no decisions about overall melodic shapes."*
Implementing the accumulators is not the same as controlling the shape.

### 2.6 What is *not* a variable: rhythm

The grid is fixed before the search. Which tick a note falls on is data; only pitch is unknown. That is a real
restriction and it should be stated rather than discovered later.

It is also the mainstream position, and for the same reason. Anders & Miranda's survey calls the general problem
*score topology* — a contrapuntal constraint needs to know which notes sound together, which are melodically
adjacent, and where the barline is, and none of that is determined until the rhythm is. PWConstraints' polyphonic
subsystem **Score-PMC makes note pitches the only variables and requires the rhythmic structure to be fully
determined in the problem definition**, because it computes its static variable ordering by sorting notes by start
time. The identical restriction, arrived at from the identical difficulty.

What it costs: no rhythmic invention, no choosing where a suspension may be prepared, no deciding a subject's
durations. What it buys: simultaneity is a lookup, the constraint graph is static, and [§2.5](#25-the-search-is-a-shortest-path)'s layered DAG exists at
all. For a fugue this is a fair trade — the subject's rhythm *is* given, and the episodes are the part where it
would matter.

### 2.7 Where a solver takes over from the DP

The DP dies at the voice count, not at the piece length. State at a tick is the product of the free voices'
domains: with a two-octave compass that is roughly `24^(V−e)` before obligations, so `V − e = 2` is comfortable,
`3` wants the harmonic automaton pruning it, and `4` or more is out of reach exactly.

**Schottstaedt reached exactly this wall in 1984 and his report is the best evidence in the literature that it is
real.** Read directly rather than through the survey, it says four things that bear on the design here:

- His stated goal was *"five to eight part mixed species counterpoint"* — the same target as [§8](#9-roadmap)'s step 5.
- Exhaustive search is hopeless and he quantifies it: a **ten-note, two-voice, first-species** problem has `16¹⁰`
  branches, about twenty minutes at one nanosecond per check. That is the smallest case in the subject.
- His branch-and-bound **is** complete — *"if any solution at all exists, we are guaranteed to find it… we are also
  guaranteed to find the best solution"* — and it is the version he could not ship: *"more complex cases drag to a
  halt."* What he shipped is a **beam**, keeping the sixteen best continuations per branch, with a branch cap and
  an acceptance threshold that decays over time *"somewhat like a person getting more and more frustrated as more
  effort is poured into a fruitless search."*
- And **layering does not work.** *"In the first attempt at multi-part counterpoint we solved one voice at a time…
  This worked well for three voices… As more voices were added however, the later layers became less and less
  acceptable. It became clear that the entire ensemble has to be calculated together."*

The last point is the one to take seriously, because the tempting shortcut here is exactly layering — fill one free
voice, then the next. It is reported as tried and failed, at three voices, forty years ago. The joint state is not
optional, which is precisely why the DP's product blows up and why the search has to become something cleverer than
enumeration.

**This is where a CDCL solver belongs, and the reason is conflict learning, not theories.** Every system in the
survey that searches over polyphony uses plain chronological backtracking; Anders & Miranda name *thrashing* as its
known weakness — the same conflict rediscovered over and over because the search never records why it failed —
and Ebcioğlu had to build backjumping into a new language (BSL) to get around it. Conflict-driven clause learning
is that fix, done properly: the conflict is learned once as a clause and never revisited anywhere in the tree. In a
five-voice texture, where a dead end in bar 30 is caused by a choice in bar 3, this is the whole game.

Three further properties matter here specifically:

- **`unsat` is a proof**, and the unsat core names the conflicting constraints. "No fifth entry can be added" comes
  back with *which pair of entries* forbids it — a musically meaningful answer, and the thing ricercar's
  `best_remaining()` existed to fake.
- **Soft constraints are native.** [§5](#5-what-this-will-not-do) is about where taste enters, and the Pareto
  front is the position taken there; Z3's optimizer supports multi-objective search in `pareto` mode directly.
- **Incrementality.** Push an entry, re-solve, pop — which is the shape of [§3](#3-stretto-capacity-and-the-subject)'s search, and precisely what
  ricercar's `capacity()` got wrong by rebuilding.

What it is *not* good for here: none of the SMT theories earn their keep. Pitch is finite-domain, interval legality
is a precomputed table, and `(pᵢ − pⱼ) mod 12` is actively unpleasant in linear integer arithmetic. The right
encoding is **one-hot Booleans per (voice, tick), table constraints for interval legality, and the automaton
unrolled as a state variable per tick with transition clauses** — which is bounded model checking's standard trick
and lands the whole problem in pure SAT. Use the solver as a very good SAT engine with an optimiser on top, not as
an SMT solver.

Two warnings, both real at five voices. **Symmetry** — voice permutation within a register, and the global
transposition of the entire texture — inflates unsat proofs badly and nothing breaks it for you; order the voices
by register and pin the first entry at `(τ = identity, d = 0, k = 0)`. And **one model is not a set of pieces**:
blocking clauses enumerate near-duplicates, so diversity has to be asked for explicitly rather than hoped for.

---

## 3. Stretto, capacity, and the subject

This is the measurement ricercar [§6.1](ricercar/readme.md#61-the-measurement) wanted and was blocked on twice.

Parameterise an entry as `(τ, d, k)` — transformation, offset in ticks, transposition — with `τ` acting relative to
the entry point. Then for any two entries the sounding interval sequence depends only on

```
( τᵢ , τⱼ , Δd , pitch offset )
```

where `Δd = dⱼ − dᵢ` and the pitch offset is `±kᵢ ± kⱼ` according to which entries are inverted. **Shifting both
entries together changes nothing**, and this holds for the whole transformation group, including retrograde and
augmentation, because every transformation is applied relative to its own entry point.

So the compatibility relation is a **precomputed table**, filled exhaustively by running the [§2.2](#22-counterpoint-is-a-finite-automaton) automaton over
each overlap. Order of magnitude: 36 transformation pairs × 128 offsets × 49 pitch offsets ≈ 2·10⁵ entries, each an
`O(n)` automaton run over a subject of a few dozen ticks. **Milliseconds, once, per subject.**

Then:

> **Densest stretto = maximum clique in the compatibility graph.**

Within a single transformation class the graph is a Cayley graph on the shift group, and a legal stretto is a set of
offsets **whose difference set is contained in the good set `A`**. Across classes it is still one small explicit
graph.

It is **not** a Sidon set, and the distinction matters. A Sidon set requires all pairwise differences to be
*distinct*; the condition here is that they all land in `A`, which highly degenerate sets satisfy. Bach's own
hyperstretto in BWV 867 is `{0, 2, 4, 6, 8}` quarters — an arithmetic progression, whose difference set repeats its
step four times over. **The densest strettos are regular, not clever**: a canon at a fixed time interval is an
arithmetic progression by construction, and if the step is legal then its multiples tend to be too. The clique
search should look for structure rather than scatter.

Three consequences.

**Infeasibility becomes a proof.** Ricercar's `best_remaining()` exists only because the greedy loop cannot tell "no
legal entry remains" from "the search gave up on a loose lower bound" — a defect the project caught by grid scan
and recorded. A complete search over a finite graph does not have the distinction to make. `best_remaining()`
deletes.

**Clique size is bounded by the voice count**, `V ≤ 5`, so this is a depth-5 search with strong pruning over a few
hundred vertices once the plausible transposition set is fixed. Maximum clique is NP-hard in general and this
instance is not the hard case; if it ever becomes one, that is an ordinary engineering problem with a large
literature behind it, not a modelling question.

**Pairwise legality is necessary, not sufficient** — dissonance treatment and harmony read the whole sonority. So
the clique is an *upper bound*, and each candidate clique is then verified against the full `V`-voice automaton.
That is exact branch and bound with an admissible bound, and it is the same shape as the packing argument it
replaces, only finite.

### 3.1 The calibration disappears

Ricercar [§7.6](ricercar/readme.md#76-step-5-θ-calibrated-against-bach) had to pin `θ` against Bach's own hyperstretto, found `θ_pair ≥ 0.821` against the `0.300` used
throughout step 4, and concluded that *"the measurement became intractable at the moment the threshold stopped
being wrong."*

Here there is no threshold. The calibration becomes a **yes-or-no test**, and since [§8](#9-roadmap)'s step 0 the target is an
exact set of integers rather than a description:

> The subject of BWV 867 is 12 quarters long. Its five final entries entries stand at quarters
> **`{266, 268, 270, 272, 274}`** — one per voice, `{0, 2, 4, 6, 8}` from the first.
> **Does `{0, 2, 4, 6, 8}` come out as a clique in that subject's compatibility graph?**

If yes, the automaton is calibrated — by construction, since Bach's five-voice hyperstretto is acceptable
counterpoint. If no, it is too strict and *that is the finding*. Nothing is tuned either way.

**The answer is both, and the split is the result** ([§8.3](#83-the-clique-test)): the full five-rule tier rejects
Bach's hyperstretto and the two-rule tier accepts it, on both contested readings of the subject.

Note how much the test tightened by having the data. Ricercar spent [§7.5](ricercar/readme.md#75-the-real-subject-and-two-things-it-broke) and [§7.6](ricercar/readme.md#76-step-5-θ-calibrated-against-bach) establishing this passage from a
score by hand and arrived at a real-valued threshold that then made the computation intractable. The same passage
is four lines of a public annotation file, and the test it supports is integer equality.

### 3.2 Capacity is a density, and it cannot be optimised

Clique *size* is the obvious capacity measure and it does not work: under the tier Bach confirms, 81% of entry
pairs are compatible and the largest legal stretto grows until the search is cut off. The measure that does work is
the **edge density of the compatibility graph** — bounded in `[0, 1]` by construction, so it cannot run away, and
it ranks subjects sensibly ([§8.4](#84-capacity-ranks-subjects-and-cannot-design-one)).

**Density ranks and cannot design.** Optimising a subject's contour against it produces a monotone, and Bach's own
contours score *below* random on their own rhythms. The reason is structural rather than a defect: both surviving
hard rules require a perfect consonance to fire, so maximising density means minimising the perfect consonances a
subject forms against its own transpositions — and a fugal answer is at the fifth. **The measure penalises the
interval the form is built on.** A design objective has to be harmonic, and ricercar
[§6.2](ricercar/readme.md#62-the-design-problem)'s design problem is therefore still open
([§9](#9-roadmap)).

### 3.3 The subject is input, and its boundary is contested

Capacity is a function of the subject, and this section assumes the subject is given. Giraud et al. built a ground truth for the
24 Bach fugues of WTC I against four musicological sources — Prout, Tovey, Keller, Bruhn, plus Charlier — and
report that **in eight of the twenty-four, at least two sources disagree about where the subject ends**, sometimes
by several notes. On Fugue No. 9 they quote Tovey to the effect that it is not worth settling where the subject
ends and the countersubject begins; the flow between them is continuous.

**The ground truth turns that from a warning into a list.** The disagreement is recorded in the data as `S alternative`
labels carrying the dissenting source, and it falls in fugues **5, 7, 9, 10, 11, 18, 19 and 22** — eight, as
claimed. The spread is not uniform: No. 19 carries three alternatives (Prout at `−5/8`, Bruhn at `−2/8`, Tovey and
Keller at `0`), No. 9 two, and **the target fugue is one of the eight** — BWV 867's subject is 3 measures for
Keller and Bruhn's "female ending" and 2 for Prout and Bruhn's "male", a difference of a third of the subject.

A subject four notes longer overlaps more, forbids more offsets, and has lower capacity. So **a single capacity
number silently encodes an editorial decision**, and a corpus ranking built from single numbers would be measuring
the editors as much as the subjects — the same failure mode as ricercar's unpinned `θ`, arriving from a completely
different direction.

Three ways to take it, in increasing order of honesty:

1. use the algomus ground truth and cite it — reproducible, but inherits one committee's view;
2. report capacity **as an interval** over the alternative subject-ends the sources give, which the ground-truth
   files record;
3. treat the subject end as a *free variable* and report the capacity profile over it — which is more interesting
   than either, because "where does this subject stop stretto-ing well" is a musical question, and the profile's
   shape may be the argument for one editor over another.

(3) is the version worth building, and it costs nothing extra: the compatibility table is computed per subject
length anyway, so the profile is a loop over prefixes of one table. It is also not optional. Capacity turns out
**non-monotonic in subject length** and the editorial choice can halve it
([§8.4](#84-capacity-ranks-subjects-and-cannot-design-one)), so a single figure is not a well-behaved function of
the one input a reader would assume it depends on.

---

## 4. Space filling, in the right category

The instinct behind ricercar was not wrong. Counterpoint really is a tiling problem. The error was the category:
not packing in `ℝᵈ`, but **factorisation of a finite abelian group**.

A tiling rhythmic canon is a partition of `ℤₙ` into translates of a rhythmic motif — every beat covered exactly
once, no gaps, no overlaps, which is space filling in the strictest sense available. The mathematics is Vuza's
canons, the Coven–Meyerowitz conditions, and Hajós groups, and it has been pursued in a music-theoretic setting by
Andreatta, Amiot and Agon.

It is discrete, it is elegant, it is not fitted to anything, and it is deep. If the aesthetic pull of this project
is *counterpoint as tiling*, that literature is where the pull is actually satisfied — and it is adjacent to [§3](#3-stretto-capacity-and-the-subject),
since a difference-set condition on entry offsets is the same kind of object.

---

## 5. What this will not do

Written in ricercar's [§8](ricercar/readme.md#8-what-this-will-not-do) form, because the point of that section is that it exists.

- **Whether the result is good.** Unchanged and irreducible. A legal fugue is not a beautiful one, and no formalism
  fixes that.
- **But the failure mode inverts, and this is worth stating plainly.** A complete solver does not fail by finding
  nothing; it fails by finding *far too much*. Completeness is not selectivity. That is ricercar [§5](ricercar/readme.md#5-the-objection-and-what-it-is-actually-an-objection-to)'s boundary
  arrived at from the other side, and it is the same boundary.

  **Measured, by Komosinski & Szachewicz (2014).** For an eleven-note *cantus firmus*, first species, two voices —
  the smallest interesting case there is — the number of legal counterpoints is **10⁵ to 3·10⁶**, growing
  exponentially in length. Whatever this method is short of, candidates is not it.

- **And the standard reply is wrong, which is the most useful thing the literature says.** The usual fix is to
  weight the broken rules and minimise `Σ pᵢ·nᵢ`. Komosinski & Szachewicz reject it on two grounds. The weights are
  unobtainable — the treatises rank rules only loosely, and they quote Fux himself declining to rank one: *"I shall
  leave to your discretion the use or avoidance of it."* And a sum is the wrong algebra, because it makes breaking
  one important rule equivalent to breaking three trivial ones, which is not how anyone hears music.

  Their alternative is to **not aggregate**: report the **Pareto front** under the dominance relation — every
  counterpoint not beaten on all criteria at once. No weights, no trade-offs asserted, nothing lost that is best at
  anything.

  **The hard/soft split is Schottstaedt's, from 1984**, as a stratified penalty table — `Infinity` for the rules
  that may not be broken (parallel fifths and unisons, dissonance, out of mode, out of range, bad cadence, no
  leading tone) and small integers for the rest. Komosinski's criticism lands on the **soft tier alone**, and there
  it lands hard, because those integers are unarguable magic numbers: a sixth followed by motion in the same
  direction costs 34, a fifth in the same position costs 8, a skip costs 1, three repeated notes 4 and four
  repeated notes 7. Nothing justifies 34 against 8.

  **So the literature offers three different algebras on the soft criteria, and the choice is the whole question.**

  | | how soft criteria combine | what it asserts |
  |---|---|---|
  | Schottstaedt 1984 | **weighted sum**, hard rules at infinity | a full exchange rate between every pair of rules |
  | Ebcioğlu 1990 | **lexicographic** — heuristics weighted by decreasing powers of two, so each outranks all below it combined | a total order on the rules |
  | Komosinski 2015 | **Pareto** — no aggregation | nothing |

  Only the third asserts nothing, which is why it should be the default here: **the automaton carries the hard
  rules, the Pareto front carries the soft ones**, and taste enters exactly once, at the end, as a person choosing
  from an incomparable set. Conveniently, this is also free — a modern optimising solver implements all three modes
  (`box`, `lex`, `pareto`), so the choice is a flag rather than a rewrite, and the three can be compared on the
  same encoding.

  Two caveats Komosinski records: the front can reach ~700 members for an eleven-note *cantus firmus*, which is too
  many to read; and exhaustive enumeration "will not be practical" for longer melodies, which is the argument for
  [§2.7](#27-where-a-solver-takes-over-from-the-dp)'s solver. Ebcioğlu had already put the first one more bluntly — in music generation *"the list of all
  solutions is of impractical length and is quite boring."*
- **The rules are stipulated, not derived.** This is the real methodological cost, and it is a genuine loss against
  ricercar. Plomp–Levelt *derives* consonance: [§7.1](ricercar/readme.md#71-step-1-result-go) found interior minima at 316, 386, 498, 702 and 884 cents —
  the minor third, major third, fourth, fifth and major sixth — falling out of summed partial pairs rather than
  being put in by hand. An automaton transcribed from Fux has consonance **stipulated in its alphabet**. It is
  transcription of an explicit theory rather than fitting to data, which is what "elegant, not fitted" asks for,
  but it is not derivation and should not be described as such.
- **A style, and a caricature of one.** Fux is not Bach, and Bach breaks Fux constantly. Whose rulebook goes into
  the automaton is an arguable, inspectable modelling choice — better than an unarguable one, but still a choice,
  and the output is bounded by it. **Both papers demonstrate the cost on themselves.** Komosinski & Szachewicz
  print a Pareto-optimal counterpoint and note in its own caption that Fux would forbid it, for a chromatic half
  step their rule set omitted. And Schottstaedt — five species, up to eight voices, the most ambitious of its
  generation — closes his report with a list of what the program does not do, which is more damning than the
  survey's secondhand verdict and should be quoted instead of it: it *"has no provision for starting a melody with
  a rest, nor does it reward invertible counterpoint and imitation. It tends to let voices get entangled in each
  other, and makes no decisions about overall melodic shapes."*

  And the rhythm, in fifth species, is not composed at all: *"we just load up an array with the legal rhythmic
  patterns and choose among them **randomly**. This approach obviously leaves much to be desired. Musical styles
  are differentiated more by rhythmic practices than melodic."* A system can satisfy every rule it was given and
  still be choosing its rhythm by coin flip. **That is the failure to expect here** — not illegal output, but legal
  output that is empty where the style lives — and [§8](#9-roadmap)'s step 1 is written to catch it early.

  Two of his omissions are pointed. *"Does not reward invertible counterpoint and imitation"* is exactly the fugal
  content this document is about; *"makes no decisions about overall melodic shapes"* is [§2.5](#25-the-search-is-a-shortest-path)'s accumulator
  boundary, reported from the far side by someone who implemented the accumulators.
- **Infeasibility is real and is not always a bug.** Komosinski & Szachewicz found *cantus firmi* for which **no**
  counterpoint satisfies even their two hard rules — the legal set is empty, not small. A complete method reports
  that as a proof rather than as a timeout, which is the right behaviour, but it means "no solution" will
  sometimes be the honest answer to a musically reasonable request.
- **Melodic invention.** The subject is input. [§3.2](#32-capacity-is-a-density-and-it-cannot-be-optimised) makes designing one cheaper, but designing for *capacity* is
  not designing for interest.
- **Robustness.** See [§6](#6-what-ricercar-still-owns).
- **Performance.** Expressive timing, dynamics, ornamentation, articulation. The output is a score, not a
  performance.

---

## 6. What ricercar still owns

Not superseded — pointed at a different question, which is the thing the project conflated.

- **Robustness under continuous perturbation.** *"This texture is legal under any tuning within ±20 cents and any
  micro-timing within ±15 ms"* is irreducibly a continuous statement, it is candidate (1) of ricercar [§3](ricercar/readme.md#3-where-the-lipschitz-property-lives-and-where-it-does-not), and no
  lattice method can produce it. The Lipschitz certificate is the right instrument and this document has nothing
  to say about it.
- **Free canon.** Continuous delay and continuous interval — ricercar [§3](ricercar/readme.md#3-where-the-lipschitz-property-lives-and-where-it-does-not)'s candidate (2). Genuinely a continuum,
  genuinely self-similar, and genuinely not a fugue.
- **A derived model of consonance**, per [§5](#5-what-this-will-not-do) above.

The honest summary is that ricercar answers the robustness question well and the fugue question badly, and that
the two were not distinguished when the domain was chosen.

---

## 7. Prior art

None of this is novel, and that is a feature — the components are known-good and the risk sits in the composition
rather than in the parts. Rows marked ✔ are in [`literature/`](literature/) and were read in full; the rest are cited from those five.
**Every DOI here was resolved against Crossref**, and the two secondary citations that could not be (Schottstaedt's
technical report and Vuza's four-part article) are marked as having none rather than given a plausible-looking one.

| source | identifier | what it gives |
|---|---|---|
| ✔ Anders & Miranda, "Constraint Programming Systems for Modeling Music Theories and Composition", *ACM Comput. Surv.* **43**(4):30, 2011 | [10.1145/1978802.1978809](https://doi.org/10.1145/1978802.1978809) | the survey to read first — music CP end to end, and the source for most rows below |
| ✔ Komosinski & Szachewicz, "Automatic species counterpoint composition by means of the dominance relation", *J. Math. & Music* **9**(1):75–94, 2015 | [10.1080/17459737.2014.935816](https://doi.org/10.1080/17459737.2014.935816) | first-species counterpoint by the **dominance relation** — the argument against weighted sums, and [§5](#5-what-this-will-not-do)'s numbers |
| ✔ Giraud, Groult, Leguy & Levé, "Computational Fugue Analysis", *Computer Music Journal* **39**(2):77–96, 2015 | [10.1162/COMJ_a_00300](https://doi.org/10.1162/COMJ_a_00300) | fugue **analysis**, and the ground-truth corpus [§8](#9-roadmap) now uses |
| ✔ Schottstaedt, *Automatic Species Counterpoint*, CCRMA Report STAN-M-19, Stanford, May 1984 | no DOI — [ccrma.stanford.edu/STANM/stanms/stanm19](https://ccrma.stanford.edu/STANM/stanms/stanm19/) | Fux, five species, up to eight voices, stratified penalties — the closest prior attempt at [§2.7](#27-where-a-solver-takes-over-from-the-dp)'s scale, printed as complete source, and the most useful negative result here |
| ✔ Ebcioğlu, "An Expert System for Harmonizing Chorales in the Style of J. S. Bach", *J. Logic Programming* **8**(1):145–185, 1990 | [10.1016/0743-1066(90)90055-A](https://doi.org/10.1016/0743-1066(90)90055-A) | ~350 rules in first-order predicate calculus, generate-and-test with **intelligent backtracking**, in a language (BSL) built because PROLOG would not do — the argument for factoring a rulebook rather than listing it |
| Pesant, "A Regular Language Membership Constraint for Finite Sequences of Variables", *CP 2004*, LNCS **3258**:482–495 | [10.1007/978-3-540-30201-8_36](https://doi.org/10.1007/978-3-540-30201-8_36) | the domain-consistent DFA-membership propagator of [§2.5](#25-the-search-is-a-shortest-path) |
| Boenn, Brain, De Vos & Ffitch, "Automatic music composition using answer set programming", *Theory and Practice of Logic Programming* **11**(2–3):397–427, 2011 | [10.1017/S1471068410000530](https://doi.org/10.1017/S1471068410000530) | the same programme in answer-set programming, which may be the most elegant surface syntax available for it |
| Coven & Meyerowitz, "Tiling the Integers with Translates of One Finite Set", *J. Algebra* **212**(1):161–174, 1999 | [10.1006/jabr.1998.7628](https://doi.org/10.1006/jabr.1998.7628) | the tiling conditions behind [§4](#4-space-filling-in-the-right-category) |
| Hiller & Isaacson, *Experimental Music: Composition with an Electronic Computer* (1959); the *Illiac Suite*, 1957 | — | rule-based counterpoint by generate-and-reject; the field starts here |
| Ebcioğlu (1980), two-part florid counterpoint | — | ~50 constraints, including the windowed melodic-peak rule that refines [§2.5](#25-the-search-is-a-shortest-path). A 16th-century strict-counterpoint program preceded CHORAL and supplied its search method |
| Laurson, PWConstraints / Score-PMC (1996); Anders, Strasheela (2007) | — | the two ends of the design space: fixed rhythm with a fast static ordering, versus arbitrary score topology |
| Vuza, "Supplementary Sets and Regular Complementary Unending Canons", *Perspectives of New Music*, 1991–93; Andreatta, Amiot, Agon | — | tiling rhythmic canons — [§4](#4-space-filling-in-the-right-category) |
| Fux, *Gradus ad Parnassum* (1725) | — | the rulebook itself, and — per [§8.2](#82-the-rulebook-stratified-by-two-corpora) — a book about a repertoire this project mostly did not test it on |

Deliberately excluded: Cope's EMI and everything downstream of it. Recombinant methods are fitted to a corpus by
construction, which is the constraint this document was written under.

**Two things the survey says that bear directly on [§2.3](#23-harmony-is-a-second-automaton) and [§2.4](#24-form-is-a-grammar).** Its conclusion names the gaps: *"Other
neglected fields include harmonic counterpoint, and the modeling of melody and musical form."* And, more precisely,
*"no system supports that the hierarchic structure of the score can be constrained freely, but such a feature would
be highly useful for modeling musical form."* The harmonic automaton and the form grammar are therefore **not**
reinventions — they are the two things this literature reports as missing. That is the strongest reason to think
the composition is worth attempting even though every part is off the shelf.

**And a calibration on speed**, from the same conclusion: an all-interval series or first-species Fuxian
counterpoint solves in milliseconds; harmonising a melody or **two-voice florid counterpoint takes seconds**. So
[§8](#9-roadmap)'s realisation step should be budgeted in seconds for two voices, and five voices should be treated as genuinely
open rather than as more of the same.

**One methodological note, from Giraud.** Discussing why they did not learn their thresholds: machine learning
*"could improve the thresholds and weights of these models, but strategies have to be designed to address the
problem of overfitting, a concern for data sets as small as these are prone."* Thirty-six fugues is not a corpus
you can fit anything to. The no-fitting constraint this document was written under has an empirical justification
as well as an aesthetic one.

---

## 8. What is built, and what it measures

`cargo run --release` in this directory. Everything below is the current state; how it was arrived at, including
four claims that did not survive the experiment after the one that produced them, is in
[`CHANGELOG.md`](CHANGELOG.md).

Pitch is a diatonic step with an alteration rather than a semitone integer, because a diminished fifth and an
augmented fourth are the same six semitones and different intervals. Time is in ticks of 1/960 of a whole note —
the smallest base making every duration in both corpora exact, dotted values and Renaissance coloration included.
No rounding anywhere.

### 8.1 The automaton

| | |
|---|---:|
| alphabet | 1600 |
| crude product of the state components | 1280 |
| **reachable states** | **513** |
| distinct obligation sets | 128 of 256 |
| hard rules / soft criteria | 2 / 9 |

All three verdict tests pass, including the two ricercar could not state at all. Parallel fifths are flagged. A
bare fifth is consonant — the roughness field measured it at `0.089`, among the least rough intervals there are,
which is why [§7.2](ricercar/readme.md#72-step-2-result-a-proof-not-a-sample) of that document had to substitute a
different test. And a 7–6 suspension is accepted where the same seventh, leapt into on the same beat, is rejected:
**the same instantaneous interval, distinguished by the path taken to it**, which is what a field over
instantaneous pitch cannot do.

### 8.2 The rulebook, stratified by two corpora

24 Bach fugues (114 voice pairs, 34 987 slices, 24 013 melodic moves) against 200 works of 15th-century polyphony
(299 613 slices) — Busnois, Dufay and Josquin, which is what the first 200 files in path order actually are.

| rule | Renaissance | Bach | reading |
|---|---:|---:|---|
| parallel perfect | 1.2 | **1.0** | **universal** — two centuries, two media |
| direct to perfect on downbeat | 1.5 | **0.7** | **universal** |
| forbidden melodic interval | **1.0** | 37.6 | **repertoire-specific**, ×38 — a correct rule about Renaissance vocal writing, applied to keyboard music |
| unprepared dissonance | 8.0 | 21.4 | fails in both centuries |
| unresolved dissonance | 71.1 | 90.9 | fails in both centuries |

Per thousand slices, or per thousand melodic moves for the melodic rule.

**Two rules are confirmed by both corpora**, and they are precisely the two a roughness field cannot express, since
a perfect fifth is among the smoothest intervals it knows. The part of the rulebook that most justified abandoning
the continuum is the part that survives contact with the music.

**The melodic rule is not refuted, it was mis-applied.** One violation per thousand moves in the repertoire Fux is
writing about. **The two dissonance rules fail in the very repertoire they were written for**, so they are
implementation faults rather than a repertoire mismatch, and they sit in the soft tier pending a replacement.

A control on the obvious confound: chromaticism explains 6% of the melodic rule's variance across fugues
(r = +0.249), so the difference is repertoire and medium rather than chromatic writing piece by piece.

### 8.3 The clique test

Bach's five final entries in BWV 867 stand at quarters `{266, 268, 270, 272, 274}` — `{0, 2, 4, 6, 8}` from the
first, one per voice. The transpositions are recovered from the score rather than assumed, and come out
`B♭ – F – B♭ – F – B♭` descending across five octaves, tonic and dominant alternating.

| subject reading | full 5-rule tier | 2-rule tier |
|---|---|---|
| 3 measures (Keller, Bruhn) | fail — max clique 4 of 5 | **pass — 5 of 5** |
| 2 measures (Prout, Bruhn) | fail — max clique 4 of 5 | **pass — 5 of 5** |

Both failures are the same rule, `unresolved dissonance`, on the pair `+0q` against `+6q`.

**A control on the written notes** rather than idealised transpositions, over the same bars: ten real voice pairs
give **15** violations on the full tier and **1** on the two-rule tier. The real passage fails the full tier by more
than the template does, so the fault is the rulebook's rather than the model of an entry. The single remaining
violation is a direct motion to a perfect consonance on a downbeat between the two middle voices — present in Bach,
absent from the idealisation.

### 8.4 Capacity ranks subjects, and cannot design one

Capacity is the **edge density** of the compatibility graph under the two-rule tier, over every diatonic
transposition `−7…+7` at every quarter-note offset within the subject.

| | |
|---|---|
| spread across 24 subjects | **0.321 … 0.956** |
| first | **BWV 849** (0.956) — the fugue musicians name when they name a stretto fugue |
| last three | BWV 860, 866, 865 |
| correlation with note density | **r = −0.311** |

Clique *size* is not used: under the same tier it does not converge, because 81% of entry pairs are compatible.
Under the strict five-rule tier it does converge and ranks BWV 849 first as well — but that tier is one Bach
violates, and its ranking correlates −0.750 with note density, so density under the confirmed tier is both more
defensible and less contaminated.

**Against the contested subject endings** ([§3.3](#33-the-subject-is-input-and-its-boundary-is-contested)),
capacity is not a well-behaved function of subject length: BWV 852 reads 4 or 2 depending on whether one follows
Keller or Prout, and BWV 854 goes 2 → 4 → 3 as the subject lengthens.

**As a design objective it fails**, for the structural reason in
[§3.2](#32-capacity-is-a-density-and-it-cannot-be-optimised): the unconstrained optimum is a monotone, which beats
Bach on 20 of 20 rhythms, and Bach's own contours score below random on their own rhythms (5 of 20, mean −0.0763).
Constraining the search to ≥5 distinct degrees replaces the monotone with a jagged leaping contour, mean melodic
step 3.78 — differently unmusical, not less.

### 8.5 The harmonic analyser

`src/harmony.rs`. Segments at every onset and chooses a chord path by Viterbi over 108 chords, charging a penalty
`λ` to change chord, so the harmonic rhythm emerges rather than being imposed by a window. Chords lose weight for
foreign notes rather than merely not gaining it; weight is duration doubled on beats; the bass earns a bonus for
being the root. Linear rather than quadratic in the chord vocabulary, because the transition cost is zero to stay
and `λ` to move.

Validated against the **106 typed cadence annotations** in the ground truth — the only external check available.
The penalty is swept rather than fitted:

| λ | arrival chord correct | + preceded by dominant | chance | harmonic rhythm |
|---:|---:|---:|---:|---:|
| 0.0 | **80%** | 71% | 14% | 102t |
| 0.3 | 77% | 71% | 15% | 167t |
| 1.0 | 70% | 59% | 15% | 338t |
| 2.0 | 48% | 25% | 15% | 686t |

A quarter note is 240 ticks, so λ = 1.0 gives a chord change about every 1.4 quarters, which is roughly right for
these pieces; accuracy across the musically plausible band is **70–80%**. Held out on odd- and even-numbered
fugues, the same λ is chosen either way and the accuracy transfers — 79% → 82%, 82% → 79%.

**What it is for.** An analyser infers harmony from notes; a generator does not have to, and can take the harmonic
plan as input from the form grammar. So this is the instrument that will **judge** a realiser's output rather than
the model a realiser writes against.

**What it is not.** It fits Renaissance polyphony slightly better than Bach on every chord-fit statistic (mean fit
+0.022, chord tones +1.6 points). That is probably a fact about the music — Renaissance polyphony really is more
triadic — but it means chord fit does not separate tonal from modal, and no claim about tonality should rest on it.

---

## 9. Roadmap

Steps 0 to 4 are done and reported above. What remains, in order.

5. **Realisation, and the first audible output.** Viterbi fill of the free voices against a **given** harmony — the
   plan supplies it, per [§8.5](#85-the-harmonic-analyser) — then MIDI. Escalate to a SAT/CDCL solver at four or
   more free voices per [§2.7](#27-where-a-solver-takes-over-from-the-dp), and do not layer: Schottstaedt reports
   that failing at three voices. Neither this document nor ricercar has produced a note.

   Expect the difficulty to be selectivity rather than legality. A two-rule hard tier and a permissive harmonic
   constraint will admit enormous numbers of legal fills, which is [§5](#5-what-this-will-not-do)'s inverted
   failure mode arriving where it finally matters. No further measurement resolves it; at some point the output
   has to be listened to, and that is the one test this project has never run.

6. **Form**, per [§2.4](#24-form-is-a-grammar) — which Anders & Miranda name as unsupported by any existing
   system, so expect to build rather than borrow. The packing question lives inside the stretto block.

7. Optional: **double fugue** — two shapes that must tile, which is where the shape-catalogue reading earns its
   keep.

### Open problems, in rough order of how much they block

- **Key-finding.** A real functional test needs degree successions relative to a *local* key, and fugues modulate
  constantly. Without it [§2.3](#23-harmony-is-a-second-automaton)'s functional half cannot be built or tested.
- **A replacement for the two dissonance rules**, which fail in both centuries ([§8.2](#82-the-rulebook-stratified-by-two-corpora)).
- **A design objective**, still open after two attempts ([§3.2](#32-capacity-is-a-density-and-it-cannot-be-optimised)).
  It has to reward a subject working at the fifth, which is a harmonic statement.
- **The right rulebook for the right repertoire.** Fux is 1725 and Palestrina-style vocal; the WTC is 1722 and
  keyboard. **Marpurg's *Abhandlung von der Fuge*** (1753) is the fugue treatise of Bach's own circle and
  **Kirnberger** studied with him directly. Transcribing either is exactly as unfitted as transcribing Fux — it is
  transcribing the right explicit theory.
- **The Shostakovich half of the ground truth**, which needs Marques' MIDI rather than kern.

---

## 10. Reproducing the results

Everything is deterministic. No sampling that is not seeded, and no threshold that was chosen rather than measured
or swept.

### 10.1 Environment and data

```
rustc 1.96.1   cargo 1.96.1     # no dependencies; std only
git clone --recurse-submodules <this repo>
cargo test --release            # 7 tests
```

| submodule | pinned | licence | used by |
|---|---|---|---|
| `corpus/algomus-data` | `a1801b5` | ODbL 1.0, contents DbCL 1.0 | subject positions, lengths, cadences |
| `corpus/bach-wtc-fugues` | `5095752` | Humdrum edition, David Huron | every Bach figure |
| `corpus/jrp-scores` | `52de715` | **CC BY-NC 4.0** | the Renaissance control |

The JRP licence is non-commercial; the other two are not. Attribute Giraud, Groult and Levé for the annotations and
the Josquin Research Project for the Renaissance scores.

### 10.2 Which command produces which section

| section | command |
|---|---|
| [§8.1](#81-the-automaton) state count | `cargo run --release -- states` |
| [§8.1](#81-the-automaton) verdict tests | `cargo run --release -- verdict` |
| [§8.2](#82-the-rulebook-stratified-by-two-corpora) Bach rates | `cargo run --release -- corpus`, and `diag` for the melodic breakdown |
| [§8.2](#82-the-rulebook-stratified-by-two-corpora) Renaissance | `cargo run --release -- exp3` |
| [§8.2](#82-the-rulebook-stratified-by-two-corpora) chromaticism | `cargo run --release -- exp4` |
| [§8.3](#83-the-clique-test) clique test | `cargo run --release -- stretto` |
| [§8.4](#84-capacity-ranks-subjects-and-cannot-design-one) density ranking | `cargo run --release -- exp1` |
| [§8.4](#84-capacity-ranks-subjects-and-cannot-design-one) design | `cargo run --release -- design` |
| [§8.5](#85-the-harmonic-analyser) sweep and hold-out | `cargo run --release -- sweep`, `holdout` |

`rank`, `probe`, `exp2`, `exp5`, `harmony`, `cad`, `seg`, `revisit`, `hren2` and `func` reproduce the superseded
measurements recorded in [`CHANGELOG.md`](CHANGELOG.md).

### 10.3 Parameters

| | |
|---|---|
| tick base | 960 per whole note ([`kern.rs`](src/kern.rs)) |
| hard tier | `ParallelPerfect`, `DirectPerfectOnDownbeat` ([`automaton.rs`](src/automaton.rs)) |
| candidate grid | offsets every quarter within the subject; diatonic transpositions −7…+7; one entry per offset |
| design grid | offsets every half note, same transposition range |
| harmonic analyser | onset segmentation, 9 qualities × 12 roots, bass bonus 0.2, strong-beat weight ×2 |
| PRNG | SplitMix64 inline; seeds `0x5EED`, `0xC0FFEE`, `0xBEEF`, `0xF00D`, `0xD00D` |
| trials | 400 random contours for single-subject figures, 60 per subject for corpus tables |
| hill-climbing | 12–16 restarts, first improvement accepted, one note changed at a time |

### 10.4 How the samples were taken

**The Renaissance sample is not the six composers the code appears to select.** It globs six directories, sorts by
path and truncates at 200, so it is **70 Busnois, 37 Dufay and 93 Josquin** — Ockeghem, Obrecht and La Rue never
enter.

**The Bach corpus is Book I only**, `wtc1f01`–`wtc1f24`; the submodule holds both books but the annotations cover
Book I.

**The design tables cover 20 of 24 subjects.** Subjects longer than 24 notes are skipped for search cost, which
excludes BWV 855, 860, 865 and 866 — the four densest, and the four least stretto-friendly.

**Timings are from one machine** and are quoted only where they carry an argument. They are not benchmarks.

### 10.5 What is not reproducible from this repository

- **The literature.** [`literature/`](literature/) holds five PDFs under their publishers' terms;
  [§7](#7-prior-art) gives DOIs so each can be obtained independently.
- **The Shostakovich annotations**, which have no scores here.
- **The functional-harmony layer**, which compiles but is not exercised by any reported number.
