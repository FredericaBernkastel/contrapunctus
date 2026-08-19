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
  - [7.1 Parallels within the same algorithmic family](#71-parallels-within-the-same-algorithmic-family)
- [8. What is built, and what it measures](#8-what-is-built-and-what-it-measures)
  - [8.1 The automaton](#81-the-automaton)
  - [8.2 The rulebook, stratified by two corpora](#82-the-rulebook-stratified-by-two-corpora)
  - [8.3 The clique test](#83-the-clique-test)
  - [8.4 Capacity ranks subjects, and cannot design one](#84-capacity-ranks-subjects-and-cannot-design-one)
  - [8.5 The harmonic analyser](#85-the-harmonic-analyser)
  - [8.6 Realisation, and the first notes](#86-realisation-and-the-first-notes)
  - [8.7 The species as a whitelist, and why it does not tighten anything](#87-the-species-as-a-whitelist-and-why-it-does-not-tighten-anything)
  - [8.8 A criterion that is not local, and the shape of every step-6 failure](#88-a-criterion-that-is-not-local-and-the-shape-of-every-step-6-failure)
  - [8.9 A better plan, and the first lever that moves more than a point](#89-a-better-plan-and-the-first-lever-that-moves-more-than-a-point)
  - [8.10 Replacing the soft tier, and the degenerate optimum of every positive criterion](#810-replacing-the-soft-tier-and-the-degenerate-optimum-of-every-positive-criterion)
  - [8.11 Marpurg's tonal answer: one rule exact, one wrong, and the treatise knew which](#811-marpurgs-tonal-answer-one-rule-exact-one-wrong-and-the-treatise-knew-which)
  - [8.12 The fourth, and the scope a dissonance is judged in](#812-the-fourth-and-the-scope-a-dissonance-is-judged-in)
  - [8.13 Are episodes sequences, and how much of a fugue is episode](#813-are-episodes-sequences-and-how-much-of-a-fugue-is-episode)
  - [8.14 Key-finding, and the ground truth that was already here](#814-key-finding-and-the-ground-truth-that-was-already-here)
  - [8.15 Does the form grammar derive the book?](#815-does-the-form-grammar-derive-the-book)
  - [8.16 A fugue, from a subject](#816-a-fugue-from-a-subject)
- [9. Roadmap](#9-roadmap)
- [10. Reproducing the results](#10-reproducing-the-results)
  - [10.1 Environment and data](#101-environment-and-data)
  - [10.2 Which command produces which section](#102-which-command-produces-which-section)
  - [10.3 Parameters](#103-parameters)
  - [10.4 How the samples were taken](#104-how-the-samples-were-taken)
  - [10.5 What is not reproducible from this repository](#105-what-is-not-reproducible-from-this-repository)
  - [10.6 Using it as a library](#106-using-it-as-a-library)
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

The realiser then produces notes, and with them the method's real difficulty, which is the inverse of the usual
one: **a complete search does not fail by finding nothing but by finding far too much.** Holding one of Bach's own
subject entries fixed, taking his rhythm and a harmonic plan derived only from the voices the search is not allowed
to see, and filling the rest exactly, the rulebook leaves seven to seventeen pitches open at every note and between
`10¹²` and `10¹⁸` complete legal fills of a three-bar span. Across every combination of rulebook and plan,
**agreement with what Bach wrote barely moves while the chance baseline nearly triples** — so the constraints are
doing all of the work and the objective almost none. Two controls place it: reversing the sign of the objective,
and drawing from the legal set **uniformly** instead of optimising over it at all, which the search can do exactly
because it already counts the paths through its own DAG. Minimising scores 7.8%, a uniform draw 6.9%, maximising
4.9% — so the soft tier is weak, real, and **better than not optimising**, which is the opposite of what the
per-note baseline had suggested before a like-for-like control existed. Where taste enters is therefore the central
problem rather than an afterthought, and the position
taken here is the **Pareto front** over soft criteria rather than a weighted sum, on the ground that no weighting
in the literature is defensible and Fux declines to supply one.

Two further results fall out of building it. The melodic prohibition is **repertoire-specific as a description and
load-bearing as a constraint** — the corpus stratified it out of the hard tier for flagging Bach thirty-eight times
more often than the Renaissance, yet without it nothing bounds a free voice's line at all, and adding it back
halves the pitches left open at every note. And the exact search meets its wall at **two** free voices rather than
the predicted four, because the multiplier is the compounding obligation state rather than the product of pitch
domains — an argument for conflict-learning search that is stronger and arrives earlier than the one this document
set out with.

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

**The generative half is built too**, and the claim in the first paragraph — that the two automata compose — is now
a fact about running code rather than a reading of the literature. [§8.6](#86-realisation-and-the-first-notes)'s
realiser runs a chord-membership obligation system beside [§2.2](#22-counterpoint-is-a-finite-automaton)'s over one
grid: a note foreign to the prevailing chord must be prepared or approached by step, and owes a resolution on the
next articulation. The two systems needed no knowledge of each other, and turning the harmonic one *off* takes the
number of spans the exact search can finish from 83 of 117 to 36. **Harmony is not a refinement of the
counterpoint constraint; it is most of the constraint.**

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
domains: with a two-octave compass that is roughly `24^(V−e)` before obligations, so `V − e = 2` looks comfortable,
`3` wants the harmonic automaton pruning it, and `4` or more is out of reach exactly.

**That arithmetic is wrong in an instructive way, and [§8.6](#86-realisation-and-the-first-notes) measures how.**
The phrase doing the damage is *"before obligations"*. Two free voices give a few hundred pitch pairs; the built
search reaches tens of thousands of live states on the same spans, because a dissonance owed in one pair and a leap
owed in another are independent bits that compound across every pair at once. **The wall is at two free voices, and
the multiplier is the obligation set rather than the pitch product.** Everything below is therefore an
understatement of the case for a solver rather than an overstatement, which is the direction an argument should err
in.

**Schottstaedt reached exactly this wall in 1984 and his report is the best evidence in the literature that it is
real.** Read directly rather than through the survey, it says four things that bear on the design here:

- His stated goal was *"five to eight part mixed species counterpoint"* — the same target [§8.6](#86-realisation-and-the-first-notes) sets out for.
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

Here there is no threshold. The calibration becomes a **yes-or-no test**, and since [§9](#9-roadmap)'s step 0 the target is an
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

  **And measured here**, on this rulebook rather than theirs, in [§8.6](#86-realisation-and-the-first-notes):
  `10¹²` to `10¹⁸` legal fills of a three-bar span of a Bach fugue, and agreement with what Bach wrote that stays
  put however much of the rulebook is switched on. The paragraph below is not a caution about a future difficulty.
  It is the current one, and the numbers are ten orders of magnitude worse than the ones that prompted it.

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
  output that is empty where the style lives — and [§9](#9-roadmap)'s step 1 is written to catch it early.

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
rather than in the parts. Rows marked ✔ are in [`literature/`](literature/) and were read in full; the music rows
below them are cited from those five — except Marpurg, which is a primary source this project has **located and
not transcribed**, and says so — and the last seven, which are not about music at all, from the
WaveFunctionCollapse README and its own bibliography, for the reasons
[§7.1](#71-parallels-within-the-same-algorithmic-family) gives. **Every DOI here was resolved against Crossref**,
and the three citations that have none (Schottstaedt's technical report, Vuza's four-part article, and Gumin's
repository) say so rather than carry a plausible-looking one.

| source | identifier | what it gives |
|---|---|---|
| ✔ Anders & Miranda, "Constraint Programming Systems for Modeling Music Theories and Composition", *ACM Comput. Surv.* **43**(4):30, 2011 | [10.1145/1978802.1978809](https://doi.org/10.1145/1978802.1978809) | the survey to read first — music CP end to end, and the source for most rows below |
| ✔ Komosinski & Szachewicz, "Automatic species counterpoint composition by means of the dominance relation", *J. Math. & Music* **9**(1):75–94, 2015 | [10.1080/17459737.2014.935816](https://doi.org/10.1080/17459737.2014.935816) | first-species counterpoint by the **dominance relation** — the argument against weighted sums, and [§5](#5-what-this-will-not-do)'s numbers |
| ✔ Giraud, Groult, Leguy & Levé, "Computational Fugue Analysis", *Computer Music Journal* **39**(2):77–96, 2015 | [10.1162/COMJ_a_00300](https://doi.org/10.1162/COMJ_a_00300) | fugue **analysis**, and the ground-truth corpus [§9](#9-roadmap) now uses |
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
| Marpurg, *Abhandlung von der Fuge* (1753; new edition 1806) — **located, not read** | no DOI — [archive.org/details/abhandlungvonder00marp](https://archive.org/details/abhandlungvonder00marp) | the fugue treatise of Bach's own circle, and the answer to [§9](#9-roadmap)'s standing question about which rulebook fits the WTC. What it holds that this project does not: the **tonal answer** as a table of degree correspondences, **invertible counterpoint** at three and four parts, and the **repercussion**. Scans are not tracked here — see [§10.5](#105-what-is-not-reproducible-from-this-repository) |
| Gumin, *WaveFunctionCollapse*, 2016 | no DOI — [github.com/mxgmn/WaveFunctionCollapse](https://github.com/mxgmn/WaveFunctionCollapse) | the whitelist/blacklist contrast of [§7.1](#c1-is-a-whitelist-and-fux-is-a-blacklist), and Weak C2 |
| Karth & Smith, "WaveFunctionCollapse is Constraint Solving in the Wild", *FDG 2017* | [10.1145/3102071.3110566](https://doi.org/10.1145/3102071.3110566) | the CSP reading made explicit, with backtracking and global constraints |
| Karth & Smith, "WaveFunctionCollapse: Content Generation via Constraint Solving and Machine Learning", *IEEE Trans. Games* **14**(3):364–376, 2022 | [10.1109/TG.2021.3076368](https://doi.org/10.1109/TG.2021.3076368) | the same argument at journal length |
| Merrell, "Example-Based Model Synthesis", *I3D 2007*, 105–112 | [10.1145/1230100.1230119](https://doi.org/10.1145/1230100.1230119) | the predecessor WFC generalises; adjacency by AC-3 |
| Mackworth, "Consistency in Networks of Relations", *Artificial Intelligence* **8**(1):99–118, 1977 | [10.1016/0004-3702(77)90007-8](https://doi.org/10.1016/0004-3702(77)90007-8) | arc consistency |
| Mohr & Henderson, "Arc and Path Consistency Revisited", *Artificial Intelligence* **28**(2):225–233, 1986 | [10.1016/0004-3702(86)90083-4](https://doi.org/10.1016/0004-3702(86)90083-4) | AC-4, the propagator WFC uses |
| Efros & Leung, "Texture Synthesis by Non-parametric Sampling", *ICCV 1999*, 1033–1038 | [10.1109/ICCV.1999.790383](https://doi.org/10.1109/ICCV.1999.790383) | the texture-synthesis line WFC descends from |

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
[§9](#9-roadmap)'s realisation step should be budgeted in seconds for two voices, and five voices should be treated as genuinely
open rather than as more of the same.

**One methodological note, from Giraud.** Discussing why they did not learn their thresholds: machine learning
*"could improve the thresholds and weights of these models, but strategies have to be designed to address the
problem of overfitting, a concern for data sets as small as these are prone."* Thirty-six fugues is not a corpus
you can fit anything to. The no-fitting constraint this document was written under has an empirical justification
as well as an aesthetic one.

### 7.1 Parallels within the same algorithmic family

**[WaveFunctionCollapse](https://github.com/mxgmn/WaveFunctionCollapse)** (Gumin, 2016) synthesises images: given a
small bitmap it produces larger ones locally indistinguishable from it. It is the same object as this document in a
different category, it was arrived at independently and from the opposite direction, and **one difference between
the two explains [§8.6](#86-realisation-and-the-first-notes)'s central number in a sentence**. Its own README says
as much in the vocabulary used here — *"WFC translates a texture synthesis problem into a constraint satisfaction
problem"*, and *"the overlapping model relates to the simple tiled model the same way higher order Markov chains
relate to order one Markov chains"*, which is [§2.2](#22-counterpoint-is-a-finite-automaton)'s bounded-order
automaton stated for pixels. Gumin also notes that *"one of the dimensions can be time"*, and the ports list
includes a piano-roll application.

| WaveFunctionCollapse | this document |
|---|---|
| cell of the output grid | slice on the tick lattice ([§2.6](#26-what-is-not-a-variable-rhythm)) |
| `N × N` pattern | order-*N* window — the automaton state, order ≤ 3 ([§2.2](#22-counterpoint-is-a-finite-automaton)) |
| adjacency data | the compatibility table ([§3](#3-stretto-capacity-and-the-subject), [`stretto.rs`](src/stretto.rs)) |
| the *wave*: a superposition per cell | the live state set of a DP layer ([§8.6](#86-realisation-and-the-first-notes)) |
| propagation, by AC-4 | forward propagation along the layered DAG |
| *observe*: collapse the minimal-entropy cell | **nothing** — the DP is exact and left to right, so it needs no variable ordering |
| contradiction, then restart | the `dead` column of [§8.6](#86-realisation-and-the-first-notes) |
| **(C1)** *"the output should contain only those `N×N` patterns of pixels that are present in the input"* | the hard tier: only the transitions the automaton permits |
| **(Weak C2)**, the distribution condition | **nothing whatever** — and that is the finding |

#### C1 is a whitelist and Fux is a blacklist

WFC's constraint says *only these configurations may occur*. This project's says *these five things may not*. A
whitelist drawn from a real artefact is enormously tighter than a handful of prohibitions, and that difference is
the whole of `10¹⁵` legal fills of three bars.

It also inverts the failure mode, which is the clue that the difference is structural rather than one of degree.
Gumin's practical problem is running out of options: *"it may happen that during propagation all the coefficients
for a certain pixel become zero"*, and the algorithm restarts. The search here has **never once failed for being
over-constrained** — across every row of [§8.6](#86-realisation-and-the-first-notes) the `dead` column tops out at
eight of 117, while `refused`, which counts searches abandoned for having too many states, reaches 81.

**The unfitted route to a whitelist is already in the source material.** Species counterpoint *is* an enumeration:
Fux sets out the permitted note-against-note configurations species by species, and this project transcribed the
prohibitions while leaving the enumeration on the table. Transcribing the species as permitted figures is exactly
as unfitted as transcribing the prohibitions — it is the same book — and it is the structural change most likely to
move `10¹⁵` toward a number at which choosing means anything.

#### Weak C2 answers §8.6's question, and answers it by not optimising

[§8.6](#86-realisation-and-the-first-notes) measured that minimising the soft criteria scores 7.8%, maximising them
4.9%, against a per-note baseline of **16.2%**, and reported that as an open problem.

WFC does not have the problem, because it never forms an objective. Its answer to *which of the many legal outputs*
is **(Weak C2)**: *"probability to meet a particular pattern in the output should be close to the density of such
patterns in the input"*, implemented as *"collapse this element into a definite state according to its coefficients
and the distribution of `N×N` patterns in the input."* Sample proportionally; aim to be **typical** rather than
optimal.

**That prediction was testable here and it failed**, which is the more useful outcome. Drawing uniformly from the
legal set — Weak C2 with the corpus half removed — scores **6.9%**, below the 7.8% of the objective it was meant to
improve on ([§8.6](#86-realisation-and-the-first-notes)). Typical does not beat extremal in this repertoire. What
the exercise bought instead was the honest baseline: the 16.2% figure turns out to be an artefact of handing the
scorer Bach's own preceding note, and a generator that has to live with its own mistakes gets 6.9%.

Two versions of that are available and only one is permitted by this project's founding constraint.

- **Uniform sampling from the legal set** asserts nothing and needs no data. It is also nearly built: the search
  computes exact path counts through the DAG, checked against brute-force enumeration
  ([§8.6](#86-realisation-and-the-first-notes)), so drawing a uniformly random legal fill is a backward walk
  weighting each predecessor by its count. That turns the 16.2% column from a baseline into a generator.
- **Frequency weighting proper** needs frequencies. Taking them from a corpus is what [§0](#0-where-this-comes-from)
  rules out; taking them from a treatise is not, since Fux states preferences — imperfect consonances over perfect,
  and so on — and transcribing a stated preference is not fitting.

WFC's *overlapping* model learns both C1 and Weak C2 from a bitmap and is therefore fitted by construction. Its
**simple tiled** model is not: *"it's convenient to initialize the simple tiled model with a list of tiles and their
adjacency data"*, authored by hand. Structurally this project is already the simple tiled model, with the adjacency
table transcribed from treatises instead of drawn by an artist.

#### Where the analogy breaks, and in which direction

**Time's arrow is worth a great deal.** Two-dimensional texture has no canonical order, which is why WFC needs a
variable-ordering heuristic — Gumin's minimal-entropy rule, minimum-remaining-values by another name — and
bidirectional arc consistency. Music is ordered, so an exact left-to-right dynamic programme is available, and it
buys something WFC structurally cannot have: **WFC cannot say how many outputs satisfy its constraints, and the DP
here can, exactly.** That count is the whole of [§8.6](#86-realisation-and-the-first-notes).

**The backtracking contrast supports [§2.7](#27-where-a-solver-takes-over-from-the-dp) rather than undermining it.**
WFC as published has no backtracking at all — contradiction, restart — and Gumin reports that working because *"in
practice, however, the algorithm runs into contradictions surprisingly rarely."* That holds when the constraint
graph is two-dimensional and local. [§8.6](#86-realisation-and-the-first-notes) measured this project's state
explosion as driven by **obligations compounding across voice pairs**, which is precisely the regime in which
restart-on-failure thrashes; and every serious derivative — Karth & Smith, and the community ports — has added
backtracking. The complexity claim is the one [§2.7](#27-where-a-solver-takes-over-from-the-dp) makes: deciding
whether a bitmap admits nontrivial outputs satisfying C1 *"is NP-hard, so it's impossible to create a fast solution
that always finishes."*

#### What it changes

Two entries for [§9](#9-roadmap)'s step 6, neither of which was visible from inside this project.

1. **Transcribe the species as a whitelist**, not only the prohibitions as a blacklist. Same book, same no-fitting
   position, and the only proposal so far that attacks `10¹⁵` at its root rather than choosing better within it.
2. **Sample uniformly from the legal set** instead of optimising over it. Built and measured, and it does *not*
   beat the objective — but it supplies the like-for-like baseline the section lacked, and it is the only way to
   ask how many of those `10¹⁵` fills are any good, since it can now draw them.

Neither is an argument for adopting WFC. It is an argument that a difficulty which looked specific to counterpoint
— a complete search over a permissive rulebook returning far too much — has a well-studied shape, a name, and at
least one answer that costs nothing this document is unwilling to spend.

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
| rules transcribed | 11 — 5 written hard, 6 written soft |
| **hard in both corpora** ([§8.2](#82-the-rulebook-stratified-by-two-corpora)) | **2** |

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
`B♭4 – F4 – B♭3 – F3 – B♭2`, tonic and dominant alternating, the five heads descending **two octaves** and the
whole texture spanning a little over three.

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
the model a realiser writes against. Until the form grammar exists there is no other source of a plan, so
[§8.6](#86-realisation-and-the-first-notes) uses it as one — and derives the plan from the *fixed* voices alone, so
that the notes being generated never inform the harmony they are generated against.

**What it is not.** It fits Renaissance polyphony slightly better than Bach on every chord-fit statistic (mean fit
+0.022, chord tones +1.6 points). That is probably a fact about the music — Renaissance polyphony really is more
triadic — but it means chord fit does not separate tonal from modal, and no claim about tonality should rest on it.

### 8.6 Realisation, and the first notes

`src/realise.rs`, `src/midi.rs`. [§2.5](#25-the-search-is-a-shortest-path)'s shortest path, built as stated there:
rhythm is given and pitch is the only variable ([§2.6](#26-what-is-not-a-variable-rhythm)), the layers are the
slices at which some voice articulates, a node carries [§2.2](#22-counterpoint-is-a-finite-automaton)'s automaton
state for every pair involving a free voice, and an edge is a transition no hard rule refuses. A free voice is
handed in as a `Voice` whose **pitches are discarded and whose onsets are obeyed**, so the search cannot choose
when a note happens even by accident.

[§2.3](#23-harmony-is-a-second-automaton)'s harmony runs beside it as a second obligation system over the same
grid: a note foreign to the prevailing chord is legal only if prepared or approached by step, and it owes a
resolution on the next articulation. The two obligation systems do not know about each other and needed no
special-casing to compose, which is the part of §2.3 that was a claim until now.

**The stretto is audible.** `out/stretto.mid` is [§8.3](#83-the-clique-test)'s clique — BWV 867's five entries at
`{0, 2, 4, 6, 8}` quarters, 50 notes, two violations on the full tier and **none** on the confirmed tier, the same
verdict the clique test gives. No search was involved: with five entries in five voices there are no free voices at
all, which is §2.5's cost profile arriving exactly as predicted. `out/stretto-bach.mid` is the same bars as Bach
wrote them, so the idealisation and the original can be compared by ear rather than by table.

Each pair of files is written **in score order, top voice first**, and every track is named with its register, its
compass and its role — `1 top F4..G♭5 — entry 1 of 5, enters +0q`. Track *n* of one file is therefore the same line
as track *n* of the other, which is the only property that makes a two-file comparison worth writing. It is also
the property the first version lacked: tracks came out in `**kern` spine order, lowest first, named by that index,
so the top voice of three arrived as `voice 2` and `entry 1` of the idealisation paired with `voice 4` of Bach.

Both were reported from a DAW rather than found here, as was a third: the division was 240 ticks per quarter, which
is legal, uncommon, and came back at exactly half length — the signature of a host substituting an assumed timebase
for the one in the header. It is now 960, an exact `×4` of the internal lattice, and the **time signature** is
written from the score's own `*M` interpretation instead of left to the host to guess. None of this is a fact about
music, and all of it is the difference between a file that can be compared and one that cannot.

**The round trip was then checked against an independent implementation**, which is the only way any of the
exactness claims above stop being self-reports. `fill.mid` was imported into FL Studio and its top voice read back
out of the piano roll:

| | |
|---|---|
| notes | 20 out of 20 identical in **pitch, onset and duration** |
| timebase | 960 ticks per quarter in the file, 96 in the host — an exact `÷10` |
| onsets | every one still exactly on the semiquaver grid: **no rounding anywhere** |
| velocity | written 80, reported `0.625` = 80/128 exactly |
| span | exactly 8.0 quarters |

**And one thing does not survive, which is worth more than the five that do.** A MIDI note number is a semitone
integer — precisely the representation [§2.1](#21-exact-arithmetic-and-therefore-no-certificates) exists to reject
— so the diatonic spelling is destroyed at the file boundary. The passage is in C minor, and **13 of those same 20
notes come back under the wrong name**: `D#5` for E♭, `A#4` for B♭, `G#4` for A♭. Nothing is broken and no host is
at fault; there is no spelling in the file to read, so it guesses sharps, which in a three-flat key is the worst
guess available.

That fixes the status of these files. **MIDI is an output format here and never an interchange one**: nothing may
be read back from one into the model, because a round trip would re-spell every accidental by the reader's
convention rather than the key's, and the interval qualities [§2.2](#22-counterpoint-is-a-finite-automaton)'s
automaton switches on would return decided by a coin flip. The corpus is read from `**kern`, which spells its
pitches, for exactly this reason. The one consolation is that the *track names* keep what the note data loses —
`1 top A♭4..F5` is correctly spelled above notes that are not.

**Four things are verified rather than asserted**, by `cargo test --release`:

- the generator and the checker assemble a slice's symbol through **one shared function**, and a test asserts the
  fill passes the checker — a generator that computes the lo/hi roles even slightly differently from its own
  checker can emit counterpoint the checker then flags, and then neither number means anything;
- the count of legal fills is checked against **brute-force enumeration** on a small instance;
- the search never chooses a rhythm;
- a non-chord tone is approached and left by step.

#### Reconstructing Bach's free voices

For every annotated subject entry in the book, hold the entry voice, **discard the other voices' pitches while
keeping their rhythm**, and fill them. Bach's own notes are the answer key and the search never sees them.

Three rulebooks and three sources of harmony, crossed:

| | |
|---|---|
| `confirmed(2)` | the two rules [§8.2](#82-the-rulebook-stratified-by-two-corpora) found universal — this document's endorsed tier |
| `conf+melodic` | those two plus the melodic prohibition |
| `full(5)` | all five rules written hard, dissonance rules included |
| `none` | no harmonic plan — the control for how much [§2.3](#23-harmony-is-a-second-automaton) is carrying |
| `clean` | the plan analysed from the **fixed voices only**: the honest condition, since a form grammar would supply one and the notes being generated must not inform it |
| `leaky` | the plan analysed from the whole texture, the answer included. Cheating, and run to price the cheat |

| tier | plan | solved | dead | refused | notes | exact | pitch class | chance | log₁₀ legal fills | open/note | time |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| confirmed(2) | none | 36 | 0 | 81 | 617 | 4.9% | 15.9% | 5.9% | 18.3 | 17.1 | 91s |
| confirmed(2) | clean | **83** | 5 | 29 | 1647 | **6.9%** | 13.5% | 7.4% | 16.6 | 14.1 | 59s |
| confirmed(2) | leaky | 81 | 1 | 35 | 1676 | 10.4% | 21.1% | 7.2% | 16.7 | 14.4 | 70s |
| conf+melodic | none | 42 | 0 | 75 | 733 | 5.3% | 15.8% | 9.6% | 16.3 | 11.1 | 94s |
| conf+melodic | clean | **99** | 6 | 12 | 2129 | **7.8%** | 13.2% | 16.2% | 14.7 | 7.0 | 34s |
| conf+melodic | leaky | 110 | 1 | 6 | 2523 | 9.3% | 17.4% | 15.5% | 15.8 | 7.3 | 41s |
| full(5) | none | 96 | 1 | 20 | 2079 | 6.4% | 12.7% | 9.8% | 15.8 | 10.9 | 69s |
| full(5) | clean | **108** | 8 | 1 | 2466 | **7.0%** | 13.2% | 16.2% | 11.7 | 7.0 | 11s |
| full(5) | leaky | 115 | 1 | 1 | 2731 | 10.5% | 17.3% | 15.5% | 12.5 | 7.3 | 11s |

Every column but the last is deterministic and reproduces exactly between runs; `time` is wall clock on one
machine and moves by a few seconds, quoted because tractability is one of the findings.

**Two things about the `leaky` rows were wrong and are repaired in [§8.9](#89-a-better-plan-and-the-first-lever-that-moves-more-than-a-point).** They are not paired with the
`clean` rows — a plan that solves one span may refuse another, so `9.3%` against `7.8%` compares two sets of notes
rather than two plans. And the difference this table attributes to *having* a plan is mostly the difference between
a **right** plan and a **wrong** one: the `clean` plan names the same chord as the `leaky` one on **16%** of the
span.

**The notes are not determined, and that is the result.** Read `exact` against `chance` rather than against zero:
the baseline is what picking at random from the pitches actually open at that note would have scored, computed
under the same tier and the same plan the search used. Agreement never gets far from it in either direction — the
best row reaches 1.4 times chance and the tighter tiers sit at **half** of it. Meanwhile the rules and the plan
together leave seven to seventeen pitches open at every note, and the number of complete legal fills of a
three-bar span runs to eleven or more orders of magnitude even under the full five-rule tier. **This is [§5](#5-what-this-will-not-do)'s inverted failure mode, arriving
where it finally matters and measured rather than predicted.** A complete search over this rulebook does not fail
by finding nothing.

The sharpest way to put it is that **`exact` barely moves across the entire table while `chance` nearly triples.**
Every constraint added — the melodic rule, the two dissonance rules, a harmonic plan — shrinks the legal set and so
raises what a random legal choice is worth. None of it changes what the search picks out of that set by more than a
few points. Constraint is doing all the work; the objective is doing almost none.

**Is the objective wrong, or merely weak?** Three runs settle it, all on the same tier, the same plan and the same
99 spans. Reverse the sign of the objective; and — [§9](#9-roadmap) step 6's first proposal — **draw from the legal
set uniformly instead of optimising over it at all**, which the search can do exactly because it already counts the
paths through its own DAG ([§8.6](#86-realisation-and-the-first-notes) again). Eight draws per span, seeded.

| conf+melodic, clean plan | exact | pitch class |
|---|---:|---:|
| soft criteria **minimised** | **7.8%** | 13.2% |
| **uniform draw from the legal set** | 6.9% | 13.4% |
| soft criteria **maximised** | 4.9% | 13.0% |
| per-note `chance` baseline | 16.2% | — |

**The middle row is the like-for-like control, and it changes the reading.** It commits to a whole path, its errors
compound exactly as the search's do, and the same function scores it — so the ordering `4.9 < 6.9 < 7.8` is
entirely the objective's doing. Minimising the soft criteria beats maximising them, ~~*and* beats not optimising at
all. The soft tier is weak and it is real, and using it is better than ignoring it.~~

**The struck sentence is withdrawn** by [§8.10](#810-replacing-the-soft-tier-and-the-degenerate-optimum-of-every-positive-criterion). The
three figures above are pooled over notes, which weights a span by how many it has and counts each of the eight
draws' notes separately. Paired per span — the accounting
[§8.2](#82-the-rulebook-stratified-by-two-corpora) established and everything after
[§8.8](#88-a-criterion-that-is-not-local-and-the-shape-of-every-step-6-failure) uses — the uniform draw is
`−0.74 ± 0.97` against the tier on these very spans, which is nothing, and `+1.07 ± 0.31` and `+4.64 ± 0.61` *for*
it on the larger window sample. What survives is `4.9 < 6.9`: minimising beats maximising by `3.00 ± 1.01`, so the
criteria are not noise and do point the right way. What does not survive is that using them beats leaving them
out.

**And the `chance` column is now shown to be a large overestimate of what any generator can do.** The gap between
16.2% and 6.9% is precisely the caveat this section already carried, measured at last: `chance` is computed per
note with **Bach's own preceding note** in hand, and more than half of its apparent advantage is that handout. The
honest baseline for a generator is the 6.9%. What 16.2% establishes is a different and still useful thing — a
ceiling on how much the constraint alone determines when the previous note is *given*: about one note in six.

#### A treatise weighting was tried, and it is Bach's rather than counterpoint's

The uniform draw asserts nothing, and asserting nothing is why it does no better than the objective. The obvious
next move is [§7.1](#71-parallels-within-the-same-algorithmic-family)'s **Weak C2** with the corpus half removed:
draw each fill in proportion to how much the *treatise* likes it. Fux supplies the directions — the six soft
criteria are the things he says to avoid — and supplies no magnitudes at all, which is Komosinski's objection to
Schottstaedt's weights. So there is **one** number rather than six: an inverse temperature `β`, with each fill drawn
in proportion to `exp(−β × soft cost)`. `β = 0` is the uniform draw, `β → ∞` is the cheapest fill, and everything
between is a preference rather than an optimisation. It is swept and reported as a curve rather than chosen, on
[§8.5](#85-the-harmonic-analyser)'s precedent with `λ`.

**Whether that generalises is [§8.2](#82-the-rulebook-stratified-by-two-corpora)'s question, so it gets §8.2's
instrument**: the same measurement on two corpora three centuries apart. One protocol for both, since the
Renaissance corpus has no subject annotations — hold the **top voice**, free up to two others, window at eight
quarters. The rule was fixed before the run: *keep it only if one `β` beats the uniform draw on both corpora by more
than twice the standard error.*

| β | Bach agreement | gain on β = 0 | 15th-c. polyphony | gain on β = 0 |
|---:|---:|---|---:|---|
| 0.00 | 6.8% | — | 9.8% | — |
| 0.25 | 7.4% | +0.26 ± 0.35 | 9.9% | −0.17 ± 0.31 |
| 0.50 | 7.8% | +0.78 ± 0.53 | 9.7% | −0.46 ± 0.33 |
| 1.00 | **8.3%** | **+1.33 ± 0.62** | 9.1% | **−1.04 ± 0.40** |
| 2.00 | 8.2% | +1.29 ± 0.74 | 8.4% | −1.88 ± 0.45 |
| 4.00 | 8.2% | +1.41 ± 0.88 | 7.6% | **−2.68 ± 0.52** |

67 Bach spans from 24 fugues, 577 Renaissance spans from 200 works, eight draws each. Gains are **paired per-span
differences** against the same spans at `β = 0`, one standard error; the span is the unit of replication, because
eight draws sharing one span's fixed voices and plan are not eight independent observations.

**The verdict is repertoire-specific, so it was rolled back.** The weighting buys Bach a little over a point at
`β = 1` — itself only about two standard errors, across five swept values, so not much — and it *costs* the earlier
repertoire more than that, **monotonically worse the harder it is applied**. Every production call site passes
`β = 0`; the parameter survives only so this table stays reproducible, which is what
[§10.2](#102-which-command-produces-which-section) asks of every superseded result.

Two things about it are worth more than the rollback.

**The first run said the opposite.** On 60 works and 112 spans the Renaissance gain came out **+0.6 points** and the
verdict printed `GENERAL`. Five times the data reversed the sign and made it significant. Nothing was wrong with the
code; the decision rule simply said *"improves on both"* and never said *by how much against what noise* — so a
figure inside its own error bar was allowed to decide. That is this project's recurring pattern arriving in the
test-design rather than in the measurement.

**And the direction is backwards from what the source would predict.** Fux is writing about Palestrina, so a
weighting transcribed from Fux ought to fit 16th-century vocal polyphony *better* than it fits Bach, and it does the
reverse. The likeliest reading is in the corpus rather than the weighting: per
[§10.4](#104-how-the-samples-were-taken) the control is **70 Busnois, 37 Dufay and 93 Josquin** — 15th-century
Franco-Flemish music, a century *before* Fux's subject, in which open fifths and octaves are idiomatic and
equal-range voices cross constantly. Three of the six criteria penalise exactly those. So what this table shows is
that the weighting fails to span 1450 to 1722; **Fux's own repertoire sits between the two and remains untested**,
and testing it needs a Palestrina corpus this project does not have. That is a real limit on the claim and it does
not rescue the weighting, which was asked to generalise and did not.

**The search is given Bach's rhythm** — every onset and every tie of the voices it is reconstructing, which is a
large part of the answer — and it still cannot find the pitches. One span whole, because percentages hide what is
actually happening. BWV 847 at bar 11, two free voices, the melodic tier and an honest plan:

```
 Bach   E♭2 A♭3  G3  F3 E♭3 D♭3  C3 B♭2 A♭2  C4 B♭3 A♭3  G3  F3  G3 A♭3
 fill    G3  F3  G3 A♭3  G3  D3  D3 E♭3  D3  C3 B♭2  A2  D3  C3  D3  D3
                                            ^^^^^^^^^^^^
                                            the right three notes, an octave low
```

**Pitch class is recovered about twice as often as pitch**, right across the table, and the three notes marked above
are the whole of that gap in miniature. The harmonic plan pins down *which note of the chord* far better than
chance; what it says nothing whatever about is **which octave**, and neither does anything else in the rulebook,
because every soft criterion in the tier looks at one slice or two. Register is a property of a line over a phrase.
It is the first concrete thing [§9](#9-roadmap)'s "criterion that is not local" would have to supply, and this is
how it would be measured.

**Every constraint buys tractability, and none of them buys agreement.** Both axes of the table say the same thing.
Tightening the tier from two rules to five, with the plan held fixed, takes the spans the exact search can finish
from 83 to 108 of 117, the refusals from 29 to 1, the legal fills down by five orders of magnitude, and the running
time from 61 seconds to 11 — while `exact` moves from 6.9% to 7.0%. Supplying a plan where there was none, with the
tier held fixed, takes the finished spans from 42 to 99 and `exact` from 5.3% to 7.8% — but takes `chance` from
9.6% to 16.2% at the same time, so the ratio falls. Nothing in the table improves on picking at random from what
the constraints leave open.

**The melodic rule is repertoire-specific as a description and load-bearing as a constraint**, and those are
different questions with different answers. [§8.2](#82-the-rulebook-stratified-by-two-corpora) stratified it out of
the hard tier because Bach breaks it thirty-eight times more often than the Renaissance does — which settles what
it is worth as a *description* of Bach and says nothing about what it is worth as a *constraint on a generator*.
96% of his melodic moves obey it; and without it **nothing whatever bounds a free voice's line**, since the
two-rule tier permits a two-octave leap between quavers. Adding it halves the pitches left open at every note and
is most of the difference between the first three rows and the rest.

**[§2.7](#27-where-a-solver-takes-over-from-the-dp)'s wall is at two free voices, not four, and the estimate was
wrong about why.** That section put the state at roughly `24^(V−e)`, the product of the free voices' pitch domains,
and called two free voices comfortable. Two free voices give about **225** pitch pairs. The measured peak is
**59 598 live states in a single layer** — and that figure is the budget biting rather than a ceiling, since a
layer that passes 60 000 is refused rather than counted, so the true number is unknown and larger. **The multiplier
is the obligation set, not the pitch product**: a dissonance owed in one pair and a leap owed in another are
independent bits, and they compound across every pair at once. The correction moves
[§2.7](#27-where-a-solver-takes-over-from-the-dp)'s conclusion in the direction it was already pointing — it makes
the case for a conflict-learning solver stronger and earlier, since compounding independent obligations is exactly
the state that clause learning collapses and enumeration cannot.

Both budgets are refusals, not beams. A span that exceeds either is reported in the `refused` column and excluded,
never silently truncated to the best few thousand states — because §2.7 predicts this failure and a quiet beam
would hide the prediction coming true. Under the full tier with a plan, one span of 117 is refused; under the
two-rule tier with none, 81 are.

#### The weighting is a choice, and it changes every note

| objective minimised | cost | direct→perf | perfect | direct | crossing | unrec. leap | repeat | the line it chooses |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| uniform | 9.0 | 0 | 0 | 1 | 0 | 7 | 1 | E♯3 D♯3 D♯3 C♯3 G♯2 G♯2 A♯2 B♯2 C♯3 G2 |
| direct to perfect | 0.0 | 0 | 6 | 2 | 0 | 7 | 6 | D2 C♯2 D♯2 C♯2 C♯2 C♯2 C♯2 C♯2 E♯2 D♯2 |
| perfect consonance | 0.0 | 0 | 0 | 7 | 0 | 7 | 5 | D2 C♯2 D♯2 C♯2 C♯2 E♯2 E♯2 D♯2 E♯2 F♯2 |
| direct motion | 1.0 | 0 | 6 | 1 | 0 | 7 | 6 | D2 C2 C2 C♯2 C♯2 C♯2 C♯2 C♯2 E♯2 D♯2 |
| voice crossing | 0.0 | 1 | 6 | 3 | 0 | 7 | 7 | D2 C♯2 D♯2 C♯2 C♯2 C♯2 C♯2 C♯2 C♯2 D♯2 |
| unrecovered leap | 2.0 | 1 | 1 | 7 | 9 | 2 | 1 | D2 C♯2 D♯2 C♯2 **G♯4 F♯4 G♯4 A♯4 B♯4 A♯4** |
| repeated note | 1.0 | 2 | 5 | 4 | 0 | 7 | 1 | D2 C2 C2 D♯2 C♯2 C♯2 D♯2 C♯2 E♯2 D♯2 |

BWV 848, bars 42–44, one free voice, two-rule tier, the plan from the two fixed voices. The counts are read back off
the **checker** rather than off the search's own accounting. A single objective disagrees with the uniform one on
up to **100%** of notes, and **two of the seven fills are mutually non-dominated** — the uniform one and, of all
things, the one that minimises unrecovered leaps by sending the free voice above the others for half the
passage — the only crossings in the table, and legal, because crossing is soft.

Every one of these fills is legal, on the same rhythm, against the same plan, in the same fugue. They are what
changes when the *only* thing that changes is which soft criterion the sum is taken over — and §5's position is
that no such sum is defensible. The Pareto front is not a refinement to add later; it is what is left once the
scalarisation is admitted to be arbitrary, and the front here is not a single point.

Several of these lines are also nearly static, because repeating a note is cheap under most of these objectives and
the rulebook has nothing to say against a voice that does almost nothing. **No further measurement resolves this.**
The output has to be listened to, and that is now possible for the first time in either document.

#### The first listening test disagrees with the numbers, and it is worth recording that it does

`out/fill.mid` against `out/fill-bach.mid`, one listener, unblinded, the passage tabulated above. The report:

> voice 2 [the inner voice] spans a smaller pitch range compared to fill-bach — but overall the result is on par,
> nothing is better or worse than Bach himself.

The first half confirms the register finding by ear; the compasses printed in the track names say the same thing.
**The second half contradicts what the agreement figures might be taken to imply, and the contradiction is the
useful part.** These two statements are both true and not in tension:

- the search reproduces Bach's *particular notes* at half the rate of a random legal choice;
- the notes it picks instead are, to one listener, no worse.

What that pulls apart is the assumption quietly linking them. `exact` measures **identity with Bach**, which is a
proxy for quality and not quality itself, and a proxy is only as good as the assumption that the target is the
unique good answer. The median span under this tier and plan admits about `10¹⁵` legal fills; if even a small
fraction of those are musically acceptable, a low agreement rate is a fact about *how many acceptable answers
there are* and not about how bad the chosen one is.

So the honest reading of this section is narrower than it first appears: **the rulebook plus a harmonic plan is
enough to write acceptable counterpoint and nowhere near enough to write Bach's.** That relocates the open problem
in [§9](#9-roadmap) from *quality* to *stylistic identity*, and makes a criterion that is not local the thing which
would distinguish a composer rather than the thing which rescues the output.

The obvious cautions, stated because one listener on six seconds is thin evidence in both directions: unblinded,
one passage, one listener, a flat MIDI piano with no dynamics, and a texture in which the top voice is Bach's in
both files and one of the three parts is therefore identical. A real test is an A/B over many spans with the
sources hidden, and it has not been run.

### 8.7 The species as a whitelist, and why it does not tighten anything

[§9](#9-roadmap) step 6's other proposal, and the one that attacks `10¹⁵` at its root rather than choosing better
within it. Fux's book *is* a whitelist and was transcribed here as a blacklist: the species enumerate the permitted
note-against-note figures one at a time — first species consonance throughout, second the passing tone, third the
neighbour, fourth the suspension tied over and resolving down. `src/species.rs` transcribes that enumeration and
nothing else.

**A whitelist is a checker before it is a constraint**, and [§8.2](#82-the-rulebook-stratified-by-two-corpora)'s
method decides whether it earns its place: one measurement on two corpora three centuries apart, asking what
fraction of the dissonances real music writes are figures Fux lists. A whitelist that cannot account for the music
is not a tighter rulebook but a wrong one, and generating against it would be pointless.

| corpus | reading | dissonances | explained | unlisted per 1000 slices |
|---|---|---:|---:|---:|
| Bach | strict | 12 208 | 61.9% | 133.0 |
| Bach | figures only | 12 208 | 76.0% | 83.8 |
| Bach | fourth consonant | 8 410 | **77.4%** | **54.3** |
| 15th-c. | strict | 70 050 | 59.3% | 95.0 |
| 15th-c. | figures only | 70 050 | 74.8% | 58.8 |
| 15th-c. | fourth consonant | 39 426 | **82.2%** | **23.5** |

**It fails, and it fails symmetrically.** At its most generous the enumeration cannot account for one dissonance in
five — 23% of Bach's and 18% of the Renaissance's — and rejects 54.3 and 23.5 slices per thousand. The two rules it
was written to replace flag 21.4 and 90.9 per thousand in Bach, 8.0 and 71.1 in the Renaissance, so the whitelist
lands **between them in both centuries**: better than *unresolved dissonance*, worse than *unprepared*. That is the
same band, not an improvement, so it does not go into the tier. Note that the failure is even across the two
corpora, unlike the melodic rule's ×38 in [§8.2](#82-the-rulebook-stratified-by-two-corpora) — this is an
enumeration that is *incomplete*, not one that belongs to a repertoire.

Two things fall out that are worth more than the proposal was.

**The perfect fourth is a large classification artefact, and it may be most of an older mystery.** Reclassifying it
as a consonance removes **31% of Bach's flagged dissonances and 44% of the Renaissance's** — from 12 208 to 8 410
and from 70 050 to 39 426. `pitch.rs` calls the fourth a dissonance, which is the classical two-voice position that
Schottstaedt and Komosinski both adopt, and its own comment warns that a texture judged this way will flag things
that are not errors. In three parts or more a fourth between upper voices over a supporting bass **is** a
consonance; only a fourth against the bass is not. A pairwise walk through a four-voice fugue cannot see the
difference. [§8.2](#82-the-rulebook-stratified-by-two-corpora) reports the two dissonance rules failing in the very
repertoire they were written for and calls them implementation faults awaiting a diagnosis; this is a candidate
diagnosis, and it is measurable — those rules should be re-run with the fourth resolved against the lowest sounding
voice rather than pairwise.

**Fux's metric condition costs fourteen points in both centuries.** Requiring suspensions on the beat and passing
tones off it drops the explained fraction from 76.0% to 61.9% in Bach and 74.8% to 59.3% in the Renaissance. Real
counterpoint strikes dissonances on strong positions far more often than the species allow — which is the
difference between a pedagogical exercise and the repertoire it is supposed to be teaching, arriving as a number.

The residue after all of that is seconds and sevenths, which are the intervals a *chord* explains rather than a
melodic figure. That is [§2.3](#23-harmony-is-a-second-automaton)'s claim from the other side: what is left over
when every voice-leading figure has been accounted for is exactly what harmony is for.

### 8.8 A criterion that is not local, and the shape of every step-6 failure

[§8.6](#86-realisation-and-the-first-notes) says where to look and
[§2.5](#25-the-search-is-a-shortest-path) says what it would cost. Pitch class is recovered about twice as often as
pitch, so the octave is wrong, and **register is a property of a line over a phrase** that no one-slice criterion
can see. The accumulators that would express it are finite-state, and carrying a running minimum and maximum per
free voice would multiply an already-exploding search by a few hundred.

So the criterion is applied **after** the search rather than inside it. [§8.6](#86-realisation-and-the-first-notes)'s
sampler draws whole legal fills; a criterion over a complete line can rank them afterwards, which needs no state at
all. Three criteria, each transcribed from Fux and each reported alone, since combining them needs weights and
[§5](#5-what-this-will-not-do) is about exactly that: **one climax**, a **compass** inside a tenth, and **variety**
— not standing on one note. 32 draws per span, ranked; the control is the same draws unranked.

| criterion | Bach | gain on unranked | 15th-c. | gain on unranked |
|---|---:|---|---:|---|
| unranked | 7.2% | — | 9.7% | — |
| climax | 7.3% | +0.15 ± 0.26 | **12.0%** | **+2.21 ± 0.82** |
| compass | 7.2% | +0.01 ± 0.27 | 7.2% | **−2.51 ± 0.61** |
| variety | 7.1% | −0.10 ± 0.26 | **11.7%** | **+1.99 ± 0.80** |
| all three | **8.1%** | **+0.96 ± 0.29** | 7.6% | **−2.15 ± 0.62** |

690 Bach spans and 577 Renaissance, one protocol for both; gains are paired per-span differences against the same
draws unranked. Nothing clears the bar on both corpora, so **nothing is adopted**. Three things in the table are
worth more than that verdict.

**The stratification runs the other way from [§8.7](#87-the-species-as-a-whitelist-and-why-it-does-not-tighten-anything)'s
and [§8.6](#86-realisation-and-the-first-notes)'s.** The treatise *weighting* helped Bach and hurt the earlier
repertoire; these *shape* criteria do the reverse — climax and variety are worth better than two points to
15th-century polyphony and are indistinguishable from nothing in Bach. And that is
[§8.2](#82-the-rulebook-stratified-by-two-corpora)'s melodic finding arriving a second time. That section measured
Fux's melodic *interval* prohibition at 1.0 violations per thousand moves in the Renaissance against 37.6 in Bach
and called it repertoire-specific; his melodic *shape* prescriptions now stratify the same way and in the same
direction. **Fux's melodic doctrine is Renaissance doctrine**, twice measured on independent evidence.

**The compass criterion makes things actively worse, and the reason indicts the whole approach.** It costs the
Renaissance 2.5 points, four standard errors. Fux's rule is an *upper* bound — keep the line inside a tenth — and
[§8.6](#86-realisation-and-the-first-notes)'s diagnosed failure is that the fills are too **narrow**: Bach's inner
voice covers `F3..A♭4` where the fill covers `F3..C4`, the same floor and a ceiling a fourth lower. Ranking by an
upper bound selects for narrowness, so it pushes precisely the wrong way on the failure it was chosen to address.

Which is the general form of every step-6 result, and the most useful thing the step produced:

> **The treatise is a list of prohibitions against excess. The generator's failure is deficiency.** A rulebook
> written to restrain a human writer who naturally does too much is the wrong instrument for a search that
> naturally does too little.

That reading is consistent across all four experiments. Removing the objective entirely changed almost nothing
([§8.6](#86-realisation-and-the-first-notes)), because the objective was never the binding problem. Weighting the
prohibitions harder helped one repertoire and hurt the other. Enumerating the permitted figures could not account
for a fifth of what real music writes ([§8.7](#87-the-species-as-a-whitelist-and-why-it-does-not-tighten-anything)).
And a shape criterion drawn from the same source pushes the wrong way on register. Nothing transcribed from Fux
tells a generator what a line should **do**; the book assumes a writer who already knows, and constrains what they
must not.

### 8.9 A better plan, and the first lever that moves more than a point

`src/plan.rs`. [§9](#9-roadmap) step 6's fourth proposal, and the only one that began with a number already on the
table: [§8.6](#86-realisation-and-the-first-notes)'s `leaky` row scores three points above its `clean` row, which
is the largest single effect there and three times what reversing the objective buys. Two things were wrong with
reading it as it stood.

**The rows are not paired.** A tighter plan solves spans a looser one refuses and refuses spans a looser one
solves — `clean` finishes 99 of 117 entry spans and `leaky` 110 — so `9.3%` against `7.8%` compares two different
sets of notes rather than two plans. **And the oracle is not a plan any grammar could emit.**
[§2.4](#24-form-is-a-grammar)'s productions name a key plan and a cadence schedule; they cannot name a chord per
onset, because the onsets belong to the notes the grammar is asking for.

So: [§8.8](#88-a-criterion-that-is-not-local-and-the-shape-of-every-step-6-failure)'s windows, both corpora, nine
plans, and every gain a **paired per-span difference against [§8.6](#86-realisation-and-the-first-notes)'s own plan
on the spans both conditions finished**. Two candidates that never see the answer — `λ` varied, since
[§8.5](#85-the-harmonic-analyser) swept it against a **full** texture while this plan is analysed from one or two
voices out of three or four, and the plan **gated on its own `fit`**, since a plan is a hard constraint and a wrong
one forbids the right note. Three ceilings that do see it: the oracle, and the oracle coarsened to a beat and to a
bar.

**Bach, 690 spans**

| plan | right | log₁₀ fills | agreement | gain on `clean` |
|---|---:|---:|---:|---|
| `none` | 0% | 21.4 | 3.8% | −2.61 ± 0.61 |
| **`clean λ=1`** | **16%** | 19.0 | **6.1%** | — |
| `clean λ=0` | 4% | 14.7 | 5.9% | −0.32 ± 0.42 |
| `clean λ=2` | 15% | 20.0 | 5.5% | −0.51 ± 0.28 |
| `clean fit≥.6` | 16% | 19.0 | 6.1% | +0.01 ± 0.08 |
| `clean fit≥.8` | 15% | 19.0 | 6.1% | +0.05 ± 0.12 |
| oracle | 100% | 20.3 | 8.4% | **+2.36 ± 0.42** |
| oracle / beat | 91% | 20.6 | 7.7% | **+1.73 ± 0.38** |
| oracle / bar | 58% | 20.4 | 6.7% | +0.88 ± 0.46 |

**15th-century, 577 spans**

| plan | right | log₁₀ fills | agreement | gain on `clean` |
|---|---:|---:|---:|---|
| `none` | 0% | 3.9 | 5.7% | +0.47 ± 0.70 |
| **`clean λ=1`** | **20%** | 3.0 | **5.2%** | — |
| `clean λ=0` | 10% | 2.5 | 6.2% | +1.13 ± 0.66 |
| `clean λ=2` | 20% | 3.0 | 5.1% | −0.05 ± 0.26 |
| `clean fit≥.6` | 20% | 3.0 | 5.1% | −0.06 ± 0.06 |
| `clean fit≥.8` | 19% | 3.0 | 5.2% | +0.06 ± 0.09 |
| oracle | 100% | 3.0 | 8.9% | **+3.74 ± 0.70** |
| oracle / beat | 97% | 3.1 | 8.9% | **+3.77 ± 0.73** |
| oracle / bar | 89% | 3.1 | 7.9% | **+2.77 ± 0.70** |

`right` is the fraction of the span on which the plan names the same chord as the answer-key analysis. `agreement`
is over each row's own solved spans and is not comparable across rows; `gain` is, and it is the paired difference.

**The plan the realiser has been writing against is wrong five times in six.** Sixteen per cent in Bach, twenty in
the Renaissance. [§8.5](#85-the-harmonic-analyser) measured the analyser at 70–80% correct on cadence arrivals
*with the whole texture in front of it*; asked the same question from one or two voices out of three or four, it
names the same chord as its own full-texture analysis on a sixth of the span. That number had never been measured,
and it reframes every `clean` row in [§8.6](#86-realisation-and-the-first-notes): those rows do not price a
harmonic plan, they price a mostly wrong one. In the Renaissance the plan is worth **nothing at all** — `none` and
`clean` are `+0.47 ± 0.70` apart, which is zero.

**A correct plan is worth more than everything else step 6 tried put together.** `+2.36` in Bach and `+3.74` in the
Renaissance, both several standard errors clear, in the same direction, on both corpora — the first condition in
the whole of step 6 to do that. For scale: the treatise weighting bought `+1.33` in Bach and `−1.04` in the
Renaissance; the shape criteria `+2.21` in the Renaissance and nothing in Bach; minimising the soft tier rather
than not optimising at all, about a point. Harmony is the only lever measured that moves more than a point in the
same direction in both centuries — and it is a **ceiling rather than a candidate**, which is exactly the point.

**It survives coarsening, and the resolution at which it stops surviving is a specification for step 7.**
[§2.4](#24-form-is-a-grammar)'s grammar can emit a chord schedule, so the question worth asking of the oracle is
how coarse a schedule still buys the gain. At **beat** resolution — 91% and 97% of the oracle's chords — nearly all
of it: `+1.73` and `+3.77`, both clear. At **bar** resolution the Renaissance keeps `+2.77` and **Bach falls to
`+0.88 ± 0.46`, which does not clear two standard errors.** So a form grammar has to schedule harmony *per beat*;
a chord per bar is not enough for the WTC. That is the first quantitative requirement step 7 has been handed from
outside itself.

**Neither candidate that stays inside the fixed voices works, and the gate fails for a reason worth keeping.**
Gating on `fit` removes almost nothing — coverage falls only from 100% to 95–98% at `fit ≥ 0.8` — and changes
nothing where it does. The reason is in the same row: the analyser reports high confidence on 95% of a span whose
chords it gets right 15% of the time. **On a thin texture its confidence is uncorrelated with its correctness**,
because two voices are easy to explain with many chords, and `fit` measures how well the notes suit the chord
rather than how likely the chord is. No gate built on `fit` can separate the segments worth keeping, and that
closes a family of repairs rather than one.

Retuning `λ` fails differently and more usefully. `λ = 0` re-chooses the chord at every onset, and in Bach that
removes **10⁴·³ of the legal fills** — four orders of magnitude — for a gain of `−0.32 ± 0.42`. It is tight and it
is wrong: 4% right, against `clean`'s 16%.

Which is the sharpest statement this project has of what constraint is and is not:

> **Neither tightness nor looseness predicts agreement. Correctness does.** The loosest plan here is the worst, the
> tightest is no better than the middle one, and the plan that wins admits **twenty times more** legal fills than
> the plan it beats.

That is [§8.6](#86-realisation-and-the-first-notes)'s thesis with a confound removed. That section watched
constraint raise `chance` without raising `exact` and concluded the objective was doing nothing; this watches a
*correct* constraint raise `exact` by two to four points while making the legal set **larger**. Constraint was
never the variable. Correct constraint is.

**And the price of correctness is quotable.** The three ceilings differ from `clean` in exactly one measurable
respect, so dividing each gain by that difference turns "improve the analyser" into an exchange rate: **0.024
points of note agreement per point of chord agreement in Bach and 0.045 in the Renaissance**, near enough constant
across all three. In Bach that is about forty points of chord accuracy per point of note agreement, and since a
perfect analyser is 84 points above the present one, the entire envelope for this lever is the `+2.4` the oracle
row shows. Large by this project's standards, and still nowhere near music.

**Nothing is adopted**, since no plan that stays inside the fixed voices beats
[§8.6](#86-realisation-and-the-first-notes)'s. What moves is where step 6's open problem points.
[§8.8](#88-a-criterion-that-is-not-local-and-the-shape-of-every-step-6-failure) closed with the treatise having
nothing to say about what a line should **do**; this section says what does — the harmony under it — and that the
instrument for supplying it is not a better analyser but [§2.4](#24-form-is-a-grammar)'s grammar, which never has
to infer the harmony because it decides it.


### 8.10 Replacing the soft tier, and the degenerate optimum of every positive criterion

`Problem::prescribe` in [`realise.rs`](src/realise.rs). [§9](#9-roadmap) step 6's last proposal, and the only one
whose stated destination this repository cannot reach: it points at **Marpurg** and **Kirnberger**, and neither is
transcribed here. ([Marpurg is freely available](https://archive.org/details/abhandlungvonder00marp) and
[§9](#9-roadmap) now records what is in it; it was not read when this ran, and reading it is not this
experiment.) What can be asked without them are the two questions that have to be
answered before any replacement is worth transcribing — **is the tier one criterion or six**, and **does saying the
same thing positively do better?**

Six one-hot ablations answer the first. Three positive criteria answer the second, each charged **in place of** the
tier rather than beside it — `weights` goes to zero, so it is a replacement and not a seventh prohibition. They are
the three such statements this project can make from what it already holds: **move by step**, **move against the
other voice**, and **state the harmony**, the last being the one
[§8.9](#89-a-better-plan-and-the-first-lever-that-moves-more-than-a-point) points at rather than Fux.

Two controls, and the second is easy to omit and necessary. The uniform draw; and **`tie-break only`**, which
charges nothing at all, so every path ties at zero and the search keeps whichever of them it reached first. An
ablation is read against *that* rather than against the uniform draw, because most paths tie under one criterion
too. Every row searches **the same graph** — a prescription reorders the legal set and never prunes it, which
`realise`'s tests assert — so `done` is constant by construction and every difference is the objective's.

**Bach, 690 spans, 491 solved**

| objective | mean \|step\| | compass | agreement | gain on `soft(6)` |
|---|---:|---:|---:|---|
| no objective (uniform) | 2.78 | 10.89 | **7.2%** | **+1.07 ± 0.31** |
| `tie-break only` | 0.76 | 1.91 | 1.3% | −4.77 ± 0.31 |
| **`soft(6)`** | 1.45 | 6.97 | **6.1%** | — |
| only direct→perfect | 0.78 | 2.17 | 1.4% | −4.68 ± 0.31 |
| only perfect consonance | 1.46 | 6.61 | 3.9% | −2.16 ± 0.30 |
| only direct motion | 0.88 | 4.10 | 2.7% | −3.37 ± 0.35 |
| only crossing | 0.80 | 2.24 | 1.4% | −4.71 ± 0.30 |
| only leap | 1.08 | 5.49 | 2.0% | −4.07 ± 0.33 |
| only repetition | 1.25 | 3.03 | 1.8% | −4.31 ± 0.31 |
| → move by step | 1.04 | 3.55 | 2.2% | −3.88 ± 0.34 |
| → move against | 0.88 | 4.10 | 2.7% | −3.37 ± 0.35 |
| → state the harmony | 0.91 | 3.83 | 2.1% | −3.96 ± 0.33 |
| → all three | 0.99 | 5.02 | 4.6% | −1.49 ± 0.39 |
| *the composer's own* | *1.66* | *6.94* | *100%* | |

**15th-century, 577 spans, 556 solved**

| objective | mean \|step\| | compass | agreement | gain on `soft(6)` |
|---|---:|---:|---:|---|
| no objective (uniform) | 2.35 | 3.14 | **9.8%** | **+4.64 ± 0.61** |
| `tie-break only` | 0.40 | 0.47 | 4.0% | −1.21 ± 0.74 |
| **`soft(6)`** | 1.25 | 1.63 | **5.2%** | — |
| only direct→perfect | 0.41 | 0.48 | 4.1% | −1.10 ± 0.75 |
| only perfect consonance | 1.02 | 1.39 | 4.4% | −0.72 ± 0.59 |
| only direct motion | 0.36 | 0.52 | 4.6% | −0.56 ± 0.77 |
| only crossing | 0.55 | 0.66 | 4.4% | −0.80 ± 0.73 |
| only leap | 0.41 | 0.51 | 4.0% | −1.17 ± 0.75 |
| only repetition | 0.99 | 1.11 | 5.3% | +0.13 ± 0.80 |
| → move by step | 0.94 | 1.04 | 5.3% | +0.15 ± 0.75 |
| → move against | 0.36 | 0.52 | 4.6% | −0.56 ± 0.77 |
| → state the harmony | 0.31 | 0.50 | 5.0% | −0.21 ± 0.78 |
| → all three | 0.79 | 0.97 | 6.7% | +1.57 ± 0.88 |
| *the composer's own* | *1.14* | *1.39* | *100%* | |

`mean |step|` is the average melodic interval of the free voices in scale steps and `compass` their whole range
over the span — [§8.6](#86-realisation-and-the-first-notes)'s narrowness and
[§8.8](#88-a-criterion-that-is-not-local-and-the-shape-of-every-step-6-failure)'s deficiency as numbers, with the
answer key's own values on the same voices in the last row.

**The tie-break is the generator's real failure mode, and it has been read as the rulebook's.** With no criterion
at all the search returns a line whose mean melodic interval is **0.76 scale steps** and whose entire compass over
the span is **1.91** — against the composer's 1.66 and 6.94 on the very same voices. It scores **1.3%**.
[§8.6](#86-realisation-and-the-first-notes) diagnosed the fills as too narrow and
[§8.8](#88-a-criterion-that-is-not-local-and-the-shape-of-every-step-6-failure) built a criterion to widen them;
both were describing what a shortest path does when nothing distinguishes its paths, which is to keep the first one
it found, and the first one barely moves. That is a fact about [§2.5](#25-the-search-is-a-shortest-path)'s search,
not about Fux.

**The tier is six, not one.** Read against `tie-break only`, all six together are worth `+4.8` in Bach; the best
single prohibition, `perfect consonance`, is worth `+2.6`; and four of the six are worth under a point. No subset
carries it.

**And the same fact explains both halves of the experiment: every positive criterion has a cheapest way to be
satisfied, and a shortest path finds it.**

- **move by step** is satisfied *perfectly* by oscillating between two adjacent notes. It got exactly what it asked
  for — mean interval `1.04`, against the composer's `1.66` — and left the compass at `3.55`. The prescription was
  obeyed and the line still goes nowhere.
- **state the harmony** is satisfied by holding one chord tone: mean interval `0.91` in Bach, `0.31` in the
  Renaissance, compass `3.83` and `0.50`.
- **move against** charges similar motion and leaves oblique motion free, so never moving is optimal: `0.88` and
  `0.36`.

> **A prohibition composes safely under a minimiser and a prescription does not.** Not doing something is what a
> search does by default. Doing something has a cheapest way to be done, and the minimiser finds that instead of
> the thing meant.

Which finally explains the six. The tier does not collapse because `repeated note` charges the degenerate solution
the other five would otherwise take. The criteria are **mutually blocking degeneracies** — that is why no subset
works, and why three prescriptions cannot stand in for them.

**And `move against` turns out to be `direct motion` restated.** The two rows are identical to every printed digit
— same mean interval, same compass, same agreement, same standard error, on both corpora. The soft rule fires on
similar or parallel motion; the prescription charges every voice this one moves *with*; they are one predicate
reached from two directions, and the coincidence is a free cross-implementation check on both. It also says what
the prohibition/prescription distinction is *not*: not the sign of the sentence, but whether the criterion has a
degenerate optimum.

**The tier reproduces the composer's melodic statistics and does not reproduce the composer's notes.**

| | `soft(6)` | *the composer* | uniform draw |
|---|---:|---:|---:|
| Bach windows — mean interval | 1.45 | *1.66* | 2.78 |
| Bach windows — compass | 6.97 | *6.94* | 10.89 |
| Bach entry spans — mean interval | 1.43 | *1.49* | 2.80 |
| Bach entry spans — compass | 6.41 | *6.14* | 10.29 |
| 15th-c. — mean interval | 1.25 | *1.14* | 2.35 |
| 15th-c. — compass | 1.63 | *1.39* | 3.14 |

Three protocols and two centuries, and the aggregate match is close every time — a compass of `6.97` against
`6.94`. Meanwhile the uniform draw misses both statistics by nearly a factor of two in every row and **scores
higher on note agreement**. Matching a composer's melodic statistics is not writing a composer's notes, and here
the two come apart far enough to be measured.

#### A claim in §8.6 does not survive, and step 6's first proposal is reinstated

[§8.6](#86-realisation-and-the-first-notes) reads: *"Minimising the soft criteria beats maximising them, and beats
not optimising at all. The soft tier is weak and it is real, and using it is better than ignoring it."* The first
clause holds. The second does not.

Run on §8.6's **own** spans, with §8.6's own tier and plan, both accountings side by side:

| objective | pooled over notes | paired per span |
|---|---:|---|
| no objective (uniform) | 6.9% (−0.95) | **−0.74 ± 0.97** |
| `tie-break only` | 0.8% (−7.00) | −6.93 ± 1.01 |
| `soft(6)` minimised | 7.8% | — |
| `soft(6)` maximised | 4.9% (−2.96) | −3.00 ± 1.01 |

The pooled column reproduces §8.6's table exactly, which is what makes the paired column readable: **`−0.74 ± 0.97`
is nothing.** Pooling weights a span by how many notes it has and counts each of the eight draws' notes separately,
and that is where the claim came from. On [§8.8](#88-a-criterion-that-is-not-local-and-the-shape-of-every-step-6-failure)'s
windows, with five times the spans, the same paired comparison runs the *other* way and clears the bar on both
corpora: **+1.07 ± 0.31 in Bach and +4.64 ± 0.61 in the Renaissance.** The two paired estimates are 1.8 standard
errors apart, which is to say they agree with each other; what neither supports is the sentence. **Using the soft
tier is not better than ignoring it, and on the larger sample it is worse.**

So [§9](#9-roadmap) step 6's *first* proposal is reinstated. It was recorded as "done, and it does not work" on the
strength of `6.9%` against `7.8%` — the pooled comparison. Paired, drawing uniformly from the legal set is never
significantly worse than optimising over it and is significantly better on both corpora at the larger sample.
**Sampling worked; the accounting hid it.**

With one practical caveat that the `tie-break only` row exists to supply: the objective can be dropped **by
sampling** and not by setting the weights to zero. To a shortest path, "no objective" means every path ties and the
first one wins, and that scores `1.3%` — five points below the tier it replaced. The uniform draw is a different
object and it is the one that wins.

**Nothing among the prescriptions is adopted**, none having beaten the tier on both corpora. What clears the bar is
**removing the objective and drawing instead** — which is the only change step 6 has produced that survives its own
decision rule, and it arrives by retracting a claim rather than by adding a criterion.

**And it is now the endorsed configuration in code**, as `Problem::drawing()`, so that step 7 generates the way
this section says rather than the way [§8.6](#86-realisation-and-the-first-notes) did. It exists as a constructor
rather than as advice because the advice has a trap in it: zeroing the weights is *not* dropping the objective,
since to a shortest path it means every path ties and the first found wins — the `1.3%` row. `drawing()` therefore
also asks for a draw, and `Solution::chosen()` returns that draw rather than the tied path. A test asserts the two
differ, so the trap is closed rather than described.


### 8.11 Marpurg's tonal answer: one rule exact, one wrong, and the treatise knew which

`src/answer.rs`. The first thing this project has transcribed from a treatise of **Bach's own circle** rather than
from Fux, and [§9](#9-roadmap)'s standing open problem — Fux is 1725 and Palestrina-style vocal, the WTC is 1722
and keyboard — is why. Marpurg, *Abhandlung von der Fuge* (1753), drittes Hauptstück, *"Vom Gefährten"*.

The chapter rests on **two Grundsätze**. The answer's melody must be made *similar* to the subject's — same figure,
same note values, same intervals in the same proportion. And *"es muß recht moduliert werden"*: it must not carry
the music into a foreign key. **The two conflict**, because *"die Octave aus zwey ungleichen Hälften besteht"* —
tonic up to dominant is five notes and dominant up to tonic is four — so a subject crossing between the halves
cannot both keep every interval and stay in the key.

Marpurg's resolution is a *Vertauschung*: skipping a degree in the larger half or doubling one in the smaller,
which he tabulates as a substitution of melodic intervals — a unison for a second, a second for a third, up to a
seventh for an octave, and the reverse. **One interval changes, by exactly one degree.** That is what the
transcription models: transposing one note up a fifth and the next up a fourth widens or narrows the interval
between them by one degree and by nothing else, so a single change of leg along the subject *is* a single
*Vertauschung*.

Where the change falls, Marpurg settles by a rule of thumb — look forward rather than back — and then by thirty
worked examples on his plates. **A rule of thumb is not transcribable and worked examples are not a rule**, so
`answer.rs` does not pick a point: it enumerates every point the stated rules leave open, and what it returns is a
set. [§8.7](#87-the-species-as-a-whitelist-and-why-it-does-not-tighten-anything)'s question then applies unchanged
— a whitelist is worth having only if the music stays inside it *and* staying inside it means something — so the
set's size is reported beside its coverage.

**One instance first**, since a percentage over 24 cases is worth nothing without a case a reader can check. BWV
856, F major, by scale degree with 1 the tonic:

| | | |
|---|---|---|
| Führer | `5 6 5 4 5 7 1 2 3 4 5 4` | opens on the **dominant** |
| Gefährte, Bach's | `1 3 2 1 2 4 5 6 7 1 2 1` | |
| plain fifth | `2 3 2 1 2 4 5 6 7 1 2 1` | wrong on the first note |
| Marpurg | `1 3 2 1 2 4 5 6 7 1 2 1` | right, and 1 of the 11 answers his rules admit |

The subject opens on the dominant; Bach answers on the tonic, which transposition does not do. The mutation falls
at the first interval, where a second becomes a third — a line of Marpurg's own substitution table — and everything
after it is a plain fifth. This is the thing the chapter is about, reproduced.

#### The two rules, each on the note it is about

The unit is the exposition's `Führer`/`Gefährte` pair, the first two annotated entries, compared **by scale
degree** since the answer sits in another voice at another octave. All 24 fugues yield a usable pair.

| | all cases | where it differs from a plain fifth |
|---|---:|---:|
| **Rule I** — first note: tonic and dominant answer each other | **100.0%** of 21 | **100.0%** of 7 |
| **Rule II** — last note: tonic/dominant, and third for third | 66.7% of 24 | **0.0%** of 4 |
| Rule II, retried at every subject end the ground truth offers | — | **0.0%** of 4 |

**The right-hand column is the whole measurement.** Where a rule says *answer at the fifth* it is saying what
transposition does anyway; only where it says *answer at the fourth* is it earning anything. Read that way:

**Rule I is exact.** Seven WTC subjects open on the dominant, and in all seven Bach answers on the tonic — not on
the supertonic that transposing up a fifth would give. Zero exceptions. Set beside
[§8.2](#82-the-rulebook-stratified-by-two-corpora), where Fux's transcribed rules were measured at 8.0, 21.4, 71.1
and 90.9 violations per thousand, **this is the first rule this project has transcribed that Bach does not break
once.**

**Rule II is not weak but wrong.** Its 66.7% is entirely the free cases; in all four where it says something, Bach
does the opposite. And that is not an artefact of where the subject is cut, which
[§3.3](#33-the-subject-is-input-and-its-boundary-is-contested) would otherwise be entitled to object: the ground
truth records dissenting readings of every subject's end, the rule was retried at each of them, and it fails at
every one.

**And Marpurg knew.** He states Rule I flatly — *"die Haupttonsnote und die Dominante müssen allezeit einander
antworten auf der ersten Note"*. He hedges Rule II himself, in the sentence that states it: it *"öfters nach
Beschaffenheit der Umstände ihre Ausnahmen leiden kann"*. **The treatise's own confidence tracks the measurement,
rule for rule**, which is not something Fux's text ever did here — §8.2 had to discover the stratification from
outside, because the *Gradus* asserts its melodic prohibition exactly as firmly as its parallel-fifth one.

#### Whole answers, and what is still missing

| condition | agrees with Bach | median set |
|---|---:|---:|
| real answer, up a fifth | 41.7% | 1 |
| real answer, up a fourth | 4.2% | 1 |
| Marpurg, Rules I and II | 41.7% | 1 |
| **Marpurg, Rule I only** | **62.5%** | 14 |

Thirteen of the 24 answers are neither plain transposition — the tonal ones, which are the only place a treatise
can earn anything — and Marpurg's set contains three of them.

**Applying Rule II as a filter costs more than it buys.** It leaves coverage exactly where transposition already
was, because it gains three tonal answers and **refuses four that Bach wrote as plain transpositions** — BWV 848,
854, 863 and 865. A rule that admits too little is wrong in a way a loose one is not, and dropping it takes
coverage from 41.7% to **62.5%**.

**What remains is the mutation's place, and it is exactly what Marpurg declined to state as a rule.** With Rule I
alone the admissible set has a median of **14** members and contains Bach's answer five times in eight. So the
transcription localises the answer from the whole space of transpositions to a shortlist of about fourteen, and
then stops — because the chapter stops, and hands the reader thirty worked examples instead. That is a
**fourteen-fold** shortlist against [§8.6](#86-realisation-and-the-first-notes)'s `10¹²`, and it is the first time
in this document that a transcribed rulebook has narrowed anything to a number a person could read.

#### And whether plain transposition is the right model for the rest

[§8.3](#83-the-clique-test) and [§8.4](#84-capacity-ranks-subjects-and-cannot-design-one) place **every** entry by
plain diatonic transposition, which the above has just shown is wrong for the comes. Whether it is wrong for the
others is a separate question, and the same machinery answers it — all 232 annotated entries against the first, at
every diatonic level:

| | |
|---|---:|
| an exact diatonic transposition of the subject | **78.9%** |
| not that, but an answer Marpurg's rules admit | 2.2% |
| neither | 19.0% |

Levels taken, commonest first: **unison ×100, fifth ×36**, fourth ×16, sixth ×13, third ×10, seventh ×5, second ×3.

**So §8.3's model is broadly right and §8.4's is the exposed one.** Tonal answers are five entries in 232, because
mutation is a feature of the *exposition's comes* and most entries are not that — which is exactly where Marpurg's
chapter puts it. The clique test, which places entries at a fixed transposition and asks how many fit, is
measuring something real. [§8.4](#84-capacity-ranks-subjects-and-cannot-design-one)'s design objective is the one
that turns on the dux/comes relation specifically, and the 13 of 24 above are its population, not the 5 of 232.

The 19% that are neither are Bach varying his own entries, and they are also a caveat on the annotation: an entry
altered past recognition is still annotated as an entry.

**Two things this does not claim.** It is not tested on two corpora, and cannot be: the tonal answer is a device of
tonal fugue and the 15th-century control has neither the annotations nor the exposition
([§10.4](#104-how-the-samples-were-taken)). And 24 pairs is a small sample — Rule I's seven discriminating cases
are seven, not seven hundred. What makes them worth reporting is that they are **seven out of seven**, on a rule
stated before they were looked at, in the one place the rule makes a claim transposition does not.

### 8.12 The fourth, and the scope a dissonance is judged in

`corpus::Fourth`. [§9](#9-roadmap)'s oldest open problem, and the one lead it had.
[§8.2](#82-the-rulebook-stratified-by-two-corpora) measured `UnpreparedDissonance` and `UnresolvedDissonance` at
**8.0 and 71.1** per thousand slices in the Renaissance and **21.4 and 90.9** in Bach, which is why they were
stratified out of the hard tier; and
[§8.7](#87-the-species-as-a-whitelist-and-why-it-does-not-tighten-anything)'s whitelist could not replace them.

What §8.7 did leave was a suspicion about **which interval** is doing it: the perfect fourth accounts for 31% of
Bach's flagged dissonances and 44% of the Renaissance's. And the fourth is the one interval whose quality a *pair*
cannot determine. Over a supporting bass it is a consonance; only against the bass is it not.
[§2.2](#22-counterpoint-is-a-finite-automaton)'s automaton judges a pair and therefore cannot see the difference,
which would make these two rules wrong in their **scope** rather than in their content — *"a change to the scope a
rule is judged in rather than to the rule"*, as §8.7 put it.

[§8.11](#811-marpurgs-tonal-answer-one-rule-exact-one-wrong-and-the-treatise-knew-which) then supplied a second and
independent reason to look at exactly this interval. Marpurg's chapter on invertible counterpoint arrives at it
from the other side: the fifth must be handled as a dissonance **because inversion turns it into a fourth**. Two
unrelated routes to one suspicion is the strongest signal this project has had about these rules.

Three scopes, both corpora. `pairwise` is what §8.2 measured; `over a bass` is the proposal; `consonant` exempts
every fourth and is the blunt control, so that the principled rule cannot take credit for what merely dropping the
interval would do.

| corpus | scope | unprepared /1k | unresolved /1k | both |
|---|---|---:|---:|---:|
| Bach | pairwise | 21.4 | 90.9 | 112.3 |
| Bach | **over a bass** | 18.8 | 83.8 | **102.6** |
| Bach | consonant | 16.3 | 70.7 | 87.1 |
| 15th-c. | pairwise | 8.0 | 71.1 | 79.1 |
| 15th-c. | **over a bass** | 3.4 | 57.8 | **61.2** |
| 15th-c. | consonant | 2.9 | 46.9 | 49.8 |

The `pairwise` rows reproduce §8.2's four figures exactly, which is what makes the rest of the table readable.

**The lead is answered, and it is not the answer.** Judging the fourth against the bass removes **9%** of what the
two rules flag in Bach and **23%** in the Renaissance, and leaves them firing at **102.6** and **61.2** per
thousand. A rule that fires a hundred times in a thousand slices of the music it was written for is not a hard
rule, and rescoping does not make it one. The oldest open problem stands, with one hypothesis eliminated.

**Why so little, and the number that explains it.** Only **38%** of Bach's flagged fourths have a voice below them;
the other 62% are against the bass, where the pairwise rule was right to fire all along. In the Renaissance it is
**61%**, which is why the same correction is worth nearly three times as much there. That is a repertoire split of
the kind §8.2 keeps producing, and for once the mechanism is plain rather than inferred: more voices and more
upper-voice writing mean more fourths with something underneath them.

**The correction is right anyway, and it is not adopted for that reason.** That a fourth over a bass is a
consonance is not a contested claim, and `corpus::Fourth::OverBass` implements it. But this document's tables are
the record of the runs as made, the two rules are already outside the endorsed tier
([§8.2](#82-the-rulebook-stratified-by-two-corpora)), and a scope correction worth 9% does not change any verdict
that rests on them. It is measured, kept runnable, and left off — and what it establishes is that **the fourth was
never the whole of the problem**, so the replacement §9 still wants has to explain the other 90% too.

### 8.13 Are episodes sequences, and how much of a fugue is episode

`src/episode.rs`. [§2.4](#24-form-is-a-grammar)'s grammar has exactly one production that claims something about
the music rather than about structure:

```
Episode → Sequence(motive, transposition pattern, n)
```

`Exposition` is now backed by [§8.11](#811-marpurgs-tonal-answer-one-rule-exact-one-wrong-and-the-treatise-knew-which)
and `Middle+` says almost nothing, but this one step 7 would inherit unexamined —
[§8.6](#86-realisation-and-the-first-notes) and [§8.10](#810-replacing-the-soft-tier-and-the-degenerate-optimum-of-every-positive-criterion)
are both cases where a claim went several steps before anyone checked it.

**An episode is a span where no annotated entry sounds**, which is the definition the ground truth supports without
any judgement added. **A sequence** is the whole texture restated after a fixed period with every voice moved the
same non-zero number of diatonic steps and the rhythm exact — a restatement at the *same* pitch is a repetition,
not a sequence, so zero is excluded. That is strict: a sequence Bach decorates or reharmonises on its second
statement is not found, so every figure below is a **floor**. Which is why the entry spans go through the identical
detector as the control.

| | mean of the span | spans with none at all |
|---|---:|---:|
| **episodes** (154) | **13.3%** | 70.8% |
| entry spans (284) | 1.3% | 95.4% |

Difference **+12.0 ± 2.0** points, unpaired — an episode and an entry span are different objects and there is no
pairing to be had. Six standard errors.

**The production is directionally right and quantitatively weak.** Episodes are where sequences live, at **ten
times** the rate of the passages that are not episodes, and that is not in doubt. But 70.8% of episodes contain no
strict sequence at all, so `Episode → Sequence` is not a definition. **A grammar that emitted every episode as a
sequence would be wrong about seven episodes in ten** — and the honest form of the production is a *tendency with a
rate*, not a rewrite rule. Some of that 70.8% is the detector's strictness and some of it is Bach, and this
measurement cannot separate them; what it can say is that the strict reading is not available.

**And the number that was not being asked for.** Episodes are **54% of the book by duration**, 154 of them across
24 fugues, median **3.0 bars** long. More than half of a fugue is not subject.

That is the largest single fact about fugal form this document has, and it lands on
[§8.6](#86-realisation-and-the-first-notes) rather than on §2.4. The realiser fills free voices **against a held
entry**; in a little over half the music there is no entry to hold. Step 7 therefore cannot be a matter of
scheduling entries and calling the existing search between them — the majority case is a passage with nothing
fixed in it at all, which is a different problem from the one
[§8.6](#86-realisation-and-the-first-notes) solves and a harder one, since
[§8.9](#89-a-better-plan-and-the-first-lever-that-moves-more-than-a-point) showed that what makes a fill agree with
Bach is mostly the harmony under it and an episode has to supply its own.

### 8.14 Key-finding, and the ground truth that was already here

`src/key.rs`. [§9](#9-roadmap)'s second open problem, stated there as *"a real functional test needs degree
successions relative to a **local** key, and fugues modulate constantly"*. Since
[§8.9](#89-a-better-plan-and-the-first-lever-that-moves-more-than-a-point) there is a second reason: a form grammar
is a key plan before it is anything else, and **a key plan nothing can check is not a claim.**

Built by [§8.5](#85-the-harmonic-analyser)'s instrument — Viterbi over the 24 keys, charging `μ` to change, linear
in the vocabulary because the transition is zero to stay and `μ` to move. Segments are **bars** rather than onsets:
a chord lasts a beat and a key lasts phrases, and segmenting a key search at every onset would ask a smoothing
parameter to undo a segmentation mistake.

One thing a collection cannot do, and the fix is ordinary theory rather than a parameter. **C major and A minor are
the same seven pitch classes** — the objection `Piece::tonic` exists to answer for the global key. What separates
them is the **raised seventh**, so a minor key here carries its leading tone. That has a cost worth stating: minor
then admits eight pitch classes to major's seven, and a fit statistic leans minor on its own. A tonic-triad bonus
leans back, fixed at the 0.2 §8.5 uses for its bass bonus; only `μ` is swept.

**The validation is external, and it was in the repository from the beginning.** The 106 typed cadences §8.5 used
carry Hepokoski–Darcy labels — `I:PAC`, `V:PAC`, `vi:PAC`, `III:PAC` — and **a roman numeral names the local key**.
All 106 parse. Somebody else annotated them, for another purpose, before this question was asked.

| μ | key correct | tonic correct | **away from home** | bars per key |
|---:|---:|---:|---:|---:|
| 0.00 | 45.3% | 54.7% | **36.5%** | 1.7 |
| **0.25** | **45.3%** | **56.6%** | **35.1%** | **6.4** |
| 0.50 | 34.9% | 48.1% | 27.0% | 14.7 |
| 1.00 | 30.2% | 45.3% | 17.6% | 28.6 |
| 2.00 | 24.5% | 39.6% | 10.8% | 47.3 |
| 4.00 | 23.6% | 38.7% | 10.8% | 52.5 |

**Naming the piece's own key at every cadence scores 30.2% and costs nothing**, so the third column is the
measurement — the **74** cadences that are somewhere else, where a guess of *home* scores zero by construction.
That reading is [§8.11](#811-marpurgs-tonal-answer-one-rule-exact-one-wrong-and-the-treatise-knew-which)'s, arriving
for the second time in two sections.

**It works, and it is weak.** A third of the modulations, and a little over half the cadences if the mode is
forgiven. Set against §8.5, where the chord analyser reached 70–80% on these same cadences with a 14% baseline,
this is a much harder problem answered much less well.

**Two things are nevertheless in its favour.** `μ = 0` scores the same as `μ = 0.25` overall and produces a key
rhythm of **1.7 bars**, which is not a key rhythm — it is the analyser re-choosing at every opportunity and being
right by accident as often as by analysis. At `μ = 0.25` the key changes every **6.4 bars**, which is roughly right
for these pieces, and the accuracy peak sits exactly there. **The accuracy peak coinciding with the musically
plausible band is the second time this instrument has done that**, §8.5 being the first, and it is the main reason
to trust the shape of the curve even where its height is poor.

And the eleven points between `key correct` and `tonic correct` are almost all mode errors, which is the predicted
cost of the eight-note minor collection, showing up where it was said it would.

**What step 7 gets.** A key plan can now be checked rather than merely asserted — which is more than it had. But at
roughly a third on the modulations, the check is **coarse**: it can catch a plan that wanders somewhere Bach never
goes, and it cannot referee between two plausible plans. [§2.3](#23-harmony-is-a-second-automaton)'s functional
half still should not be built on this, and §9's open problem is **narrowed rather than closed**.

### 8.15 Does the form grammar derive the book?

`src/form.rs`. [§2.4](#24-form-is-a-grammar) writes ten lines of productions and asserts that form is a grammar.
[§8.13](#813-are-episodes-sequences-and-how-much-of-a-fugue-is-episode) checked one of them; this checks the rest,
in the only way a grammar can be checked — **does it derive the sentences it claims to be a grammar of?**

What is parsed is the **plan**, not the notes. The ground truth annotates every subject entry and every typed
cadence, which is what §2.4's non-terminals range over and what a form grammar would have to emit. Each production
gets its own rate, on [§8.2](#82-the-rulebook-stratified-by-two-corpora)'s principle that a rulebook is not one
thing: a grammar failing 22 of 22 for one reason is a different object from one failing for four.

**22 fugues**, ten of them in four voices or more.

| production | holds |
|---|---:|
| `Exposition`: one entry per voice, every voice used | 59.1% |
| `Exposition`: alternating dux level and comes level | 40.9% |
| `Exposition`: those entries run unbroken, no episode | **18.2%** |
| `Fugue → Exposition Middle+`: there is a middle | 95.5% |
| `Final → … Cadence`: the last cadence is at home | **100.0%** |
| **the whole derivation** | **13.6%** |

**The grammar derives three fugues in twenty-two, and the failure is entirely in one production.**

**What holds, holds completely.** Every one of the 22 ends with its last annotated cadence in the home key — a
production that never fails once. Twenty-one of 22 have at least one middle. So §2.4 is **right about the shape of
a fugue**: exposition, then middles, then home. `Middle+` runs from 0 to 9 with a median of 3, `Stretto?` is taken
by 5 of 22, and the episodes between entry groups run to a median of **3.0 bars** — the same median
[§8.13](#813-are-episodes-sequences-and-how-much-of-a-fugue-is-episode) found by an unrelated route, which is the
cross-check that says the two sections are measuring the same object.

**And `Exposition` is wrong on every count.** It is the one production that says something detailed, and the one
[§8.11](#811-marpurgs-tonal-answer-one-rule-exact-one-wrong-and-the-treatise-knew-which) was written to supply. Its
answer rule is exact; the production that uses it is not. Only 59% state the subject once in each voice across the
first `V` entries; only 41% alternate; and **82% of expositions contain an episode**, which the production has no
symbol for at all.

That last figure is the largest and the least surprising to anyone who has looked at a fugue: the link between
expositional entries is ordinary practice, and `Entry (Countersubject Entry){V−1}` forbids it by construction. So
the corrected production is roughly

```
Exposition → Entry (Link? Countersubject Entry){V−1} Redundant?
```

with the link optional, and a redundant entry allowed — which is what the 41% that fail the first row look like.

**Three faults in the instrument were found and fixed before these numbers**, and they are worth recording because
each produced a plausible table. Grouping entries by the distance between their *starts* rather than from where one
**ends** split every exposition into as many groups as it had voices and reported 0% on all 22. A verdict for
`Middle → Episode Entry+` was true by construction once a group is defined as a run with no episode in it — a check
that cannot fail, replaced by a measurement of the `+` it actually claims: middle groups hold **1.35** entries on
average. And judging an entry's level by whether its first note is the tonic called seven expositions
non-alternating for alternating perfectly, since a subject beginning on the dominant has a dux on degree 4;
[§8.11](#811-marpurgs-tonal-answer-one-rule-exact-one-wrong-and-the-treatise-knew-which)'s Rule I is what *level*
means, and using it moved that row from 22.7% to 40.9%.

> **A grammar with an unbounded `+` in it is hard to falsify, and the parts of §2.4 that survive here are the parts
> that say least.** `Middle+` accepts any number of middles and 21 of 22 satisfy it. `Final → … Cadence` is exact
> and is one bit. `Exposition` is the only production with a shape, and it is the only one that fails.

**What step 7 gets.** Not a grammar to implement, but four facts to build one from: the exposition takes links and
sometimes a redundant entry; a fugue has a median of three middle entry groups of about 1.35 entries each; episodes
between them run about three bars; and it ends at home, always. Together with
[§8.13](#813-are-episodes-sequences-and-how-much-of-a-fugue-is-episode)'s finding that episodes are **54% of the
book by duration**, the object to build is now clear and it is not the one §2.4 describes.

### 8.16 A fugue, from a subject

`src/compose.rs`, `out/fugue.mid`. Everything before this filled voices **against music that already existed**.
[§8.6](#86-realisation-and-the-first-notes) held one of Bach's entries and reconstructed the others;
[§8.3](#83-the-clique-test) placed entries into a span Bach had written. This emits the span too, so for the first
time nothing in the output is Bach's except the subject.

The grammar is [§8.15](#815-does-the-form-grammar-derive-the-book)'s corrected one and its numbers are §8.15's and
§8.13's rather than §2.4's: an exposition **with a link**, three middle entry groups, episodes of **three bars**, a
close at home. From BWV 847's subject, in three voices:

| bar | | key |
|---:|---|---|
| 1 | entry, voice 0 | home |
| 3 | entry, voice 1 — the comes, by [§8.11](#811-marpurgs-tonal-answer-one-rule-exact-one-wrong-and-the-treatise-knew-which)'s Rule I | V |
| 5 | episode — the link §2.4 forbids and 82% of expositions contain | home |
| 6 | entry, voice 2 | home |
| 8, 11 | episode, then entry | V |
| 13, 16 | episode, then entry | VI |
| 18, 21 | episode, then entry | IV |
| 23, 26 | episode, then the last entry | home |

**Twelve blocks, 27 bars, filled in 0.5 seconds.** Read back through §8.15's own parser it covers the voices,
alternates, has a middle and ends at home — and fails `exposition runs unbroken`, which is the link, written on
purpose. Against §8.2's checker: **785 slices, zero violations on the confirmed tier**, and 58 on the full five —
`73.9` per thousand against Bach's `112.3`
([§8.12](#812-the-fourth-and-the-scope-a-dissonance-is-judged-in)).

**That is the fourth version, and the first two corrections came from a listener rather than from a test.** The two
subsections below are about how, because in both cases the answer was worth more than the fugue. The fourth
correction came from **clippy**, which had never been run: the episode plan read
`quality: if deg == 4 { 0 } else { 0 }`, an intention written down and not finished, and giving the local dominant
its seventh took `73.8` to `70.1`. Splitting the repository into a library and a binary
([§10.6](#106-using-it-as-a-library)) moved it once more, by one bar of link — the exposition's link now goes to a
voice chosen by the same rule as every other block rather than to voice 0 by hand.

| | dissonance /1000 | confirmed-tier violations | fill |
|---|---:|---:|---:|
| first — `conf+melodic` | 366.2 | 0 | 3.9s |
| second — the full tier | 90.8 | 1 | 0.8s |
| third — continuous voices | 73.8 | 0 | 0.5s |
| **fourth — a seventh on the dominant, and the library split** | **73.9** | **0** | **0.6s** |

The last figure is one draw of many. Over twelve seeds the rate is **74.1 ± 2.3**, running from `70.1` to `77.7`,
and every one of them is far below Bach's `112.3`. That matters more than the middle of it: **the seed changes
which notes are written and barely changes how good they are.** A caller re-drawing to hear something different is
exploring the legal set, not hunting a better score — which is the honest thing to tell anyone given a button that
does it.

#### The second listening test, and a rule that is wrong as a description and right as a constraint

The first version of this was generated on `conf+melodic`, the tier
[§8.6](#86-realisation-and-the-first-notes) onwards uses, and it was listened to. The report: **"very large
dissonance, and at times it sounds like a cacophony."**

That is worth recording for a reason beyond the fault it names.
[§8.6](#86-realisation-and-the-first-notes) records a listening test that **disagreed** with the numbers — the
reconstruction scored 7.8% against Bach and a listener called it *"on par, nothing is better or worse than Bach
himself"*. This one **agrees** with them, and the number it agrees with is `366.2` violations per thousand slices
against Bach's `112.3`. An instrument that matches the ear once and misses it once is more useful than one that has
never been checked, and the difference between the two cases is which rules were being counted.

**The fix is one word and it inverts §8.2.** Generated on the **full five-rule tier** instead:

| tier the generator writes against | dissonance rules fire, per thousand | time |
|---|---:|---:|
| `conf+melodic` — §8.6's tier | **366.2** | 3.9s |
| `full(5)` | **90.8** | 0.8s |
| *Bach himself* ([§8.12](#812-the-fourth-and-the-scope-a-dissonance-is-judged-in)) | *112.3* | |

**A factor of four, and it lands below Bach's own rate.** It is also five times faster, because a tighter tier
prunes the search — which is [§8.6](#86-realisation-and-the-first-notes)'s `full(5)` rows arriving again in a place
where they matter.

Now, [§8.2](#82-the-rulebook-stratified-by-two-corpora) **stratified those two rules out of the hard tier**, and
was right to: they fire at 21.4 and 90.9 per thousand on Bach, so as a description of what Bach does they are
badly wrong. But a generator that omits them writes cacophony, and one that enforces them writes less dissonance
than Bach.

> **A rule can be wrong as a description and right as a constraint.** §8.2 asked which rules describe the
> repertoire and answered correctly. A generator is asking a different question — what may I write — and the
> answer is not the same. The two dissonance rules capture something real about how dissonance has to be
> *handled*; they are merely too crude to catch the exceptions Bach takes.

This is the first place in this document where the endorsed tier and the generating tier come apart, and it is why
`--gen-tier` exists separately from `--tier`. Everything §8.6 to §8.15 measures is a description and uses
`conf+melodic`; §8.16 generates and uses `full(5)`.

**It costs one thing, and the cost is the seam rather than the tier.** On the full tier the piece carries **one**
confirmed-tier violation where `conf+melodic` carried none — a single parallel at a block join, which is the
limitation the section above describes and not a consequence of the stricter rules. Tightening the tier makes each
block harder to fill, so more of them run close to the edge of what is legal, and the one place the search cannot
see is where that shows.

#### The third listening test, and four tenths of a second of nothing

The second version was listened to. Dissonance *"reduced meaningfully"*, and then a new report: **"0.4s long
silence breaks repeating every 3-6s."**

That is a diagnosis, not a complaint. At the 76 to the minute this writes, four tenths of a second is **an eighth
note** — and BWV 847's subject begins on an **upbeat**, its first note at tick 120 rather than 0. Every free voice
was given the subject's rhythm *with its onsets*, tiled, so every voice inherited the same 120-tick gap at the head
of every tile. They all rested in the same place, together, once per tile. Three voices breathing in unison is
silence, and no rule in this document has anything to say about it: the checker counts what sounds against what
sounds, and nothing sounding is not a violation of anything.

The fix is to lay the subject's note **values** end to end from the first tick rather than copying its onsets, and
to rotate the sequence by a different phase per voice so that even the note boundaries do not line up. The
dissonance rate fell again as a side effect — `90.8` to `73.8` — because a texture with no holes in it is a
texture the search has more chances to get right, and the piece gained 144 slices of actual counterpoint.

Fixing it exposed **two more of the same kind**, both found by tests once the fixture was given an upbeat of its
own. A voice that finished an entry and immediately took the next episode's motive **leapt nine steps between
them**, because both lines are *placed* and nothing in the search stands between the end of one and the start of
the other; the derivation now never gives two consecutive blocks to the same voice. And the test fixture itself had
no upbeat, so **it could not have caught the bug it was written for** — a subject beginning on the beat has no gap
to share. It has one now.

> **The checker cannot hear silence.** Every instrument in this document measures a relation between notes that
> sound. A fault that consists of *nothing sounding* is invisible to all of them, and was found by the only
> instrument that had not been used until §8.6 — somebody listening. The test that now guards it asserts a property
> of the piece rather than of the rhythm function, so any future way of inventing rhythm has to satisfy it too.

#### Three things it had to be built around, and what each cost

**Two free voices.** [§2.7](#27-where-a-solver-takes-over-from-the-dp) predicted the wall at four and §8.6 measured
it at two, so three voices is the scope and half the book is out of reach. But a fugue is exactly the case where
*which* voice is free changes: the subject moves. So the fill runs **one block at a time**, the placed voice held
and the other two free — two, which is the wall exactly.

That created a seam, and the seam had a fault in it. The search's state resets at every block edge, so a parallel
fifth *across* a join was invisible to it and visible to the checker — the fault
[§8.6](#86-realisation-and-the-first-notes)'s first test exists to prevent, arriving at the join rather than in the
middle. `Problem::prior` now carries the previous slice's pitches across for **every** voice, not only the free
ones: a parallel is a fact about two voices moving together, so a search that knows where its own voices came from
but not where the held one came from cannot see one. What still resets is the obligation state, so a dissonance
owed across a boundary is forgiven. **No block contains counterpoint the checker flags**, and that is what the test
asserts — not that the whole piece contains none, because those are different claims and only the first is one the
search can make.

**Episodes have nothing held in them.** §8.13 found episodes are 54% of the book, and in one no subject sounds, so
all three voices would be free. The way out is §2.4's own `Sequence(motive, …)`: a motive from the subject's head is
*placed* in one voice and sequenced down by step, leaving two free. **This is a commitment §8.13 already priced** —
only 13.3% of Bach's episodes are strictly sequential, so this writes a kind of episode that is a minority of the
book's, and it writes every one of them that way.

**Rhythm is data.** [§2.6](#26-what-is-not-a-variable-rhythm) makes rhythm an input, which is what keeps the search
a shortest path — and a reconstruction gets it from the piece it is reconstructing. A generator has to invent it.
Every free voice here takes **the subject's own rhythm**, tiled. It is the cheapest defensible choice and a real
limitation: a fugue whose accompanying voices all move in the subject's rhythm is a stiffer thing than Bach writes,
and this is where that shows.

#### What the tests found, which is the part worth keeping

Four faults, each in code that had already produced a plausible-looking fugue.

The **parallel fifth at the seam**, above. A voice **entering by a leap of an eleventh**, because an entry's first
note is placed by the derivation and no care in the fill can reach it — the fix is Bach's, to have the voice **rest
before it enters**, and there is then nothing to leap from. §8.15's parser reporting `FAIL` on all four checks
against the generator's own output, because it read each entry's degree at the block's first tick and **BWV 847's
subject has an upbeat**, so nothing sounded there and every entry was silently dropped. And a test asserting that
no voice leaps more than an octave across a join, which is true only where the join was **kept** — a block whose
join the generator dropped enters cold, and a voice with nothing behind it may legally leap a tenth.

That last one is why `Relaxed` reports *which* blocks lost a constraint and not only how many. **One of twelve lost
the join, and none lost the plan** — and the order matters: the join is this generator's own convenience and the
plan is §2.3's obligation system, which is *also* what keeps the search tractable. Dropping the plan first turns a
dead block into an exploded one, which is worse, and that is how it was found.

**What this is not.** It is not a good fugue. The dissonance rate is now below Bach's and the listening report
that produced that fix was about dissonance, so the next faults to hear are the ones the numbers cannot see: every
accompanying voice moves in the subject's rhythm, every episode is a strict sequence where
[§8.13](#813-are-episodes-sequences-and-how-much-of-a-fugue-is-episode) measured 13.3% of Bach's that way, and the
harmonic plan is the subject's own analysis transposed rather than anything that knows what a fugue's middle is
for. What it
is, is the first thing here that produces a whole piece and then submits it to every instrument this document has
built — the grammar it came from, the rulebook, and a checker that does not know it was the generator.

---

## 9. Roadmap

Steps 0 to 5 are done and reported above. The project now produces notes and can be listened to. What remains, in
order.

6. **Selectivity**, which [§8.6](#86-realisation-and-the-first-notes) turned from a prediction into a number:
   `10¹²` to `10¹⁸` legal fills of a three-bar span, and agreement with Bach that does not respond to anything the
   rulebook does. The table there is unambiguous about which direction *not* to go: more of the same kind of
   constraint buys tractability and nothing else — though [§8.9](#89-a-better-plan-and-the-first-lever-that-moves-more-than-a-point) later sharpens that
   into *correct* constraint being the variable rather than *more* of it. **All five proposals below have now been
   built and measured, and the step is closed.** Three failed in ways worth keeping. The fourth failed while
   measuring the largest effect in the project. The fifth failed too — and in failing, retracted the claim that had
   ruled the first one out, so the step ends by adopting the proposal it opened with.

   - ~~Sample the legal set instead of optimising over it.~~ **Done, and — after one retraction — it is the one
     thing in this step that works.** Uniform sampling is built: a backward walk through the DAG weighting each
     predecessor by its path count, drawing every legal fill with probability exactly `1/total` and verified flat
     on an instance small enough to enumerate. It was recorded here as a failure on the strength of **6.9%**
     against the objective's 7.8%, and that comparison was **pooled over notes**. Paired per span
     ([§8.10](#810-replacing-the-soft-tier-and-the-degenerate-optimum-of-every-positive-criterion)) it is `−0.74 ± 0.97` on those same spans — nothing — and `+1.07 ± 0.31` and `+4.64 ± 0.61`
     in its favour on a sample five times the size, on both corpora. **Drawing beats optimising**, and it is the
     only change step 6 has produced that clears its own decision rule. One caveat the same section supplies: the
     objective must be dropped *by drawing*, not by zeroing the weights — to a shortest path "no objective" means
     every path ties and the first one wins, which scores `1.3%`. **Frequency-weighted** sampling was tried too,
     with the weights taken from Fux rather than from a corpus, and it is **repertoire-specific** — it helps Bach
     by about a point and costs 15th-century polyphony more than that
     ([§8.6](#86-realisation-and-the-first-notes)). That half stays rolled back.
   - ~~Enumerate the species as a whitelist.~~ **Done, and it does not account for the music.** Transcribed and
     run as a checker before being used as a constraint, Fux's four figures cannot explain one dissonance in five
     in either century, and reject slices at a rate between the two rules they were written to replace
     ([§8.7](#87-the-species-as-a-whitelist-and-why-it-does-not-tighten-anything)). Not adopted. It paid for
     itself twice over anyway: the perfect fourth turns out to account for **31% of Bach's flagged dissonances and
     44% of the Renaissance's**, which is a candidate diagnosis for the two dissonance rules' long-standing
     failure, and Fux's metric condition costs fourteen points in both corpora.
   - ~~A criterion that is not local.~~ **Done, and it is Fux's melodic doctrine that is local to the
     Renaissance.** One climax, a bounded compass and variety, each transcribed and each used to rerank uniform
     draws rather than to enter the search — which is how a long-range criterion is bought without the state
     explosion [§2.5](#25-the-search-is-a-shortest-path) predicts. Climax and variety are worth better than two
     points to 15th-century polyphony and nothing at all to Bach; the compass bound makes things *worse*, because
     it is an upper bound and the diagnosed failure is narrowness
     ([§8.8](#88-a-criterion-that-is-not-local-and-the-shape-of-every-step-6-failure)). Not adopted, and it is the
     experiment that produced step 6's one general finding: **the treatise prohibits excess and the generator's
     failure is deficiency.**
   - ~~A better plan.~~ **Done, and it is the largest lever this project has measured — and it belongs to step
     7.** Priced properly, paired per span and on both corpora, a *correct* harmonic plan is worth **+2.36** in
     Bach and **+3.74** in the Renaissance ([§8.9](#89-a-better-plan-and-the-first-lever-that-moves-more-than-a-point)): the only condition in the whole of step 6 to move more
     than a point in the same direction in both centuries. It is a **ceiling and not a candidate**, because
     neither honest repair to the analyser buys any of it, and the reason is a number nobody had measured — the
     plan [§8.6](#86-realisation-and-the-first-notes) has been writing against names the right chord **one time in
     six**, while reporting high confidence throughout, so no gate on that confidence can help. Two consequences.
     The gain survives coarsening to a **beat** and not to a **bar**, which is a specification handed to
     [§2.4](#24-form-is-a-grammar)'s grammar rather than to this step. And the row that removes four orders of
     magnitude of legal fills for nothing settles what constraint is: **neither tightness nor looseness predicts
     agreement, correctness does.**
   - ~~Replacing the soft tier rather than reweighting it.~~ **Done, and it is the tie-break rather than the tier
     that was the problem.** Marpurg and Kirnberger remain unread — the first is
     [freely available](https://archive.org/details/abhandlungvonder00marp) and located since, the second not — so what
     was asked is what can be asked without them: six one-hot ablations, and three positive criteria charged **in
     place of** the tier ([§8.10](#810-replacing-the-soft-tier-and-the-degenerate-optimum-of-every-positive-criterion)). Three findings. The tier is **six and not one**: no subset carries it,
     because each criterion's cheap degenerate solution is charged by another, so they are **mutually blocking
     degeneracies**. Every prescription collapses onto one — *move by step* is satisfied by oscillating between two
     adjacent notes, *state the harmony* by holding a chord tone — because **a prohibition composes safely under a
     minimiser and a prescription does not**. And the narrowness that
     [§8.6](#86-realisation-and-the-first-notes) and
     [§8.8](#88-a-criterion-that-is-not-local-and-the-shape-of-every-step-6-failure) laid at the rulebook's door is
     the shortest path's **tie-break**: with no criterion at all the search returns a line of mean interval `0.76`
     and compass `1.91`, against the composer's `1.66` and `6.94`. Nothing adopted, and the tier is now recommended
     *off* — see the first bullet.

   And escalate to a SAT/CDCL solver, which [§2.7](#27-where-a-solver-takes-over-from-the-dp) put at four or more
   free voices and [§8.6](#86-realisation-and-the-first-notes) measured at **two**. Do not layer: Schottstaedt
   reports that failing at three voices.

7. ~~**Form**, per [§2.4](#24-form-is-a-grammar)~~ — **built, and it produces a fugue**
   ([§8.16](#816-a-fugue-from-a-subject)). Twelve blocks, 27 bars, three voices, from BWV 847's subject and nothing else of
   Bach's; it parses under the grammar it came from and has **zero violations on the confirmed tier** over 628
   slices. Anders & Miranda named form as unsupported by any existing system, and it was built rather than
   borrowed. What it is not is a good fugue, and §8.16 says where to look: three times Bach's dissonance rate,
   every accompanying voice in the subject's rhythm, and every episode a strict sequence when
   [§8.13](#813-are-episodes-sequences-and-how-much-of-a-fugue-is-episode) measured only 13.3% of Bach's that way.
   The packing question still lives inside the stretto block, and **four voices still need the solver**.

   **Started, from the outside in.** The grammar's `Exposition` rule reads `Entry (Countersubject Entry){V−1}`,
   and the second `Entry` is an *answer* rather than a transposition — a distinction this document did not have
   until [§8.11](#811-marpurgs-tonal-answer-one-rule-exact-one-wrong-and-the-treatise-knew-which), since
   [§8.3](#83-the-clique-test) places every entry by plain diatonic transposition. Marpurg's Rule I now supplies
   it and is exact on Bach; his Rule II is measured and rejected; and what is still missing is where the mutation
   falls, which narrows the answer to about fourteen candidates and no further. Two things follow for the grammar.
   It must emit an **answer**, not a placement, so `Entry` is not one production. And [§8.9](#89-a-better-plan-and-the-first-lever-that-moves-more-than-a-point)'s
   requirement stands beside it: the harmony has to be scheduled **per beat**, since a chord per bar loses more
   than half of what a correct plan is worth.

   **The grammar has now been parsed against the book, and it derives three fugues in twenty-two**
   ([§8.15](#815-does-the-form-grammar-derive-the-book)). What holds, holds completely: every one of the 22 ends with its last cadence at
   home, and 21 have a middle — §2.4 is right about the *shape*. What fails is `Exposition`, the one production
   with any detail in it and the one [§8.11](#811-marpurgs-tonal-answer-one-rule-exact-one-wrong-and-the-treatise-knew-which)
   was written to serve: 59% state the subject once per voice, 41% alternate, and **82% contain an episode**,
   which the production has no symbol for. The corrected form is `Exposition → Entry (Link? Countersubject
   Entry){V−1} Redundant?`. Three faults in the parser were found and fixed first, each of which had produced a
   plausible table.

   **And the grammar's one substantive production has been checked** ([§8.13](#813-are-episodes-sequences-and-how-much-of-a-fugue-is-episode)). Episodes really
   are where sequences live — ten times the rate of the passages that are not episodes, six standard errors — but
   **seven episodes in ten contain no strict sequence at all**, so `Episode → Sequence` is a tendency with a rate
   and not a rewrite rule. The same measurement produced the largest single fact about fugal form this document
   has: **episodes are 54% of the book by duration**. The realiser fills free voices against a *held entry*, and in
   more than half the music there is no entry to hold, so step 7 cannot be a matter of scheduling entries and
   calling [§8.6](#86-realisation-and-the-first-notes)'s search between them.

8. Optional: **double fugue** — two shapes that must tile, which is where the shape-catalogue reading earns its
   keep.

### Open problems, in rough order of how much they block

- **A criterion that selects.** [§8.6](#86-realisation-and-the-first-notes) is the whole of step 6 and now the
  central problem of the project: everything downstream of it generates legal music that nothing prefers. Step 6 is
  closed and it did not solve this. What it established is where the problem is *not*: not in a heavier rulebook,
  not in a treatise weighting, not in the species list, not in a shape criterion, and not in the soft tier, which
  [§8.10](#810-replacing-the-soft-tier-and-the-degenerate-optimum-of-every-positive-criterion) shows is no better than leaving the objective out. The two things that moved anything moved it
  out of step 6 — **draw from the legal set rather than optimise over it**, and **supply the harmony**, which is
  step 7's job ([§8.9](#89-a-better-plan-and-the-first-lever-that-moves-more-than-a-point)).
- **Key-finding.** A real functional test needs degree successions relative to a *local* key, and fugues modulate
  constantly. Without it [§2.3](#23-harmony-is-a-second-automaton)'s functional half cannot be built or tested.
  **Built and measured, and the problem is narrowed rather than closed** ([§8.14](#814-key-finding-and-the-ground-truth-that-was-already-here)): Viterbi over
  24 keys by bar, validated on the 106 cadence labels, whose roman numerals name the local key — ground truth that
  was in the repository from the beginning, annotated by somebody else for another purpose. It reaches **35% of the
  74 modulations** against a null that scores zero on them, and its accuracy peak sits exactly at the key rhythm
  that is musically plausible, 6.4 bars. Enough to catch a key plan that wanders somewhere Bach never goes; not
  enough to referee between two plausible ones, and not enough to build the functional half on.
- **A replacement for the two dissonance rules**, which fail in both centuries ([§8.2](#82-the-rulebook-stratified-by-two-corpora)),
  and which step 6's whitelist could not supply
  ([§8.7](#87-the-species-as-a-whitelist-and-why-it-does-not-tighten-anything)). **Its one lead has now been
  followed and it is not the answer** ([§8.12](#812-the-fourth-and-the-scope-a-dissonance-is-judged-in)): judging the
  fourth against the lowest sounding voice rather than pairwise is right, and removes 9% of what the rules flag in
  Bach and 23% in the Renaissance, leaving them at 102.6 and 61.2 per thousand. Only 38% of Bach's flagged fourths
  have a voice below them. Whatever replaces these rules has to explain the other ninety per cent.
- **A design objective**, still open after two attempts ([§3.2](#32-capacity-is-a-density-and-it-cannot-be-optimised)).
  It has to reward a subject working at the fifth, which is a harmonic statement.
- **The right rulebook for the right repertoire.** Fux is 1725 and Palestrina-style vocal; the WTC is 1722 and
  keyboard. **Marpurg's *Abhandlung von der Fuge*** (1753) is the fugue treatise of Bach's own circle and
  **Kirnberger** studied with him directly. Transcribing either is exactly as unfitted as transcribing Fux — it is
  transcribing the right explicit theory. **Its third chapter is now transcribed and measured**
  ([§8.11](#811-marpurgs-tonal-answer-one-rule-exact-one-wrong-and-the-treatise-knew-which)), and it is the first
  rulebook this project has read that Bach does not break: Marpurg's rule for the answer's *first* note holds in
  every case where it says anything, seven of seven. His rule for the *last* note fails in every such case, and he
  is the one who hedged it. What remains unread is the rest of him, and Kirnberger entirely. Marpurg is
  [freely available](https://archive.org/details/abhandlungvonder00marp); the
  engraved examples are a separate volume from the text in every edition, so both are needed. Three things in it
  answer questions open here — **surveyed from the scans, not read in full**, which is why the row in
  [§7](#7-prior-art) says located rather than read. **Hauptstück 3, *Vom
  Gefährten***, gives the **tonal answer** as a table of degree correspondences with a two-case rule — transpose
  note for note where the subject stays in the key, alter it where the subject reaches the dominant — which is a
  finite map this document's formalism can hold and currently does not have at all, since
  [§8.3](#83-the-clique-test) places entries by plain diatonic transposition. Its plates give some thirty worked
  *Dux*/*Comes* pairs, attributed, two of them Bach's — the treatise's own examples, usable as
  [§8.7](#87-the-species-as-a-whitelist-and-why-it-does-not-tighten-anything) used Fux's figures and for the same
  reason. The second part, on **invertible counterpoint**, states its rules in a form
  [§2.2](#22-counterpoint-is-a-finite-automaton)'s automaton already speaks: no two consecutive fourths, since
  inversion makes them fifths, and the fifth handled as a dissonance, since inversion makes it a fourth. That is
  the fourth-and-fifth exchange the bullet above arrived at from the other direction, by measurement. And **Vom
  Wiederschlage** is about the order and cadence of entries, which is [§2.4](#24-form-is-a-grammar)'s subject.
  None of it addresses [§8.10](#810-replacing-the-soft-tier-and-the-degenerate-optimum-of-every-positive-criterion)'s
  problem, which is what to prefer among the legal fills; this is a book about fugal devices, not about choosing.
- **The Shostakovich half of the ground truth**, which needs Marques' MIDI rather than kern.

---

## 10. Reproducing the results

Everything is deterministic. No sampling that is not seeded, and no threshold that was chosen rather than measured
or swept.

### 10.1 Environment and data

```
rustc 1.96.1   cargo 1.96.1     # one dependency: clap, for the command line
git clone --recurse-submodules <this repo>
cargo test --release            # 64 library tests, 10 in the binary, 5 reference checks
cargo run --release -- list     # every command and the section it produces
cargo run --release -- --help   # that, plus §10.3's parameters as flags
cargo run --release -- realise  # writes out/*.mid
```

**Everything below is a flag.** §10.3 used to be a table of constants that had to be edited and recompiled to
vary; it is [`src/cli.rs`](src/cli.rs) now. The defaults are exactly the published runs — that is the contract of
that module, and **no figure in §8 was produced with a non-default flag**.

**The cross-references are enforced, not proofread.** `tests/references.rs` fails the build if a `§` reference
anywhere in this document, in [`CHANGELOG.md`](CHANGELOG.md) or in `src/**.rs` names a section that does not exist;
if a link's visible text and its anchor disagree about which section that is; if the numbering has a gap; or if the
Contents and the headings have drifted apart. It exists because they had: the first run over an already-tidied
repository found **53 stale references**, most of them doc comments still naming the `9`–`17` numbering that the
restructure folded into §8, and four links in this file reading `§8` while pointing at §9. Anchors had been checked
all along, which is exactly why the rot was invisible — the links resolved, and only the words were wrong.

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
| [§8.2](#82-the-rulebook-stratified-by-two-corpora) Bach rates | `cargo run --release -- corpus` |
| [§8.2](#82-the-rulebook-stratified-by-two-corpora) melodic breakdown | `cargo run --release -- diag` |
| [§8.2](#82-the-rulebook-stratified-by-two-corpora) Renaissance | `cargo run --release -- renaissance` |
| [§8.2](#82-the-rulebook-stratified-by-two-corpora) chromaticism | `cargo run --release -- chromatic` |
| [§8.3](#83-the-clique-test) clique test | `cargo run --release -- stretto` |
| [§8.4](#84-capacity-ranks-subjects-and-cannot-design-one) density ranking | `cargo run --release -- density` |
| [§8.4](#84-capacity-ranks-subjects-and-cannot-design-one) design | `cargo run --release -- design` |
| [§8.5](#85-the-harmonic-analyser) sweep | `cargo run --release -- sweep` |
| [§8.5](#85-the-harmonic-analyser) hold-out | `cargo run --release -- holdout` |
| [§8.6](#86-realisation-and-the-first-notes) stretto render | `cargo run --release -- render` |
| [§8.6](#86-realisation-and-the-first-notes) reconstruction | `cargo run --release -- reconstruct` |
| [§8.6](#86-realisation-and-the-first-notes) scalarisations | `cargo run --release -- scalarisations` |
| [§8.6](#86-realisation-and-the-first-notes) treatise weighting, both corpora | `cargo run --release -- generality` |
| [§8.7](#87-the-species-as-a-whitelist-and-why-it-does-not-tighten-anything) species whitelist | `cargo run --release -- species` |
| [§8.8](#88-a-criterion-that-is-not-local-and-the-shape-of-every-step-6-failure) shape criteria | `cargo run --release -- shape` |
| [§8.9](#89-a-better-plan-and-the-first-lever-that-moves-more-than-a-point) harmonic plans | `cargo run --release -- plan` |
| [§8.10](#810-replacing-the-soft-tier-and-the-degenerate-optimum-of-every-positive-criterion) tier ablation and prescriptions | `cargo run --release -- soft` |
| [§8.10](#810-replacing-the-soft-tier-and-the-degenerate-optimum-of-every-positive-criterion) the objective, paired | `cargo run --release -- objective` |
| [§8.11](#811-marpurgs-tonal-answer-one-rule-exact-one-wrong-and-the-treatise-knew-which) Marpurg's tonal answer | `cargo run --release -- answer` |
| [§8.12](#812-the-fourth-and-the-scope-a-dissonance-is-judged-in) the fourth's scope | `cargo run --release -- fourth` |
| [§8.13](#813-are-episodes-sequences-and-how-much-of-a-fugue-is-episode) episodes and sequences | `cargo run --release -- episode` |
| [§8.14](#814-key-finding-and-the-ground-truth-that-was-already-here) key-finding | `cargo run --release -- key` |
| [§8.15](#815-does-the-form-grammar-derive-the-book) the grammar, parsed | `cargo run --release -- form` |
| [§8.16](#816-a-fugue-from-a-subject) a whole fugue | `cargo run --release -- fugue` |
| every cross-reference in the repository | `cargo test --release --test references` |

**This table is checked against the program.** `tests/references.rs` runs `list` and fails the build if a row here
names a command the binary does not have, if a command's section disagrees with the one it prints, or if a
command that produces a reported figure is missing from the table. It exists because the previous command line
answered an unknown argument by silently running something else, so a mistyped command came back as a measurement
of something.

`realise` runs the three §8.6 commands together. The short names this table used to carry — `exp1`–`exp5`,
`h1`–`h3`, `r1`–`r3`, `gen`, `cad`, `hren2`, `obj` — all still work as aliases, because they are cited in
[`CHANGELOG.md`](CHANGELOG.md) and a citation that cannot be run is not a citation.

`rank`, `probe`, `pareto`, `revisit`, `ncts`, `harmony-design`, `harmony-corpus`, `cadence`, `hren`, `seg`,
`modal-control`, `func` and `binding-harmony` reproduce the **superseded** measurements recorded in
[`CHANGELOG.md`](CHANGELOG.md); `list` prints them under that heading rather than mixing them with the reported
ones.

The MIDI files land in `out/`, which is not tracked, and `--out` moves them. `reconstruct` is the only command
here that takes minutes rather than seconds, and the reason is the subject of
[§8.6](#86-realisation-and-the-first-notes); `plan` and `soft` take an hour, and the reason is that they run nine
and thirteen conditions over the same 1 267 spans.

### 10.3 Parameters

Everything with a flag beside it is [`src/cli.rs`](src/cli.rs)'s, and `--help` prints the same list. **The defaults
are the published runs**: no figure anywhere in §8 was produced with a flag set, and a test asserts each default is
the value the table names.

| | | flag |
|---|---|---|
| tick base | 960 per whole note ([`kern.rs`](src/kern.rs)) | |
| hard tier | `ParallelPerfect`, `DirectPerfectOnDownbeat` ([`automaton.rs`](src/automaton.rs)) | |
| realiser tier | `conf+melodic` wherever one tier is used rather than all three crossed | `--tier` |
| candidate grid | offsets every quarter within the subject; diatonic transpositions −7…+7; one entry per offset | |
| design grid | offsets every half note, same transposition range | |
| harmonic analyser | onset segmentation, 9 qualities × 12 roots, bass bonus 0.2, strong-beat weight ×2 | |
| realiser plan | `λ = 1.0`, the middle of [§8.5](#85-the-harmonic-analyser)'s plausible band | `--lambda` |
| realiser compass | each voice's range over the **whole piece**, which a form grammar would supply; never the passage's own range, which would be circular | |
| realiser budgets | 60 000 states per layer, 4 000 000 edges per span — both refusals, never beams ([`realise.rs`](src/realise.rs)) | |
| sampler | uniform over the legal set; the treatise weighting `β` is swept in [§8.6](#86-realisation-and-the-first-notes) and left at **0** everywhere else, being repertoire-specific | `--beta` |
| draws per span | 8 where a section averages them; 32 in [§8.8](#88-a-criterion-that-is-not-local-and-the-shape-of-every-step-6-failure), which ranks them instead | `--samples`, `--rerank` |
| objective | six soft criteria at equal weight in every table reported here, and **recommended off** by [§8.10](#810-replacing-the-soft-tier-and-the-degenerate-optimum-of-every-positive-criterion), which measures a uniform draw at `+1.07 ± 0.31` and `+4.64 ± 0.61` against it. The tables stand as the record of the runs as made | |
| prescriptions | `Problem::prescribe`, three positive criteria charged *instead of* the tier; all zero everywhere but [§8.10](#810-replacing-the-soft-tier-and-the-degenerate-optimum-of-every-positive-criterion), where each is measured and none adopted | |
| windows per work | 30 per Bach fugue and 3 per 15th-century work from [§8.8](#88-a-criterion-that-is-not-local-and-the-shape-of-every-step-6-failure) onwards, since 24 fugues stand against 200 works; [§8.6](#86-realisation-and-the-first-notes)'s weighting table predates that and runs 3 | `--bach-windows`, `--ren-windows`, `--gen-windows` |
| corpora | 24 WTC Book I fugues, 200 JRP works | `--kern`, `--jrp`, `--ren-works` |
| PRNG | SplitMix64 inline; seeds `0x5EED`, `0xC0FFEE`, `0xBEEF`, `0xF00D`, `0xD00D` | `--seed` |
| trials | 400 random contours for single-subject figures, 60 per subject for corpus tables | |
| hill-climbing | 12–16 restarts, first improvement accepted, one note changed at a time | |
| MIDI output | format 1, **960 ticks per quarter** (an exact ×4 of the internal lattice), tempo and time signature from the score, tracks top voice first ([`midi.rs`](src/midi.rs)) | `--out` |

### 10.4 How the samples were taken

**The Renaissance sample is not the six composers the code appears to select.** It globs six directories, sorts by
path and truncates at 200, so it is **70 Busnois, 37 Dufay and 93 Josquin** — Ockeghem, Obrecht and La Rue never
enter.

**The Bach corpus is Book I only**, `wtc1f01`–`wtc1f24`; the submodule holds both books but the annotations cover
Book I.

**The design tables cover 20 of 24 subjects.** Subjects longer than 24 notes are skipped for search cost, which
excludes BWV 855, 860, 865 and 866 — the four densest, and the four least stretto-friendly.

**[§8.6](#86-realisation-and-the-first-notes) covers 117 of 153 annotated entry spans.** A span qualifies if at
least one other voice sounds through at least half of it; of those, the ones with more than two free voices are not
attempted, because [§2.7](#27-where-a-solver-takes-over-from-the-dp)'s wall makes an exact answer impossible and
this project does not report a beam as if it were a search. Spans refused by the state or work budget are counted
in the `refused` column rather than dropped, so the shrinking sample is visible in the table that reports it.

**The generality test does not use the annotations.** The Renaissance corpus has none and never will, so
[§8.6](#86-realisation-and-the-first-notes)'s treatise-weighting table holds the **top voice** and windows at a
fixed eight quarters in both corpora rather than holding an annotated subject entry, as do
[§8.8](#88-a-criterion-that-is-not-local-and-the-shape-of-every-step-6-failure)'s and [§8.9](#89-a-better-plan-and-the-first-lever-that-moves-more-than-a-point)'s. That is a slightly harder
problem than the reconstruction table's, and the two are therefore not directly comparable to each other — only
within themselves, which is all the paired test needs.

**A pooled figure and a paired one are not the same measurement.** [§8.6](#86-realisation-and-the-first-notes)'s
percentages are pooled over notes, which weights a span by how many notes it has and — in the sampled row — counts
all eight draws separately. Everything from [§8.8](#88-a-criterion-that-is-not-local-and-the-shape-of-every-step-6-failure)
onwards pairs per span, with the span as the unit of replication. Where the two disagree the paired one is
reported, and [§8.10](#810-replacing-the-soft-tier-and-the-degenerate-optimum-of-every-positive-criterion) is the case where they did.

**Timings are from one machine** and are quoted only where they carry an argument — in
[§8.6](#86-realisation-and-the-first-notes) they carry one, since tractability is the finding. They are not
benchmarks.

### 10.5 What is not reproducible from this repository

- **The literature.** [`literature/`](literature/) holds five PDFs under their publishers' terms;
  [§7](#7-prior-art) gives DOIs so each can be obtained independently. **Marpurg is deliberately not among them**:
  the scans run to 92 MB, they are public domain rather than licensed, and a link serves as well as a copy —
  [archive.org/details/abhandlungvonder00marp](https://archive.org/details/abhandlungvonder00marp). `.gitignore` keeps them
  out if they are downloaded into that directory.
- **The Shostakovich annotations**, which have no scores here.
- **The functional-harmony layer**, which compiles but is not exercised by any reported number.

### 10.6 Using it as a library

The repository is a **library and a binary**. The library is the model and the
generator; the binary is the measurement of them, and every figure in
[§8](#8-what-is-built-and-what-it-measures) comes out of it. The line falls where
it does because of one test: **a caller who wants to compose a fugue needs the
first and none of the second.**

```
src/lib.rs        the model and the generator — 17 modules
src/main.rs       the command line and the drivers, §10.2's table
src/cli.rs        §10.3's parameters as flags
src/step5.rs      the drivers for §8.6 onwards
src/experiments.rs  the five that resolved §8.2's deadlock
```

Everything else in `src/` is library. `cargo test` runs both halves separately —
**64 tests in the library, 10 in the binary, 5 reference checks** — and
`cargo build --lib` builds the model with no knowledge that a command line
exists.

#### Composing

```rust
use contrapunctus::{automaton::HARD, compose, kern};

let piece = kern::read(Path::new("corpus/bach-wtc-fugues/kern/wtc1f02.krn"))?;
let design = compose::Design {
  subject: kern::clip(&piece.voices[1], 0, 2 * piece.measure),
  voices: 3,
  key: piece.key,
  tonic: 0,
  measure: piece.measure,
  beat: piece.beat,
  compass: vec![(33, 45), (28, 40), (21, 33)],
};

let out = compose::fugue(&design, &compose::Layout::default(), HARD, 0x5EED)?;
compose::write(&out, &design, Path::new("fugue.mid"), 76)?;
```

**[`Design`] is what the music is made of and [`Layout`] is what is done with
it**, and the split is the one a user interface wants: a control per field of
`Layout` — how many middle entries and in which keys, how long an episode runs,
whether the exposition takes a link, whether it closes at home — over a `Design`
that changes rarely. `Layout::default()` is the book's own shape as
[§8.15](#815-does-the-form-grammar-derive-the-book) and
[§8.13](#813-are-episodes-sequences-and-how-much-of-a-fugue-is-episode) measured
it, and is what every published figure uses.

**`Outcome` carries the notes and every judgement this document can pass on
them** — the derivation block by block, whether it parses under §8.15's own
grammar, the rule firings by §8.2's checker, which blocks had a constraint
dropped, and the wall clock. One struct rather than five return values, because
a result that *can* be displayed without being checked is one that will be.

| you want | you call |
|---|---|
| a whole fugue, checked | `compose::fugue` |
| **one block rewritten, the rest untouched** | `compose::refill` |
| the derivation only, to draw a plan | `compose::derive` |
| the notes only | `compose::generate` |
| a fill against voices you already have | `realise::fill` |
| a subject out of a score | `kern::read`, `kern::clip` |
| the annotated entries and cadences | `refdata::read` |
| what a texture breaks | `corpus::check_voices`, `corpus::check_melody` |
| a chord path, or a key path | `harmony::analyse_viterbi`, `key::analyse` |
| Marpurg's answer | `answer::admissible` |
| MIDI out, tracks named and ordered | `midi::write_score` |

#### Editing one block without recomposing the piece

An interface over this wants to change one thing and see one thing change. It can:
[`compose::refill`](src/compose.rs) rewrites a single block and leaves every other note where it was, which a test
asserts over the whole piece rather than at the seam.

It works because the fill is blockwise and the only thing crossing a block boundary is **the pitch each voice ends
on**. `Problem::terminal` pins those — the mirror of `Problem::prior` — so a refilled block is a drop-in
replacement and nothing after it is searched again. On a twelve-block fugue that is a twelfth of the work; at five
voices, where one block costs more than a whole three-voice piece, it is the difference between an editor that
responds and one that recomputes.

Two limits, both refusals rather than surprises. **Span-preserving edits only** — changing a block's key or its
voice keeps the piece the same length and refills locally; lengthening an episode or adding a middle moves every
later bar, and no pin makes that local. And the pinned ending may simply be **unreachable** once the block's
contents have changed, which is reported so the caller can fall back to [`compose::fugue`].

The seed is keyed on **what a block is** — its kind, voice, key and length — and not on its position, so editing
one block does not reseed the others. The index-keyed seed it replaced would have redrawn the whole piece under any
edit that changed the block list.

**What this does not solve is five voices.** Refilling reduces how many blocks are searched; it does not make one
searchable. A block with four or five free voices is past [§2.7](#27-where-a-solver-takes-over-from-the-dp)'s wall,
and that is the solver in [§9](#9-roadmap). The two fit together well — a CDCL solver is natively incremental, so
*re-solve one block against a changed constraint* is the operation it is built for.

> The same architecture gives §8.16 its one parallel fifth at a seam and gives an editor its locality. The
> automaton's state resets at a block edge, so the search cannot see across it — **the defect and the capability
> are one fact**, and a single global search would fix the first by destroying the second.

#### Two things a caller should know before trusting it

**The tier to generate against is not the tier this document endorses.**
[§8.2](#82-the-rulebook-stratified-by-two-corpora) stratified the two dissonance
rules out because they misdescribe Bach; a generator that omits them writes
cacophony at 366 violations per thousand and one that enforces them writes 70,
below Bach's own 112. Pass `HARD`.
[§8.16](#816-a-fugue-from-a-subject) is the argument.

**Three voices.** [§2.7](#27-where-a-solver-takes-over-from-the-dp) predicted the
search's wall at four free voices and [§8.6](#86-realisation-and-the-first-notes)
measured it at two, so `compose::fugue` refuses a four-voice design rather than
beaming. Half the Well-Tempered Clavier is out of reach until a solver replaces
the DP, and the refusal says so.
