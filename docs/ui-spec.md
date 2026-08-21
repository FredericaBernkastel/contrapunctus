# Contrapunctus Workbench — interface specification

A desktop and browser interface over the `contrapunctus` library, in [egui] 0.36
through [`eframe`], which is the framework that gives the same code a window on
the desktop and a canvas in a browser. Built as `ui/`, a member of the same
workspace. Section 12 is the roadmap and says what is implemented.
[`ui-sketch.html`](ui-sketch.html) is the visual sketch this specifies; open it
beside this document.

[egui]: https://github.com/emilk/egui
[`eframe`]: https://github.com/emilk/egui/tree/master/crates/eframe

Section numbers of the form §8.16 refer to [`../readme.md`](../readme.md), which
is the argument this program is built on and the record of what it can and
cannot do. **`§` always means that document**, never this one — sections of this
specification are cited as bare numbers, *4.3 below*.

That convention is enforced rather than hoped for: `tests/references.rs` sweeps
`docs/` and resolves every `§` against the readme, so a section of *this* file
written with one is reported as naming a section that does not exist. It caught
two on the first run, and then caught a third in the sentence explaining the
rule.

---

## 1. What the interface is for

Two users, and the design problem is that they are not the same person.

- **Someone with no theory** knows what a tune is, that it comes back, and that
  it moves around. They do not know *dux*, *comes*, *episode*, *compass*, or what
  a tier is.
- **An expert** wants every one of those, and the seed as well.

Two applications would be the easy answer and the wrong one. This is **one
layout at two densities**: every control keeps its position between modes, and
Advanced *reveals* rather than rearranges. A plain-language label sits above the
parameter's real name in mono, so a beginner reads *How far it travels* and an
expert reads `middles = [4, 5, 3]` underneath. Nobody is lied to and nobody
learns a second layout.

**Plain does not mean wrong, which took a correction to get right.** The subject
was called *the tune* throughout the interface, on the grounds that a beginner
knows what a tune is. But *subject* is the name for it, it is one word, it is no
harder than *tune*, and the library has called it that all along — so the plain
label was buying nothing and costing the reader the word they will meet
everywhere else. It says **subject** now, and so does the strip: an entry's block
reads *subject* where it read *theme*.

The rule that came out of it: plain language is for **describing** a thing —
*times the subject comes back*, *how far it travels* — and never for **renaming**
one. A reader who learns *tune* here has to unlearn it; a reader who learns
*subject* here can go and read about fugues. The tooltips are where the technical
name of a *quantity* goes when the label describes it instead: the returns slider
says middle entries, and what §8.15 measured of them.

**The centrepiece is the plan strip, not the score.** Three lanes, one per voice,
the subject drawn solid where it sounds and hatched where it is away, with the key
plan as a ribbon underneath. It teaches what a fugue *is* by being looked at, and
it is the one view where a non-expert can make a judgement worth making. The
score sits below it and is secondary.

---

## 2. Decisions taken

| question | decision |
|---|---|
| Score rendering | **Staff notation.** Not a piano roll. |
| Subject | **Chosen or imported only.** No note editor in this version. |
| Plan strip | **Editable, as far as the library allows.** |
| Sound | **Both** a built-in synth and system MIDI out. |
| Four voices | **Present and disabled**, until the search can do three free voices. |
| Platform | **Desktop and web from one codebase.** No OS-specific behaviour without an objective need. |
| Settings | **Saved and loaded, plan included.** The same file reproduces the same fugue, and that is checked rather than asserted. |

---

## 3. Layout

### 3.1 Frame

```
┌──────────────────────────────────────────────────────────────────────┐
│ Contrapunctus   [Simple|Advanced]        ▶ ■   Export ▾   Save  Open │
├───────────────┬──────────────────────────────────────────────────────┤
│ THE SUBJECT   │  PLAN        ← the centrepiece, editable             │
│  staff        ├──────────────────────────────────────────────────────┤
│  Choose…      │                                                      │
│  Import…      │  SCORE       ← staves, playhead, follows the plan    │
│               │                                                      │
│ THE SHAPE     │                                                      │
│  …controls…   │                                                      │
│               ├──────────────────────────────────────────────────────┤
│ [Compose]     │  HOW IT TURNED OUT                                   │
│ [Try another] │                                                      │
└───────────────┴──────────────────────────────────────────────────────┘
```

A single `egui::SidePanel::left` (fixed 284 px, resizable), a
`egui::TopBottomPanel::top` for the toolbar, and a `CentralPanel` holding the
three stacked views. The two modes differ only in which groups the side panel
draws and which columns the report shows.

### 3.2 Simple mode controls

| control | plain label | library field |
|---|---|---|
| Subject picker | The subject | `Design::subject` |
| Voice count | How many voices | `Design::voices` |
| Return count | Times the subject comes back | `Layout::middles.len()` |
| Journey preset | How far it travels | `Layout::middles` |
| Strictness | How strictly it follows the rules | `tier` |
| Reroll | Try a different one | `seed` |

The three journey presets:

| preset | `middles` | reads as |
|---|---|---|
| Stays home | `[4]` | one trip to the dominant and back |
| Wanders | `[4, 5, 3]` | the default — §8.15's median of three |
| Roams | `[4, 5, 1, 3, 6]` | five, near the top of the book's range |

Strictness maps to the tier, and **the default is `HARD`**, which is not the tier
§8.2 endorses for describing the repertoire. §8.16 is the argument: a generator
on `conf+melodic` writes dissonance at 366 per thousand and a listener calls it
cacophony; on the full five it writes about 74, below Bach's own 112. The tooltip
says so in one sentence.

### 3.3 Advanced mode adds

- **Design** — subject provenance, voices, key and metre (read from the source),
  and per-voice compass as draggable ranges on a staff.
- **Layout** — the middles as reorderable chips each carrying a degree; episode
  length in bars; the link's position and length; close-at-home.
- **Search** — tier as three named radio options; seed as a hex field with a
  reroll; the analyser's `λ`.
- **Report** — the five grammar verdicts individually, the tally by rule, the
  per-block relaxation log, and timings.

Nothing in Advanced is hidden behind a further click. It is one longer panel.

---

## 4. The plan strip

The most important view and the most interesting to build.

### 4.1 Drawing

One lane per voice, `x` proportional to tick. For each `Outcome::blocks[i]`:

- `Kind::Entry` — a filled rounded rect in the voice's colour, labelled
  *theme* or *answer* (the latter when `tonal`).
- `Kind::Episode` — the same rect, hatched and outlined, in the voice that
  carries the motive.
- Beneath the lanes, a **key ribbon** segmenting the piece by `Block::key_of`,
  merging adjacent equal values.
- Blocks in `Outcome::relaxed.cold` carry a warning outline and a tooltip
  saying which constraint was dropped.
- A playhead line, shared with the score view.

### 4.2 Editing

Every gesture maps onto a `Layout` field. Nothing else is editable, because
nothing else is a parameter.

| gesture | effect | `Layout` |
|---|---|---|
| drag a block **vertically** to another lane | that entry or episode changes voice | *see 4.3 below* |
| drag a return **horizontally** past a neighbour | reorder the journey | `middles` reordered |
| click a return, and choose a degree | where that return goes | `middles[k]` |
| drag an episode's **right edge** | lengthen or shorten every episode | `episode_bars` |
| drag the exposition link's edge | its length, or off at zero | `link` |
| click a **+** in the handle row | add a return there, at the dominant | `middles` grown |
| a return's menu, *take this return out* | remove it | `middles` shortened |
| the closing block's menu, or the **+ close** handle | close at home, or stop after the last return | `close_at_home` |
| click a block, and choose *write these bars again* | refill just that block with a new draw | `rerolls`, keyed on the block's identity |

**Both directions are always on screen, which is what took a second attempt.**
Adding and removing a return were first tried as gestures on the blocks
themselves, and came out **one-way**: click the final block to stop closing at
home and the block you clicked is gone, with nothing left to click to bring it
back. A gesture whose undo lives somewhere else is worse than a checkbox. The
answer was not a better gesture but a **row of handles** under the key ribbon —
a `+` at each place a return could go, and one more at the end for the close when
it is off — so whichever state the plan is in, the way out of it is visible.

The same three edits are in the Advanced panel, and **they go through the same
path**: they apply at once rather than waiting for Compose. They used to set the
layout directly, which meant taking a return out in the panel and taking one out
on the strip did visibly different things.

**Every block's menu opens on either mouse button.** Which one a person reaches
for is a habit rather than a decision, there is nothing here for the two to mean
differently, and a block that answered only one of them would have a dead spot
under the other. `Popup::menu` alone gives the primary button; the secondary is
added explicitly.

**Two classes of edit, and the interface must not blur them.** Which class an
edit is in is `Edit::touches` in the code — a method the dispatch actually goes
through — and a test checks its answer against what `compose::derive` does
rather than against the claim. That test earned its place immediately:
**reordering the returns was written down here as span-changing and is not.**
`derive` gives every return an episode and an entry of the same lengths whatever
degree it carries, so shuffling the order changes keys and moves not one bar. It
goes down the fast path.

*Span-preserving* — a key change, a reordering, a per-block reroll. The piece
stays the same length, and `compose::refill_span` rewrites **from the first
affected block to the end**, so every bar *before* the edit is untouched.

### A claim withdrawn, and the bug that withdrew it

This section used to promise more: that a span-preserving edit rewrote only the
blocks it owned and **every other note stayed exactly where it was**. That was
true of the notes and false of the piece, and it broke section 8.

Refilling only the edited blocks means pinning the last of them to what the piece
sounded *before* the edit. That pin is real information and it is **history** —
it says what some earlier version of this fugue happened to end on, and a
settings file records no history. So an edited fugue was not one the generator
would write from its own settings, and saving it and opening it again produced a
different piece. The report was a fingerprint mismatch between engine `0.1.0` and
engine `0.1.0`, which is a sentence that tells a reader nothing and blames the
wrong thing twice over.

> **A local edit that pins its own boundary is not a function of the settings, it
> is a function of the history.** The pin cannot go in the file, because it is
> not a parameter of the piece — it is a fact about a piece that no longer
> exists.

Running to the end takes no pin, and then the result is exactly what
`compose::fugue` writes. A test in `compose` asserts precisely that, and one in
the interface asserts it over thirty-odd edits.

What is left of the locality is the half worth having: **the bars before the edit
do not move**, which is the part already heard. The bars after it follow from the
change — which is what they should do anyway. A return sent somewhere else, with
everything after it carrying on as though nothing had happened, was never the
more musical answer; it was the faster one.

The cost is real and it is stated: an edit is now most of a generate rather than
one block of one, so the *hundred milliseconds* this section used to promise is
a few hundred. At three voices that is still immediate. At five it will not be,
and the answer there is the same worker 7.3 wants, not a pin.

The 110-of-144 figure this section used to carry — how often a pinned refill
could reach its old ending — is withdrawn with the pin that produced it. It
measured something the program no longer does.

*Span-changing* — episode length, the link's length, adding or removing a middle,
toggling the close. Everything after moves in time. `compose::refill_span`
refuses these by design; the interface re-runs `compose::fugue`.

**And the fading turned out to be worth more than fading.** `compose::derive` is
pure and costs nothing — it produces the plan without searching for a note — so
a drag shows *the exact plan it would commit to*, live, with the blocks that are
about to move drawn faded behind it. Not a hint that something will change: the
thing it will change into. One function, `Edit::applied`, produces both the
preview and the commit, so the picture cannot promise what the commit does not
do.

The horizontal scale stays pinned to the committed piece for the length of a
drag. Rescaling to the preview would move the edge out from under the pointer
dragging it, which is a feedback loop rather than an interface.

### 4.3 A limit worth stating, and what it turned out to be

`compose::derive` chains the lanes: after the exposition, each block takes the
voice **after its predecessor's**. So a block's lane is not independently
settable, and dragging one to another lane rotates the ones after it.

**This section used to give the reason as "so that no two consecutive blocks are
placed in the same voice", and that is false.** The default layout — the one
every published figure uses — derives as `0E 1E 2L 2E 0m 1m 2m …`: the link and
the entry after it are both in voice 2, because the exposition's entries are
written one per voice top-down rather than chained, and the link takes the lane
after the entry it follows. There is no such invariant to protect. What the
chain is actually for is §8.16's finding: a voice that ends an entry and then
starts the next episode's motive leaps whatever separates them, and handing the
motive on costs nothing.

So the honest parameter is not a lane but a **rotation**, and that is
`Layout::turns` — keyed on `compose::identities_of`, each entry rotating its own
block and every block after it. Dragging a block down a lane sets one. The chain
survives intact: after the turn every step is still one lane on from the last.

Three things fell out of building it, and all three are the interesting part.

- **A turn reaches one block further back than it looks.** `fill_block` asks
  which voice holds the *next* block and rests that voice for a bar at the end of
  this one, so it does not enter by a leap. Turn a block and its predecessor is
  told a different voice is coming — so it changes too. `Edit::touches` refills
  from there and the ghost fades from there. Refilling from the turn itself would
  leave one block stale; fading from it would show less than moves.
- **The exposition cannot be turned in part.** Its entries are one per voice by
  construction, so a rotation of a tail of them states the subject twice in one
  voice and never in another. `compose::turnable` says so, `Run::new` refuses a
  layout that does it, and the drag never offers one. Turning at the very first
  block rotates the whole run together and is fine.
- **A legal turn can still fail to compose**, and for a reason that connects to
  3.3. A turn moves a *placed* subject into another lane, and a placed subject
  ignores the compass — so an entry can land far outside the compass of the voice
  now holding it, leaving the free voices an awkward job. Rotating one subject's
  whole piece by a lane hits §2.7's wall at bar 26. That is a hard search and not
  an illegal layout, and the two are kept apart: the interface does not offer the
  second and does report the first.

The ghosting 4.2 already does is what shows all of this: the drag draws the plan
it would commit to, with everything from the turn's predecessor onward faded. No
gesture fakes independence, because none is offered.

### 4.4 Which voices are sounding — the grammar's part built, the rest on the roadmap

Every voice used to sound in every bar. `compose::fill_block` gave the held voice
the subject and every other voice a tiled rhythm, and a voice with no notes was an
error — so a three-voice fugue had three voices from bar one, and an exposition,
whose whole identity is voices arriving one at a time, had no arrival in it.

That was noticed by a reader with the *Art of the Fugue* in hand. It was the
plainest musical fault the interface displayed, and it turned out to be the same
question readme §9 asks about four voices.

**Candidates 1 and 2 below are built.** `compose::resting` rests a voice until it
has entered, so the piece opens with one voice, then two, then three; and
`Layout::rests` rests a voice again after it has been heard, which is what reaches
four voices. Asking for four sets a pattern rather than handing over a refusal.

**The strip says who is accompanying, which it never used to.** A block was drawn
in the lane holding it and every other lane left blank, so a voice playing an
accompaniment and a voice saying nothing looked identical — and until this they
always were identical. Faint bars now show the accompaniment, the exposition fills
up lane by lane, and a rest is visible as the gap it is. Clicking one toggles it.

Candidate 3 is roadmap item 1, and readme §8.17's four-voice figures are what says
whether it is worth wanting.

**readme §8.17 measures it.** The exact search's wall moves with the number of
voices it must *choose*, not with the number sounding:

| free voices | peak states | with four voices sounding |
|---|---|---|
| 1 | 68 | 4 voices, 2 resting |
| 2 | 8 434 | 4 voices, 1 resting |
| 3 | refused | 4 voices, none resting |

Four voices with one resting costs what three voices costs, to the state, in the
same 150 ms with nothing relaxed. So the field that fixes the texture is also the
one that buys the voice count — and §9's four-voice item stops being a solver
problem.

**What the interface would show.** A lane already exists per voice, and a resting
voice is a lane with nothing drawn in it for those bars, which needs no new
vocabulary: the strip becomes a picture of the texture rather than of a full
grid, and the exposition finally looks like one. The gesture is a click on a
block's lane to silence that voice there, and the ghost 4.2 already draws shows
what the rest of the piece does about it.

#### Where the pattern comes from, and what each answer gives somebody using this

Three candidates. They are not alternatives — each is a layer on the one before,
and each is worth having on its own. What follows is what each *delivers*, and
readme §8.17 now measures the first one's limit rather than assuming it.

**1. The grammar supplies it: a voice says nothing until it has entered. — BUILT**

No parameter, no gesture, no settings field. The exposition already says who
enters when, so this is a rule and not a choice.

What it gives: the opening finally sounds and looks like a fugue. One voice
alone, then two, then three, and the plan strip shows lanes filling up rather
than a full grid from bar one. That is the single most recognisable event in the
form and it is currently absent — this is a **correction, not a feature**, and it
should happen whatever else does.

What it does not give, and the measurement is exact about it:

| | free voices per block | worst |
|---|---|---|
| 3 voices | `E0 E1 L1 E2 m2 m2 … c2` | 2 — fills |
| 4 voices | `E0 E1 L1 E2 **E3** m3 … c3` | 3 — §2.7's wall |

**It does not buy four voices.** At four, the fourth entry is already three free
voices, and so is every block after the exposition, so the piece refuses at the
block that completes its own exposition. It also does nothing for texture after
the exposition: from bar five onward every voice still plays continuously. What
it does buy beyond the opening is speed — the first blocks drop to one and two
free, 68 states where there were 8 434.

**2. `Layout` carries it, per block. — BUILT**

A field keyed on `identities_of` like the rerolls and the turns, and a gesture:
click a block's lane to silence that voice there. The furniture exists.

What it gives: **four voices**, and the first thing in this program that lets
somebody *shape* a piece rather than pick from presets. Thinning the texture
before a big entry, dropping the bass under an episode, bringing all four in for
the close — that is fugal craft, and none of it is expressible today. It saves
and reloads with the piece, so a texture is part of what a settings file
reproduces.

What it costs: **the constraint is invisible until it is violated.** Four voices
with nobody resting refuses, and a refusal a person cannot diagnose is worse than
a control that is missing. §9's disabled four-voice button already sets the
precedent for how to handle that — offer it with the condition stated — and the
interface should maintain a legal pattern by default rather than leave somebody to
discover the rule by hitting it.

**3. The search chooses it. — BUILT, off by default, and it does not work**

What it gives: four voices that simply work, with nothing to learn and no
constraint to respect. And *Try a different one* becomes a much larger musical
difference than it is now — a different draw would change who is playing, not
only what they play.

The cost argument is in its favour rather than against. Resting **shrinks** the
problem: choosing one of three voices to rest and then filling two free is three
subproblems of 8 434 states, about 25 000 in total, against refusing outright at
three free. The choice pays for itself immediately.

The objection is this project's founding constraint. Something has to decide
*which* voice rests, and no treatise says. A heuristic — rest whoever has played
longest — is exactly the hand-rolled rule §1 forbids.

There is a formulation that invents nothing: **make the rest patterns part of the
legal set and draw uniformly from it**, which is what
[§8.10](../readme.md) already established beats optimising. No new rule, the same
mechanism over a larger domain. The unmeasured risk is that uniform draws give
*incoherent* texture — a voice appearing and vanishing block by block, where real
fugal texture has shape. Repairing that is where an invented rule would creep back
in, and it is the reason this one is called the interesting candidate rather than
the recommended one.

**Measured, and the risk was the outcome.** readme §8.17 built it and counted: a
pattern with one more voice playing admits thousands of times more fills, so the
draw returns the densest legal texture essentially always — one block in 156 chose
to be thinner than it had to be, and at three voices the flag changes not one
note. What it does buy is the choice among *equally dense* patterns, spread 24 /
9 / 31 / 36 across the four voices, which replaces `rests_that_fit`'s heuristic
with something that invents nothing. A correction, not a different piece.

**So it ships switched off, with a control to turn it on**, and that is a
requirement rather than a nicety. It is a `Layout` field, not interface state: it
changes what is generated, so section 8 requires it in the settings file or a
saved fugue would not reproduce. In Advanced under SEARCH, phrased for what it
does rather than for how — *let the search decide who rests* — with the honest
caveat beside it, that resting is what makes four voices possible and that nothing
has yet measured whether a drawn texture sounds like one. The precedent is
`--gen-tier` existing separately from `--tier`: a change that alters what is
generated and has not been measured against the book does not get to be the
default. It also gives somebody a way back to a texture they can predict, which
matters more here than usual, since the alternative is a piece where *Try a
different one* changes who is playing.

**And one thing four voices does not give**, which the measurement made plain and
the panel now says out loud: four voices state the subject and **three sound at
once**. Four sounding together *is* three free voices, which is the wall itself —
no arrangement of rests moves it. So this reaches four voices in the sense the
form cares about, four entries and a grammar that passes, and not four-part
texture. Two things were being called four voices and only one of them was ever
§9's solver item.

**Build order, as it went.** 1 was a bug fix. 2 delivered four voices and the
compositional control. 3 was built to see whether it could replace 2 and cannot —
it is on by choice or not at all, and what it corrects is small. All three are in;
what is left is not a fourth candidate but a different kind of thing, and item 4
of the roadmap says so.

**And one thing the measurement does not cover.** A voice has to leave and
re-enter, and `Problem::prior` carries one pitch per voice — a resting voice has
none, so every re-entry is a cold start. `fill_block` already does that
deliberately before an entry, for §8.16's leap-of-an-eleventh reason, and has
never done it anywhere else. Whether the counterpoint survives being cold that
often is not something §8.17 asked.

### 4.5 Building a plan rather than choosing one — not built

`Layout` is a **plan generator**: six parameters, and `derive` expands them. The
alternative a reader asks for once they understand the strip is a **palette** —
blocks dragged onto the lanes, and the plan authored rather than derived.

It is possible, and the interesting part is what `derive` currently supplies for
free:

- **Blocks tile time.** `Block::at` accumulates, so gaps and overlaps are not
  expressible. Authored blocks make both expressible and the strip would have to
  close them itself.
- **`origins` names what a block came from.** Every edit in 4.2 is phrased over
  `Origin::Middle(k)` — *this return goes to the dominant*. A hand-built plan has
  no returns to index, so those edits need a second vocabulary at the block level.
- **`identities_of` keys the rerolls and the turns**, off the chain a layout
  derives. Authored blocks need identities that are theirs.

**The grammar is already the safety net, and this is the argument for doing it.**
`form::parse` judges *any* plan against §2.4's grammar and returns five
independent verdicts — the exposition covers the voices, alternates tonic and
dominant, runs unbroken, there is a middle, it ends at home. That is what the
generated fugues are scored against already; nothing new is needed to score an
authored one.

So the palette should **not** refuse an illegal drop. It should let the plan be
built and let the verdict say which of the five things it is missing. That is
1's two-user problem exactly: presets for somebody who wants a fugue, and a
palette plus a live verdict for somebody who wants to know what a fugue is. A
palette that only permits legal plans teaches nothing, because everything it
allows is already legal.

---

## 5. The score

### 5.1 Staff notation, and why it is cheap here

`Pitch { step, alter }` is a diatonic step and an accidental (§2.1). **Staff
position is `step` and nothing else** — no lookup table, no key-signature
reasoning, no enharmonic guessing. The argument §2.1 makes for the lattice turns
out to be the argument for it being drawable.

- **Vertical**: `y = baseline − (step − reference) × half_space`. Ledger lines
  where the position falls outside the five.
- **Accidental**: from `alter`, drawn when it differs from the key signature's
  value for that letter — `Piece::key[step.rem_euclid(7)]`, which the library
  already carries.
- **Horizontal**: `x` proportional to `Note::onset`. The tick lattice is exact
  (960 per whole note), so spacing is exact.
- **Duration**: `Note::dur` against `TICKS_PER_WHOLE` gives the note value
  directly. Beams join runs of eighths and shorter within a beat.

**Beaming, and what it turned out to hinge on.** Two rules decide a group and
the tick lattice answers both exactly: notes join when one **ends where the next
begins**, and when they fall in the **same beat**. A beam across a beat would
hide the metre, which is the one thing a beam is there to show. Each beam level
is drawn over every maximal run of notes short enough to need it, so a lone
sixteenth among eighths gets its second beam as a stub, and a short note with
nothing to join gets a flag.

The count of beams is taken by halving **down from a quarter** until the value is
no longer than the note, rather than doubling up from the note. The difference is
**dots**: a dotted eighth is three quarters of a quarter, doubling it overshoots,
and the first version called it a quarter and drew it with no beam at all — a
note the corpus is full of, rendered wrong. A test found it on the first run.
- **Clef**: treble for voices whose mean `step` is above middle C, bass below —
  the same mean-pitch ordering `midi::write_score` already uses for track order.

Rendered with `egui::Painter` primitives: `line_segment` for staves, stems,
beams and ledger lines; `circle_filled` scaled into an ellipse, or a small
filled convex path, for noteheads. Accidentals are drawn from line segments —
three glyphs, and they are the only shapes here simple enough to be worth
drawing rather than setting.

**The clefs are a font, and the reason is the geometry rather than the drawing.**
`ui/assets/music.otf` is four glyphs of Bravura, SMuFL's reference font: 16 kB of
868, which costs the browser build 0.07 MB. SMuFL specifies that **one em is four
staff spaces** and that **each clef's origin sits on the line it names**, so a
clef drawn at a size of four staff spaces with its baseline on the G or F line is
placed exactly right *by construction* — there is no constant in `glyph.rs` that
anybody tuned by eye, which there certainly would have been had the outlines been
drawn by hand. egui anchors a galley by its box rather than its baseline, and the
two differ by the ascent, so the baseline is read off the laid-out glyph
(`Glyph::pos`) rather than guessed at.

Do not rely on a system music font: there is not one on the web. And note what
the licence costs, because it is not nothing — Bravura is OFL 1.1 with **Reserved
Font Name**, so the subset had to be renamed to be redistributable at all, and
its licence travels with it. That is the argument against lifting the outlines
into source instead: they would still be derived from the font, and the
obligation would be far less visible sitting in a `.rs` file than beside a file
plainly marked as a font.

### 5.2 Behaviour

**Wheel to pan, ctrl and wheel to zoom** — over the score, and over nothing else.
A score is a horizontal thing, so a vertical wheel moves along it; the horizontal
delta counts too, for whoever has a trackpad that sends one. The scroll bar still
drags, because taking that away would be taking something and giving nothing.

**Where the zoom comes from is not a detail.** When a wheel event carries egui's
zoom modifier, egui moves the wheel into `zoom_factor_delta` and leaves
`smooth_scroll_delta` at zero — so an implementation that reads the scroll delta
and the ctrl key separately has a zoom branch that can *never* run, while panning
goes on working perfectly. That is what shipped, and what it looked like from
outside was a zoom that did nothing. Read `InputState::zoom_delta` instead, which
is the factor already, and which prefers a trackpad pinch when there is one.

Zoom is **about the pointer**: the bar under it stays under it. Zooming about the
left edge looks fine on the first notch and has thrown the reader across the page
by the fourth, and keeping the point costs one ratio. `score::View` holds the two
numbers and does that arithmetic, apart from the drawing, because it is the part
of zooming that can be *wrong* rather than merely ugly — and a struct with two
numbers can be tested where a wheel event over a rectangle cannot.

**The plan strip does not zoom and does not pan.** It fits the whole piece, which
is its entire job; a view that shows everything has nothing to zoom to. What it
gained instead is the other half of the pair: once the score is zoomed in, the
strip **shades what is off the page**, so the overview says where the detail view
is looking. That is the reason to have both views rather than a preference
between them.

- Horizontal scroll locked to the plan strip, so the two views always agree.
- Entry blocks tinted behind the notes in the voice colour, so the subject is
  findable on the page without reading it.
- Playhead follows audio position; clicking a bar seeks.
- No editing. The score is an output.

---

## 6. Sound

Both outputs, chosen in the toolbar. They share one scheduler and differ only in
the sink.

### 6.1 The scheduler

`Outcome::voices` is already exact: ticks per whole note is 960, and
`kern::meter` gives the metre. Convert once to a flat, sorted list of
`(sample_index, voice, Option<Pitch>)` events at the chosen tempo, then play it.
Nothing about this is platform-specific and nothing about it needs a timer — the
audio callback's own sample clock is the position, which is also what makes the
playhead exact rather than approximately synchronised.

**There is no timer anywhere in this program.** The playhead is not synchronised
with the sound; it is *read off* it, through an atomic the callback writes and
the frame reads. Nothing else can be as accurate, and nothing else keeps being
accurate when a frame is late.

Ties are merged here rather than left to the synth. A `Note` with
`attack == false` is the same sound continuing, and re-striking it would turn
every suspension into a repeated note — the one articulation §2.2's rules exist
to tell apart.

### 6.2 Built-in synth — the default

`cpal` for the device, a hand-written voice per part. Three voices of
counterpoint need nothing richer than a triangle or a stack of two or three sine
partials, with a short attack and release so entries do not click.

- Works on desktop and on the web: `cpal` targets WebAudio on
  `wasm32-unknown-unknown`.
- No external dependency on what the user has installed.
- The point of this tool is hearing the counterpoint **clearly**, not hearing it
  sound good. Those are different goals and only the first is ours.

**Built, and the shape of it is the interesting part.** Three files, split by
what can be tested:

| file | what | tested |
|---|---|---|
| `schedule.rs` | a fugue to samples, and back | yes — ties, tempo, both directions |
| `synth.rs` | samples to sound | yes — clicks, clipping, seeking |
| `audio.rs` | the sound card | no, and it cannot be |

No build machine here has speakers, so the third file can never be exercised.
That is an argument for making it as small as possible rather than for giving
up: everything that can be *wrong* — a tie re-struck as two notes, a tempo out
by a factor of four, three voices clipping, an entry that clicks — lives in the
two that render into a buffer and can be asserted about.

Three constants and their reasons. **4 ms attack, 14 ms release**, which is a
tenth of the shortest note the generator writes at 76 — long enough that no
entry clicks, short enough that it reads as articulation rather than a swell,
and articulation is what makes a repeated note and a tied one different to the
ear. **Peak 0.72** across the texture, asserted rather than assumed, because a
headroom constant is exactly the thing that gets invalidated by adding a partial
and never revisited.

One design note worth keeping: **`Score` owns its own tick-to-sample
conversion**. Free functions would take the tempo as an argument, and a caller
that passed a tempo the score was not built at would get a playhead that drifted
— slowly, plausibly, and only after the tempo had been changed once.

#### Equal amplitude is not equal loudness

The fifth listening report: *"higher pitches are significantly louder than lower
ones."* True, and mine. Every note got the same amplitude, and the ear is far
more sensitive around two to five kilohertz than it is low down, so the soprano
dominated a texture the code believed was balanced. Nothing was wrong with the
synth except that it treated a physical quantity as a perceptual one.

The correction is a **tilt** — decibels of attenuation per octave above a pivot
near the bottom of the compass, so it only ever quietens and cannot cost the
headroom a clipping test has to keep. What it achieves is measured rather than
asserted, using **A-weighting as the instrument**, over C3 to F6:

| tilt | A-weighted spread |
|---:|---:|
| 0 dB/8ve | 15.7 dB — the complaint |
| 1.5 | 10.6 |
| 3.0 | 6.4 |
| **4.5** | **3.1 — the minimum** |
| 6.0 | 5.7 — overshoots; the bass becomes the loud end |

**The first default written was 3 dB, chosen because it seemed about right, and
the measurement beat it by a third.** A test now asserts the default is the
narrowest of the settings tried, so a later change to the waveform that moves the
optimum fails rather than quietly leaving a stale number.

A-weighting is used to *measure* and not to *correct*, and the difference
matters: it describes the ear at a quiet level and the equal-loudness contours
flatten as the level rises, so inverting it into the synth would over-correct for
anybody playing this loudly. Measuring with one curve and correcting with a
plainer one keeps the two honest about each other — and it is why the tilt is a
control rather than a constant.

**A tilt and not a bank of bands**, which is what was asked for. A multi-band
equalizer would be the right answer to a timbre that is wrong; this is a
*balance* that is wrong, across the register, and the shape of the fix is a
slope. It is one knob in the header beside the voice toggles, it goes to zero,
and past about five it makes things worse in the other direction — which the
table above is the reason to know.

### 6.3 System MIDI out — optional

`midir`, which also targets Web MIDI on wasm.

- Ports enumerated into a dropdown; nothing sent until one is chosen.
- On the web, Web MIDI needs a permission prompt and is not available in every
  browser. **Absence is a disabled dropdown with a reason, never a silent
  failure.**
- Behind a cargo feature so a build without it has no dependency.

### 6.4 The stream does not survive a stall, and is rebuilt

Found by pressing Play and then Compose: the new music played, chopped, and
stayed chopped until the page was reloaded. Web only.

`cpal`'s WebAudio backend schedules each buffer from an `onended` callback **on
the main thread**, and its cursor synchronises with the context clock only for
the *first* buffer — after that it advances by one buffer step and never looks at
the clock again:

```rust
if *time_at_start_of_buffer > 0.0 { *time_at_start_of_buffer }   // every buffer after the first
else { now + base_latency_secs + buffer_time_step_secs }         // only the first
```

So blocking the main thread past the ~85 ms already queued leaves every later
buffer scheduled at a time that has passed. The browser plays them immediately,
the two workers overlap, and **nothing ever re-synchronises** — which is exactly
why only a reload helped.

> **A stall on the main thread is not a hiccup where the audio clock is also on
> the main thread.** It is permanent damage, because the thing that would notice
> and correct it is the thing that was blocked.

The fix is to rebuild the stream after anything that stalls the frame: a
generate, an edit, a settings load. Dropping `cpal::Stream` closes the
`AudioContext`, so this does not accumulate contexts against the browser's
per-page limit — which is the thing that would have made the cure worse than the
disease. The position and the play state are carried across, so a rebuild costs
a moment of silence and nothing else.

Native audio has its own thread, nothing breaks, and it keeps its stream. That
is one of the few places this program behaves differently by target, and 7.5
asks for a reason rather than for uniformity.

**None of this is tested.** It lives in the one file no test here can reach, and
it was found by a person pressing two buttons in a browser.

### 6.5 Export

MIDI file, by `midi::write_score`, which already orders tracks top voice first
and names each with its range and role.

---

## 7. Running on the web

egui compiles to `wasm32-unknown-unknown` via `eframe`. The interface must not
assume otherwise. Four consequences, and each has a concrete rule.

### 7.1 No filesystem in the library's path

Three functions in the library take a `&Path` and immediately delegate:

| now | needs |
|---|---|
| `kern::read(&Path) -> Piece` | `kern::parse(&str, id: &str) -> Piece` |
| `refdata::read(&Path, …) -> Vec<SubjectSpec>` | `refdata::parse(&str, …)` |
| `midi::write(&Path, …) -> io::Result<()>` | `midi::encode(…) -> Vec<u8>` |

Each is a two-line split: the path form reads the file and calls the new one.
**Do this before writing any interface code.** It is small now and becomes an
excuse later.

The corpus itself ships as bytes: `embedded::FUGUES` and `embedded::ANNOTATIONS`
behind the `embedded-corpus` feature, so a web build has the subject list with no
network fetch. **295 kB**, not the two megabytes first estimated. A test asserts
the embedded text is byte-identical to the files, which is what makes a browser
build's figures comparable with §8's rather than merely similar.

### 7.2 Files in and out

`rfd` for open and save dialogs. It has an async API that works on both targets —
a native dialog on desktop, and on the web a file input for opening and a
download for saving. **The interface always uses the async form**, so there is
one code path rather than two.

### 7.3 No blocking the frame

A generate is about 0.6 s. Blocking a desktop frame for that is rude; blocking a
browser frame for that freezes the tab.

The blockwise fill makes the answer easy: **generate one block per frame** and
keep the previous `Outcome` on screen until the new one is complete. Twelve
blocks at 60 fps is a fifth of a second of animation, and the plan strip can fill
in block by block as it goes, which is a better progress indicator than a bar.

**And on the web it would not be enough, which was learned the hard way.** This
section used to end by saying the approach needs no threads and therefore works
identically on both targets, reaching for a worker only if a five-voice block
ever mattered. The arithmetic says otherwise. `cpal`'s WebAudio backend queues
two buffers of 2048 frames — about **85 ms** at 48 kHz — and one block of this
generator is tens of milliseconds native and more in a browser. Smaller units
help and they do not clear the bar; a **worker** does, and is the honest answer
for the web whenever the sound has to survive the work.

Until there is one, the stream is rebuilt around anything long — 6.4.

### 7.4 Built, and how

`ui/index.html` holds a canvas the entry point looks up by id; `main.rs` has one
`#[cfg(target_arch = "wasm32")]` function beside the desktop one, and that is the
whole of the difference. Everything else — the interface, the search, the corpus,
the synth — is one build.

```
rustup target add wasm32-unknown-unknown
cargo build -p contrapunctus-ui --target wasm32-unknown-unknown --release   # the binary
cd ui && trunk build --release                                             # the page
cd ui && trunk serve --release                                             # and served
```

`trunk` fetches `wasm-bindgen` and `wasm-opt` itself; nothing else has to be
installed. Run it from `ui/`, where it finds `index.html` and the manifest
beside it, and the output lands in `ui/dist/`.

**Size, measured at each step.** 7.3 MB out of `cargo build`; **5.04 MB** after
`wasm-opt`; **2.10 MB** over the wire gzipped, which is what a reader actually
waits for. The corpus is 295 kB of that and everything else is the interface,
the search and their dependencies.

Served, the three assets come back `200 text/html`, `200 application/wasm` and
`200 text/javascript`, and the served page still carries the canvas id the entry
point looks up — which is worth checking rather than assuming, because the whole
document passes through a template on the way.

**What the build proves about the port**, without a browser to run it in: the
generated bindings name the browser APIs each subsystem needs, so their presence
is a fact about what was compiled rather than a hope.

| bound in the glue | which means |
|---|---|
| `AudioContext`, `createBuffer`, `sampleRate` | the synth reached WebAudio |
| `WebGlRenderingContext`, `WebGl2RenderingContext` | the renderer found a context |
| `HtmlCanvasElement`, `getElementById` | the entry point found its canvas |
| `HTMLInputElement`, `createObjectURL` | `rfd` opens by file input and saves by download |

All four are there, which is as far as evidence goes here. **What is still not
verified is that it renders and runs**: there is no browser on this machine to
put it in front of, and a page that builds, links, optimises and serves can still
come up blank. That is the one claim left to somebody who opens it.

**The `cargo build` is worth running before the browser one, and it is not the
test.** It compiles the entire interface for a target with no filesystem, no
threads and no environment, so anything reaching for *those* fails there and
nowhere else. It also fails usefully in one way already seen: while the entry
point was a stub it passed with 74 dead-code warnings, which is what a vacuous
check looks like, and with a real entry it passes with none.

**What it does not catch is a clock, and that is how the page first came up.**

```text
panicked at library/std/src/sys/time/unsupported.rs:13:9:
time not implemented on this platform
```

> **A compile check catches what does not compile, and time compiles.**
> `std::time::Instant::now()` builds for `wasm32-unknown-unknown`, links,
> survives `wasm-opt`, is served without complaint, and panics when it is called.
> A path or a thread would have failed to build. The sentence in this document
> claiming the check covered "anything of the kind" was therefore stronger than
> the check underneath it, and the difference was found by somebody opening the
> page — which is the same way the three listening tests found what no number was
> looking at.

So the rules this section states in prose are enforced in `tests/portable.rs`
instead: the clock lives in `src/clock.rs` alone, the interface names no
filesystem, environment or process, and a thread is spawned only behind a target
check. Each was verified to fail on a deliberately broken copy. It is a lint over
source text and not a proof — something reached indirectly through a dependency
would slip past it — but it covers the way this fault actually arrived, which was
somebody writing `std::time` in a file a browser compiles.

Size, measured rather than guessed: **7.3 MB of wasm, 2.4 MB gzipped**, before
`wasm-opt`. The corpus is 295 kB of that and the rest is the interface, the
search and their dependencies.

### 7.5 Nothing else OS-specific

No paths in application state — a loaded subject is bytes plus a display name.
No environment variables. No shelling out.

And **no `std::time` anywhere**, which is a correction: this section used to say
`Instant` "works on wasm under `eframe`", and it does not. `eframe` works on wasm
because it uses `web-time`, not because `std` does. The generate timing that
`Outcome::seconds` reports goes through `contrapunctus::clock`, which is `std`'s
`Instant` on the desktop and `web-time`'s — `performance.now()` — in a browser.
The audio path has no clock at all: the sample count is the clock, which is 6.1.

---

## 8. Saving and loading

**The requirement: loading a settings file produces the same fugue, note for
note.** That is achievable because everything that determines the output is
already explicit — there is no hidden state and no clock — but it has to be
written down completely and checked.

### 8.1 What determines a fugue

```
Design   subject notes · voices · key · tonic · measure · beat · compass
Layout   middles · episode_bars · link · close_at_home
Search   tier · seed
Engine   the code that turns those into notes
```

The first three go in the file. The fourth cannot, so it is **recorded and
verified** instead.

### 8.2 Format

**JSON, through `serde`**, behind the library's optional `serde` feature. The
interface enables it; the measurement binary does not, which is what keeps
readme §10.5's claim that no reported figure passes through a crate.

`Serialize` and `Deserialize` are derived on `Pitch`, `Note`, `Voice`,
`Design`, `Layout`, `Kind`, `Block` and `automaton::Tier`, each behind
`cfg_attr` so the core still compiles with no dependency at all.

```json
{
  "format": 1,
  "engine": "0.1.1",
  "design": {
    "subject": { "notes": [ { "onset": 120, "dur": 120, "pitch": { "step": 28, "alter": 0 }, "attack": true } ] },
    "voices": 3,
    "key": [0, 0, -1, 0, 0, -1, -1],
    "tonic": 0,
    "measure": 960,
    "beat": 240,
    "compass": [[33, 45], [28, 40], [21, 33]]
  },
  "layout": {
    "middles": [4, 5, 3],
    "episode_bars": 3,
    "link": [1, 1],
    "close_at_home": true,
    "rerolls": [[11512035283069334625, 1]]
  },
  "tier": "full",
  "seed": 24301,
  "fingerprint": 11512035283069334625
}
```

The subject is stored as **notes**, in the library's own units, so one imported
from anywhere round-trips exactly and a file does not depend on the corpus being
present to open.

`rerolls` is 4.2's per-block reroll, and it is here for a reason worth stating:
a block asked for again is **a parameter of the piece**, not a passing state of
the interface. Left out of the file it would come back as a different block on
load, and the promise this whole section exists for would be false for exactly
the pieces somebody had worked on hardest. It is keyed on the block's identity —
what it is, not where it sits — so it survives an edit that inserts something
before it.

**The engine version earns its place here.** A file written by `0.1.0` and opened
by `0.1.1` says so, and that is exactly what happened: the edit path changed, the
music it writes changed with it, and files saved before it report a mismatch on
load with both numbers named. A saved file that stops reproducing can be
re-saved from the interface, or re-stamped by the `restamp_presets` tool —
**deliberately**, never as a side effect of opening one, which would be the
fingerprint quietly forgiving itself.

`format` is the format's version, bumped when a field's meaning changes rather
than when one is added: **unknown keys are ignored**, so an older build opens a
newer file with the settings it understands. A `format` *newer* than the build
reads is refused rather than half-read. Both are tested. `rerolls` was added and
did not move it, and a test opens a file without the field to say so.

### 8.3 The fingerprint

A hash over the generated notes — every `(voice, onset, dur, step, alter)`.
Written on save, recomputed on load.

- **Match** — silent. The guarantee held.
- **Mismatch** — a banner: *this file was written by engine 0.1.0 and this is
  0.2.0; the settings loaded, and the music is not the same.* With a control to
  keep the new music or to stop.

This is what makes the guarantee a fact rather than a claim. Every published
figure in §8 moved at some point because the code underneath it changed —
§8.16's rate alone moved four times, and each was correct — so a file format
that quietly assumed stability would be promising something the project's own
history says is not true.

### 8.4 What is not saved

The playhead, the scroll position, the mode toggle, the chosen MIDI port. Those
are interface state, not music, and they belong in whatever the platform offers
for preferences — `eframe`'s own persistence, which works on both targets.

---

## 9. Four voices

The control is present, showing `4`, and **disabled with a reason on hover**:

> Four voices needs three free voices at once. The search is exact up to two
> (§8.6), and refuses beyond rather than beaming. §9's solver item.

Not hidden. §2.7 predicted the wall at four free voices and §8.6 measured it at
two; half the Well-Tempered Clavier is out of reach until a CDCL solver replaces
the dynamic programme. An interface that silently omitted the option would be
concealing a fact about the program, and this repository's habit is the
opposite.

When the solver lands, the control enables and nothing else in the interface
changes — which is the test of whether this specification put the boundary in
the right place.

---

## 10. What the library still needs

Ordered by whether an interface can start without it.

| # | change | why | size |
|---|---|---|---|
| 1 | Blocks addressable one at a time, for 4.5's palette | so a plan can be built rather than derived | medium |
| 2 | A CDCL solver | **four parts sounding together**, and stretto packing | §9 |

**Item 1 is measured and it is larger than it looks.** readme §8.17 asked what a
resting voice costs and found that the search's wall moves with the number of
voices it must *choose*, not with the number sounding: four voices with one
resting costs 8 434 peak states, which is what three voices costs to the state,
and fills in the same 150 ms with nothing relaxed. So the same field that fixes
the texture also buys the voice count that readme §9 had assigned to a solver.
`compose::fill_block` already takes a `silent` argument and `compose::block_cost`
is the measurement's way in.

**Both halves of the texture are built**: `compose::resting` for the grammar's own
rule and `Layout::rests` for the choice it cannot make, with
`compose::rests_that_fit` supplying a pattern that keeps a piece under the wall.
readme §8.17 has a whole four-voice fugue — 29 bars, all four grammar checks
passing, three of thirteen blocks losing the join where the three-voice piece
loses one.

**Item 3 is narrower than it was and clearer for it.** Four voices *sounding
together* is three free voices, which is the wall itself and no arrangement of
rests moves it — the four-voice piece runs `1 1 2 2 3 3 3 ...` and never reaches
four at once. So the solver is for **texture density**, not for the voice count.
Two things were being called four voices, and only one of them was ever a solver
problem.

Everything else on this list is now done:

- **`Layout::turns`** — a rotation of the voice chain from one block on, which is
  what 4.3's voice drag turned out to be a parameter *for*. That section had said
  ghosting needed "a voice to be settable at all"; it does, and what is settable
  is the rotation rather than the lane, because the lane is derived. Keyed on
  `identities_of`, refused inside the exposition, and the ghost fades from the
  turn's predecessor because that is how far it really reaches.
- **`compose::Run`** — a resumable generate, one block per `step`. `generate` and
  `fugue` are both this loop run to completion, and that is the code rather than
  a promise: `stepping_a_run_writes_what_generating_it_would_have` compares them
  note for note. The interface fills what fits in six milliseconds of each frame
  and draws the plan filling in as it goes.
- **`identities_of`** replaced the public `identities`, which took blocks. A
  caller holding an `Outcome` would naturally pass its blocks and would get
  answers that moved whenever a lane did — so the question that can be asked
  wrongly is no longer askable. Making it private caught one real call site in
  the plan strip the moment it compiled.

- **`Layout::rerolls`** — a per-block nudge keyed on `compose::identities`, so
  4.2's reroll changes one block and survives a save. It went in the layout
  rather than beside the seed because it is a parameter of the piece; section 8
  is the argument.
- **No `&Path` required anywhere.** `kern::parse`, `refdata::parse`,
  `midi::encode` and `midi::encode_score` take and return bytes; the path forms
  are wrappers on them.
- **`embedded-corpus`** compiles the 24 fugues and the annotations in, 295 kB,
  byte-identical to the files and asserted so.
- **`settings::Settings`** — JSON through `serde`, a format version, unknown
  keys ignored, a newer format refused, and a fingerprint that makes *same file,
  same fugue* a checked fact rather than a promise.
- **`automaton::Tier`** names a tier where a `&'static [Rule]` cannot go — a
  settings file, a command line. `cli::TierArg` is a two-line wrapper on it now.

Both features are **off by default**, so the measurement binary still passes
through no crate and readme §10.5's claim about §8's figures holds.

Already done before this and needing nothing further:

- `compose::Design` / `Layout` / `Outcome` — every control in §3 is a field that
  exists.
- `compose::refill_span` — a span-preserving edit rewrites forward from the
  first affected block and leaves every bar before it alone, which a test asserts
  over the whole piece. Run to the end it is exactly `compose::fugue`, which is
  what keeps 4.2's edits reproducible from a settings file; `Problem::terminal`
  is still there and is now used only where something follows the span.
- Block seeds keyed on **what a block is**, so an edit does not reseed its
  neighbours.
- `compose::fugue` returns the notes *and* every judgement §8 can pass on them,
  so a result cannot be displayed without also being able to say what is wrong
  with it.

---

## 11. Notes for whoever builds it

**The seed is not a quality dial.** Over twelve seeds the dissonance rate is
`74.1 ± 2.3`, running `70.1` to `77.7`, every one far below Bach's `112.3`. The
seed changes *which* notes are written and barely changes *how good they are*, so
*Try a different one* is for exploring the legal set and should be presented that
way — prominent, cheap, and not framed as an improvement.

**Report a number with a yardstick or not at all.** *70 per thousand* means
nothing to anyone. *70 per thousand, where Bach averages 112* means something
immediately, and the second is barely longer.

**The one seam.** The blockwise fill leaves the automaton's state reset at each
block edge, so §8.16's piece carries about one confirmed-tier violation per
piece, at a join. The same fact is what makes editing local. Do not describe it
as a bug in the interface; the report shows it, and §8.16 explains it.

**Three listening tests have corrected this project so far** — one disagreed with
the numbers, one agreed with them, and one found something no number was looking
at (three voices resting in unison, four tenths of a second, every six seconds).
The interface's real job is to make the next one easier to get.

Two more have happened since, both on the synth rather than on the music. The
fourth corrected nothing — the timbre was described as chiptune and passed — and
is recorded anyway, because a log that keeps only the reports which found
something will read, in hindsight, as though listening always finds something.

The fifth found a real fault that no test here was looking for: **the high voices
were significantly louder than the low ones**, which was true, was mine, and was
15.7 dB of it. 6.2 has the account. It is worth noticing what kind of fault that
is. Every existing test asked whether the right *signal* came out — no clicks, no
clipping, the right duration, nothing swallowed — and every one passed, because
the signal was right. What was wrong was the relation between the signal and an
ear, and this repository had no instrument for that until the report arrived and
one was written.

---

## 12. Roadmap

Where the implementation is. `ui/` is a workspace member depending on the
library; the arrow never points back, which is what keeps readme §10.5's claim
about §8's figures checkable.

Status is one of **done**, **partial** — usable and honestly incomplete — or
**not started**. A row is only moved to *done* when something checks it.

### 12.1 Built

| section | what | evidence |
|---|---|---|
| — | `ui/` as a workspace member, `eframe` 0.36 | `cargo build --workspace` |
| 3.1 | The frame: toolbar, side panel, three stacked views | `every_view_paints` |
| 3.2 | Simple controls — subject, voices, returns, journey, strictness, reroll | as above |
| 3.3 | Advanced: layout, the middles as a list, episode length, link, close-at-home, seed | as above |
| 4.1 | The plan strip, drawn — lanes, entries, hatched episodes, key ribbon, cold outlines, tooltips | as above |
| 5.1 | Staff notation — position from `step`, ledger lines, accidentals as paths | as above |
| 7.1 | The corpus with no filesystem: all **24** subjects from `embedded::FUGUES` | `every_offered_subject_composes` |
| 9 | Four voices present and disabled, with the reason on hover | by inspection |
| 11 | Every number with its yardstick; the reroll framed as exploration | by inspection |
| 8 | Save and load, as JSON, with the fingerprint checked and a banner when it does not match | `Fidelity` is shown, not swallowed |
| 6.5 | Export MIDI, through `compose::encode` | by inspection |
| 4.2 | The plan strip's key edit — click a middle, choose where it goes | `a_local_key_change_leaves_every_other_note_alone` |
| 7.2 | One code path for files: `rfd`'s async API, on both targets | `ui/src/files.rs` mentions no `Path` |
| 6.1 | The scheduler — ticks to samples, ties merged, both directions | `the_clock_runs_both_ways`, `the_piece_lasts_what_the_tempo_says` |
| 6.2 | The built-in synth, and the sound card behind it | `a_note_begins_and_ends_at_silence`, `a_full_texture_does_not_clip` |
| 4.1, 5.2 | The playhead in both views, and a click to listen from there | the position is the callback's own sample count |
| 6.2 | Silencing a voice while listening, which is how anyone learns to follow one | `a_muted_voice_goes_quiet_and_the_rest_do_not` |
| 7 | The browser entry, and the whole interface building for `wasm32` | `cargo build --target wasm32-unknown-unknown` links, with no warnings |
| 6.4 | Rebuilding the audio stream around anything that stalls the frame | nothing — see below |
| 4.2 | Dragging an episode's edge, the link's edge, and the order of the returns | `an_edit_preserves_the_span_exactly_when_derive_says_it_does` |
| 4.2 | A live preview of the plan a drag would commit to, with what moves drawn faded | `the_preview_and_the_commit_are_the_same_function` |
| 3.2 | Choosing which voice of an imported file is the subject | `an_imported_subject_replaces_the_design` |
| 4.2 | Asking for one block to be written again, and it surviving a save | `a_rerolled_block_survives_a_round_trip` |
| 4.2 | Adding and removing a return, and the close, from the strip and the panel alike | `the_shapes_the_handles_can_reach_all_compose` |
| 5.1 | Beams, by beat and by contiguity, with stubs and flags | `a_beam_stays_inside_its_beat`, `a_duration_wants_the_beams_it_should` |
| 5.1 | Clefs, from a four-glyph SMuFL subset placed by its own geometry | `both_clefs_have_ink_in_them`, `a_clef_is_the_size_smufl_says` |
| 3.3 | Sharps and flats in text, which no font egui ships has, from the same subset | `the_accidentals_are_reachable_from_text`, `an_accidental_sits_on_the_baseline_and_a_clef_does_not` |
| 5.2 | Wheel to pan and ctrl-wheel to zoom the score, about the pointer, with the strip shading what is off the page | `a_bar_under_the_pointer_stays_under_it`, `the_wheel_pans_and_ctrl_and_the_wheel_zooms` |
| 3.2 | Importing a subject from a `**kern` file | `an_imported_subject_replaces_the_design` |
| 5.2 | The score following the playhead while it plays | by inspection |
| 3.3 | Each voice's compass, as bars dragged on a grand staff — ends, and the whole range | `a_drag_on_a_handle_moves_that_bound`, `an_end_stops_where_it_has_to` |
| 3.3 | And every arrangement a drag can reach still composing | `any_compass_the_drag_can_reach_still_composes` |
| 4.3 | A voice drag, as the rotation it really is, with the knock-on ghosted | `a_turn_rotates_its_block_and_the_tail_behind_it`, `a_turn_reaches_the_block_before_it` |
| 7.3 | A block a frame instead of a frame a piece, with the plan filling in as it goes | `stepping_a_run_writes_what_generating_it_would_have` |
| 4.4 | A voice says nothing until it has entered, so the exposition has arrivals in it | `a_voice_is_silent_until_it_has_entered`, `every_voice_sounds_in_every_block` |
| 4.4 | A rest anywhere, clicked in the lane, and the accompaniment drawn so it can be seen | `four_voices_compose_when_one_of_them_rests` |
| 3.2, 9 | **Four voices**, which asking for now sets a rest pattern for rather than refusing | `asking_for_four_voices_gives_four_voices`, `a_refusal_about_free_voices_names_its_own_cure` |
| 4.1 | The strip drawing the piece it was given rather than the controls, which have moved | `the_strip_is_given_the_piece_it_is_drawing`, `a_click_in_a_lane_rests_that_voice` |
| 4.4 | The search choosing the rests, off by default, with the finding beside the switch | `the_search_can_choose_who_rests`, `drawing_the_texture_is_off_and_at_three_voices_is_the_same_piece` |

Fifty-one tests, all headless. The interesting one is
`every_offered_subject_composes`: each of the 24 subjects is composed on the
shortest layout that is still a fugue, because a picker whose entries have not
been tried is a picker that wastes the one click a beginner is sure to make. It
costs 13 seconds in release and two and a half minutes unoptimised.

`tests/references.rs` now sweeps `ui/src` beside `src` and `docs/`, so the
section numbers this crate's doc comments cite are checked like every other.

One of them is worth naming on its own. `no_gap_swallows_every_voice_at_once`
asserts that the whole texture never falls silent inside the piece — which is the
**third listening test made mechanical**. A listener heard "0.4s long silence
breaks repeating every 3-6s" in a fugue whose every number looked right, because
every instrument in this repository measures a relation between notes that
*sound*, and a fault consisting of nothing sounding is invisible to all of them.
A scheduler is the first thing here that can see it, because it knows where the
silence is, in samples.

### 12.2 Next, in order

| # | section | what | blocked on |
|---|---|---|---|
| 1 | 4.5 | A block palette, judged by the grammar rather than gated by it | blocks addressable one at a time |
| 2 | 7.3 | Generating in a worker, so the sound survives the work | a worker build; 6.4 is the workaround until then |
| 3 | 6.3 | System MIDI out, behind a feature | a `midir` dependency, and a device to try it on |
| 4 | 4.4 | A texture that varies for a *musical* reason | a criterion, and readme §8.10 is about what those do |

**Texture is built, and four voices with it.** `compose::resting` rests a voice
until it has entered — no parameter, the derivation already said who enters when —
and `Layout::rests` rests one again afterwards, which is what reaches four. The
strip draws the accompaniment so a rest is visible, and a click toggles one.
Asking for four voices sets a pattern rather than handing over a refusal, and a
refusal that a rest *would* cure now says so in the message.

**The search choosing the rests is built, off, and measured.**
`Layout::drawn_texture` is in SEARCH with the finding beside it. It invents no
rule and it does not deliver a texture: readme §8.17 counted the legal sets and a
pattern with one more voice playing admits thousands of times more fills, so the
draw returns the densest legal one essentially always. At three voices it changes
not a note. What it buys is which voice rests where one must, and there it
replaces a heuristic with something principled.

**Item 4 is what that leaves.** A texture that thins for a musical reason needs
something that *prefers* one density to another, which is a positive criterion —
and readme §8.10 is the section about what positive criteria do under a minimiser.
It is on this list to be honest about being open, not because it is next.

**Item 2 is mostly interface work on machinery that exists**, and 4.5 argues the
one design point that matters: the palette should let an illegal plan be built
and let `form::parse`'s five verdicts say what is wrong with it, rather than
refusing the drop. A palette that permits only legal plans teaches nothing.

**The voice drag has come off this list, and not in the shape it was written
in.** It said a voice needed to be settable and called that a §9 decision about
how the library assigns lanes. It is not: what the drag wants is not a lane but a
*rotation* of the chain from one block on, and that changes nothing about how
`derive` assigns anything — every step after a turn is still one lane on from the
last. `Layout::turns` is the field, 4.3 has the three things building it found,
and the ghost was already there.

### 12.3 Known gaps in what is built

Stated rather than left to be discovered:

- **The score has no time signature and no key signature.** Both are drawn by
  every other engraver and neither is drawn here; the font subset would need a
  few more glyphs for the first, and the second is accidentals this program
  already knows how to draw.
- **The synth reads as chiptune.** That is a fair description of what was 
  built — three odd partials with
  a hard envelope, no filter, no decay, is close to what an FM chip does, and
  nothing here rounds it off. It is not a defect against the stated goal, which
  is hearing the counterpoint clearly rather than hearing it sound good, and the
  report says it works. Worth writing down because the next person to touch the
  synth should know the timbre was heard, described, and left alone deliberately
  — and because a report that confirms is evidence too, of a different and
  weaker kind than the three that corrected.
- **No metronome, and one tempo for the whole piece.** Neither has been asked
  for by anything yet.
- **The register tilt is a correction and not a model.** It makes the A-weighted
  spread across the compass small; it does not make the synth sound like an
  instrument, and it takes no account of how loudly anybody is playing it. If a
  listener still finds one voice dominating after it, the next thing to add is a
  **per-voice trim** rather than more bands — the imbalance a listener notices in
  a fugue is nearly always one *voice*, and each voice keeps to a register.
- **The audio stream is rebuilt on every compose, on the web.** It is a cure for
  6.4 and not a design: a worker doing the generation would let the sound run
  through it untouched, and 7.3 now says so. What this costs is a moment of
  silence where there should be continuity.
- **One string in the built wasm is unexplained.** `time not implemented on this
  platform` is still in the binary after the fix. It is not in a build of any
  single dependency tried alone — not `std`, `std::fs`, `mpsc`, `Mutex`,
  `parking_lot`, `eframe`, `cpal`, `rfd`, or this library — so something in the
  combination still *references* the panic, and referencing is not calling. The
  path it was reached by is gone; whether the string is reachable at all is
  unknown, and saying so is better than reporting a clean binary.
- **Generation no longer blocks the frame, and on the web that is still not
  enough.** `compose::Run` fills a block at a time and the interface spends six
  milliseconds of each frame on it, so the window stays answerable and the plan
  strip fills in as it goes. What has not changed is the arithmetic 7.3 gives:
  `cpal`'s WebAudio backend queues about 85 ms and one block is tens of
  milliseconds, so the sound can still be crowded by the work. A worker is the
  honest answer there and 6.4's stream rebuild is what stands in until there is
  one.
- **A drag previews the plan and not the notes.** `derive` is free and the search
  is not, so what a drag shows is exactly right about where every bar will be and
  says nothing about what will be in them. That is the honest half to show; the
  other half costs half a second.
- **The plan strip does not scroll, and the two views are deliberately not
  locked.** 5.2 asked for a locked scroll; building it made the reason against
  clear. The strip's whole job is to show the shape of the piece at once, and an
  overview that scrolls has stopped being an overview. So the strip fits, the
  score pans and zooms, and the strip shades the part the score is not showing.
  At sixty bars the strip will get cramped and that is a different problem from
  this one.
- **Nothing is persisted between runs.** 8.4's interface state wants `eframe`'s
  persistence feature, which is not enabled. The *music* is persisted, through 8.
- **An edit rewrites everything after it**, which is 4.2's withdrawn claim and
  the price of a settings file that reproduces. The bars after an edit are not
  faded while it happens; at three voices it is over before anything could be
  seen, and at five it will not be.
- **The compass bounds the search, not the piece.** A voice's compass is the
  domain the free voices are filled from, and a *stated* subject is given rather
  than searched — so an entry sounds where its entry puts it whether the compass
  admits those notes or not. The first draft of the widget said otherwise, in a
  doc comment and on the screen, and measuring it is what settled it: every
  arrangement a drag can produce composes, including three voices squeezed into
  one octave and the bass sitting above the soprano. The minimum span the widget
  enforces is therefore a floor on usefulness and not on legality, and it now
  says so where a reader will see it.
- **Eight faults have now been found by somebody using this, and one by
  building.** The eight: four listening reports, a clock that compiles and
  panics, an audio stream a stalled frame destroys, a settings file that did not
  reproduce what it recorded, and a zoom that read the wrong input channel. Every
  one was in something the tests were not pointed at — the ear, the browser, the
  sound card, the difference between a piece and its history, and the wheel. The
  tests here check what the program *computes*; all eight were in what it
  *does*. The zoom is the sharpest case, because its arithmetic had four tests
  and all four passed: they covered the half that was right.

  The ninth came a different way and is worth separating. Building the compass
  meant deciding what a drag must be **stopped** from producing, and asking that
  question found `realise::domain` sizing a vector from `high - low + 1` — which
  on an inverted compass goes negative, and `as usize` turns into a request for
  nine million million pitches. A `Design` read from a settings file could take
  the process down, and settings files are text a person can edit. So the route
  was neither a test nor a session at the keyboard: it was designing a
  constraint, and then asking what the library did to the values on the far side
  of it. That is a cheap question and it had not been asked before.
- **The strip used to be handed the controls, not the piece.** Between changing a
  control and pressing Compose the panel describes a fugue that does not exist
  yet, and `Strip` was given the live `Design` and `Layout` while drawing an
  `Outcome` written from older ones. It draws its lanes from the voice count and
  looks every block up by identity, so at four voices it drew a fourth lane over a
  three-voice piece — a lane whose voice had entered nowhere, so a click in it was
  refused — and indexed a thirteen-block plan against a twelve-block one, so the
  three real lanes asked for rests in the wrong blocks. `App::shown` keeps what
  wrote the piece and the strip asks that.

  It was reported as manual resting not working at four voices, and it was neither
  about resting nor about four voices: **every gesture had it, for any control
  that had moved**. It had been latent since the first edit gesture — changing the
  number of returns and then dragging one has the same fault — and stayed
  invisible because the mismatch was usually one block wide and the wrong block
  was next to the right one. The voice count is the one control that changes how
  many lanes there are, so it is the one that made the mismatch impossible to
  mistake for anything else.
- **A test can pass on the piece before the one it is about.** The import test
  asserted that taking a second voice of a file composed, and it had never been
  true: that file is a whole fugue, its lines span a sixteenth, and every one of
  them hits §2.7's wall on the first block. What satisfied the assertion was the
  piece the *previous* import left on screen, while `refused` held this one's
  explosion — both true at once, and only one of them about the voice just taken.
  Keeping the last piece and saying why is the right behaviour and the panel
  shows the refusal in warning colour. The test was reading the wrong half of it,
  and only noticed because the generate stopped being synchronous and the stale
  piece stopped being there.
- **The compass shows what is allowed and not what was used.** Nothing on the
  staff says where the voices actually went, so a compass three octaves wider
  than the music needs looks the same as one the music fills. `kern::compass`
  already measures the second thing from an `Outcome`, so drawing it inside the
  bar is a small piece of work and not a decision.
