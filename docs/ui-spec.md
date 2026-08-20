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
`ui/assets/clefs.otf` is two glyphs of Bravura, SMuFL's reference font: 15 kB of
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
| 1 | Ghosting for a voice drag | 4.3 above, so the knock-on is visible | needs a voice to be settable at all — 12.2 |
| 2 | A resumable `generate` | 7.3, so a fill can run a block per frame instead of blocking one | small |
| 3 | A CDCL solver | four voices, and five | §9 |

Everything else on this list is now done:

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
| 5.1 | Clefs, from a two-glyph SMuFL subset placed by its own geometry | `both_clefs_have_ink_in_them`, `a_clef_is_the_size_smufl_says` |
| 5.2 | Wheel to pan and ctrl-wheel to zoom the score, about the pointer, with the strip shading what is off the page | `a_bar_under_the_pointer_stays_under_it`, `the_view_stays_inside_the_piece` |
| 3.2 | Importing a subject from a `**kern` file | `an_imported_subject_replaces_the_design` |
| 5.2 | The score following the playhead while it plays | by inspection |

Seventeen tests, all headless. The interesting one is
`every_offered_subject_composes`: each of the 24 subjects is composed on the
shortest layout that is still a fugue, because a picker whose entries have not
been tried is a picker that wastes the one click a beginner is sure to make. It
costs 13 seconds in release and two and a half minutes unoptimised.

`tests/references.rs` now sweeps `ui/src` beside `src` and `docs/`, so the
section numbers this crate's doc comments cite are checked like every other.

One of the fifteen is worth naming on its own. `no_gap_swallows_every_voice_at_once`
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
| 1 | 7.3 | Generating in a worker, so the sound survives the work | a worker build; 6.4 is the workaround until then |
| 2 | 4.3 | A voice drag at all | there is no `Layout` field for it — see below |
| 3 | 6.3 | System MIDI out, behind a feature | a `midir` dependency, and a device to try it on |
| 4 | 3.3 | The compass as draggable ranges on a staff | nothing |

**Item 2 needs saying properly, because 4.3 assumed something that is not
there.** That section says a voice drag should ghost its knock-on rather than
fake independence — but there is nothing to drag. `derive` assigns every voice
by a chain from the first entry, and `Layout` has no field that changes it, so a
block's lane is not a parameter at all and dragging one cannot mean anything
yet. Ghosting is the *presentation* of a feature that does not exist. Making it
exist is a change to how the library assigns voices, which 4.3 itself calls a §9
decision. The row is honest about that now; it used to read as interface work.

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
- **Two faults have been found by opening the page, and none by a test.** A
  clock that compiles and panics (7.4), and an audio stream that a stalled frame
  destroys permanently (6.4). Both are in the layer no test here reaches — the
  first was linted for afterwards, the second cannot be. That is the honest score
  for the web target: everything mechanical passes, and the two real bugs came
  from somebody pressing buttons.
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
- **Generation blocks the frame** for about 0.6 s, where 7.3 calls for one block
  per frame. Doing it properly needs a resumable generate in the library —
  `compose::generate` fills every block in one call — and on the web it needs a
  worker as well, because one block is of the same order as the whole audio
  budget. 6.4 is what stands in for both until then.
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
- **Six faults have now been found by somebody using this, and none by a test.**
  Four listening reports, a clock that compiles and panics, an audio stream a
  stalled frame destroys, and a settings file that did not reproduce what it
  recorded. Every one was in something the tests were not pointed at — the ear,
  the browser, the sound card, and the difference between a piece and its
  history. The pattern is worth naming: the tests here check what the program
  computes, and each of these was about what the program *is for*.
