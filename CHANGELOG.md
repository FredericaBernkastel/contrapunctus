# Changelog

The [readme](readme.md) states the current position. This states how it was reached — every implementation step in
the order it happened, the defects each one found, and the claims that did not survive the next experiment.

**Forward chronological**, against the usual convention, because each entry is the reason for the next and reading
the corrections before what they correct makes nonsense of both.

---

## Step 0 — the corpora

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

---

## Step 3 — the corpus ranking

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

Built in `src/harmony.rs`: segment at the notated beat, score every root-and-quality candidate by duration-weighted
membership, classify every note against the chord that prevails where it sounds.

**Reported at the time:** as a rule it fails (Bach 99.4% explained, control 98.8%); as a design objective it works,
with every step-4 diagnostic reversing — Bach beats random 17 of 20, mean advantage +0.0552, optimised contours
carry 6.8 distinct degrees rather than 1.0, and the optimiser cannot beat Bach.

**This also corrected experiment 5.** Its 78% vs 56.9% gap was measuring texture complexity, not harmonic
correctness: three transpositions of one subject is a thin self-similar texture that trivially fits a triad.

---

## Validating the harmonic layer — four external checks, all negative

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

## §17 — the analyser rebuilt

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

## Recurring pattern

Three constraints in this project have turned out too permissive to bind — the two-rule hard tier, the
non-chord-tone categories, and the functional progression rule — and all three were caught after the fact.

> **Any constraint written loosely enough to admit the target on the first try will admit almost everything, and
> the check for it is a chance baseline computed before the measurement, not after.**

Five casually chosen parameters have turned out load-bearing: the exclusive subject window, shared entry offsets,
per-pair melody counting, lyric spines, and the harmonic segmentation window.
