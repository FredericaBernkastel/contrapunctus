# Changelog

The [readme](readme.md) states the current position. This states how it was reached — every implementation step in
the order it happened, the defects each one found, and the claims that did not survive the next experiment.

**Forward chronological**, against the usual convention, because each entry is the reason for the next and reading
the corrections before what they correct makes nonsense of both.

There are no version numbers yet and nothing here is released, so **each entry cites the commit that produced it**.
The hashes are the unit of history this project actually has: `git show 20cd760` is the whole of step 1. Entries
whose work landed together say so rather than pretending to a granularity the repository does not have.

The design document itself — readme §§1–7, the argument the implementation was written to test — predates all of
this, in [`1d5d518`](../../commit/1d5d518), [`9b36d70`](../../commit/9b36d70) and
[`9fbd67d`](../../commit/9fbd67d), which is why the log below starts at step 0.

---

## Step 0 — the corpora

[`7648ec0`](../../commit/7648ec0) [`568508a`](../../commit/568508a) — and, later than it should have been,
[`e162fb0`](../../commit/e162fb0): the Renaissance submodule was pinned only when experiment 3 needed it, three
steps after the table below claims all three.

Three submodules under `corpus/`, pinned so that a ranking is reproducible against a stated revision of the ground
truth.

| | source | pinned | licence |
|---|---|---|---|
| `algomus-data` | `gitlab.com/algomus.fr/algomus-data` | `a1801b5` | ODbL 1.0, contents DbCL 1.0 |
| `bach-wtc-fugues` | `github.com/humdrum-tools/bach-wtc-fugues` | `5095752` | Humdrum edition, David Huron |
| `jrp-scores` | `github.com/josquin-research-project/jrp-scores` | `52de715` | CC BY-NC 4.0 |

The annotations carry no notes — every label is an offset into a score held elsewhere — so the second submodule
followed. The **voice-separated** edition is the right one; `humdrum-tools/bach-wtc` is the two-staff keyboard
layout and would need voice separation first.

**Found on the way.** Huron's `!!!parts` agrees with the algomus voice count on all 24 fugues. BWV 867's subject,
transcribed by hand in `ricercar` from a score, checks out against the encoding note for note — but is the
**two-measure reading**, one side of a documented editorial dispute, chosen by accident. Huron also warns that in
BWV 867 "the alto and second-soprano parts exchange registers between measures 29 and 37", so spine index is not a
stable voice identity.

*Erratum in the corpus itself:* the dataset README, the `fugues.ref` header and the project page all give
Shostakovich as "op. 57, 1952". It is op. 87, 1950–51. The journal paper has it right.

---

## Step 1 — the two-voice automaton

[`20cd760`](../../commit/20cd760)

Built in `src/automaton.rs`, `src/pitch.rs`, `src/kern.rs`, `src/corpus.rs`.

**Result.** 513 reachable states against a crude product of 1280; all three verdict tests pass, including the two
`ricercar` could not state at all. Run as a checker over 24 fugues it stratifies its own rulebook: parallel perfect
consonances and direct motion to a perfect consonance on a downbeat occur about once per thousand slices; the
dissonance and melodic prohibitions fire two orders of magnitude more often.

**The first version had 40 reachable states.** The count grew to 513 when the debt split below was applied. The
state count is a property of how carefully the rules are stated, not a constant of counterpoint.

### Defects

- **A dissonance does not always resolve downward.** The first rule demanded a descending step from every
  dissonance. Only a *suspension* must descend; a passing note leaves by step in whichever direction it was going —
  Schottstaedt's third-species comment says "can be if passing either way". One wrong word produced **79% of all
  violations** in the first corpus run, which flagged Bach at 290 per thousand slices. Fixed by splitting the debt
  into `RESOLVE` (suspension, downward) and `LEAVE` (passing, either way).
- **Melody was counted per pair.** In a five-part fugue each voice belongs to four pairs, so every interval it sang
  was counted four times. Moved to a per-voice pass with its own denominator.
- **Roles were tracked instead of voices.** The pairwise walk kept the previous *lower* and *upper* pitch rather
  than each voice's own history, so at every voice crossing it measured melodic intervals between two different
  singers and corrupted the motion type parallel detection rests on. Found by reading, not by running.

---

## Step 2 — the compatibility table and the clique

[`c9f2c22`](../../commit/c9f2c22)

Built in `src/stretto.rs`.

**Result.** Bach's five-voice hyperstretto in BWV 867 — entries at `{0, 2, 4, 6, 8}` quarters — is **not** a clique
under the full five-rule tier and **is** one under the two-rule tier, on both contested readings of the subject.
A control on the written notes rather than idealised transpositions gives 15 violations on the full tier and 1 on
the two-rule tier, so the fault is the rulebook's rather than the template's.

### Defects

- **The subject window was exclusive.** The corpus defines a length as measured "between the offsets of the start
  and the end", where "the end of the pattern denotes the impact of the last note" — inclusive. The exclusive cut
  gave a five-note subject where `ricercar`'s independent hand transcription of the same span has six.
  Disagreeing with that transcription was the signal.

### Claims retired

- **A stretto is not a Sidon set.** An earlier reading of §3 called it "the same object as a Sidon set or a
  difference family". A Sidon set requires all pairwise differences to be *distinct*; the condition here is that
  they all land in the good set. Bach's own is an arithmetic progression, which is as far from Sidon as five points
  can be. The useful form of the correction: **densest strettos are expected to be regular, not clever.**
- **"Descending across five octaves"** — the five entry heads are `B♭4 F4 B♭3 F3 B♭2`, which descend **two**
  octaves; the whole texture spans a little over three. Five entries, not five octaves. Corrected when the entries
  were rendered to MIDI in step 5 and the pitches could be counted off the file.

---

## Step 3 — the corpus ranking

[`e162fb0`](../../commit/e162fb0) — one commit carries step 3, the five experiments and step 4, because none of
the three was worth reporting without the other two.

Built in `src/refdata.rs` — a reader for the `.ref` ground truth, including its fractional measure-offset grammar.

**Result.** Capacity by *clique size* does not converge under the two-rule tier: 81% of entry pairs are compatible
and the search returns whatever cap it is given. Only the strict five-rule tier — the one Bach violates — yields a
finite number. Under it BWV 849 ranks first of 24, which is the fugue musicians name when they name a stretto
fugue, and it survives a control for note density as the largest positive residual (r = −0.750 with notes per
quarter; BWV 849's residual +3.76).

**The structural finding**: the rules Bach never breaks are precisely the rules that almost never bind, so no
single tier both accepts his hyperstretto and discriminates between subjects.

Capacity also proved **non-monotonic in subject length** — BWV 854 goes 2 → 4 → 3 as the subject lengthens — and
the editorial choice of ending can halve it, so a single figure is not a well-behaved function of its input.

### Defects

- **Entries were allowed to share an offset.** The measure therefore counted harmonising the subject in parallel
  thirds as a five-voice stretto, and returned exactly whatever clique cap it was given for every subject in the
  corpus. A stretto is a succession of entries, not a chord of them. The tell was the suspiciously perfect
  agreement between the answer and the parameter.

---

## Five experiments, to resolve the tier deadlock

[`e162fb0`](../../commit/e162fb0)

### 12.0 — a parser defect that corrected everything upstream

Reading the Renaissance corpus exposed a fault present in every earlier result. **Vocal music interleaves `**text`
and `**silbe` lyric spines with the `**kern` ones**, and the reader pushed a spine only for `**kern` while indexing
data fields by position — so every note after the first lyric column was read from the wrong field. 60 of 200
Renaissance files failed outright and the 140 that parsed were a biased sample of the textless ones.

The same fault was in **Bach**: BWV 869's header carries an empty column that had been shifting its spines for the
entire project. Fixed by tracking every spine and marking which bear notes. All step 1 figures moved by under 4%
and no conclusion changed; BWV 869 entered the step 3 ranking, in last place.

### The five

| | experiment | outcome |
|---|---|---|
| 1 | **density** instead of clique size | **resolves the deadlock** — spread 0.321–0.956, BWV 849 first, r = −0.311 with note density |
| 2 | **Pareto** calibration against Bach's stretto | **degenerate** — the limit contains zeros |
| 3 | the same rulebook on **16th-century polyphony** | **decisive** — splits the three refuted rules |
| 4 | **chromaticism** against the melodic rule | **negative** — r = +0.249 |
| 5 | **harmony** as the binding constraint | supportive at the time; later withdrawn |

**Experiment 2's failure is worth keeping.** Bach's Stretto II contains no voice crossing and no unrecovered leap,
so the componentwise-maximum limit has zeros in it, and a zero in a domination limit is an absolute prohibition.
Capacity collapsed and BWV 867 scored 2 — the calibration could not reproduce the passage it was calibrated on.
**Calibrating a domination limit on the componentwise maximum of a single passage turns every criterion that
passage happens not to exhibit into a hard prohibition.**

**Experiment 3 split the three "refuted" rules.** Against 200 works of 15th-century polyphony:

| rule | Renaissance | Bach | ratio |
|---|---:|---:|---:|
| parallel perfect | 1.2 | 1.0 | — |
| direct to perfect on downbeat | 1.5 | 0.7 | — |
| **forbidden melodic interval** | **1.0** | **37.6** | **×38** |
| unprepared dissonance | 8.0 | 21.4 | ×2.7 |
| unresolved dissonance | 71.1 | 90.9 | ×1.3 |

The melodic rule is **not refuted; it was never about Bach.** It is a correct rule about Renaissance vocal writing
applied to eighteenth-century keyboard music. The dissonance rules fail in the very repertoire they were written
for, so they are implementation faults rather than a repertoire mismatch.

### Claims retired

- **"Three rules are refuted by Bach"** (step 1's language) becomes: one rule is repertoire-specific and correct
  where it belongs; two are wrong in both centuries.

---

## Step 4 — subject design

[`e162fb0`](../../commit/e162fb0)

**Result.** The measure is valid for ranking and invalid for design.

Optimising a pitch contour against density gives a **monotone** — density exactly 1.000, and the optimiser beat
Bach on 20 of 20 rhythms with a single repeated pitch. A static subject never moves, so `Motion::None` at every
slice, so neither surviving rule can fire. Constraining to ≥5 distinct degrees does not rescue it: the optimum
becomes jagged instead of static, mean melodic step 3.78.

Worse, **Bach's own contours score below random on their own rhythms** — 5 of 20, mean advantage −0.0763.

**Why, and it is structural.** Both surviving rules require a perfect consonance to fire, so maximising density
means minimising the perfect consonances a subject forms against its own transpositions. A fugal answer is at the
fifth. **The measure penalises the interval the form is built on.**

### Claims retired

- **"Step 4 is unblocked"** (experiment 1's conclusion). Density ranks subjects; it cannot design one.

---

## §2.3 — the harmonic layer, first attempt

[`7b633d4`](../../commit/7b633d4)

Built in `src/harmony.rs`: segment at the notated beat, score every root-and-quality candidate by duration-weighted
membership, classify every note against the chord that prevails where it sounds.

**Reported at the time:** as a rule it fails (Bach 99.4% explained, control 98.8%); as a design objective it works,
with every step-4 diagnostic reversing — Bach beats random 17 of 20, mean advantage +0.0552, optimised contours
carry 6.8 distinct degrees rather than 1.0, and the optimiser cannot beat Bach.

**This also corrected experiment 5.** Its 78% vs 56.9% gap was measuring texture complexity, not harmonic
correctness: three transpositions of one subject is a thin self-similar texture that trivially fits a triad.

---

## Validating the harmonic layer — four external checks, all negative

[`b0163d4`](../../commit/b0163d4)

Every figure above was self-referential: they measure whether the chord templates explain notes, not whether the
labels are right.

1. **Cadences.** The ground truth carries **106 typed cadence annotations**, unused until now — the only external
   check available. The analyser identifies the arrival chord **38%** of the time against a **23%** chance rate,
   and confirms the dominant before it 18% of the time.
2. **Modal control.** A tonal vocabulary should fit modal polyphony *worse*. It fits it **better** on every
   statistic — mean fit +0.061, chord tones +7.0 points, untreated −3.3.
3. **Segmentation.** The untreated-dissonance rate varies **elevenfold** across plausible windows (3.1 at the half
   beat, 5.5 at the beat, 17.7 at the half measure, 34.3 at the measure), and mean fit is monotone in window
   fineness because fewer notes always fit some chord.
4. **A graded objective**, meant to break the design objective's saturation at 1.000, made it worse: the monotone
   scores **1.0000** against Bach's 0.9444 and the optimiser beats Bach on 16 of 20.

**The first cadence test was itself wrong** — it checked against the home tonic, but the label names the *key* of
the cadence and only 39 of the 106 are in the home tonic. `III:PAC` arrives on III. Corrected by resolving each
Roman numeral against the mode, with half cadences on the local dominant and deceptive ones on its submediant.

### Claims retired

- **"Harmony is a working design objective."** It is better than the contrapuntal one, and the diagnosis of *why*
  stands; on its own terms it fails.

---

## The analyser rebuilt

[`6acc533`](../../commit/6acc533)

The repair is architectural. A fixed window **imposes** the harmonic rhythm; segmenting at every onset and charging
a penalty `λ` to change chord lets it **emerge**. Scoring also changed: foreign notes lose weight rather than
merely not gaining it, weight is duration doubled on beats, and the bass earns a bonus for being the root. Viterbi
is linear rather than quadratic in the chord vocabulary because the transition cost is zero to stay and `λ` to move.

**Result.** Cadence arrival accuracy **80%** against a 14% baseline at λ ≤ 0.1, and 70% at λ = 1.0 where the
harmonic rhythm is musically plausible; the old analyser managed 38% against 23%. Held out on odd- and
even-numbered fugues, the same λ is chosen either way and the accuracy transfers — 79% → 82%, 82% → 79%.

The modal gap narrows fourfold and keeps its sign, which now looks less like a defect in the analyser than a fact
about the music: Renaissance polyphony really is more triadic than the WTC, so any chord-fit statistic flatters it.

**The functional layer is vacuous.** `progression_ok` accepts root motion by nine of twelve intervals, so **75%
passes before any music is consulted**; Bach scores 80.4% and Renaissance 86.9%.

---

## Step 5 — realisation, and the first notes

[`e6e4af2`](../../commit/e6e4af2)

`src/realise.rs` builds §2.5's shortest path; `src/midi.rs` writes it out. Rhythm is given, pitch is the only
variable, and §2.3's harmony runs beside §2.2's counterpoint as a second obligation system over one grid. The two
compose with no special-casing, which was the part of §2.3 still owed.

### Defects of correctness

1. **The generator and the checker computed a slice differently.** They now share `corpus::pair_sym`, which exists
   only to make that impossible. A generator whose lo/hi role assignment drifts from its own checker's can emit
   counterpoint the checker then flags, and at that point neither number means anything. A test asserts the fill
   passes the checker; a second checks the count of legal fills against brute-force enumeration.
2. **The soft-criterion report called the checker with the voices swapped.** `crossed` is *"the higher-indexed voice
   sounds below the lower"*, so the report was the exact negation of voice crossing — visible because a fill that
   scored zero on crossing appeared to cross at every slice. The search itself was right; only the report was wrong.
3. **The chance baseline did not move with the condition it was the baseline for.** It was computed against the
   full texture's harmonic analysis in every row, including the rows run with no plan at all, so a search choosing
   under one set of constraints was being scored against a baseline built from another. It now uses whatever plan
   the search used. The correction changed the reading of the table: several `chance` figures moved by six points
   and one row crossed from above the baseline to below it.
4. **The domain offered two spellings of one sound.** In C-sharp major the chord respelling produced `E##` beside
   `F#` and `B##` beside `C#`, doubling the branching factor and putting double sharps in the output. One spelling
   per sounding pitch is kept, the key's own preferred. This is not a retreat to semitones: alternatives merge only
   when they sound alike *and* one is the key's own, so §2.1's augmented fourth and diminished fifth stay distinct.

### Defects of cost

The corpus run went from not finishing to eight minutes, in four steps, none of which changed a single reported
figure. Recorded because each was invisible until measured and each was worth an order of magnitude.

1. **The product was taken first.** Enumerating the joint assignment of all free voices and then testing it. A free
   voice's melody, its harmony, and every pair it forms with a *fixed* voice depend on its own note alone, so those
   are decided per voice and only the free-against-free pairs need the product.
2. **`automaton::step` allocated a `Vec` for its fired-rule list on every call** — several million calls per layer.
   `step_into` writes into a caller-owned buffer; `step` is now a wrapper over it.
3. **The candidate list was collected into a `Vec` per state per voice per slice**, and the option counts into
   another. Both are now iterated by index.
4. **The analyser rescanned every note for each of 108 chord candidates.** Hoisting the per-segment weights out of
   the chord loop left every published figure of §8.5 identical and took the λ sweep from ten minutes to 1.7
   seconds. It had been that way since the analyser was rebuilt.

Finished layers also kept their hash index, which is dead once the next layer is built. That one was a guess at the
cause and turned out not to be it — the process never exceeded 15 MB — but the change is right regardless.

### What the measurement found

Reconstructing Bach's free voices from his own entries, his rhythm, and a plan built only from the voices the search
cannot see: `10¹²` to `10¹⁸` legal fills per three-bar span, and **agreement that does not respond to the rulebook**.
Tightening the hard tier from two rules to five moves the spans the exact search can finish from 83 to 108 of 117
and the legal fills down by five orders of magnitude, while agreement moves from 6.9% to 7.0%. §5's inverted failure
mode, arriving where it matters and quantified. The full table is in readme §8.6.

**The reversed-objective control is the part that was not anticipated.** Running the identical search minimising and
then maximising the soft criteria gives 7.8% and 4.9% against a random-legal baseline of 16.2%. So the criteria are
not noise — the sign is right — and they are not usable as an objective either, because *both* extremes lose to
picking at random from the legal set. Taking the extremum of a nearly orthogonal objective lands in an atypical
corner. No choice of weights repairs that, which is a sharper objection than §5's and lands in the same place.

Two further findings. **The melodic rule is repertoire-specific as a description and load-bearing as a constraint** —
§8.2 stratified it out of the hard tier, and without it nothing bounds a free voice's line at all. And **§2.7's wall
is at two free voices rather than four**, because the multiplier is the compounding obligation state and not the
product of pitch domains, which §2.7 had assumed.

### The first listening test, and what it narrows

Reported by one listener on `fill.mid` against `fill-bach.mid`, unblinded: the filled inner voice covers a smaller
range than Bach's — confirming the register finding by ear — but *"overall the result is on par, nothing is better
or worse than Bach himself."*

Both halves are consistent with the table and the second half narrows what the table licenses. `exact` measures
identity with Bach, which is a proxy for quality only under the assumption that Bach's notes are the unique good
answer; the median span admits about `10¹⁵` legal fills, so a low agreement rate can be a fact about how many
acceptable answers exist rather than about the quality of the chosen one. **The claim §8.6 supports is that the
rulebook plus a plan writes acceptable counterpoint and not Bach's**, which moves step 6's open problem from
quality to stylistic identity.

Recorded with its limits rather than as a result: one listener, one six-second passage, no blinding, a flat MIDI
piano, and one of the three voices identical between the two files.

### Two defects in the output files, both found in a DAW rather than here

- **Tracks were written in `**kern` spine order and named by that index**, so the top voice of three arrived as
  `voice 2`, and `stretto.mid`'s `entry 1` was `stretto-bach.mid`'s `voice 4` with nothing saying so. Files are now
  written top voice first, ordered by *measured* mean pitch, each track named with its position, compass and role.
- **The division was 240 ticks per quarter** — legal, uncommon, and reported back by a DAW at exactly half length,
  the signature of a host substituting an assumed timebase. Now 960, an exact `×4` of the internal lattice, with a
  **time signature** taken from the score's own `*M` interpretation rather than assumed. Two tests assert the
  timebase stays a whole multiple of the lattice, since a fractional one would round every onset in every file and
  §2.1 exists to prevent exactly that.

---

## §7.1 — the prior art that was found last, from outside the field

[`6b57e3d`](../../commit/6b57e3d) [`35d9b13`](../../commit/35d9b13) [`e49c2dd`](../../commit/e49c2dd) — written,
wired into the roadmap, then moved inside §7.

Readme §7.1. **WaveFunctionCollapse** (Gumin, 2016) is texture synthesis for images and is the same object as this
document in a different category — constraint propagation over a discrete lattice with local compatibility rules,
arrived at independently and from the opposite direction. Its own README states §2.2's bounded-order automaton for
pixels: *"the overlapping model relates to the simple tiled model the same way higher order Markov chains relate to
order one Markov chains."*

Two things came out of the comparison that were not visible from inside the project.

- **C1 is a whitelist and Fux is a blacklist.** WFC admits only configurations it has seen; this project forbids
  five things and admits the rest. That difference is the whole of §8.6's `10¹⁵` legal fills, and it inverts the
  failure mode — WFC's practical problem is contradictions, and this project has never once failed for being
  over-constrained. The unfitted route to a whitelist is already in the source: species counterpoint *is* an
  enumeration of permitted figures, and only the prohibitions were transcribed.
- **Weak C2 answers §8.6's question by not forming an objective.** WFC samples proportionally to pattern frequency
  — typical rather than optimal — and §8.6's own control already showed typical beating extremal by a factor of
  two. The unfitted version, uniform sampling from the legal set, is mostly built: the exact path counts are
  already computed and already checked against enumeration.

Written first as a standalone top-level section and then moved inside §7, where it belongs: its seven sources merged into that
section's table rather than standing in a second one making the same Crossref claim, which turned up a stale count
— three citations now carry no DOI, not two.

---

## Step 6 — sampling the legal set, and a claim retracted

[`f171273`](../../commit/f171273)

Readme §8.6, §9 step 6. The first of the two proposals §7.1 produced, built and measured.

**Uniform sampling from the DAG.** The search already counts the paths through itself, so drawing one uniformly is
a backward walk: choose a final node in proportion to its count, then take each predecessor in proportion to *its*
count. The factors telescope — `count[j]` is by construction the sum over j's predecessors — so every complete fill
comes out with probability exactly `1/total`. Only the edge list is new, and only when a sample is asked for;
`(from, to)` suffices, because a node's key *is* the pitches the free voices took.

Verified on an instance small enough to enumerate: 116 legal fills, 20 000 seeded draws, **all 116 drawn**, and
chi-squared **113 against 115 degrees of freedom**, which is where a flat distribution is expected to land. Two
further tests assert that every sampled fill passes the checker and that more than one distinct fill comes back.

### Claims retired

- **"Optimising this objective is worse than not optimising."** Refuted by the control that sentence was waiting
  for. Minimising the soft criteria scores 7.8%, a uniform draw **6.9%**, maximising 4.9%. Sampling was supposed to
  win — §7.1 read WaveFunctionCollapse's Weak C2 as saying *typical beats extremal* — and it loses. The soft tier
  is weak, real, and **better than not optimising**, which is the opposite of what §8.6 concluded from a baseline
  that was not like-for-like.
- **The `16.2%` baseline as a figure any generator could approach.** It is computed per note with Bach's own
  preceding note in hand. A generator that lives with its own mistakes gets **6.9%**, so more than half of that
  baseline was the handout. 16.2% still measures something real — how much the constraint determines when the
  previous note is *given* — but that is a ceiling on a different problem.

### Defect

- **The §7.1 entry above overwrote the `## Recurring pattern` heading** when it was added in
  [`6b57e3d`](../../commit/6b57e3d), leaving that section's body orphaned under it for three commits. A scripted
  replacement matched `--- ## Recurring pattern` and substituted rather than inserting before it. Restored here.
  Nothing was lost, and nothing said so either — which is the argument for the heading check the repair added.

---

## Step 6b — a treatise weighting, measured against two corpora and rolled back

[`2d8cd4a`](../../commit/2d8cd4a)

Readme §8.6. §7.1 asked whether WaveFunctionCollapse's **Weak C2** could be had without a corpus. Fux supplies the
directions — the six soft criteria are the things he says to avoid — and no magnitudes, so the unfitted form is a
single swept inverse temperature `β`, drawing each fill in proportion to `exp(−β × soft cost)`. `β = 0` is the
uniform draw and `β → ∞` the cheapest fill, so the sweep interpolates between two figures §8.6 already reports.

Tested on §8.2's instrument: the same measurement on two corpora three centuries apart, one annotation-free
protocol for both. **The decision rule was fixed and printed before the numbers** — keep only if a single `β` beats
the uniform draw on both corpora by more than twice the standard error of the paired per-span difference.

**Result: repertoire-specific, and rolled back.** At `β = 1` Bach gains `+1.33 ± 0.62` points and 15th-century
polyphony *loses* `−1.04 ± 0.40`, worsening monotonically to `−2.68 ± 0.52` at `β = 4`. Production call sites pass
`0.0`; the parameter survives so the table stays reproducible, per §10.2's standard for superseded results.

### Defect, in the test rather than the code

**The first run said `GENERAL`.** On 60 works and 112 Renaissance spans the gain came out `+0.6` points and the
verdict printed accordingly. Five times the data reversed the sign and made it significant. The code was right; the
decision rule said *"improves on both"* and never said *by how much, against what noise*, so a figure well inside
its own error bar was allowed to decide. Fixed by making the span the unit of replication — eight draws sharing one
span's fixed voices and plan are not eight independent observations — and by requiring two standard errors on a
**paired** per-span difference.

### A limit on what was shown

Fux describes Palestrina, so a weighting transcribed from Fux ought to fit 16th-century vocal polyphony better than
Bach, and it does the reverse. The control is **70 Busnois, 37 Dufay and 93 Josquin** (§10.4) — 15th-century music,
a century *before* Fux's subject, where open fifths and octaves are idiomatic and equal-range voices cross
constantly, and three of the six criteria penalise exactly those. So the demonstrated failure is to span 1450–1722.
**Fux's own repertoire sits between the two and is untested**, and testing it needs a Palestrina corpus this project
does not have. That does not rescue the weighting, which was asked to generalise and did not.

---

## Step 6c — the species as a whitelist, checked and not adopted

[`da12028`](../../commit/da12028)

Readme §8.7. §9 step 6's other proposal: Fux's book is a whitelist and was transcribed here as a blacklist, so
transcribe the enumeration instead — first species consonance, second the passing tone, third the neighbour, fourth
the suspension. `src/species.rs` is that and nothing else.

**Run as a checker before being used as a constraint**, on §8.2's instrument. At its most generous reading the four
figures cannot account for **23% of Bach's dissonances and 18% of the Renaissance's**, rejecting 54.3 and 23.5
slices per thousand against the 21.4/90.9 and 8.0/71.1 of the two rules it was written to replace — the same band,
not an improvement. Not adopted. The failure is *even* across the two corpora, so unlike the melodic rule this is an
enumeration that is incomplete rather than one belonging to a repertoire.

### What it found instead

- **The perfect fourth is a large classification artefact.** Counting it as a consonance removes **31% of Bach's
  flagged dissonances and 44% of the Renaissance's**. `pitch.rs` calls it a dissonance — the classical two-voice
  position, and its own comment warns what that costs — but in three parts a fourth over a supporting bass is a
  consonance and only a fourth against the bass is not, which a pairwise walk cannot see. §8.2 has called the two
  dissonance rules implementation faults since step 1 without a diagnosis; this is a candidate, and it is testable:
  resolve the fourth against the lowest sounding voice rather than pairwise.
- **Fux's metric condition costs fourteen points in both centuries** — 76.0% → 61.9% in Bach, 74.8% → 59.3% in the
  Renaissance. Real counterpoint strikes dissonances on the beat far more often than the species permit.
- **The residue is seconds and sevenths**, the intervals a chord explains rather than a melodic figure — §2.3's
  claim arriving from the other side.

---

## Cross-references made checkable

[`75f7978`](../../commit/75f7978)

`tests/references.rs`. Section references had been going stale silently, and the reason they were invisible is
that the *anchors* were checked all along: a link to `#86-realisation-and-the-first-notes` was verified to resolve,
and nothing compared it with the `§8.6` a reader actually sees. Doc comments in `src/` had no check of any kind,
and there are 160 references in them.

Four checks, all mechanical, all `cargo test`: numbered headings form a contiguous sequence; `Contents` lists each
exactly once with the right anchor; every link resolves, every relative path exists, and where a link reads `[§N]`
the heading it points at **is** section N; and every bare `§N` across `readme.md`, `CHANGELOG.md` and `src/**.rs`
names a section that exists. References to `ricercar`'s sections resolve against that document instead, identified
without guesswork — either the link target names the file, or the word immediately before is `ricercar`.

**The first run found 53 stale references** in a repository that had just been tidied by hand. Four links in the
readme read `§8` while pointing at `#9-roadmap`; the rest were doc comments and command banners still naming the
`9`–`17` numbering that the restructure folded into §8 — `12.5`, `13.3`, `16.2`, `17.4` and so on. Live
references were remapped; the ones naming superseded experiments now say what the experiment was, since the
numbering they used no longer exists anywhere.

Each of the four checks was then verified to fail on a deliberately broken copy, because a linter that has never
failed is indistinguishable from one that cannot.

### The alternative that was not taken

Stable identifiers — permanent slugs referenced instead of numbers — would prevent the rot rather than detect it,
at the cost of the numbers themselves, which carry a reader's sense of where in the argument a reference points
and which this document's voice is built on. Detection is cheaper and loses nothing. What remains true is that
**renumbering is the operation that breaks references**, so it stays worth avoiding; the checker turns a silent
breakage into a failing build rather than making renumbering free.

---

## Step 6d — a criterion that is not local

[`8f90d28`](../../commit/8f90d28)

Readme §8.8. §8.6 says the octave is what goes wrong and §2.5 says a running range would multiply the search state
by a few hundred, so the criterion is applied **after** the search: rank 32 uniform draws per span by a criterion
over the whole line. Three, each transcribed from Fux and each reported alone because combining needs weights —
one climax, a compass inside a tenth, variety.

**Nothing clears the bar on both corpora, so nothing is adopted.** But climax and variety are worth `+2.21 ± 0.82`
and `+1.99 ± 0.80` to 15th-century polyphony and are indistinguishable from zero in Bach, which is the **reverse**
of step 6b's weighting and the second independent measurement of §8.2's melodic finding: Fux's melodic doctrine is
Renaissance doctrine.

**The compass criterion costs the Renaissance `−2.51 ± 0.61`.** Fux's rule is an upper bound and §8.6's failure is
narrowness — the fill covers `F3..C4` where Bach covers `F3..A♭4` — so ranking by it selects for exactly the fault
being treated.

## Step 6e — a better plan, and the largest effect this project has measured

[`b3668d5`](../../commit/b3668d5)

Readme §8.9, `src/plan.rs`. Step 6's fourth proposal began with a number already on the table — §8.6's `leaky` row
is three points above its `clean` row — and two faults in how that number was being read. The rows are **not
paired**: a plan that solves one span refuses another, so `9.3%` against `7.8%` compares two sets of notes. And the
oracle is **not a plan any grammar could emit**, since §2.4's productions cannot name a chord per onset when the
onsets belong to the notes being asked for. Nine plans, both corpora, §8.8's windows, every gain a paired per-span
difference.

**The plan §8.6 has been writing against names the right chord one time in six.** 16% of the span in Bach, 20% in
the Renaissance, against the analyser's own full-texture reading. §8.5 measured 70–80% on cadence arrivals *with
the whole texture in front of it*; from one or two voices out of three or four it is mostly guessing. Nobody had
measured this. In the Renaissance the plan is worth **nothing at all** — `none` and `clean` differ by
`+0.47 ± 0.70`.

**A correct plan is worth `+2.36` in Bach and `+3.74` in the Renaissance** — the only condition in the whole of
step 6 to move more than a point in the same direction in both centuries. It is a **ceiling, not a candidate**:
neither honest repair helps. Retuning `λ` for the thinner texture loses on both. Gating the plan on the analyser's
own `fit` does nothing, because on a thin texture confidence is **uncorrelated with correctness** — high
confidence on 95% of a span whose chords are 15% right — which closes every repair built on that statistic rather
than one.

**The gain survives coarsening to a beat and not to a bar.** `oracle/beat` keeps `+1.73` and `+3.77`; `oracle/bar`
keeps `+2.77` in the Renaissance but falls to `+0.88 ± 0.46` in Bach, inside two standard errors. So a form
grammar has to schedule harmony **per beat** — the first quantitative requirement step 7 has been handed from
outside itself.

**And `λ = 0` removes four orders of magnitude of legal fills for `−0.32 ± 0.42`.** It is tight and it is wrong:
4% right against `clean`'s 16%. Set beside an oracle that admits *twenty times more* fills than the plan it beats
by two points:

> **Neither tightness nor looseness predicts agreement. Correctness does.**

Which is §8.6's thesis with the confound removed. That section watched constraint raise `chance` without raising
`exact`; this watches a *correct* constraint raise `exact` by two to four points while making the legal set
**larger**. Constraint was never the variable.

The price is quotable: **0.024 points of note agreement per point of chord agreement in Bach, 0.045 in the
Renaissance**, near enough constant across all three ceilings. A perfect analyser is 84 points above the present
one, so `+2.4` is the entire envelope for this lever.

## Step 6f — replacing the soft tier, and a claim withdrawn

[`be14082`](../../commit/be14082)

Readme §8.10, `Problem::prescribe`. Step 6's last proposal points at **Marpurg** and **Kirnberger** and
`literature/` holds neither, so what was asked is what can be asked without them: **is the tier one criterion or
six**, and **does saying the same thing positively do better?** Six one-hot ablations for the first; three positive
criteria — move by step, move against, state the harmony — charged *in place of* the tier for the second, with
`weights` at zero so it is a replacement and not a seventh prohibition. Both corpora, §8.8's windows, paired per
span. A prescription reorders the legal set and never prunes it, which a test asserts, so every row searches the
same graph and `done` is constant.

**The narrowness §8.6 and §8.8 laid at the rulebook's door is the shortest path's tie-break.** With no criterion at
all the search returns a line of mean melodic interval **0.76** scale steps and compass **1.91** over the whole
span, against the composer's **1.66** and **6.94** on the same voices, and it scores **1.3%**. When nothing
distinguishes the paths the search keeps the first one it found, and the first one barely moves. That is a fact
about §2.5's search, not about Fux.

**The tier is six, not one.** Read against that tie-break control, all six together are worth `+4.8` in Bach; the
best single prohibition is worth `+2.6`; four of the six are worth under a point.

**Every prescription collapses onto a degenerate optimum.** `move by step` is satisfied *perfectly* by oscillating
between two adjacent notes — it got mean interval `1.04` and left the compass at `3.55`. `state the harmony` is
satisfied by holding one chord tone (`0.91` in Bach, `0.31` in the Renaissance). `move against` leaves oblique
motion free, so never moving is optimal.

> **A prohibition composes safely under a minimiser and a prescription does not.** Not doing something is what a
> search does by default; doing something has a cheapest way to be done, and the minimiser finds that instead.

Which explains the six: `repeated note` charges the degenerate solution the other five would take. The tier is a
set of **mutually blocking degeneracies**, so no subset works and three prescriptions cannot stand in.

**`move against` turned out to be `direct motion` restated** — the two rows are identical to every printed digit on
both corpora, a free cross-implementation check, and a reminder that the distinction is not the sign of the
sentence but whether the criterion has a cheap way to be satisfied.

**And the tier reproduces the composer's melodic statistics without reproducing the composer's notes**: compass
`6.97` against `6.94`, mean interval `1.45` against `1.66`, and the same closeness on three protocols and two
centuries — while a uniform draw that misses both by nearly a factor of two scores *higher* on note agreement.

### Withdrawn: "using it is better than ignoring it"

§8.6 read: *"Minimising the soft criteria beats maximising them, and beats not optimising at all. The soft tier is
weak and it is real, and using it is better than ignoring it."* The first clause holds — minimising beats
maximising by `3.00 ± 1.01`. The second is withdrawn.

Re-run on §8.6's own spans with both accountings side by side, the pooled column reproduces §8.6's table exactly —
`6.9%`, `7.8%`, `4.9%` — and the paired column puts the uniform draw at **`−0.74 ± 0.97`**, which is nothing.
Pooling weights a span by its note count and counts each of the eight draws' notes separately, and that is where
the claim came from. On the window sample, five times the size, the same paired comparison runs the other way and
clears the bar on both corpora: **`+1.07 ± 0.31` in Bach and `+4.64 ± 0.61` in the Renaissance.**

So step 6's *first* proposal is reinstated. Uniform sampling was ruled out on `6.9%` against `7.8%` — the pooled
figures. Paired, **drawing beats optimising**, and it is the only change step 6 has produced that clears its own
decision rule. One caveat, which the tie-break row supplies: the objective must be dropped *by drawing*, not by
zeroing the weights, since to a shortest path "no objective" means every path ties and the first wins, at `1.3%`.

**Step 6 is closed.** Five proposals, four failures, and one adoption that arrived by retracting the measurement
that had ruled it out.

### What step 6 established

Six experiments. Five failures against the rulebook, one measurement that moved the problem to another step, and
one adoption:

> **The treatise is a list of prohibitions against excess. The generator's failure is deficiency.**

Removing the objective changed nothing, because the objective was never binding. Weighting the prohibitions harder
split by repertoire. Enumerating the permitted figures left a fifth of real dissonance unexplained. A shape
criterion from the same source pushes the wrong way. Nothing transcribed from Fux tells a generator what a line
should *do* — the book assumes a writer who knows, and constrains what they must not. That is an argument for
reading **Marpurg** and **Kirnberger**, who write about Bach's practice, and it is the first time this project has
had a measured reason to prefer one unread treatise over another.

The fifth says where the missing selectivity actually is, and it is not in a rulebook of any kind. It is the
harmony under the line, it is worth two to four points on both corpora, and the instrument that supplies it is not
a better analyser — the analyser cannot see it from the fixed voices and cannot tell when it is guessing — but
§2.4's grammar, which never has to infer the harmony because it decides it. **Step 6's open problem is answered by
step 7.**

The sixth says the rest of it out loud. The generator's narrowness was never the rulebook's doing but the
tie-break's; the soft tier is six mutually blocking degeneracies and no smaller thing replaces it; a prescription
cannot be minimised because it has a cheapest way to be satisfied; and the tier is not worth applying at all. What
step 6 leaves behind is one line of code turned off, one turned on, and a much better map of where not to look:

> **Do not optimise. Draw. And put the effort into the harmony rather than the rules.**

---

## A command line, and §10.3 made settable

[`bb3d9b3`](../../commit/bb3d9b3)

`src/cli.rs`, the repository's first dependency. Dispatch was a `match` on `argv[1]` whose fallback arm ran the
four quick measurements, so **a mistyped command came back as a measurement of something else** — `specie` for
`species` printed the automaton's state count and exited zero. It is an error with a suggestion now.

**§10.3 was a table of constants.** `λ`, the draw counts, the windows per fugue, how many works are read, the
tier, the seed, where the corpora live and where MIDI goes: every one had to be edited and recompiled to vary, and
one of them was edited and recompiled to vary during step 6d. They are flags, and the module's stated contract is
that **the defaults are exactly the published runs** — a test asserts each default equals the value §10.3 names.

Restoring that contract caught one thing. §8.6's treatise-weighting table takes **three** windows per fugue and
everything from §8.8 on takes **thirty**; a single flag for both would have silently rewritten §8.6's `67 Bach
spans` into 690. They are separate flags, and `gen` prints `67 spans from 24 fugues` again. Spot-checked against
the published figures: 513 reachable states, the λ sweep's `80 / 77 / 70 / 48`, and the stretto's `2 on the full
tier, 0 on the confirmed tier`.

**§10.2 is now checked against the program.** `list` prints the section-to-command map from the same data `--help`
is built from, and a fifth reference test runs it and fails the build if the table names a command that does not
exist, files one under the wrong section, or omits one that produces a reported figure. Verified against all three
failures rather than assumed. Commands were renamed to say what they produce — `exp3` is `renaissance`, `r2` is
`reconstruct` — and every short name the readme or this file has ever cited still works as an alias, because a
citation that cannot be run is not a citation.

`list` also separates the twenty commands that produce a reported figure from the thirteen that reproduce a
superseded one and the seven batches, which the flat list never did.

---

## Marpurg's tonal answer — the first transcribed rule Bach does not break

[`731a932`](../../commit/731a932)

Readme §8.11, `src/answer.rs`. The first thing transcribed from a treatise of **Bach's own circle** rather than
from Fux, which is §9's standing open problem: the *Abhandlung von der Fuge* (1753), drittes Hauptstück, "Vom
Gefährten".

Two Grundsätze that conflict — keep the subject's intervals, and stay in the key — because the octave has two
unequal halves, five notes up to the dominant and four back. Marpurg resolves it by a *Vertauschung* he tabulates
as a substitution of melodic intervals: a unison for a second, a second for a third, and the reverse. **One
interval changes, by one degree.** Transposing one note up a fifth and the next up a fourth does exactly that, so a
single change of leg along the subject is a single mutation, and that is the model.

**Where the mutation falls, Marpurg settles by worked example rather than by rule, so the transcription does not
pick a point** — it enumerates every point the stated rules leave open and returns a set. §8.7's question then
applies unchanged, and the set's size is reported beside its coverage.

**Rule I is exact.** Seven WTC subjects open on the dominant; in all seven Bach answers on the tonic, which
transposing up a fifth does not do. **Seven of seven, zero exceptions** — against §8.2, where Fux's rules were
measured at 8.0, 21.4, 71.1 and 90.9 violations per thousand. The right-hand column is the whole measurement: where
a rule says *answer at the fifth* it says what transposition does anyway, and only *answer at the fourth* earns
anything.

**Rule II is not weak but wrong** — 0 of 4 where it says anything, and its 66.7% overall is entirely the free
cases. §3.3 would object that the subject's *end* is a reading rather than a fact, so the rule was retried at every
subject end the ground truth records. It fails at all of them.

**And Marpurg knew which was which.** He states Rule I flatly and hedges Rule II in the sentence that states it —
*"öfters nach Beschaffenheit der Umstände ihre Ausnahmen leiden kann"*. The treatise's own confidence tracks the
measurement rule for rule, which Fux's never did: the *Gradus* asserts its melodic prohibition exactly as firmly as
its parallel-fifth one, and §8.2 had to find the stratification from outside.

**Applying Rule II as a filter costs more than it buys.** It gains three tonal answers and refuses four Bach wrote
as plain transpositions (BWV 848, 854, 863, 865), leaving coverage exactly where transposition already was —
41.7%. Dropping it takes coverage to **62.5%** with a median of 14 admissible answers per subject.

So the transcription narrows the answer from the space of transpositions to a **shortlist of about fourteen**, and
then stops, because the chapter stops. Against §8.6's `10¹²` fills of three bars, that is the first time a
transcribed rulebook in this project has narrowed anything to a number a person could read.

Not tested on two corpora and cannot be: the tonal answer is a device of tonal fugue and the 15th-century control
has neither the annotations nor the exposition. Twenty-four pairs is a small sample; what makes Rule I worth
reporting is that it is seven out of seven in the one place it makes a claim transposition does not.

---

## Before step 7 — five things checked first

Step 6 spent five experiments discovering that legality is not preference, and every one of them was checking
something that had been assumed for several steps. These five were checked before step 7 assumed them.

### The endorsed configuration, and every entry against the subject

[`7f38718`](../../commit/7f38718)

§8.10 concluded the objective should be dropped and the generator should draw, and the code had no way to say so.
Saying it naively is a trap: zeroing the weights is not dropping the objective, since to a shortest path every path
then ties and the first found wins — the 1.3% row. `Problem::drawing()` zeroes the weights **and** asks for a draw;
`Solution::chosen()` returns the draw rather than the tied path; a test asserts the two differ.

And §8.11 showed plain diatonic transposition is wrong for the comes, while §8.3 and §8.4 place every entry that
way. All **232** annotated entries against the first: **78.9%** exact transpositions, 2.2% tonal answers, 19.0%
neither. Tonal answers are five entries in 232 because mutation belongs to the exposition's comes — so §8.3's
clique test is measuring something real, and §8.4's design objective is the exposed one.

### The fourth against the bass — the oldest lead, followed and closed

[`b176c50`](../../commit/b176c50)

§8.7 left one lead on §9's oldest open problem, and §8.11 supplied a second and independent reason to follow it:
Marpurg's invertible-counterpoint chapter says the fifth must be handled as a dissonance *because inversion turns
it into a fourth*.

`corpus::Fourth` adds the scope to the existing checker rather than beside it. The `pairwise` rows reproduce §8.2's
21.4/90.9 and 8.0/71.1 exactly. Judging the fourth against the bass removes **9%** of what the two rules flag in
Bach and **23%** in the Renaissance, leaving them at 102.6 and 61.2 per thousand. Only **38%** of Bach's flagged
fourths have a voice below them.

**The lead is answered and it is not the answer.** The correction is right and is left off, the two rules being
outside the endorsed tier already and 9% changing no verdict. What it establishes is that the replacement §9 wants
has to explain the other ninety per cent.

### Are episodes sequences

[`e1b1a3b`](../../commit/e1b1a3b)

§2.4's one production that claims something about the music. Episodes score **13.3%** sequential coverage against
entry spans' **1.3%** — `+12.0 ± 2.0`, six standard errors — so the production is directionally right. And
**70.8%** of episodes contain no strict sequence at all, so it is a tendency with a rate and not a rewrite rule.

The incidental finding is the larger one. **Episodes are 54% of the book by duration**, 154 of them, median three
bars. The realiser fills free voices against a *held entry*, and in most of the music there is no entry to hold.

### Key-finding

[`756c943`](../../commit/756c943)

§9's second open problem. Viterbi over 24 keys by bar, by §8.5's instrument, with minor carrying its raised seventh
because a collection cannot otherwise be told from its relative major.

**The validation was already in the repository.** The 106 typed cadences §8.5 used carry roman numerals, and a
roman numeral names the local key. All 106 parse.

Naming the piece's own key everywhere scores **30.2%** for free, so the measurement is the **74** cadences
elsewhere. At `μ = 0.25`: 45.3% overall, 56.6% forgiving the mode, **35.1%** on the modulations, key rhythm 6.4
bars. `μ = 0` scores the same overall at a key rhythm of 1.7 bars, which is not a key rhythm. **It works and it is
weak** — enough to catch a key plan that wanders somewhere Bach never goes, not enough to referee between two
plausible ones.

Two pitch-class masks were wrong when first written, and a test that spells the collections out caught both. It is
in the file.

### What the five leave step 7

The grammar's `Exposition` is backed by §8.11 and its `Episode` production is a tendency rather than a rule. The
key plan can be checked coarsely and not finely. The objective is off and the generator draws. And the largest
single fact about fugal form this project has is that **more than half of a fugue has no subject sounding in it**,
which is not the problem §8.6 solved.

---

## Step 7 — the grammar, parsed against the book

[`c815f94`](../../commit/c815f94)

Readme §8.15, `src/form.rs`. §2.4 writes ten lines of productions and asserts that form is a grammar. §8.13 checked
one of them; this checks the rest, in the only way a grammar can be checked — **does it derive the sentences it
claims to be a grammar of?** What is parsed is the **plan** rather than the notes: the annotated entries with their
voices and degrees, the typed cadences, and the length, which is exactly what the non-terminals range over and what
a form grammar would have to emit.

**It derives 3 fugues of 22, and the failure is in one production.**

**What holds, holds completely.** All 22 end with their last annotated cadence in the home key — a production that
never fails once. Twenty-one of 22 have a middle. `Middle+` runs 0 to 9 with a median of 3, `Stretto?` is taken by
5, and the episodes between entry groups have a median of **3.0 bars** — the same median §8.13 reached by an
unrelated route, which is the cross-check that the two sections measure one object.

**`Exposition` is wrong on every count**, and it is both the only production with any detail in it and the one
§8.11 was written to serve. 59.1% state the subject once in each voice, 40.9% alternate, and **82% contain an
episode**, for which the production has no symbol at all. Corrected:

```
Exposition → Entry (Link? Countersubject Entry){V−1} Redundant?
```

> **A grammar with an unbounded `+` in it is hard to falsify, and the parts of §2.4 that survive are the parts that
> say least.** `Middle+` accepts any number and 21 of 22 satisfy it; `Final → … Cadence` is exact and is one bit;
> `Exposition` is the only production with a shape and the only one that fails.

**Three faults in the parser were found and fixed before any of these numbers, and each had produced a plausible
table.** Grouping entries by the distance between their *starts* rather than from where one **ends** split every
exposition into as many groups as it had voices and reported 0% on all 22. A verdict for `Middle → Episode Entry+`
was true by construction once a group is defined as a run with no episode in it — a check that cannot fail,
replaced by a measurement of the `+`, which is 1.35 entries per middle group. And judging an entry's level by
whether its first note is the tonic failed seven expositions for alternating perfectly, since a subject beginning
on the dominant has a dux on degree 4; §8.11's Rule I is what *level* means, and using it moved that row from
22.7% to 40.9%.

A fourth was caught by the reference checker rather than by me: a section heading containing a `§` reference reads
to the link test as a citation and mangles its own anchor. Headings do not carry cross-references now.

### What step 7 has to build

Not §2.4's grammar. Four facts, and one of them is not from this section: the exposition takes links and sometimes
a redundant entry; a fugue has a median of three middle entry groups of about 1.35 entries; the episodes between
them run about three bars; it ends at home, always. And §8.13's: **episodes are 54% of the book by duration**, so
more than half of what step 7 must generate has no subject in it at all — which is not the problem §8.6 solved.


---

## Step 7 — a fugue, from a subject

[`0d94280`](../../commit/0d94280)

Readme §8.16, `src/step7.rs`, `out/fugue.mid`. Everything before this filled voices **against music that already
existed**. §8.6 held one of Bach's entries and reconstructed the others; §8.3 placed entries into a span Bach had
written. This emits the span too, so for the first time nothing in the output is Bach's except the subject.

**Twelve blocks, 27 bars, three voices, from BWV 847's subject, filled in 3.9 seconds.** Read back through §8.15's
own parser it covers the voices, alternates, has a middle and ends at home, and fails `runs unbroken` — the
expositional link, written on purpose, which §2.4 forbids and 82% of real expositions contain. Against §8.2's
checker: **628 slices, zero violations on the confirmed tier**, and 366.2 per thousand on the full five against
Bach's 112.3, since the two dissonance rules are not in the tier and §8.2 is why.

**Three constraints shaped it, and each cost something the section states.** Two free voices, so three voices is
the scope and half the book is out of reach — but a fugue is exactly where *which* voice is free changes, so the
fill runs one block at a time with the placed voice held. Episodes have nothing held in them at all, so the motive
is placed and sequenced, a commitment §8.13 already priced at 13.3%. And rhythm is data, so every free voice takes
the subject's own rhythm, which is why the result is stiffer than Bach.

### Four faults, each found by a test, each in code that had already produced a plausible fugue

**A parallel fifth across a block seam.** The search's state resets at every edge, so a parallel that straddles one
was invisible to it and visible to the checker — the fault §8.6's first test exists to prevent, arriving at the
join rather than in the middle. `Problem::prior` now carries the previous slice's pitches for **every** voice and
not only the free ones, because a parallel is a fact about two voices moving together, and a search that knows
where its own voices came from but not where the held one came from cannot see one.

**A voice entering by a leap of an eleventh.** An entry's first note is placed by the derivation, so no care in the
fill can reach it. The fix is Bach's: rest the voice before it enters, and there is nothing to leap from.

**§8.15's parser failing all four checks on the generator's own output**, because it read each entry's degree at
the block's first tick — and BWV 847's subject has an upbeat, so nothing sounded there and every entry was silently
dropped.

**And a join test that is true only where the join was kept.** A block that lost its join enters cold, and a voice
with nothing behind it may legally leap a tenth. That is why `Relaxed` reports *which* blocks lost a constraint and
not only how many.

> **One of twelve lost the join and none lost the plan**, and the order is the finding: the join is this
> generator's own convenience, the plan is §2.3's obligation system *and* what keeps the search tractable.
> Dropping the plan first turns a dead block into an exploded one, which is worse — which is how the order was
> found.

The test that says all this is not "the piece contains no violations" but **"no block contains counterpoint the
checker flags"**. Those are different claims and only the first is one the search can make; a test that conflated
them would either fail on a cost the generator already reports, or pass by not looking.


---

## Recurring pattern

Three constraints in this project have turned out too permissive to bind — the two-rule hard tier, the
non-chord-tone categories, and the functional progression rule — and all three were caught after the fact.

> **Any constraint written loosely enough to admit the target on the first try will admit almost everything, and
> the check for it is a chance baseline computed before the measurement, not after.**

Step 6 adds the corollary, which cost a retracted claim to learn:

> **A baseline must also be reachable by the thing it is a baseline for.** One computed with information the
> generator does not have measures an easier problem, and the gap is invisible until the honest control is built.

Step 6b adds a third, and it is about the *rule* rather than the measurement:

> **A decision rule that names a direction but not a magnitude will be decided by noise.** "Improves on both" is
> not a criterion; "improves on both by more than twice the standard error of the paired difference" is, and the
> two gave opposite verdicts on the same experiment.

Step 5 is the fourth instance and the largest: a rulebook that admits `10¹⁸` fills of three bars admits everything.
It is also the first one where the baseline was computed **before** the measurement rather than after, which is why
it took an afternoon rather than a step of the roadmap to notice.

Six casually chosen parameters have turned out load-bearing: the exclusive subject window, shared entry offsets,
per-pair melody counting, lyric spines, the harmonic segmentation window, and the melodic rule's absence from the
generator's tier.

Two further patterns have appeared alongside it.

**Baselines have to move with the thing they baseline.** Step 5's first table scored every row against a baseline
built from the true harmony, including the rows that were run without one. A baseline that does not vary with the
condition is not a control; it is a constant, and subtracting a constant from every row hides exactly the
comparison the table exists to make.

Step 6e adds two more, both about reading a table rather than building one.

> **Two conditions that solve different subsets of the problem cannot be compared by their averages.** §8.6's
> `clean` and `leaky` rows are scored over 99 and 110 spans, and the gap between them was quoted as the worth of a
> harmonic plan from step 5 until it was paired per span. Paired, it is smaller — and most of what remains turns
> out to be the difference between a right plan and a wrong one rather than between having one and not.

> **A statistic that says how well the data fit the model says nothing about whether the model is right.** The
> analyser's `fit` sits at 95% on spans whose chords are 15% correct, because two voices are easy to explain with
> many chords. Every repair that gates on such a number fails for the same reason, so the family closes together.

Step 7's generator adds one:

> **Relax the constraint you invented before the one the theory gave you.** Filling block by block needed a
> fallback when a block would not fill, and dropping the harmonic plan first turned dead blocks into exploded ones
> — the plan is not only §2.3's obligation system, it is what keeps the search tractable. The join was this
> generator's own convenience, and it is the one that should go first.

Step 7 adds one about checks rather than about music:

> **A check that cannot fail is worse than no check, because it reports a rate.** `Middle → Episode Entry+` scored
> 77.3% under a broken grouping and would have scored 100% under the correct one, since a group is *defined* as a
> run with no episode in it. Both numbers were meaningless and only one of them looked it.

§8.11 adds one that has been implicit since §8.2 and is now explicit:

> **A rule only earns something where it differs from the default.** Marpurg's Rule I is right in 21 of 21 cases
> and that figure means nothing, because in 14 of them it prescribes what plain transposition does anyway. Seven
> of seven, on the cases where it says otherwise, is the measurement. The same reading applies to every rule §8.2
> reports: a prohibition against something nobody does is satisfied at no cost.

The command line adds one that is not about measurement at all, but has the same shape:

> **An interface that answers an unknown input by doing something else has no error case, and every mistake it
> receives comes back as a plausible result.** `argv[1]` fell through to the default batch, so a mistyped command
> printed a real table for a different question and exited zero.

Step 6f adds two more.

> **A positive criterion has a cheapest way to be satisfied, and a minimiser will find it rather than the thing
> meant.** All three prescriptions collapsed: `move by step` onto oscillating between two notes, `state the
> harmony` onto holding a chord tone, `move against` onto never moving. Prohibitions do not have this failure mode,
> which is why a rulebook of them composes and a short list of prescriptions does not.

> **When no criterion distinguishes the candidates, the tie-break becomes the criterion.** Two sections diagnosed
> the generator's lines as too narrow and blamed the rulebook. The narrowness was the shortest path keeping the
> first of many equal-cost paths, and it was measurable all along by running with no criterion at all — a control
> that costs one row and was not there.

**Three separate defects have been argument-order or identity confusions in code that looked symmetric and was
not** — the swapped checker arguments here, the lo/hi role tracking of step 1, and the duplicate spellings. Two of
the three announced themselves as a number that was *exactly* wrong rather than approximately wrong: zero crossings
in a fill that crossed at every slice, and a rate inflated by exactly `V − 1`. That signature is worth watching for.
