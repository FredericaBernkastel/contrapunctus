# Contrapunctus: Counterpoint is a regular language

### Fugue on the lattice: 513 states, a clique, and what Bach says about the rulebook

*Design document. **Steps 0 to 4 of [§8](#8-roadmap) are built and measured. [§2.3](#23-harmony-is-a-second-automaton)'s harmonic
layer is built and [fails validation](#16-the-harmonic-layer-does-not-survive-validation) — see §§[9](#9-step-1-result-two-rules-survive-bach-three-do-not)–[16](#16-the-harmonic-layer-does-not-survive-validation).**
Steps 5 onward are not built.*

---

## Contents

- [Abstract](#abstract)
- [The name is the argument in miniature](#the-name-is-the-argument-in-miniature)
- [0. Where this comes from](#0-where-this-comes-from)
- [1. Diagnosis: §8 is two causes, not seven items](#1-diagnosis-8-is-two-causes-not-seven-items)
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
- [3. The measurement §6.1 wanted, computed exactly](#3-the-measurement-61-wanted-computed-exactly)
  - [3.1 The calibration disappears](#31-the-calibration-disappears)
  - [3.2 And §6.2 gets easier, not harder](#32-and-62-gets-easier-not-harder)
  - [3.3 The subject is input, and its boundary is contested](#33-the-subject-is-input-and-its-boundary-is-contested)
- [4. Space filling, in the right category](#4-space-filling-in-the-right-category)
- [5. What this will not do](#5-what-this-will-not-do)
- [6. What ricercar still owns](#6-what-ricercar-still-owns)
- [7. Prior art](#7-prior-art)
- [8. Roadmap](#8-roadmap)
- [9. Step 1 result: two rules survive Bach, three do not](#9-step-1-result-two-rules-survive-bach-three-do-not)
  - [9.1 The state count, measured](#91-the-state-count-measured)
  - [9.2 The verdict tests pass](#92-the-verdict-tests-pass)
  - [9.3 Bach found two bugs in the rulebook, in the first run](#93-bach-found-two-bugs-in-the-rulebook-in-the-first-run)
  - [9.4 The result, and it stratifies the rulebook](#94-the-result-and-it-stratifies-the-rulebook)
  - [9.5 The comfortable explanation is not available](#95-the-comfortable-explanation-is-not-available)
  - [9.6 What this changes](#96-what-this-changes)
- [10. Step 2 result: the clique test passes, on the tier Bach chose](#10-step-2-result-the-clique-test-passes-on-the-tier-bach-chose)
  - [10.1 The verdict](#101-the-verdict)
  - [10.2 The control, which is what makes the verdict mean anything](#102-the-control-which-is-what-makes-the-verdict-mean-anything)
  - [10.3 A defect the test caught in the reading of the corpus](#103-a-defect-the-test-caught-in-the-reading-of-the-corpus)
  - [10.4 What this settles, and what it does not](#104-what-this-settles-and-what-it-does-not)
- [11. Step 3 result: a ranking, and the rulebook that can produce it is not the one Bach confirms](#11-step-3-result-a-ranking-and-the-rulebook-that-can-produce-it-is-not-the-one-bach-confirms)
  - [11.1 The measure is vacuous on the tier §9 and §10 selected](#111-the-measure-is-vacuous-on-the-tier-9-and-10-selected)
  - [11.2 The ranking, on the strict tier, and it agrees with musicians](#112-the-ranking-on-the-strict-tier-and-it-agrees-with-musicians)
  - [11.3 How much of that is just note density](#113-how-much-of-that-is-just-note-density)
  - [11.4 §3.3 was right, and the numbers are worse than it feared](#114-33-was-right-and-the-numbers-are-worse-than-it-feared)
  - [11.5 A defect the measure found in itself](#115-a-defect-the-measure-found-in-itself)
- [12. Resolving §11.1: five experiments, and the one that works](#12-resolving-111-five-experiments-and-the-one-that-works)
  - [12.0 A parser bug found on the way, which corrected everything upstream](#120-a-parser-bug-found-on-the-way-which-corrected-everything-upstream)
  - [12.1 Experiment 1 — density instead of clique size. **This resolves it.**](#121-experiment-1--density-instead-of-clique-size-this-resolves-it)
  - [12.2 Experiment 2 — Pareto calibration. **Fails, and the failure is instructive.**](#122-experiment-2--pareto-calibration-fails-and-the-failure-is-instructive)
  - [12.3 Experiment 3 — the Renaissance control. **Decisive, and it splits the three refuted rules.**](#123-experiment-3--the-renaissance-control-decisive-and-it-splits-the-three-refuted-rules)
  - [12.4 Experiment 4 — chromaticism. **Negative.**](#124-experiment-4--chromaticism-negative)
  - [12.5 Experiment 5 — harmony. **Supportive, and not yet enough.**](#125-experiment-5--harmony-supportive-and-not-yet-enough)
  - [12.6 Where this leaves the programme](#126-where-this-leaves-the-programme)
- [13. Step 4 result: the measure is valid for ranking and invalid for design](#13-step-4-result-the-measure-is-valid-for-ranking-and-invalid-for-design)
  - [13.1 Three optima, all anti-musical](#131-three-optima-all-anti-musical)
  - [13.2 The worse finding: Bach's contours score *below* random](#132-the-worse-finding-bachs-contours-score-below-random)
  - [13.3 Why, and it is structural rather than accidental](#133-why-and-it-is-structural-rather-than-accidental)
  - [13.4 What this settles](#134-what-this-settles)
- [14. §2.3 built: harmony repairs the design objective and fails as a rule](#14-23-built-harmony-repairs-the-design-objective-and-fails-as-a-rule)
  - [14.1 As a rule, it fails — and it corrects §12.5's reading](#141-as-a-rule-it-fails--and-it-corrects-125s-reading)
  - [14.2 As a design objective, it works — and this is the repair §13.3 asked for](#142-as-a-design-objective-it-works--and-this-is-the-repair-133-asked-for)
  - [14.3 What §2.3 is and is not](#143-what-23-is-and-is-not)
- [15. Reproducing the results](#15-reproducing-the-results)
  - [15.1 Environment and data](#151-environment-and-data)
  - [15.2 Which command produces which section](#152-which-command-produces-which-section)
  - [15.3 Parameters that would otherwise have to be read out of the source](#153-parameters-that-would-otherwise-have-to-be-read-out-of-the-source)
  - [15.4 How the samples were taken, including one that is not what it looks like](#154-how-the-samples-were-taken-including-one-that-is-not-what-it-looks-like)
  - [15.5 What is not reproducible from this repository](#155-what-is-not-reproducible-from-this-repository)
- [16. The harmonic layer does not survive validation](#16-the-harmonic-layer-does-not-survive-validation)
  - [16.1 It finds Bach's cadences barely above chance](#161-it-finds-bachs-cadences-barely-above-chance)
  - [16.2 It fits *modal* polyphony better than tonal — so it is not measuring tonality](#162-it-fits-modal-polyphony-better-than-tonal--so-it-is-not-measuring-tonality)
  - [16.3 Every number depends on a window nobody justified](#163-every-number-depends-on-a-window-nobody-justified)
  - [16.4 The graded objective is worse than the binary one](#164-the-graded-objective-is-worse-than-the-binary-one)
  - [16.5 What this means for the next step](#165-what-this-means-for-the-next-step)
---

## Abstract

Machine composition of fugue is usually attempted in one of two categories: fit a model to a corpus, or search a
continuous relaxation of the score. This document argues that both are the wrong category, and that fugue is a
**word problem over a finite alphabet subject to constraints of bounded memory** — so the natural instruments are
automata, dynamic programming and exact combinatorial search, none of which require training data and all of which
return proofs rather than samples.

The argument opens as a post-mortem. [`ricercar`](ricercar/readme.md) modelled counterpoint as a Lipschitz-certifiable
roughness field over a continuum of entry placements, and its own measurements refute it. The legal region turned
out **piecewise constant at the note grid**; and rounding a certified placement onto the semitone grid costs about
ten times the margin the certificate establishes, so the proof was being taken over the wrong set. Everything
expensive in that approach existed to bound a function whose answer is constant on a lattice.

On the lattice the reformulation is small, and most of it is classical. Counterpoint is a **finite automaton** over
`(interval, motion, articulation, metric weight)` whose state is *the interval plus what you owe* — a dissonance
owes a resolution, a leap owes a recovery — and it stays finite because strict counterpoint requires debts settled
on the next event. Harmony is a second automaton over functional progressions; form is a ten-line grammar;
realising free voices against fixed entries is a shortest path, escalating to a CDCL solver at four or more free
voices, where layering is known to fail. Densest stretto becomes **maximum clique in a Cayley graph** on the shift
group, exactly computable, where the continuous formulation of the same question was killed at thirty minutes
without a single placement. Bach's own five-voice hyperstretto in BWV 867 is such a clique — and it is an arithmetic
progression, `{0, 2, 4, 6, 8}` quarters, which refutes this document's earlier guess that the object was a Sidon set.

Three steps are built and measured against the 24 fugues of the Well-Tempered Clavier, Book I, using published
ground-truth annotations and Huron's Humdrum encodings. The automaton has **513 reachable states** against a crude
product of 1280, and it distinguishes a prepared suspension from the same interval struck on the same beat — the
distinction a field over instantaneous pitch is structurally unable to make, and the device most of the repertoire
worth imitating is built from. Run as a *checker* over Bach, it then stratifies its own rulebook: **parallel
perfect consonances and direct motion to a perfect consonance on a downbeat occur about once per thousand slices
and are confirmed; the dissonance and melodic prohibitions fire two orders of magnitude more often and are
refuted.** The surviving pair is precisely the pair a roughness field cannot express at all, since a perfect fifth
is among the smoothest intervals it knows.

The clique test then selects the same rulebook a second time, by a different route. **Under the full five-rule
tier Bach's hyperstretto is not a clique; under the two-rule tier it is**, on both contested readings of the
subject, and a control on the written notes rather than on idealised transpositions confirms that the fault lies
with the rulebook rather than with the model of an entry. That two independent tests — one counting rule
frequencies across a book, one asking whether a single passage is mutually compatible — converge on the same two
rules is the strongest result here, because neither was designed to check the other.

Ranking all 24 subjects by capacity then puts **BWV 849 first**, which is the fugue musicians name when they name
a stretto fugue, and it survives a control for note density as the largest positive residual. But the same
experiment exposes the sharpest limit found so far, and it is structural: **the rules Bach never breaks are
precisely the rules that almost never bind.** Under the two-rule tier that §[§9](#9-step-1-result-two-rules-survive-bach-three-do-not)–10 selected, 81% of entry pairs are
mutually compatible and capacity never converges; only the strict five-rule tier — the one Bach violates — yields
a finite number. There is no single rulebook here that both accepts Bach's own hyperstretto and discriminates
between subjects, which is the quantitative form of the observation that completeness is not selectivity.

What the method does not do is decide whether the result is good, and its failure mode is the inverse of the usual
one: a complete search does not fail by finding nothing but by finding far too much — on the order of `10⁵`–`10⁶`
legal counterpoints to an eleven-note *cantus firmus*. Where taste enters is therefore the central design question
rather than an afterthought, and the position taken here is the **Pareto front** over soft criteria rather than a
weighted sum, on the ground that no weighting in the literature is defensible and Fux declines to supply one.

**Provenance.** §[§1](#1-diagnosis-8-is-two-causes-not-seven-items)–7 were written before reading the literature now in [`literature/`](literature/); §[§2.1](#21-exact-arithmetic-and-therefore-no-certificates), 2.5,
2.6, 2.7, 3, 5, 7 and 8 were revised against it, and every revision is marked in place. **Four claims have not
survived contact with the sources or the data**: that transposition is `x + k`, that the subject is given, that the
hard/soft split of the rulebook is recent, and that a stretto is a Sidon set. This document is the alternative to
`ricercar` rather than a repair of it, and it exists because that project's own measurements point here.

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

## 1. Diagnosis: §8 is two causes, not seven items

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
| transposition by `k` | `x + k` |
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

> **Correction, from Giraud et al. (2015).** The first row of that table is wrong for real fugues. A fugal answer
> is often **tonal**, not real: transposed to the dominant, *and* altered in its first few notes so that the
> dominant pitch maps to the tonic rather than the supertonic. Giraud names this as the reason their matcher works
> on **diatonic** intervals rather than semitones — "a scale will always match only a scale."
>
> **The target fugue exhibits it in its first two notes.** BWV 867's subject opens B♭4 → F4, a descending fourth.
> The answer at measure 3 opens F4 → B♭3 — a descending *fifth*. A real answer would have given F4 → C4; the
> dominant has been mapped back to the tonic instead, altering the very interval that defines the subject's head.
> The annotation marks the label `(tonal_answer)`, and Huron's encoding shows why.
>
> So `x + k` describes the *real* answer only. The repair is small and the parameterisation already has room for
> it: the tonal answer is **its own transformation type `τ`**, not a value of `k`, and [§3](#3-the-measurement-61-wanted-computed-exactly)'s compatibility table is
> already indexed by `(τᵢ, τⱼ, …)`. But a subject stated with one `τ` and answered with another is the normal
> case, not an extension, and the table has to carry it from the start. Pitch is therefore best held as
> `(scale degree, inflection)` rather than as a semitone integer — Giraud's argument, arrived at for a matching
> problem and applying unchanged to a generation problem.

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

| rule | as an automaton condition | order |
|---|---|---|
| parallel fifths, octaves | forbidden edge `5 → 5`, `8 → 8` under parallel motion | 2 |
| hidden / direct fifths | edge `* → 5` under similar motion with a leap above | 2 |
| passing dissonance | dissonant tick approached and left by step in one direction | 3 |
| **suspension** | consonant-and-tied → dissonant-on-strong → step down to consonance | 3 |
| neighbour tone | step away and back | 3 |
| voice crossing, overlap | `p_upper(t) ≥ p_lower(t)`, `p_upper(t) ≥ p_lower(t−1)` | 2 |
| leap recovery | a leap beyond a fourth is followed by a step against it | 2–3 |

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

**The rulebook is smaller than the model it replaces.** The crude product of the state components is on the order
of `10³` before minimisation, and DFA minimisation is a solved algorithm. The reachable count after minimisation
should be **measured and reported** rather than asserted — that is step 1 of [§8](#8-roadmap) below, and it is the sort of number
this project prefers to have measured.

### 2.3 Harmony is a second automaton

> **Built, and it does not survive validation — see [§14](#14-23-built-harmony-repairs-the-design-objective-and-fails-as-a-rule) then [§16](#16-the-harmonic-layer-does-not-survive-validation).**
> As a *rule* it fails: made permissive enough to accept Bach it accepts nearly everything. As a *design objective*
> it looked like a repair of [§13](#13-step-4-result-the-measure-is-valid-for-ranking-and-invalid-for-design)'s
> degenerate optimum, until four external checks showed it identifies cadences barely above chance and fits modal
> music better than tonal. It is a triadic-consonance detector, not a model of harmony.

A functional automaton over `(key, scale degree, inversion)`, with edges for the standard progressions, and
modulation via pivot chords to closely related keys. A cadence is then a **labelled accepting path** — `ii⁶ → V → I`
with its voice leading — and ricercar's *"nothing here knows the difference between a cadence and a stop"* is
answered by construction.

The two automata compose by intersection. This is the classical reading — harmony as a regular language, voice
leading as a transduction over it — and it is worth saying that it is classical, because the components being
known-good is the point.

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

One honest boundary. Melodic *shape* rules are not order-3: a single melodic climax, tessitura over a whole phrase,
no repeating a figure. Those are genuinely long-range, and they are what pushes a problem from dynamic programming
to constraint programming. That the boundary falls exactly between the harmonic-contrapuntal rules and the shape
rules is a real finding about counterpoint, and it is falsifiable — name a strict-counterpoint rule with longer
memory and the claim is wrong.

> **The falsification was offered twice, and both times it refines the claim rather than breaking it.**
>
> Ebcioğlu's two-part florid counterpoint carries a rule that the pitches of the voice's local maxima **within
> three measures** must be distinct — longer than order 3, but *windowed*, and a windowed rule is finite-state at a
> cost exponential in the window.
>
> Schottstaedt's source is the harder case, and it is worth reading rather than paraphrasing. Three of his
> procedures scan **the entire melody so far**: `TotalRange` (the voice's compass must not exceed an octave and a
> fifth), `PitchRepeats` (how often this pitch has already been used), and `TooMuchOfInterval` (keep a mixture of
> interval sizes). Those have unbounded lookback — not windowed at all.
>
> But every one of them is an **accumulator**: a running min and max, a running count, a running histogram. An
> accumulator is finite-state whenever its range is bounded, and each of these is — the range check only needs
> `(min, max)` so far, the repeat check only needs a count saturated at its threshold. So the accurate statement is
> not about lookback at all:
>
> **Contrapuntal rules are order ≤ 3 in events. Melodic shape rules have unbounded lookback but bounded state,
> because they are accumulators.** Everything stays finite-state; what changes is the state count, and the
> interval-mixture histogram is where it stops being small. That is the honest reason to reach for a solver rather
> than a wider DP — and note that Schottstaedt, having implemented all three, still concludes that his program
> *"makes no decisions about overall melodic shapes."* Implementing the accumulators is not the same as controlling
> the shape.

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

- His stated goal was *"five to eight part mixed species counterpoint"* — the same target as [§8](#8-roadmap)'s step 5.
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
- **Soft constraints are native.** [§5](#5-what-this-will-not-do) is about where taste enters; the answer below is the Pareto front, and
  Z3's optimizer supports multi-objective search in `pareto` mode directly.
- **Incrementality.** Push an entry, re-solve, pop — which is the shape of [§3](#3-the-measurement-61-wanted-computed-exactly)'s greedy loop, and precisely what
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

## 3. The measurement §6.1 wanted, computed exactly

This is the part to build first, because it is small and it settles the thing that has been blocked twice.

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

> **Correction, from the corpus.** An earlier draft called that "the same object as a Sidon set or a difference
> family." It is not, and the data supplies the counterexample immediately. A Sidon set requires all pairwise
> differences to be *distinct*; the condition here is that they all *land in `A`*, which is a different constraint
> and is satisfied by highly degenerate sets. Bach's own hyperstretto in BWV 867 is, in quarter notes from the
> first entry, `{0, 2, 4, 6, 8}` — **an arithmetic progression**, whose difference set `{2, 4, 6, 8}` repeats the
> step four times over. That is as far from a Sidon set as five points can be.
>
> The correction is worth more than the erratum: **the densest strettos are expected to be regular, not clever.**
> A canon at a fixed time interval is an AP by construction, and if the step is legal then every multiple of it
> tends to be legal too. So the clique search should look for structure, not scatter — which is also a hint that
> the search will be easier than a general max-clique instance suggests.

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

Here there is no threshold. The calibration becomes a **yes-or-no test**, and since [§8](#8-roadmap)'s step 0 the target is an
exact set of integers rather than a description:

> The subject of BWV 867 is 12 quarters long. Its five final entries entries stand at quarters
> **`{266, 268, 270, 272, 274}`** — one per voice, `{0, 2, 4, 6, 8}` from the first.
> **Does `{0, 2, 4, 6, 8}` come out as a clique in that subject's compatibility graph?**

If yes, the automaton is calibrated — by construction, since Bach's five-voice hyperstretto is acceptable
counterpoint. If no, the automaton is too strict and *that is the finding*, exactly as ricercar argued for its own
falsification, but without a constant to fit. Nothing is tuned, and the corpus ranking of [§6.1](ricercar/readme.md#61-the-measurement) becomes a loop over
subjects at milliseconds each.

Note how much the test tightened by having the data. Ricercar spent [§7.5](ricercar/readme.md#75-the-real-subject-and-two-things-it-broke) and [§7.6](ricercar/readme.md#76-step-5-θ-calibrated-against-bach) establishing this passage from a
score by hand and arrived at a real-valued threshold that then made the computation intractable. The same passage
is four lines of a public annotation file, and the test it supports is integer equality.

### 3.2 And §6.2 gets easier, not harder

Ricercar's design problem — optimise the subject's contour for capacity — was posed as continuous optimisation over
`N = 8..16` coordinates with `Manifold` weights decaying from the head of the subject. Here a contour is a word,
capacity costs microseconds, and the search is exhaustive or branch-and-bound over words with the head fixed. The
weighting intuition survives intact: fix the head, vary the tail, because the head is what the ear recognises on
re-entry. It is just a search order now rather than a metric.

---

### 3.3 The subject is input, and its boundary is contested

This is the second thing the literature broke, and it is worse than the tonal answer because there is no clean
repair.

Capacity is a function of the subject. [§3](#3-the-measurement-61-wanted-computed-exactly) assumes the subject is given. Giraud et al. built a ground truth for the
24 Bach fugues of WTC I against four musicological sources — Prout, Tovey, Keller, Bruhn, plus Charlier — and
report that **in eight of the twenty-four, at least two sources disagree about where the subject ends**, sometimes
by several notes. On Fugue No. 9 they quote Tovey to the effect that it is not worth settling where the subject
ends and the countersubject begins; the flow between them is continuous.

**Step 0 turns that from a warning into a list.** The disagreement is recorded in the data as `S alternative`
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
length anyway, so the profile is a loop over prefixes of one table.

---

## 4. Space filling, in the right category

The instinct behind ricercar was not wrong. Counterpoint really is a tiling problem. The error was the category:
not packing in `ℝᵈ`, but **factorisation of a finite abelian group**.

A tiling rhythmic canon is a partition of `ℤₙ` into translates of a rhythmic motif — every beat covered exactly
once, no gaps, no overlaps, which is space filling in the strictest sense available. The mathematics is Vuza's
canons, the Coven–Meyerowitz conditions, and Hajós groups, and it has been pursued in a music-theoretic setting by
Andreatta, Amiot and Agon.

It is discrete, it is elegant, it is not fitted to anything, and it is deep. If the aesthetic pull of this project
is *counterpoint as tiling*, that literature is where the pull is actually satisfied — and it is adjacent to [§3](#3-the-measurement-61-wanted-computed-exactly),
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

  **Correction, on reading Schottstaedt's source.** Two paragraphs above credited Komosinski with the hard/soft
  split. That is wrong: it is in Schottstaedt in 1984, as a stratified penalty table — `Infinity` for the rules
  that may not be broken (parallel fifths and unisons, dissonance, out of mode, out of range, bad cadence, no
  leading tone) and small integers for the rest. What Komosinski's criticism actually lands on is the **soft tier
  alone**, and there it lands hard, because those integers are unarguable magic numbers: a sixth followed by
  motion in the same direction costs 34, a fifth in the same position costs 8, a skip costs 1, three repeated
  notes 4 and four repeated notes 7. Nothing justifies 34 against 8.

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
  output that is empty where the style lives — and [§8](#8-roadmap)'s step 1 is written to catch it early.

  Two of his omissions are pointed. *"Does not reward invertible counterpoint and imitation"* is exactly the fugal
  content this document is about; *"makes no decisions about overall melodic shapes"* is [§2.5](#25-the-search-is-a-shortest-path)'s accumulator
  boundary, reported from the far side by someone who implemented the accumulators.
- **Infeasibility is real and is not always a bug.** Komosinski & Szachewicz found *cantus firmi* for which **no**
  counterpoint satisfies even their two hard rules — the legal set is empty, not small. A complete method reports
  that as a proof rather than as a timeout, which is the right behaviour, but it means "no solution" will
  sometimes be the honest answer to a musically reasonable request.
- **Melodic invention.** The subject is input. [§3.2](#32-and-62-gets-easier-not-harder) makes designing one cheaper, but designing for *capacity* is
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
| ✔ Giraud, Groult, Leguy & Levé, "Computational Fugue Analysis", *Computer Music Journal* **39**(2):77–96, 2015 | [10.1162/COMJ_a_00300](https://doi.org/10.1162/COMJ_a_00300) | fugue **analysis**, and the ground-truth corpus [§8](#8-roadmap) now uses |
| ✔ Schottstaedt, *Automatic Species Counterpoint*, CCRMA Report STAN-M-19, Stanford, May 1984 | no DOI — [ccrma.stanford.edu/STANM/stanms/stanm19](https://ccrma.stanford.edu/STANM/stanms/stanm19/) | Fux, five species, up to eight voices, stratified penalties — the closest prior attempt at [§2.7](#27-where-a-solver-takes-over-from-the-dp)'s scale, printed as complete source, and the most useful negative result here |
| ✔ Ebcioğlu, "An Expert System for Harmonizing Chorales in the Style of J. S. Bach", *J. Logic Programming* **8**(1):145–185, 1990 | [10.1016/0743-1066(90)90055-A](https://doi.org/10.1016/0743-1066(90)90055-A) | ~350 rules in first-order predicate calculus, generate-and-test with **intelligent backtracking**, in a language (BSL) built because PROLOG would not do — the argument for factoring a rulebook rather than listing it |
| Pesant, "A Regular Language Membership Constraint for Finite Sequences of Variables", *CP 2004*, LNCS **3258**:482–495 | [10.1007/978-3-540-30201-8_36](https://doi.org/10.1007/978-3-540-30201-8_36) | the domain-consistent DFA-membership propagator of [§2.5](#25-the-search-is-a-shortest-path) |
| Boenn, Brain, De Vos & Ffitch, "Automatic music composition using answer set programming", *Theory and Practice of Logic Programming* **11**(2–3):397–427, 2011 | [10.1017/S1471068410000530](https://doi.org/10.1017/S1471068410000530) | the same programme in answer-set programming, which may be the most elegant surface syntax available for it |
| Coven & Meyerowitz, "Tiling the Integers with Translates of One Finite Set", *J. Algebra* **212**(1):161–174, 1999 | [10.1006/jabr.1998.7628](https://doi.org/10.1006/jabr.1998.7628) | the tiling conditions behind [§4](#4-space-filling-in-the-right-category) |
| Hiller & Isaacson, *Experimental Music: Composition with an Electronic Computer* (1959); the *Illiac Suite*, 1957 | — | rule-based counterpoint by generate-and-reject; the field starts here |
| Ebcioğlu (1980), two-part florid counterpoint | — | ~50 constraints, including the windowed melodic-peak rule that refines [§2.5](#25-the-search-is-a-shortest-path). A 16th-century strict-counterpoint program preceded CHORAL and supplied its search method |
| Laurson, PWConstraints / Score-PMC (1996); Anders, Strasheela (2007) | — | the two ends of the design space: fixed rhythm with a fast static ordering, versus arbitrary score topology |
| Vuza, "Supplementary Sets and Regular Complementary Unending Canons", *Perspectives of New Music*, 1991–93; Andreatta, Amiot, Agon | — | tiling rhythmic canons — [§4](#4-space-filling-in-the-right-category) |
| Fux, *Gradus ad Parnassum* (1725) | — | the rulebook itself, and — per [§12.3](#123-experiment-3--the-renaissance-control-decisive-and-it-splits-the-three-refuted-rules) — a book about a repertoire this project mostly did not test it on |

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
[§8](#8-roadmap)'s realisation step should be budgeted in seconds for two voices, and five voices should be treated as genuinely
open rather than as more of the same.

**One methodological note, from Giraud.** Discussing why they did not learn their thresholds: machine learning
*"could improve the thresholds and weights of these models, but strategies have to be designed to address the
problem of overfitting, a concern for data sets as small as these are prone."* Thirty-six fugues is not a corpus
you can fit anything to. The no-fitting constraint this document was written under has an empirical justification
as well as an aesthetic one.

---

## 8. Roadmap

Each step is decidable, and each produces a number or a verdict rather than a demo. **Revised against the
literature**: step 0 is new, step 3 acquires a real corpus, and step 5 acquires an escalation ladder and an honest
expectation of cost.

0. ~~**Take the corpus, do not build one.**~~ **Done.** Both corpora are **git submodules** under `corpus/`, so the
   exact revision each result was computed against is recorded rather than merely fetched. Clone this repository
   with

   ```
   git clone --recurse-submodules <this repo>
   ```

   or run `git submodule update --init` in an existing checkout.

   | submodule | source | pinned | licence |
   |---|---|---|---|
   | [`corpus/algomus-data`](corpus/algomus-data) | `gitlab.com/algomus.fr/algomus-data` | `a1801b5` | **ODbL 1.0**, contents DbCL 1.0 — attribute Giraud, Groult and Levé |
   | [`corpus/bach-wtc-fugues`](corpus/bach-wtc-fugues) | `github.com/humdrum-tools/bach-wtc-fugues` | `5095752` | Humdrum edition by David Huron |
   | [`corpus/jrp-scores`](corpus/jrp-scores) | `github.com/josquin-research-project/jrp-scores` | `52de715` | Josquin Research Project — added for [§12.3](#123-experiment-3--the-renaissance-control-decisive-and-it-splits-the-three-refuted-rules)'s control, shallow clone |

   Pinning matters more here than it usually does. Both are living annotation projects, a subject boundary is an
   editorial judgement that can be revised ([§3.3](#33-the-subject-is-input-and-its-boundary-is-contested)), and **capacity is a function of the subject** — so a corpus
   ranking is only reproducible against a stated revision of the ground truth.

   **What is there.** All 36 fugues in one 27 kB file, [`fugues/fugues.ref`](corpus/algomus-data/fugues/fugues.ref)
   — 24 Bach WTC I and 12 Shostakovich, with `S`, `S-inc` (incomplete statements), `Sinv`, `Saug`, `CS`, `CS2`,
   cadences typed in Hepokoski–Darcy notation, and pedals. Plus 23 per-piece `.dez` files (JSON, Bach only, one
   fugue missing) and `synchro` files aligning labels to particular recordings. The `.ref` form gives offsets in
   **measures** with exact fractional extras (`29-1/16`); the `.dez` form gives them in **quarters** as plain
   integers, which is the one to parse. `syntax.ref` documents the format completely.

   **Three things worth having found.**
   - The BWV 867 stretto extracted cleanly and now anchors [§3.1](#31-the-calibration-disappears): five entries at quarters
     `{266, 268, 270, 272, 274}`, one per voice, against a subject 12 quarters long. Ricercar spent two sections
     establishing this by hand from a score.
   - The eight contested subject-ends of [§3.3](#33-the-subject-is-input-and-its-boundary-is-contested) are named, with dissenting source attached — and the target fugue is
     one of them.
   - It refuted [§3](#3-the-measurement-61-wanted-computed-exactly)'s Sidon-set framing outright, which no amount of further reasoning would have done.

   **The annotations contain no notes** — every label is an offset into a score held elsewhere, which is why the
   second submodule exists. It is 16 MB, all 48 fugues of both books, **one part per spine**, which is what the
   annotations assume: Giraud's system starts *"from a symbolic score that was already separated into voices."*
   (The sibling repository `humdrum-tools/bach-wtc` is the two-staff keyboard layout and would need voice
   separation first — the wrong one to take.) `kern.humdrum.org` was returning 503; the GitHub mirror is the
   reliable route. Shostakovich still needs Marques' MIDI, so the corpus is Bach-only for now — 24 fugues, which is
   enough for steps 1 and 3.

   **Cross-check: voice counts agree on all 24.** Huron's `!!!parts` matches the algomus voice count fugue for
   fugue. (No. 24 declares four and its own note records that *"the texture increases to five voices in the final
   two measures"* — the annotation agrees, so this is a real musical fact rather than a disagreement.)

   **And the hand transcription survives its first falsification test.** BWV 867's subject was entered into
   [`ricercar/src/main.rs`](ricercar/src/main.rs) by reading a score. Huron's top spine gives
   `2b- 2f 4r 4gg- 4ff 4ee- 4dd-` — B♭4 half, F4 half, quarter rest, then G♭5 F5 E♭5 D♭5 as quarters. That is the
   transcription **note for note and offset for offset**, including the descending fourth that ricercar [§7.5](ricercar/readme.md#75-the-real-subject-and-two-things-it-broke) warns
   memory inverts, and the minor ninth from F4 to G♭5.

   > **But it is the *short* reading, and nobody knew that.** The next event in the spine is `4cc` — C5, continuing
   > past where the transcription stops. Ricercar's six notes end at 8 quarters, which is Prout and Bruhn's
   > two-measure "male ending"; the algomus ground truth takes Keller and Bruhn's three-measure "female ending" as
   > primary. So the existing capacity work sits on one side of a documented editorial dispute, chosen by accident.
   > [§3.3](#33-the-subject-is-input-and-its-boundary-is-contested)'s capacity-over-length profile is not a refinement for this fugue — it is the only honest way to report it.

   **One more integration hazard, from Huron's own note:** *"The alto and second-soprano parts exchange registers
   between measures 29 and 37."* Spine index is therefore **not** a stable voice identity, while the annotations
   are keyed to voice letters (SATBC). Any parser that assumes spine `i` is voice `i` throughout will mislabel
   entries in exactly the register-crossing passages that matter most here.

   *(Erratum worth carrying: the dataset README, the `fugues.ref` header and the project page all give Shostakovich
   as "op. 57, 1952". It is op. 87, 1950–51 — op. 57 is the Piano Quintet. The journal paper has it right.)*
1. ~~**The two-voice automaton.**~~ **Done — see [§9](#9-step-1-result-two-rules-survive-bach-three-do-not).** Built in [`src/`](src), 40 reachable states at first and
   **513** once the obligation field widened, against a crude product of 1280. All three verdict tests pass. The
   corpus checker ran and produced the more interesting result. Original wording follows.

   Build it, minimise it, **report the reachable state count**, and split the rules
   **hard versus soft** in Komosinski & Szachewicz's manner — the automaton takes the hard ones, [§5](#5-what-this-will-not-do)'s Pareto front
   takes the rest. Verdict tests, in order of how much they would hurt to fail:
   - parallel fifths flagged; a bare fifth consonant; a suspension distinguished from an accidental dissonance of
     the same interval. Ricercar [§7.2](ricercar/readme.md#72-step-2-result-a-proof-not-a-sample) had to substitute its own test because the field could not do the first;
   - **run it as a checker over the 36-fugue corpus and count how often Bach violates it.** A rulebook that flags
     Bach on every page is the Schottstaedt failure of [§5](#5-what-this-will-not-do) arriving early and cheaply, and it is far better to learn
     that from a checker than from a composition. This test costs almost nothing and is the single most
     informative thing in the roadmap.
2. ~~**The compatibility table and the clique**, on BWV 867's subject.~~ **Done — see [§10](#10-step-2-result-the-clique-test-passes-on-the-tier-bach-chose).** The full tier
   rejects Bach's hyperstretto; the two-rule tier of [§9.4](#94-the-result-and-it-stratifies-the-rulebook) accepts it, under both readings of the subject. Carry the **tonal answer as its own `τ`** from
   the start ([§2.1](#21-exact-arithmetic-and-therefore-no-certificates)). Verdict test, per [§3.1](#31-the-calibration-disappears): *does Bach's Stretto II come out as a clique?* Pass calibrates the
   automaton; fail falsifies it. No constant is fitted either way.
3. ~~**The corpus ranking.**~~ **Done — see [§11](#11-step-3-result-a-ranking-and-the-rulebook-that-can-produce-it-is-not-the-one-bach-confirms).** BWV 849 ranks first of 24, which is the right answer;
   but the tier that produces a ranking at all is one Bach violates. Original wording follows.
   Ricercar [§6.1](ricercar/readme.md#61-the-measurement), blocked twice there, at milliseconds per subject here — over 36 real
   subjects rather than a handful. Report **capacity as a profile over subject length** ([§3.3](#33-the-subject-is-input-and-its-boundary-is-contested)), not as a single
   number, since eight of the twenty-four Bach subjects have contested endings.
4. ~~**Subject design**, per [§3.2](#32-and-62-gets-easier-not-harder) — search over contours with the head fixed.~~ **Run — see [§13](#13-step-4-result-the-measure-is-valid-for-ranking-and-invalid-for-design).** The optimum
   is a monotone, and Bach's own contours score *below* random: the measure ranks subjects but cannot design one.
5. **Realisation**, with an escalation ladder rather than one algorithm:
   - `V − e ≤ 2` free voices: Viterbi against the harmonic automaton. Exact. Budget: seconds, per the survey's
     figure for two-voice florid counterpoint;
   - `V − e = 3`: the same, pruned by harmony, or a solver;
   - `V − e ≥ 4`: SAT/CDCL per [§2.7](#27-where-a-solver-takes-over-from-the-dp) — one-hot pitch, table constraints, unrolled automaton, symmetry broken by
     register and by pinning the first entry. **Treat this as open**, not as more of the same. Schottstaedt aimed at
     five to eight parts in 1984, found his complete search dragged to a halt, and shipped a sixteen-wide beam with
     a decaying acceptance threshold. **Nothing in the literature here does five-voice florid counterpoint with a
     complete search**, and no layering shortcut is available — he reports that one failing at three voices.
     Solve the ensemble jointly or not at all.
   Then MIDI. The first audible output of either document.
6. **Form**, per [§2.4](#24-form-is-a-grammar) — which the survey names as unsupported by any existing system, so expect to build rather
   than borrow. A whole fugue, with the packing question living inside the stretto block where it belongs.

Steps 0 to 3 are the ones that pay for themselves, and step 0 is now nearly free. They are perhaps a few hundred
lines and they close a question that has been open across two blocked attempts.

---

## 9. Step 1 result: two rules survive Bach, three do not

`cargo run --release` in this directory, against the submodules of [§8](#8-roadmap) step 0. Pitch is a diatonic step with an
alteration rather than a semitone integer, because a diminished fifth and an augmented fourth are the same six
semitones and different intervals; time is in ticks of 1/128 of a whole note, which is exact for this corpus —
reciprocals `{1,2,4,8,16,32}`, at most one dot, and no tuplets in either book.

### 9.1 The state count, measured

| | |
|---|---:|
| alphabet | 1600 |
| crude product | 1280 |
| **reachable** | **513** |
| distinct obligation sets | 128 of 256 |
| hard rules / soft criteria | 5 / 6 |

[§2.2](#22-counterpoint-is-a-finite-automaton) guessed "on the order of `10³` before minimisation". The crude product is 1280 and reachability cuts it to
513, so the guess was right in magnitude and the real number is smaller. **The first version of the automaton had
40 reachable states** — the count quadrupled when [§9.3](#93-bach-found-two-bugs-in-the-rulebook-in-the-first-run)'s correction split one kind of debt into two and let
obligations persist across held notes. That is worth recording: the state count is a property of how carefully the
rules are stated, not a constant of counterpoint.

### 9.2 The verdict tests pass

All three, including the two ricercar could not state at all. Parallel fifths are flagged. A bare fifth is
consonant — the roughness field measured it at `0.089`, among the *least* rough intervals there are, which is why
[§7.2](ricercar/readme.md#72-step-2-result-a-proof-not-a-sample) of that document had to substitute a different test. And a 7–6 suspension is accepted where the same seventh,
leapt into on the same beat, is rejected: **the same instantaneous interval, distinguished by the path taken to
it**, which is what a field over instantaneous pitch cannot do.

### 9.3 Bach found two bugs in the rulebook, in the first run

The checker's first pass flagged **290 hard violations per 1000 slices** — one slice in three. Almost all of it was
mine, and localising it took one diagnostic each.

- **A dissonance does not always resolve downward.** The first version demanded a descending step from every
  dissonance. Only a *suspension* must descend; a passing note leaves by step in whichever direction it was going.
  Schottstaedt's third-species comment says it plainly — "can be if passing either way". One wrong word in one rule
  produced **79% of all violations**. Fixed by splitting the debt into two kinds, which is what widened the state
  space from 40 to 513.
- **Melody is a property of one voice, and it was being counted per pair.** In a five-part fugue each voice belongs
  to four pairs, so every interval it sang was counted four times. Moved to a per-voice pass with its own
  denominator.

A third, found by reading rather than by running: the pairwise walk tracked the previous *lower* and *upper* pitch
rather than each voice's own history, so at every voice crossing it measured melodic intervals between two
different singers and corrupted the motion type that parallel detection depends on.

### 9.4 The result, and it stratifies the rulebook

24 fugues, 114 voice pairs, 33 331 slices, 23 498 note-to-note moves.

| rule | | count | per 1000 |
|---|---|---:|---:|
| parallel perfect | H | 35 | **1.1** |
| direct to perfect on downbeat | H | 23 | **0.7** |
| unprepared dissonance | H | 717 | 21.5 |
| unresolved dissonance | H | 3059 | 91.8 |
| forbidden melodic interval | H | 883 | 37.6 |

**Two rules are confirmed by Bach and three are refuted by him.** Parallel perfect consonances and direct motion to
a perfect consonance on a downbeat occur about once per thousand slices across the whole book — which for a rule
meant to be absolute is as close to vindication as a corpus can give. Those two are also precisely the rules a
roughness field cannot express at all, since a perfect fifth is one of the *smoothest* intervals it knows. **The
part of the rulebook that most justified abandoning the continuum is the part that survives contact with Bach.**

The other three do not. What the melodic rule is objecting to, by frequency: sevenths, diminished fifths, augmented
fourths, diminished fourths and chromatic semitones — every one of them idiomatic in the Well-Tempered Clavier and
every one forbidden by Fux. That is not Bach breaking a rule; it is Fux describing a different repertoire, and
[§5](#5-what-this-will-not-do)'s warning that the rulebook encodes "a style, and a caricature of one" arriving as a measurement.

### 9.5 The comfortable explanation is not available

The obvious defence of the three failing rules is scope: a pair of voices drawn from a five-part fugue is not a
two-part exercise, and a seventh between alto and bass is ordinary when a third voice supplies the chord that
explains it. [§3](#3-the-measurement-61-wanted-computed-exactly) assumes something adjacent — that pairwise legality is *necessary but not sufficient*.

The data does not support the defence. **Fugue No. 10 is the only two-voice fugue in the book and it has the worst
hard-violation rate of all 24, at 327 per thousand against a mean of 147.** There is no third voice to explain
anything away, and it is still the worst. So the three failing rules are not mis-scoped, they are simply too strict
for free counterpoint — and [§3](#3-the-measurement-61-wanted-computed-exactly)'s assumption may be wrong in the opposite direction from the one it anticipated:
pairwise checking here is too **strict**, not too loose.

### 9.6 What this changes

- **The hard tier is smaller than assumed.** Only `ParallelPerfect` and `DirectPerfectOnDownbeat` have earned the
  status. The dissonance and melodic rules should move to the soft tier, where [§5](#5-what-this-will-not-do)'s Pareto front can rank them
  without anyone having to assert that Bach is wrong.
- **Step 2's calibration test is now the sharper question.** With a two-rule hard tier, does BWV 867's stretto
  `{0, 2, 4, 6, 8}` still come out as a clique? A rulebook this permissive will more easily say yes, which makes a
  *failure* there much more informative than it was going to be.
- **Step 5's solver has less to prove.** A hard tier of two rules is a far smaller constraint than five, so the
  five-voice realisation of [§2.7](#27-where-a-solver-takes-over-from-the-dp) is a lighter problem than Schottstaedt's — but for the same reason it constrains
  less, and the burden shifts onto the soft criteria and their ordering.

---

## 10. Step 2 result: the clique test passes, on the tier Bach chose

`cargo run --release -- stretto`. The subject is cut from Huron's encoding; the five entry offsets come from the
algomus ground truth; the transpositions are **recovered from the score** rather than assumed, and come out
`B♭ – F – B♭ – F – B♭` descending across five octaves — tonic and dominant alternating, which is what a stretto
layout is supposed to be, and a check on the extraction rather than a result.

### 10.1 The verdict

[§3.1](#31-the-calibration-disappears) promised integer equality rather than a fitted threshold, and that is what it is: are the five entries at
`{0, 2, 4, 6, 8}` quarters mutually compatible?

| subject reading | full hard tier (5 rules) | confirmed tier (2 rules, [§9.4](#94-the-result-and-it-stratifies-the-rulebook)) |
|---|---|---|
| female, 3 measures (Keller, Bruhn) | **fail** — max clique 4 of 5 | **pass** — 5 of 5 |
| male, 2 measures (Prout, Bruhn) | **fail** — max clique 4 of 5 | **pass** — 5 of 5 |

**The full rulebook rejects Bach's own hyperstretto; the tier Bach confirmed accepts it.** Both failures are the
same rule, `unresolved dissonance`, on the pair `+0q` against `+6q` — and that is the rule [§9.4](#94-the-result-and-it-stratifies-the-rulebook) had already
measured firing 91.8 times per thousand slices across the whole book. Two independent tests, on different data
and asking different questions, **select the same rulebook**: [§9](#9-step-1-result-two-rules-survive-bach-three-do-not) by counting how often each rule fires in 24
fugues, [§10](#10-step-2-result-the-clique-test-passes-on-the-tier-bach-chose) by asking whether one passage is a clique. That convergence is the strongest thing in this document,
because neither test was designed to check the other.

The result also does not depend on the editorial dispute of [§3.3](#33-the-subject-is-input-and-its-boundary-is-contested). Both readings of the subject give the same
verdict under both tiers, which is worth recording because [§3.3](#33-the-subject-is-input-and-its-boundary-is-contested) predicted the opposite — that the contested ending
would be load-bearing. Here it is not.

### 10.2 The control, which is what makes the verdict mean anything

A clique test on *templates* proves nothing about Bach if the template is a bad model of an entry: exact
transposition is an idealisation, and a failure could be the model's fault rather than the rulebook's. So the same
window of the **actual score** — all five voices, measures 67.5 to 71, ten real pairs — is checked directly.

| | full tier | confirmed tier |
|---|---:|---:|
| Bach's actual notes, 10 pairs | **15** | **1** |

The real passage fails the full tier too, and by more than the template does. So the template is not the problem
and the rulebook is: [§3.1](#31-the-calibration-disappears)'s falsification branch, taken.

The single confirmed-tier violation is worth naming rather than rounding away. It is a **direct motion to a
perfect consonance on a downbeat, between the two middle voices**, and it is in Bach and not in the template —
the idealised exact transpositions avoid it, the written music does not. One violation in ten voice pairs of the
densest passage in the book is close enough to zero to call the two-rule tier confirmed, and honest enough to say
it is not literally zero.

### 10.3 A defect the test caught in the reading of the corpus

The first run reported the male ending as a clique and the female as a failure — an apparently sharp result about
the editorial dispute, and an artefact. The subject window was cut with an **exclusive** end, and the corpus's own
`syntax.ref` defines a length as measured "between the offsets of the start and the end", where "the end of the
pattern denotes the impact of the last note" — inclusive.

It was caught by a cross-check rather than by reading the code: the two-measure cut produced a five-note subject
where ricercar's hand transcription of the same span has six. That transcription was made independently from a
score, and disagreeing with it was the signal. Corrected, the two readings agree, and the apparent finding about
the subject boundary evaporated.

### 10.4 What this settles, and what it does not

- **The hard tier is two rules.** Measured in [§9](#9-step-1-result-two-rules-survive-bach-three-do-not), confirmed here. `UnpreparedDissonance`, `UnresolvedDissonance`
  and `ForbiddenMelodic` move to the soft tier and go to [§5](#5-what-this-will-not-do)'s Pareto front, where they can be ranked without
  anyone having to claim Bach is wrong.
- **The compatibility table works**, and the clique formulation of [§3](#3-the-measurement-61-wanted-computed-exactly) is now demonstrated rather than argued.
- **It does not show the method can compose.** A two-rule hard tier admits Bach's stretto, and it will admit an
  enormous amount of bad counterpoint too — [§5](#5-what-this-will-not-do)'s inverted failure mode, arriving on schedule. Everything now rests
  on the soft criteria, which is exactly where this document has always said the difficulty lives.

---

## 11. Step 3 result: a ranking, and the rulebook that can produce it is not the one Bach confirms

`cargo run --release -- rank`. Subjects come from the ground truth for all 24 fugues — length and entry positions
from `fugues.ref`, notes from Huron's encoding — so the ranking measures the corpus rather than my transcriptions.
Candidate entries are every diatonic transposition `−7..+7` at every quarter-note offset **within** the subject, so
that each entry in a clique genuinely overlaps every other.

### 11.1 The measure is vacuous on the tier §9 and §10 selected

This is the result, and it was not the expected one.

| hard tier | graph density | capacity of BWV 867's subject |
|---|---:|---|
| confirmed, 2 rules ([§9.4](#94-the-result-and-it-stratifies-the-rulebook)) | **0.810** | 4, 6, 8, 10 … **does not converge** — it returns whatever cap it is given |
| full, 5 rules | 0.427 | **6**, and stable at caps of 8, 10 and 12 |

Under the two-rule tier, **81% of all entry pairs are mutually compatible**, and the largest legal stretto keeps
growing until the search is cut off. That is not a measurement of a subject; it is a measurement of how permissive
the rulebook is.

And the reason is structural rather than incidental:

> **The rules Bach never breaks are precisely the rules that almost never bind.**

[§9](#9-step-1-result-two-rules-survive-bach-three-do-not) selected the two-rule tier by asking which rules Bach obeys, and the answer was the two that fire about once per
thousand slices. A rule that fires once per thousand slices cannot forbid much of anything over the ten-slice
overlap of two entries — so the tier that survives Bach is exactly the tier that cannot discriminate between
subjects. **There is no single tier here that both accepts Bach's own hyperstretto and measures capacity.** [§10](#10-step-2-result-the-clique-test-passes-on-the-tier-bach-chose)
needed the permissive tier to pass; [§11](#11-step-3-result-a-ranking-and-the-rulebook-that-can-produce-it-is-not-the-one-bach-confirms) needs the strict one to say anything.

That is [§5](#5-what-this-will-not-do)'s *"completeness is not selectivity"* arriving as a number, and it is the sharpest limit this document
has found in itself.

### 11.2 The ranking, on the strict tier, and it agrees with musicians

Reported under the full five-rule tier — the only one that converges — with the standing caveat that Bach violates
three of its rules. The last row is BWV 869, which entered the table only after [§12.0](#120-a-parser-bug-found-on-the-way-which-corrected-everything-upstream)'s
parser fix; before it, that fugue's spines were misread and it produced no subject. Nothing above it moved.

| | fugue | subject | notes | capacity |
|---|---|---|---|---:|
| 1 | **wtc-i-04, BWV 849 (C♯ minor, 5 voices)** | 12q | 5 | **11** |
| 2 | wtc-i-14, BWV 859 (F♯ minor) | 18q | 20 | 9 |
| 3= | wtc-i-12, BWV 857 (F minor) | 12q | 11 | 6 |
| 3= | **wtc-i-22, BWV 867 (B♭ minor, 5 voices)** | 12q | 10 | 6 |
| 5 | wtc-i-19, BWV 864 (A major) | 9q | 16 | 5 |
| … | | | | |
| 22= | wtc-i-20, BWV 865 (A minor) | 12q | 31 | 2 |
| 23 | wtc-i-21, BWV 866 (B♭ major) | 12q | 38 | 2 |
| 24 | wtc-i-24, BWV 869 (B minor) | 12q | 21 | 2 |

**BWV 849 comes first.** That is the fugue musicians name when they name a stretto fugue — ricercar [§6.0](ricercar/readme.md#60-the-benchmark-bwv-867) singled it
out as "a triple fugue, which is the multi-shape tiling problem and strictly harder", and it is the piece whose
austere five-note subject Bach strettos more thoroughly than any other in the book. **BWV 867, the other famous
stretto fugue, is joint third of twenty-four.** Ricercar [§6.1](ricercar/readme.md#61-the-measurement) asked for a measure "falsifiable against what
musicians already believe"; on its own terms, this one passes.

### 11.3 How much of that is just note density

The honest check, since a sparse subject obviously strettos more easily than a busy one:

| | |
|---|---:|
| notes per quarter vs capacity | **r = −0.750** (Spearman −0.682) |
| subject length vs capacity | r = +0.554 |
| note count vs capacity | r = −0.382 |

So **about 56% of the variance is note density**, which is a real musical fact and not a deep one. The measure is
substantially a proxy for how much a subject is already doing.

What survives that is the residual — subjects that stretto better than their density predicts — and it is the same
piece again:

| residual | fugue | |
|---:|---|---|
| **+3.76** | wtc-i-04, BWV 849 | 5 notes in 12q, capacity 11 |
| +3.15 | wtc-i-14, BWV 859 | 20 notes in 18q, capacity 9 |
| −2.28 | wtc-i-08, BWV 853 | 14 notes in 10q, capacity 3 |

BWV 849 is both the highest capacity and the largest positive residual, so its ranking is not merely an artefact of
its sparseness. That is the one place where this measure says something a note count does not.

### 11.4 §3.3 was right, and the numbers are worse than it feared

Capacity computed at the primary reading and at every dissenting source's:

| fugue | primary | alternatives |
|---|---|---|
| wtc-i-05 | 4q → 3 | 2q → 2 |
| wtc-i-07 | 6q → **4** | 8q → **2** |
| wtc-i-09 | 2q → 2 | 4q → **4**, 6q → 3 |
| wtc-i-18 | 6q → 3 | 3q → 3 |
| wtc-i-22 | 12q → 6 | 8q → **7** |

Two things, both worse than [§3.3](#33-the-subject-is-input-and-its-boundary-is-contested) anticipated. **The editorial choice can halve the number** — BWV 852 reads 4 or 2
depending on whether one follows Keller or Prout. And **capacity is not monotonic in subject length**: BWV 854 goes
2 → 4 → 3 as the subject lengthens. So a single capacity figure is not merely imprecise, it is not even a
well-behaved function of the one input a reader would assume it depends on. Any published ranking has to be a
profile, exactly as [§3.3](#33-the-subject-is-input-and-its-boundary-is-contested) proposed, and [§11.2](#112-the-ranking-on-the-strict-tier-and-it-agrees-with-musicians)'s table should be read with that in mind.

### 11.5 A defect the measure found in itself

The first implementation let several entries share an offset. It therefore counted harmonising the subject in
parallel thirds as a five-voice stretto, and returned **exactly whatever clique cap it was given, for every subject
in the corpus** — 3 for a cap of 3, 6 for a cap of 6. A stretto is a succession of entries, not a chord of them,
and Bach's own is five distinct offsets. Fixed by requiring one entry per offset; the symptom that gave it away was
the suspiciously perfect agreement between the answer and the parameter.

---

## 12. Resolving §11.1: five experiments, and the one that works

[§11.1](#111-the-measure-is-vacuous-on-the-tier-9-and-10-selected) left the programme stuck: the rules Bach never breaks are the rules that almost never bind, so no tier both
accepts his hyperstretto and discriminates between subjects. Five ways out were proposed and all five were run.

### 12.0 A parser bug found on the way, which corrected everything upstream

Reading the Renaissance corpus exposed a defect that had been in every earlier result. **Vocal music interleaves
`**text` and `**silbe` lyric spines with the `**kern` ones**, and the reader pushed a spine only for `**kern` while
indexing data fields by position — so every note after the first lyric column was read from the wrong field. 60 of
200 Renaissance files failed outright, and the 140 that parsed were a biased sample of the textless ones.

The same bug was quietly present in **Bach**: fugue 24's header carries an empty column, which had been shifting
its spines for the entire project. Fixed by tracking every spine and marking which of them bear notes. All figures
in [§9](#9-step-1-result-two-rules-survive-bach-three-do-not) are restated below at their corrected values; nothing moved by more than 4%, and no conclusion changed.

| | before | after |
|---|---:|---:|
| parallel perfect | 35 (1.1/1k) | **36 (1.0/1k)** |
| direct to perfect on downbeat | 23 (0.7/1k) | 23 (0.7/1k) |
| unprepared dissonance | 717 (21.5) | **750 (21.4)** |
| unresolved dissonance | 3059 (91.8) | **3179 (90.9)** |
| forbidden melodic | 883 (37.6) | **904 (37.6)** |

[§10](#10-step-2-result-the-clique-test-passes-on-the-tier-bach-chose)'s clique verdicts are unchanged: full tier fails, confirmed tier passes, 15 against 1 on the control.

### 12.1 Experiment 1 — density instead of clique size. **This resolves it.**

Clique *size* saturates because the permissive tier admits almost everything. Graph *density* cannot saturate: it
is bounded in `[0, 1]` by construction.

| | |
|---|---|
| spread of 2-rule density across 24 subjects | **0.321 … 0.956** |
| ranking | **BWV 849 first** (0.956); the three lowest are BWV 860, 866, 865 |
| density vs note density | **r = −0.311** |
| clique capacity vs note density ([§11.3](#113-how-much-of-that-is-just-note-density)) | r = −0.750 |
| 2-rule density vs 5-rule density | r = +0.639 |

Density under the tier **Bach confirms** discriminates cleanly, ranks BWV 849 first exactly as the strict-tier
clique did, and is **less than half as contaminated by note density** as clique capacity was. That is the measure
to use: it needs no rule Bach violates, and it answers [§11.3](#113-how-much-of-that-is-just-note-density)'s objection at the same time.

### 12.2 Experiment 2 — Pareto calibration. **Fails, and the failure is instructive.**

Keep the permissive hard tier; let the soft criteria filter edges, calibrated against Bach's own stretto as [§10](#10-step-2-result-the-clique-test-passes-on-the-tier-bach-chose)
calibrated the clique test. Bach's Stretto II gives, as the worst of its ten real pairs per slice:

```text
direct to perfect 0.077   perfect consonance 0.400   direct motion 0.500
voice crossing    0.000   unrecovered leap   0.000   repeated note  0.286
```

**Two of those are zero, and a zero in a domination limit is an absolute prohibition.** Bach's five entries happen
to contain no voice crossing and no unrecovered leap, so the calibration silently forbids both outright. Capacity
collapses to between 1 and 4, and **BWV 867 itself scores 2** — the calibration cannot reproduce the passage it was
calibrated on.

The ranking survives — BWV 849 and BWV 859 still lead at 4 — but the measure is degenerate. The lesson is specific
and worth keeping: **calibrating a domination limit on the componentwise maximum of a single passage turns every
criterion that passage happens not to exhibit into a hard prohibition.** A usable version needs a reference set
broad enough that no component is zero.

### 12.3 Experiment 3 — the Renaissance control. **Decisive, and it splits the three refuted rules.**

Fux describes 16th-century vocal polyphony. The same rulebook, unchanged, over 200 works from the Josquin Research
Project. The sample is the first 200 files in path order across six composer directories, which — checked while
writing [§15](#15-reproducing-the-results) rather than assumed — is **70 Busnois, 37 Dufay and 93 Josquin**, and no Ockeghem, Obrecht or La Rue at all:
the sort truncates before reaching them. 299 613 slices.

| rule | Renaissance | Bach | ratio |
|---|---:|---:|---:|
| parallel perfect | 1.2 | 1.0 | — |
| direct to perfect on downbeat | 1.5 | 0.7 | — |
| **forbidden melodic interval** | **1.0** | **37.6** | **×38** |
| unprepared dissonance | 8.0 | 21.4 | ×2.7 |
| unresolved dissonance | 71.1 | 90.9 | ×1.3 |

**The melodic rule is vindicated completely.** One violation per thousand moves in the repertoire Fux is writing
about, against 37.6 in Bach — a factor of thirty-eight. It is not a broken rule; it is a correct rule about
Renaissance vocal writing, applied to eighteenth-century keyboard music. [§9.4](#94-the-result-and-it-stratifies-the-rulebook) called it "refuted by Bach"; the
honest statement is that **it was never about Bach**.

**The dissonance rules are not rescued.** 71.1 per thousand in the Renaissance is barely better than Bach's 90.9,
and this is the repertoire those rules were written for. They are wrong as I implemented them, in both centuries —
my error, not a repertoire mismatch. That is the sharpest correction of the five experiments, and it points
directly at [§12.5](#125-experiment-5--harmony-supportive-and-not-yet-enough).

**And the two confirmed rules hold in both.** 1.2 against 1.0, and 1.5 against 0.7. A prohibition on parallel
perfect consonances is apparently invariant across two centuries and two media, which is a stronger claim for those
two rules than [§9](#9-step-1-result-two-rules-survive-bach-three-do-not) could make from Bach alone.

### 12.4 Experiment 4 — chromaticism. **Negative.**

If the melodic rule were objecting to chromatic writing, its rate should track how chromatic each fugue is.

| | |
|---|---:|
| chromaticism vs melodic rule | r = +0.249 |
| chromaticism vs dissonance rule | r = +0.031 |

It does not — six per cent of the variance. Combined with [§12.3](#123-experiment-3--the-renaissance-control-decisive-and-it-splits-the-three-refuted-rules)'s factor of thirty-eight, the melodic difference is
a property of **repertoire and medium**, not of chromatic writing fugue by fugue. Keyboard music leaps differently
from voices whatever its harmonic language.

### 12.5 Experiment 5 — harmony. **Supportive, and not yet enough.**

The hypothesis: dissonance in tonal music is governed harmonically, so a harmonic constraint should be one Bach
*satisfies* and arbitrary placements *violate* — the property no current rule has.

| | ≥3-note sonorities explained by a triad or seventh chord |
|---|---:|
| Bach, all 24 fugues | **78.0%** (5983 of 7672) |
| arbitrary 3-entry strettos from the same subjects | **56.9%** (1355 of 2382) |

The gap is real and in the right direction: harmony separates Bach from arbitrary placement by 21 points, where the
dissonance rules separate them by nothing. But 78% for Bach is too low to be a hard rule — the missing 22% is
suspensions, passing tones and appoggiaturas, which are non-chord tones *by design*. The crude template test is
therefore the right idea and the wrong instrument. It needs the non-chord-tone treatment that [§2.3](#23-harmony-is-a-second-automaton)'s harmonic
automaton was always meant to supply, at which point Bach's figure should approach 100% and the constraint becomes
usable.

### 12.6 Where this leaves the programme

- **Capacity is measured by density under the two-rule tier** ([§12.1](#121-experiment-1--density-instead-of-clique-size-this-resolves-it)). [§11.1](#111-the-measure-is-vacuous-on-the-tier-9-and-10-selected)'s deadlock is resolved, with no rule
  Bach violates and less proxy contamination than the clique measure it replaces.
- **The melodic rule is restored** — to the hard tier for Renaissance repertoire, and dropped for Bach ([§12.3](#123-experiment-3--the-renaissance-control-decisive-and-it-splits-the-three-refuted-rules)). It
  was never refuted, only mis-applied, and [§9.4](#94-the-result-and-it-stratifies-the-rulebook)'s language should be read accordingly.
- **The two dissonance rules are my bugs**, demonstrated by their failing in the very repertoire they were written
  for. They should be removed rather than demoted, and replaced.
- **The replacement is harmonic** ([§12.5](#125-experiment-5--harmony-supportive-and-not-yet-enough)), which promotes [§2.3](#23-harmony-is-a-second-automaton) — the one designed component never built — from
  optional extension to next piece of work. It is also what Anders & Miranda name as the neglected field.
- ~~**Step 4 is unblocked.**~~ **Wrong — see [§13](#13-step-4-result-the-measure-is-valid-for-ranking-and-invalid-for-design).** Density ranks subjects but its preferences run against
  Bach's contours, because both surviving rules need a perfect consonance to fire and a fugal answer is at the
  fifth. The measure penalises the interval the form is built on.

---

## 13. Step 4 result: the measure is valid for ranking and invalid for design

`cargo run --release -- design`. Rhythm is held fixed from a real subject and only the pitch contour varies, with
the head pinned — [§3.2](#32-and-62-gets-easier-not-harder)'s weighting intuition surviving as a search order. The objective is [§12.1](#121-experiment-1--density-instead-of-clique-size-this-resolves-it)'s density under
the two-rule tier, the measure [§12](#12-resolving-111-five-experiments-and-the-one-that-works) had just established. Hill-climbing with restarts, deterministic seed.

### 13.1 Three optima, all anti-musical

| | density | contour |
|---|---:|---|
| Bach's own subject (BWV 867) | 0.844 | 7 distinct degrees, no repeats, mean step 2.00 |
| random contours, same rhythm (400) | 0.830 ± 0.060 | — |
| **unconstrained optimum** | **1.000** | `[0,0,0,0,0,0,0,0,0,0]` — **one repeated note** |
| constrained: ≥5 degrees, no triple repeat | 0.959 | `[0,0,1,0,−4,1,3,−4,2,−6]`, mean step 3.78 |

**The unconstrained optimum is a monotone, and it is optimal by construction.** A static subject never moves, so
every slice is `Motion::None`, so neither `ParallelPerfect` (which needs parallel motion) nor
`DirectPerfectOnDownbeat` (which needs similar motion) can fire at all. Density 1.000, exactly, for every subject.
**The optimiser beat Bach on 20 of 20 rhythms, and every winner was a single repeated pitch.**

Constraining it to at least five distinct degrees with no triple repeat does not rescue it: the optimum simply
becomes jagged instead of static, mean melodic step 3.78 — a contour of leaps that no one would sing.

This is ricercar [§5](ricercar/readme.md#5-the-objection-and-what-it-is-actually-an-objection-to)'s warning arriving in the discrete setting, where that document had already stated it and then
argued its way out: *"maximum distance from every rule boundary is the safest counterpoint, hence the blandest."*
[§5](ricercar/readme.md#5-the-objection-and-what-it-is-actually-an-objection-to) rescued the maximin step by pointing out that circle packing *consumes* the clearance it finds. Nothing consumes
it here, so the warning lands undiluted.

### 13.2 The worse finding: Bach's contours score *below* random

The decisive control is per-subject — the mean density of random contours **on Bach's own rhythm**, so rhythm is
held exactly constant and only the pitch design varies.

| | |
|---|---:|
| Bach's contour beats a random one on the same rhythm | **5 of 20** |
| mean advantage of Bach's contour | **−0.0763** |
| Bach density vs random-on-same-rhythm, across subjects | r = +0.375 |

**Bach's contours are worse than random on this measure, on average and in three quarters of cases.** That is not
blindness — `r = +0.375` shows the measure is sensitive to contour — it is a measure whose preferences run
*against* Bach's, while [§12.1](#121-experiment-1--density-instead-of-clique-size-this-resolves-it)'s across-subject ranking still comes out musically right.

### 13.3 Why, and it is structural rather than accidental

Both surviving rules require a **perfect consonance** to fire: `ParallelPerfect` needs two of them in succession,
`DirectPerfectOnDownbeat` needs one arrived at by similar motion. So maximising density means **minimising the
perfect consonances a subject forms against its own transpositions**.

And a fugal answer is at the fifth. The entire form is built on a subject sounding against itself at a perfect
consonance — that is what an answer *is*, and [§10](#10-step-2-result-the-clique-test-passes-on-the-tier-bach-chose) recovered exactly that from BWV 867's stretto: entries at
`B♭ – F – B♭ – F – B♭`, tonic and dominant alternating.

> **The measure penalises precisely the interval the form is built on.**

A subject optimised for density is therefore a subject optimised to make a *bad answer*. That is not a defect in
the implementation and no amount of constraint tuning will remove it; it follows from which two rules survived [§9](#9-step-1-result-two-rules-survive-bach-three-do-not),
and those two survived because they are the only ones Bach never breaks.

### 13.4 What this settles

- **Density is valid for ranking and invalid for design.** These are different uses with different validity, and
  [§12.1](#121-experiment-1--density-instead-of-clique-size-this-resolves-it)'s claim to have resolved [§11.1](#111-the-measure-is-vacuous-on-the-tier-9-and-10-selected) stands only for the first. [§12.6](#126-where-this-leaves-the-programme)'s last bullet — "step 4 is unblocked" —
  was wrong, and this section is the correction.
- **Across-subject ranking survives.** BWV 849 still comes first, and [§12.1](#121-experiment-1--density-instead-of-clique-size-this-resolves-it)'s numbers are unaffected: nothing here
  touches the comparison between subjects, only the attempt to move within one.
- **What a design objective would need** is something that *rewards* a subject working at the fifth rather than
  penalising it — which is a harmonic statement, not a contrapuntal one. A fugal answer at the dominant is a
  harmonic relationship, and [§12.5](#125-experiment-5--harmony-supportive-and-not-yet-enough) already identified harmony as the missing constraint on independent grounds.
  Step 4 and step 6 therefore need the same thing, and it is [§2.3](#23-harmony-is-a-second-automaton).
- **Step 4 is blocked, not failed.** It ran, it produced a clean answer, and the answer is that the objective is
  wrong. Ricercar [§6.2](ricercar/readme.md#62-the-design-problem) anticipated the shape of this: *"If the optimized subject scores below Bach's, that is the
  more interesting result and should be reported as such."* The optimised subject scores *above* Bach's, on a
  measure that prefers monotones — which is the same finding with the sign flipped, and it is more interesting
  than a ranking would have been.

---

## 14. §2.3 built: harmony repairs the design objective and fails as a rule

`cargo run --release -- harmony`. [`src/harmony.rs`](src/harmony.rs) segments a texture at the notated beat, scores
every root-and-quality candidate by duration-weighted membership, takes the best, and then classifies every note
against the chord that prevails where it sounds — chord tone, or a named dissonance: suspension, passing,
neighbour, appoggiatura, escape, or untreated. Two open questions sent me here, and it answers them differently.

### 14.1 As a rule, it fails — and it corrects §12.5's reading

[§12.5](#125-experiment-5--harmony-supportive-and-not-yet-enough) measured bare chord membership at **78.0%** on Bach against **56.9%** on arbitrary strettos and read the
21-point gap as harmony discriminating. Accounting for non-chord tones was supposed to lift Bach toward 100% and
make the constraint usable. It lifts Bach to **99.4%** — and lifts the control to **98.8%**.

| statistic | Bach | control | separation |
|---|---:|---:|---:|
| explained, binary | 99.4% | 98.8% | +0.7 pts |
| chord tones, not dissonances | 80.0% | **82.7%** | **−2.6 pts** |
| mean chord fit | 0.868 | **0.881** | **−0.012** |
| **untreated dissonances per 1000 notes** | **5.5** | **12.5** | **+6.9** |

**The control fits chords better than Bach does.** That is not a paradox, it is the control being unfair: three
transpositions of one subject is a thin, self-similar texture with few distinct pitch classes at any instant, and
such a thing trivially fits a triad. Bach's fugues carry countersubjects and free voices, and the resulting
sonorities are richer and harder to name. **[§12.5](#125-experiment-5--harmony-supportive-and-not-yet-enough)'s 21-point gap was measuring texture complexity, not harmonic
correctness**, and the conclusion drawn from it — that harmony is the constraint Bach satisfies and arbitrary
placement violates — is not supported once non-chord tones are accounted for.

What survives is the last row, and only that row: **untreated dissonances, 5.5 per thousand notes in Bach against
12.5 in arbitrary strettos.** A factor of 2.3, correctly signed, and it is the statistic that corresponds to an
actual rule rather than to a property of the texture. But 5.5 is five times the rate of the two confirmed
contrapuntal rules, so it is a soft criterion, not a hard one.

And the binary version fails for the reason [§11.1](#111-the-measure-is-vacuous-on-the-tier-9-and-10-selected) already named, arriving now in the harmonic domain: **widening
the rule until Bach passes widens it until everything passes.** Six named dissonance categories cover nearly every
approach-and-departure pair; only leaping in *and* leaping out is left over.

### 14.2 As a design objective, it works — and this is the repair §13.3 asked for

[§13](#13-step-4-result-the-measure-is-valid-for-ranking-and-invalid-for-design) optimised a contour against contrapuntal density and got a monotone, because both surviving rules need a
perfect consonance to fire and a fugal answer is at the fifth. The harmonic objective scores a subject sounding
**against its own answer a fifth below** — so the answer relationship is what is being rewarded rather than
penalised. Every diagnostic reverses:

| | [§13](#13-step-4-result-the-measure-is-valid-for-ranking-and-invalid-for-design), contrapuntal | [§14](#14-23-built-harmony-repairs-the-design-objective-and-fails-as-a-rule), harmonic |
|---|---:|---:|
| Bach's contour beats random on the same rhythm | **5 of 20** | **17 of 20** |
| mean advantage of Bach's contour | **−0.0763** | **+0.0552** |
| distinct degrees in the optimised contour | **1.0** — a monotone | **6.8** |
| Bach vs random, BWV 867 | +0.23 sd | **+0.91 sd** |
| could the optimiser beat Bach? | **yes, 20 of 20** | **no** |

On BWV 867 the search returns Bach's own contour — `B♭ F G♭ F E♭ D♭ C D♭ E♭ F`, seven distinct degrees, mean step
2.00 — and cannot improve on it. **The objective that penalised the interval the form is built on now prefers the
contours Bach actually wrote.**

> **Withdrawn — see [§16](#16-the-harmonic-layer-does-not-survive-validation).** Every figure in this section is
> self-referential. Four external checks were run afterwards and all four failed: the analyser identifies Bach's
> annotated cadences correctly 38% of the time against a 23% baseline, fits *modal* polyphony better than tonal,
> and reports effect sizes that vary elevenfold with a segmentation window nobody justified. The comparison against
> [§13](#13-step-4-result-the-measure-is-valid-for-ranking-and-invalid-for-design)'s contrapuntal objective stands;
> the claim that this one works does not.

The remaining flaw is a ceiling rather than a perversity. The measure saturates: Bach scores 1.000, and so does a
monotone. So it has stopped *preferring* degenerate contours, but it cannot *rank* among the ones it accepts. It is
a filter, not an ordering — which is exactly the shape [§5](#5-what-this-will-not-do) says the soft criteria and their Pareto front are for.

### 14.3 What §2.3 is and is not

- **The functional layer is written but untested.** `degree_of`, `progression_ok` and `is_cadence` implement [§2.3](#23-harmony-is-a-second-automaton)'s
  claim that a cadence is a labelled path rather than a coincidence. Nothing in [§14](#14-23-built-harmony-repairs-the-design-objective-and-fails-as-a-rule) exercises them; that is step 6's
  work, and it should not be reported as built until it has been run against something.
- **Harmony is a soft criterion, not a hard rule.** The one usable statistic is the untreated-dissonance rate, at a
  2.3× separation. It joins [§5](#5-what-this-will-not-do)'s Pareto front rather than the automaton.
- **Harmony is the right design objective** and the contrapuntal measure was the wrong one. That was the question
  [§13.4](#134-what-this-settles) left open, and it is now answered by measurement rather than by argument.
- **[§12.5](#125-experiment-5--harmony-supportive-and-not-yet-enough) is corrected in place.** Its interpretation of the 21-point gap does not survive its own follow-up, which
  is the fourth claim in this document to be refuted by the next experiment after the one that produced it.

---

## 15. Reproducing the results

Everything above is deterministic. There is no floating-point time, no sampling that is not seeded, and no
threshold that was chosen rather than measured. What follows is what a reader needs to get the same numbers, and
what they should know about how the samples were taken.

### 15.1 Environment and data

```
rustc 1.96.1   cargo 1.96.1     # no dependencies; std only
git clone --recurse-submodules <this repo>
cargo test --release            # 7 tests
```

| submodule | pinned | licence | used by |
|---|---|---|---|
| `corpus/algomus-data` | `a1801b5` | ODbL 1.0, contents DbCL 1.0 | subject positions and lengths, all steps |
| `corpus/bach-wtc-fugues` | `5095752` | Humdrum edition, David Huron | every Bach figure |
| `corpus/jrp-scores` | `52de715` | **CC BY-NC 4.0** | [§12.3](#123-experiment-3--the-renaissance-control-decisive-and-it-splits-the-three-refuted-rules) only |

The JRP licence is non-commercial; the other two are not. Anything redistributing these results should carry the
attribution each dataset asks for — Giraud, Groult and Levé for the annotations, the Josquin Research Project for
the Renaissance scores.

### 15.2 Which command produces which section

| section | command | tier |
|---|---|---|
| [§9.1](#91-the-state-count-measured) state count | `cargo run --release -- states` | — |
| [§9.2](#92-the-verdict-tests-pass) verdict tests | `cargo run --release -- verdict` | full |
| [§9.4](#94-the-result-and-it-stratifies-the-rulebook) corpus rates | `cargo run --release -- corpus` | both, reported per rule |
| [§9.3](#93-bach-found-two-bugs-in-the-rulebook-in-the-first-run) melodic breakdown | `cargo run --release -- diag` | full |
| [§10](#10-step-2-result-the-clique-test-passes-on-the-tier-bach-chose) clique test | `cargo run --release -- stretto` | both |
| [§11](#11-step-3-result-a-ranking-and-the-rulebook-that-can-produce-it-is-not-the-one-bach-confirms) ranking | `cargo run --release -- rank` | full (only one that converges) |
| [§11.1](#111-the-measure-is-vacuous-on-the-tier-9-and-10-selected) saturation | `cargo run --release -- probe` | both |
| [§12.1](#121-experiment-1--density-instead-of-clique-size-this-resolves-it)–[§12.5](#125-experiment-5--harmony-supportive-and-not-yet-enough) | `cargo run --release -- exp1` … `exp5`, or `exp` | as stated per experiment |
| [§13](#13-step-4-result-the-measure-is-valid-for-ranking-and-invalid-for-design) subject design | `cargo run --release -- design` | confirmed |
| [§14](#14-23-built-harmony-repairs-the-design-objective-and-fails-as-a-rule) harmony | `cargo run --release -- harmony`, or `h1`/`h2`/`h3` | — |

### 15.3 Parameters that would otherwise have to be read out of the source

| | |
|---|---|
| tick base | 960 per whole note — the smallest exact for both corpora ([`kern.rs`](src/kern.rs)) |
| hard tiers | `HARD` = 5 rules; `CONFIRMED` = `ParallelPerfect`, `DirectPerfectOnDownbeat` ([`automaton.rs`](src/automaton.rs)) |
| candidate grid, [§11](#11-step-3-result-a-ranking-and-the-rulebook-that-can-produce-it-is-not-the-one-bach-confirms) | offsets every **quarter** within the subject; diatonic transpositions **−7…+7**; one entry per offset; clique anchored at the subject; cap 12 |
| candidate grid, [§13](#13-step-4-result-the-measure-is-valid-for-ranking-and-invalid-for-design)–[§14](#14-23-built-harmony-repairs-the-design-objective-and-fails-as-a-rule) | offsets every **half note** (a coarser grid, for search cost), same transposition range |
| harmonic segmentation | at the notated beat, from the time signature; chord scored by duration-weighted membership; 9 qualities × 12 roots ([`harmony.rs`](src/harmony.rs)) |
| [§14.2](#142-as-a-design-objective-it-works--and-this-is-the-repair-133-asked-for) objective | subject against its own answer a **fifth below**, entering at `len/3` |
| PRNG | SplitMix64, inline. Seeds: `0x5EED` ([§13](#13-step-4-result-the-measure-is-valid-for-ranking-and-invalid-for-design)), `0xC0FFEE` ([§14.2](#142-as-a-design-objective-it-works--and-this-is-the-repair-133-asked-for)), `0xBEEF` ([§14.3](#143-what-23-is-and-is-not)) |
| random-contour trials | 400 for the single-subject figures, 60 per subject for the corpus tables |
| hill-climbing | 12–16 restarts, first improvement accepted, one note changed at a time |

### 15.4 How the samples were taken, including one that is not what it looks like

**The Renaissance sample is not the six composers it appears to be.** The code globs six directories, sorts by
path, and truncates at 200 files. Sorting by path means `Bus` and `Duf` come first, so the 200 are **70 Busnois,
37 Dufay and 93 Josquin** — Ockeghem, Obrecht and La Rue never enter. [§12.3](#123-experiment-3--the-renaissance-control-decisive-and-it-splits-the-three-refuted-rules)
originally described the sample by its directory list, which was wrong, and it has been corrected. The finding
itself is unaffected: 200 works of fifteenth-century polyphony, 299 613 slices, and the melodic rule fires 38 times
less often than in Bach. But a reader checking the claim would have looked for Ockeghem and not found him.

**The Bach corpus is Book I only** — 24 fugues, `wtc1f01`–`wtc1f24`. The submodule contains both books; Book II is
never read, because the ground-truth annotations cover Book I.

**The design tables cover 20 of the 24 subjects, not all of them.** Subjects longer than 24 notes are skipped for
search cost, which excludes exactly BWV 855, 860, 865 and 866 — the four densest, at 25, 31, 31 and 38 notes. They
are the four at the bottom of [§11.2](#112-the-ranking-on-the-strict-tier-and-it-agrees-with-musicians)'s ranking,
so the design experiments of [§13](#13-step-4-result-the-measure-is-valid-for-ranking-and-invalid-for-design) and
[§14](#14-23-built-harmony-repairs-the-design-objective-and-fails-as-a-rule) are run on a sample that omits the
least stretto-friendly quarter of the corpus. The ranking of
[§11](#11-step-3-result-a-ranking-and-the-rulebook-that-can-produce-it-is-not-the-one-bach-confirms) covers all 24.

**Timings are from one machine** and are reported only where they carry an argument — the ten-minute cutoff in
[§11](#11-step-3-result-a-ranking-and-the-rulebook-that-can-produce-it-is-not-the-one-bach-confirms) and the
saturation probe. They are not benchmarks and should not be read as any.

### 15.5 What is not reproducible from this repository

- **The literature.** [`literature/`](literature/) holds five PDFs under their publishers' terms and is not
  redistributable; [§7](#7-prior-art) gives DOIs so each can be obtained independently.
- **The Shostakovich half of the ground truth.** `fugues.ref` annotates 36 fugues; only Bach's 24 have scores here,
  because the Shostakovich scores are MIDI from a separate source that has not been fetched.
- **The functional-harmony layer.** `degree_of`, `progression_ok` and `is_cadence` are written and compiled but no
  reported number exercises them ([§14.3](#143-what-23-is-and-is-not)).

---

## 16. The harmonic layer does not survive validation

[§14](#14-23-built-harmony-repairs-the-design-objective-and-fails-as-a-rule) reported harmony as a working design
objective on the strength of numbers that were all self-referential: they measure whether my chord templates
explain notes, not whether the labels are right. Four external checks were run before building anything on it.
**All four come back negative**, and one of them says the analyser is not measuring what its name claims.

### 16.1 It finds Bach's cadences barely above chance

The ground truth has carried **106 typed cadence annotations** since
[§8](#8-roadmap) step 0 — Hepokoski–Darcy labels like `III:PAC`, `vi:DC`, `V:HC` — and nothing had ever read them.
They are the only external check available.

The first run scored 15% against a 12% baseline, which was **my test being wrong, not the analyser**: the label
names the *key* of the cadence, and only 39 of the 106 are in the home tonic. `III:PAC` is a perfect cadence in the
mediant and arrives on III. Corrected to resolve each Roman numeral against the mode, and to place a half cadence
on the local dominant and a deceptive one on its submediant:

| | |
|---|---:|
| cadences parsed | 106 of 106 |
| **arrival chord correct** | **40 (38%)** |
| and preceded by its dominant | 19 (18%) |
| chance rate for the same lookup | **23%** |

**38% against a 23% baseline.** The analyser carries real signal — 1.65× chance is not nothing — but it is wrong
about the arrival chord three times in five, and it confirms the dominant before it less than one time in five.
That is not something a realiser can be built on.

### 16.2 It fits *modal* polyphony better than tonal — so it is not measuring tonality

The prediction was straightforward and is the kind [§12.3](#123-experiment-3--the-renaissance-control-decisive-and-it-splits-the-three-refuted-rules)
made work: a vocabulary of triads, sevenths and functional progressions describes eighteenth-century tonal music,
so it should fit fifteenth-century modal polyphony **worse**. Same 200 Renaissance works, 188 201 notes:

| statistic | Renaissance | Bach | difference |
|---|---:|---:|---:|
| mean chord fit | **0.929** | 0.868 | **+0.061** |
| chord tones | **87.0%** | 80.0% | **+7.0** |
| explained, binary | 99.8% | 99.4% | +0.3 |
| untreated per 1000 notes | **2.3** | 5.5 | **−3.3** |

**It fits Josquin better than Bach, on every statistic.** The prediction is not merely unconfirmed, it is
falsified with the sign reversed.

The explanation is not flattering. Renaissance polyphony is more triadic and far less chromatic than the WTC, so a
detector that rewards notes for lying inside a triad will prefer it. **The analyser is measuring triadic
consonance, not tonal function** — and a consonance measure over simultaneous pitches is, to within a
psychoacoustic model, what [ricercar](ricercar/readme.md)'s roughness field was. Having spent fourteen sections
getting away from that, [§2.3](#23-harmony-is-a-second-automaton) arrived back at it from the other side.

### 16.3 Every number depends on a window nobody justified

The segmentation is at the notated beat, chosen because it seemed reasonable. Re-run at three other plausible
windows:

| window | fit | chord tones | explained | untreated/1000 |
|---|---:|---:|---:|---:|
| half beat | 0.926 | 88.4% | 99.7% | **3.1** |
| **beat** (as used) | 0.868 | 80.0% | 99.4% | **5.5** |
| half measure | 0.787 | 71.5% | 98.2% | **17.7** |
| measure | 0.709 | 64.2% | 96.6% | **34.3** |

**The untreated-dissonance rate varies eleven-fold** across windows any analyst might defend. That was the one
statistic [§14.1](#141-as-a-rule-it-fails--and-it-corrects-125s-reading) found separating Bach from arbitrary
placement, at 5.5 against 12.5 — and it is now clear the separation is a property of the window as much as of the
music.

Worse, the trend has a trivial cause: a shorter window contains fewer notes, and few notes always fit some chord.
In the limit of one note per segment the fit is exactly 1. **Mean chord fit is monotone in window fineness and
carries no information until the window is fixed on principled grounds**, which nothing here does.

This is the fifth free parameter in this project to turn out load-bearing after being chosen casually — after the
exclusive subject window ([§10.3](#103-a-defect-the-test-caught-in-the-reading-of-the-corpus)), shared offsets
([§11.5](#115-a-defect-the-measure-found-in-itself)), per-pair melody counting
([§9.3](#93-bach-found-two-bugs-in-the-rulebook-in-the-first-run)) and lyric spines
([§12.0](#120-a-parser-bug-found-on-the-way-which-corrected-everything-upstream)).

### 16.4 The graded objective is worse than the binary one

[§14.2](#142-as-a-design-objective-it-works--and-this-is-the-repair-133-asked-for)'s objective saturated: Bach
scored 1.000 and so did a monotone. Grading it — mean chord fit minus the untreated rate, both fractions of the
same denominator, so nothing is weighted against anything — was supposed to break the tie.

| | §13 contrapuntal | §14.2 binary | §16.4 graded |
|---|---:|---:|---:|
| Bach's subject | — | 1.000 | 0.9444 |
| **a monotone** | won outright | 1.000 | **1.0000** |
| Bach above random | +0.23 sd | **+0.91 sd** | +0.31 sd |
| optimiser beats Bach | 20 of 20 | **0 of 20** | 16 of 20 |
| Bach beats random, same rhythm | 5 of 20 | 17 of 20 | **18 of 20** |
| distinct degrees in an optimum | 1.0 | 6.8 | 6.8 |

**The monotone now beats Bach outright.** A single repeated pitch has perfect chord fit and no dissonances to
treat, so it scores exactly 1. Grading did not break the tie at the ceiling; it broke it in the wrong direction.

Two things survive. Optimised contours are still melodic — 6.8 distinct degrees, not [§13](#13-step-4-result-the-measure-is-valid-for-ranking-and-invalid-for-design)'s
monotone — and Bach still beats a random contour on his own rhythm 18 times in 20, the best figure yet. So the
harmonic family of objectives really does align with Bach's choices in the middle of its range. It is the top of
the range that is degenerate, and grading made that worse rather than better.

### 16.5 What this means for the next step

- **[§14](#14-23-built-harmony-repairs-the-design-objective-and-fails-as-a-rule)'s central claim is withdrawn.**
  Harmony is not a working design objective. It is better than the contrapuntal one — that comparison stands,
  and [§13.3](#133-why-and-it-is-structural-rather-than-accidental)'s diagnosis of *why* stands with it — but on
  its own terms it fails: wrong about cadences three times in five, better on modal music than tonal, and
  dependent on an unjustified window for the size of every effect it reports.
- **Step 5 must not be built on this.** A realiser fills voices against the prevailing harmony, and the prevailing
  harmony is currently identified correctly about 38% of the time.
- **The repair is a real harmonic analyser**, not a better statistic over this one. Chord segmentation is a solved
  problem with published baselines — the Roman-numeral-analysis literature, and Giraud's own group has a
  cadence-detection system with figures to compare against. Substituting a validated analyser is ordinary work and
  would let every §14 and §16 measurement be re-run unchanged.
- **The functional layer is still untested.** `progression_ok` was never exercised: with the arrival chord wrong
  three times in five there is nothing yet to test it against.
