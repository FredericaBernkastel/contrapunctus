# Contrapunctus Workbench — interface specification

A desktop and browser interface over the `contrapunctus` library, in [egui] 0.36.
[`ui-sketch.html`](ui-sketch.html) is the visual sketch this specifies; open it
beside this document.

[egui]: https://github.com/emilk/egui

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

**The centrepiece is the plan strip, not the score.** Three lanes, one per voice,
the theme drawn solid where it sounds and hatched where it is away, with the key
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
│ THE TUNE      │  PLAN        ← the centrepiece, editable             │
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
| Subject picker | The tune | `Design::subject` |
| Voice count | How many voices | `Design::voices` |
| Return count | Times the tune comes back | `Layout::middles.len()` |
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
| drag a middle-entry block **horizontally** past a neighbour | reorder the journey | `middles` reordered |
| click a block's key chip | choose the degree | `middles[k]` |
| drag an episode's **right edge** | lengthen or shorten every episode | `episode_bars` |
| drag the exposition link's edge | its length, or off at zero | `link` |
| toggle the final block | close at home or stop after the last middle | `close_at_home` |
| double-click a block | refill just that block with a new seed | `seed` for that block |

**Two classes of edit, and the interface must not blur them.**

*Span-preserving* — a key change, a voice change, a per-block reroll. The piece
stays the same length, `compose::refill` rewrites that block alone, and
**every other note stays exactly where it was**. Sub-100 ms; repaint immediately.

*Span-changing* — episode length, adding or removing a middle, toggling the
close. Everything after moves in time. `compose::refill` refuses these by
design; the interface re-runs `compose::fugue`. Blocks after the edit are drawn
faded during the recompute, because they really are about to change and pretending
otherwise is a lie the user will notice.

### 4.3 A limit worth stating

`compose::derive` gives each block the voice **after its predecessor's**, so that
no two consecutive blocks are placed in the same voice — the rule that stopped a
voice leaping an eleventh between its own entry and the next episode's motive
(§8.16).

The consequence is that **a block's voice is not independently settable**:
dragging one block to another lane rotates the ones after it. Two honest ways to
present that:

1. **Show it.** On drag, ghost the downstream blocks in their new lanes so the
   knock-on is visible before the drop. Recommended.
2. **Change the library.** Assign voices per block with a local adjacency fix-up
   instead of a chain. This changes the generated music and is a decision for
   §9, not for the interface.

Do (1) now. Do not fake independence.

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
- **Clef**: treble for voices whose mean `step` is above middle C, bass below —
  the same mean-pitch ordering `midi::write_score` already uses for track order.

Rendered with `egui::Painter` primitives: `line_segment` for staves, stems,
beams and ledger lines; `circle_filled` scaled into an ellipse, or a small
filled convex path, for noteheads. Accidentals and clefs are the only glyphs
that want a font — either a SMuFL subset embedded with `egui::FontDefinitions`,
or drawn as paths. **Embed the glyphs**; do not rely on a system music font,
because there is not one on the web.

### 5.2 Behaviour

- Horizontal scroll locked to the plan strip, so the two views always agree.
- Entry blocks tinted behind the notes in the voice colour, so the theme is
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

### 6.2 Built-in synth — the default

`cpal` for the device, a hand-written voice per part. Three voices of
counterpoint need nothing richer than a triangle or a stack of two or three sine
partials, with a short attack and release so entries do not click.

- Works on desktop and on the web: `cpal` targets WebAudio on
  `wasm32-unknown-unknown`.
- No external dependency on what the user has installed.
- The point of this tool is hearing the counterpoint **clearly**, not hearing it
  sound good. Those are different goals and only the first is ours.

### 6.3 System MIDI out — optional

`midir`, which also targets Web MIDI on wasm.

- Ports enumerated into a dropdown; nothing sent until one is chosen.
- On the web, Web MIDI needs a permission prompt and is not available in every
  browser. **Absence is a disabled dropdown with a reason, never a silent
  failure.**
- Behind a cargo feature so a build without it has no dependency.

### 6.4 Export

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

The corpus itself ships as bytes: the 24 fugues embedded with `include_str!`
behind a feature, so a web build has the subject list without a network fetch.
Roughly 2 MB of `**kern`, which is acceptable; if it is not, embed only the 24
subjects rather than the whole scores.

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

This needs no threads, so it works identically on both targets. Reach for a
worker only if a five-voice block ever exceeds a frame by enough to matter, and
then reach for it on both platforms at once.

### 7.4 Nothing else OS-specific

No paths in application state — a loaded subject is bytes plus a display name.
No environment variables. No shelling out. No `std::time::Instant` in the audio
path (the sample clock is the clock); `Instant` is fine for the generate timing
that `Outcome::seconds` already reports, and works on wasm under `eframe`.

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

A small line-oriented text format, hand-written, with no serialisation
dependency — the library has exactly one dependency today and §10.6 makes a
point of it. The data is flat and small; a reader is under a hundred lines, and
the format stays legible in a diff.

```
contrapunctus/1
engine     0.1.0
voices     3
key        0 0 -1 0 0 -1 -1
tonic      0
measure    960
beat       240
compass    33 45 | 28 40 | 21 33
subject    wtc-i-02 entry 1
note       0 120 28 0 1
note       120 120 27 1 1
...
middles    4 5 3
episode    3
link       1 1
close      yes
tier       full
seed       24301
fingerprint  9f3c1a7b5e2d4408
```

- `note` is `onset dur step alter attack`, in the library's own units, so a
  subject imported from anywhere round-trips exactly.
- `subject` is provenance only, for display. The notes are the truth.
- Unknown keys are **ignored with a warning**, so a newer file opens in an older
  build with the settings it understands rather than not at all.

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
| 1 | `kern::parse`, `refdata::parse`, `midi::encode` | no filesystem on the web | two lines each |
| 2 | Embedded corpus behind a feature | a subject list without a fetch | small |
| 3 | Settings read/write | §8, and the fingerprint | ~150 lines |
| 4 | Ghosting for a voice drag | 4.3 above, so the knock-on is visible | interface only |
| 5 | A CDCL solver | four voices, and five | §9 |

Already done and needing nothing further:

- `compose::Design` / `Layout` / `Outcome` — every control in §3 is a field that
  exists.
- `compose::refill` with `Problem::terminal` — a span-preserving edit rewrites
  one block and leaves every other note alone, which a test asserts over the
  whole piece.
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
The interface's real job is to make the fourth easier to get.
