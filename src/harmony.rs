//! §2.3, built at last: harmony as a second automaton.
//!
//! Two independent findings converged on this file. §12.5 showed that a
//! harmonic constraint is the only one Bach *satisfies* while arbitrary
//! placements *violate* it — the property the contrapuntal rules lack — but
//! that a bare chord-membership test tops out at 78% on Bach, because the
//! missing 22% is suspensions and passing tones, which are non-chord tones by
//! design. §13.3 showed that a purely contrapuntal objective penalises the
//! perfect fifth, and a fugal answer *is* at the fifth, so no contrapuntal
//! measure can serve as a design objective.
//!
//! Both need the same thing: a prevailing harmony, and a rule for what may
//! sound against it. That is what this provides.
//!
//! The analysis is deliberately the simplest defensible one — segment at the
//! notated beat, score every candidate chord by duration-weighted membership,
//! take the best. It is not a theory of tonal function; §13.4 wanted the
//! *relationship*, and the relationship is what a chord label carries.

use crate::{kern, kern::Voice, pitch::Pitch};

/// Chord qualities as pitch-class masks on a root of 0, with a name.
pub const QUALITIES: [(&str, u16); 9] = [
  ("", 0b0000_1001_0001),      // major triad   0 4 7
  ("m", 0b0000_1000_1001),     // minor triad   0 3 7
  ("dim", 0b0000_0100_1001),   // diminished    0 3 6
  ("aug", 0b0001_0001_0001),   // augmented     0 4 8
  ("7", 0b0100_1001_0001),     // dominant 7    0 4 7 10
  ("m7", 0b0100_1000_1001),    // minor 7       0 3 7 10
  ("ø7", 0b0100_0100_1001),    // half-dim 7    0 3 6 10
  ("o7", 0b0010_0100_1001),    // dim 7         0 3 6 9
  ("M7", 0b1000_1001_0001),    // major 7       0 4 7 11
];

const NAMES: [&str; 12] = ["C", "C#", "D", "E-", "E", "F", "F#", "G", "A-", "A", "B-", "B"];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Chord {
  pub root: u8,
  pub quality: usize,
}

impl Chord {
  pub fn mask(&self) -> u16 {
    let m = QUALITIES[self.quality].1 as u32;
    ((((m << self.root as u32) | (m >> (12 - self.root as u32))) & 0xFFF) as u16) as u16
  }
  pub fn contains(&self, p: Pitch) -> bool {
    self.mask() & (1 << p.chroma().rem_euclid(12)) != 0
  }
  pub fn name(&self) -> String {
    format!("{}{}", NAMES[self.root as usize], QUALITIES[self.quality].0)
  }
}

#[derive(Clone, Debug)]
pub struct Segment {
  pub start: i64,
  pub end: i64,
  pub chord: Option<Chord>,
  /// Fraction of the segment's sounding weight that the chord explains.
  pub fit: f64,
}

/// Sounding pitches in `[t0, t1)` with the total time each is held, which is
/// the weight a chord candidate is scored against. Duration matters: a
/// semiquaver passing note should not outvote a minim.
fn weights(voices: &[Voice], t0: i64, t1: i64) -> Vec<(Pitch, i64)> {
  let mut out: Vec<(Pitch, i64)> = vec![];
  for v in voices {
    for n in &v.notes {
      let a = n.onset.max(t0);
      let b = (n.onset + n.dur).min(t1);
      if b > a {
        match out.iter_mut().find(|(p, _)| p.chroma().rem_euclid(12) == n.pitch.chroma().rem_euclid(12)) {
          Some((_, w)) => *w += b - a,
          None => out.push((n.pitch, b - a)),
        }
      }
    }
  }
  out
}

/// The chord that best explains one span.
pub fn best_chord(voices: &[Voice], t0: i64, t1: i64) -> (Option<Chord>, f64) {
  let w = weights(voices, t0, t1);
  let total: i64 = w.iter().map(|(_, x)| *x).sum();
  if total == 0 {
    return (None, 0.0);
  }
  let mut best: Option<(Chord, i64)> = None;
  for root in 0..12u8 {
    for quality in 0..QUALITIES.len() {
      let c = Chord { root, quality };
      let hit: i64 = w.iter().filter(|(p, _)| c.contains(*p)).map(|(_, x)| *x).sum();
      // Prefer the simpler chord on ties: triads are listed before sevenths, so
      // a plain `>` keeps the first (simplest) winner.
      if best.map_or(true, |(_, h)| hit > h) {
        best = Some((c, hit));
      }
    }
  }
  match best {
    Some((c, hit)) => (Some(c), hit as f64 / total as f64),
    None => (None, 0.0),
  }
}

/// Segment a texture at the notated beat and analyse each span.
pub fn analyse(voices: &[Voice], beat: i64, end: i64) -> Vec<Segment> {
  let mut out = vec![];
  let mut t = 0;
  while t < end {
    let (chord, fit) = best_chord(voices, t, t + beat);
    out.push(Segment { start: t, end: t + beat, chord, fit });
    t += beat;
  }
  out
}

/// How a non-chord tone is treated.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Nct {
  Suspension,
  Passing,
  Neighbour,
  Appoggiatura,
  Escape,
  /// Leapt into and leapt away from — the genuinely unexplained case.
  Untreated,
}

/// Classify one note against the harmony prevailing where it sounds.
fn classify(v: &Voice, i: usize, seg: &Segment) -> Option<Nct> {
  let n = v.notes[i];
  let c = seg.chord?;
  if c.contains(n.pitch) {
    return None; // a chord tone owes nothing
  }
  let prev = i.checked_sub(1).map(|j| v.notes[j].pitch);
  let next = v.notes.get(i + 1).map(|m| m.pitch);
  let step = |a: Option<Pitch>| {
    a.map(|x| (x.step - n.pitch.step).abs() == 1).unwrap_or(false)
  };
  let in_step = step(prev);
  let out_step = step(next);
  // A note at the end of its voice is not left at all, so nothing is owed.
  if next.is_none() {
    return None;
  }
  Some(match (n.attack, in_step, out_step) {
    (false, _, _) => Nct::Suspension, // tied over from the previous harmony
    (true, true, true) => {
      let (a, b) = (prev.unwrap().step, next.unwrap().step);
      if (n.pitch.step - a).signum() == (b - n.pitch.step).signum() {
        Nct::Passing
      } else {
        Nct::Neighbour
      }
    }
    (true, false, true) => Nct::Appoggiatura,
    (true, true, false) => Nct::Escape,
    (true, false, false) => Nct::Untreated,
  })
}

#[derive(Default, Debug, Clone)]
pub struct Report {
  pub chord_tones: usize,
  pub suspension: usize,
  pub passing: usize,
  pub neighbour: usize,
  pub appoggiatura: usize,
  pub escape: usize,
  pub untreated: usize,
  /// Mean fit of the chord chosen for each segment.
  pub mean_fit: f64,
}

impl Report {
  pub fn total(&self) -> usize {
    self.chord_tones + self.explained_ncts() + self.untreated
  }
  pub fn explained_ncts(&self) -> usize {
    self.suspension + self.passing + self.neighbour + self.appoggiatura + self.escape
  }
  /// The measure §12.5 asked for: everything a chord explains, either as a
  /// chord tone or as a properly treated dissonance against it.
  pub fn explained(&self) -> f64 {
    let t = self.total();
    if t == 0 { 0.0 } else { (self.chord_tones + self.explained_ncts()) as f64 / t as f64 }
  }
  pub fn merge(&mut self, o: &Report) {
    self.chord_tones += o.chord_tones;
    self.suspension += o.suspension;
    self.passing += o.passing;
    self.neighbour += o.neighbour;
    self.appoggiatura += o.appoggiatura;
    self.escape += o.escape;
    self.untreated += o.untreated;
  }
}

/// Analyse a texture and account for every note.
pub fn report(voices: &[Voice], beat: i64) -> Report {
  let end = voices
    .iter()
    .flat_map(|v| v.notes.iter().map(|n| n.onset + n.dur))
    .max()
    .unwrap_or(0);
  if end == 0 || beat == 0 {
    return Report::default();
  }
  let segs = analyse(voices, beat, end);
  let mut r = Report::default();
  let mut fits = 0.0;
  for s in &segs {
    fits += s.fit;
  }
  r.mean_fit = fits / segs.len().max(1) as f64;

  for v in voices {
    for (i, n) in v.notes.iter().enumerate() {
      let si = (n.onset / beat) as usize;
      let Some(seg) = segs.get(si) else { continue };
      match classify(v, i, seg) {
        None => r.chord_tones += 1,
        Some(Nct::Suspension) => r.suspension += 1,
        Some(Nct::Passing) => r.passing += 1,
        Some(Nct::Neighbour) => r.neighbour += 1,
        Some(Nct::Appoggiatura) => r.appoggiatura += 1,
        Some(Nct::Escape) => r.escape += 1,
        Some(Nct::Untreated) => r.untreated += 1,
      }
    }
  }
  r
}

/// A functional progression automaton over scale degrees in a key.
///
/// The state is the degree of the prevailing chord; the edges are the standard
/// functional moves. This is the part of §2.3 that makes a cadence a *labelled
/// path* rather than a coincidence — `V → I` is an edge with a name, and a
/// progression that never reaches it never cadences.
pub fn degree_of(c: Chord, tonic: u8) -> u8 {
  ((c.root as i16 - tonic as i16).rem_euclid(12)) as u8
}

/// Does `a → b` (as semitone degrees above the tonic) follow a standard
/// functional progression? Root motion by fourth, fifth, second or third, which
/// is the classical account, plus repetition.
pub fn progression_ok(a: u8, b: u8) -> bool {
  let d = (b as i16 - a as i16).rem_euclid(12);
  matches!(d, 0 | 5 | 7 | 2 | 10 | 3 | 4 | 8 | 9)
}

/// The cadential figure: dominant to tonic.
pub fn is_cadence(a: Chord, b: Chord, tonic: u8) -> bool {
  degree_of(a, tonic) == 7 && degree_of(b, tonic) == 0
}

/// Convenience: analyse a whole parsed piece.
pub fn report_piece(p: &kern::Piece) -> Report {
  report(&p.voices, p.beat)
}

// ------------------------------------------------- a real analyser ---------
//
// §16 killed the fixed-window analyser above on three counts: it identified
// annotated cadences 38% of the time against a 23% baseline, it fitted *modal*
// polyphony better than tonal — so it was measuring triadic consonance rather
// than harmony — and every effect size it reported varied elevenfold with a
// segmentation window nobody justified.
//
// The third is the one to fix first, because it is architectural. A fixed
// window *imposes* the harmonic rhythm. Segmenting at every onset and paying a
// penalty `lambda` to change chord lets the harmonic rhythm **emerge**: the
// analysis holds a chord until the evidence for changing outweighs the cost of
// doing so. The window parameter disappears and a change penalty replaces it —
// which would be no gain at all if the penalty were then fitted, so §17 sweeps
// it and reports the whole curve instead of choosing a value.
//
// The other two are addressed by scoring a chord properly rather than counting
// membership: wrong notes are penalised rather than merely not rewarded, the
// bass is privileged because it carries the root, and metrically strong notes
// weigh more than ornamental ones.

/// Weight of one sounding note: how long it is held, doubled if it is struck on
/// a beat. An ornamental semiquaver should not outvote a minim.
fn note_weight(n: &kern::Note, t0: i64, t1: i64, beat: i64) -> f64 {
  let held = (n.onset + n.dur).min(t1) - n.onset.max(t0);
  if held <= 0 {
    return 0.0;
  }
  let strong = beat > 0 && n.onset % beat == 0;
  held as f64 * if strong { 2.0 } else { 1.0 }
}

/// Everything about one span that does not depend on which chord is proposed:
/// what sounds, how much it weighs, and what the bass is.
///
/// Gathered once and reused across all 108 candidates. Re-deriving it inside the
/// chord loop is the same answer and a hundred times the work, which turned the
/// step-5 corpus run from seconds into ten minutes before it was noticed.
struct Span {
  notes: Vec<(Pitch, f64)>,
  total: f64,
  bass: Option<Pitch>,
}

fn span(voices: &[Voice], t0: i64, t1: i64, beat: i64) -> Span {
  let mut sp = Span { notes: vec![], total: 0.0, bass: None };
  let mut lowest = i16::MAX;
  for v in voices {
    for n in &v.notes {
      if n.onset >= t1 || n.onset + n.dur <= t0 {
        continue;
      }
      let w = note_weight(n, t0, t1, beat);
      if w <= 0.0 {
        continue;
      }
      sp.total += w;
      sp.notes.push((n.pitch, w));
      let ch = n.pitch.chroma();
      if ch < lowest {
        lowest = ch;
        sp.bass = Some(n.pitch);
      }
    }
  }
  sp
}

/// How well a chord explains one span: chord tones earn their weight, foreign
/// notes lose it, and the bass earns extra for being the root.
fn observation(sp: &Span, c: Chord) -> f64 {
  if sp.total <= 0.0 {
    return 0.0;
  }
  let score: f64 = sp.notes.iter().map(|&(p, w)| if c.contains(p) { w } else { -w }).sum();
  let mut s = score / sp.total;
  if let Some(p) = sp.bass {
    // the lowest voice carries the root: a strong cue, worth a fifth of the
    // whole segment's evidence
    if p.chroma().rem_euclid(12) as u8 == c.root {
      s += 0.2;
    }
  }
  s
}

/// Analyse by Viterbi over every onset, with `lambda` charged for each change
/// of chord. `lambda = 0` re-chooses freely at every note; large `lambda`
/// forces one chord for the piece.
///
/// Linear in the number of chords rather than quadratic: the transition cost is
/// zero to stay and `lambda` to move, so the best predecessor for any chord is
/// either itself or the best of the whole previous column.
pub fn analyse_viterbi(voices: &[Voice], beat: i64, lambda: f64) -> Vec<Segment> {
  let mut times: Vec<i64> = voices.iter().flat_map(|v| v.notes.iter().map(|n| n.onset)).collect();
  times.sort_unstable();
  times.dedup();
  if times.is_empty() {
    return vec![];
  }
  let end = voices.iter().flat_map(|v| v.notes.iter().map(|n| n.onset + n.dur)).max().unwrap_or(0);
  times.push(end);

  let chords: Vec<Chord> =
    (0..12u8).flat_map(|r| (0..QUALITIES.len()).map(move |q| Chord { root: r, quality: q })).collect();
  let n = chords.len();
  let steps = times.len() - 1;

  let mut best = vec![f64::NEG_INFINITY; n];
  let mut back: Vec<Vec<u16>> = Vec::with_capacity(steps);
  let mut obs_all: Vec<Vec<f64>> = Vec::with_capacity(steps);

  for s in 0..steps {
    let (t0, t1) = (times[s], times[s + 1]);
    let sp = span(voices, t0, t1, beat);
    let obs: Vec<f64> = chords.iter().map(|&c| observation(&sp, c)).collect();
    let mut cur = vec![f64::NEG_INFINITY; n];
    let mut bk = vec![0u16; n];
    if s == 0 {
      cur.copy_from_slice(&obs);
    } else {
      let (mut gi, mut gv) = (0usize, f64::NEG_INFINITY);
      for (i, &v) in best.iter().enumerate() {
        if v > gv {
          gv = v;
          gi = i;
        }
      }
      for j in 0..n {
        let stay = best[j];
        let moved = gv - lambda;
        if stay >= moved {
          cur[j] = stay + obs[j];
          bk[j] = j as u16;
        } else {
          cur[j] = moved + obs[j];
          bk[j] = gi as u16;
        }
      }
    }
    best = cur;
    back.push(bk);
    obs_all.push(obs);
  }

  // trace back
  let mut j = best.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(i, _)| i).unwrap_or(0);
  let mut path = vec![0usize; steps];
  for s in (0..steps).rev() {
    path[s] = j;
    j = back[s][j] as usize;
  }

  (0..steps)
    .map(|s| Segment {
      start: times[s],
      end: times[s + 1],
      chord: Some(chords[path[s]]),
      // map [-1,1] onto [0,1]; the bass bonus can push the raw score past
      // 1, so clamp rather than report a 'fraction' greater than one
      fit: ((obs_all[s][path[s]] + 1.0) / 2.0).clamp(0.0, 1.0),
    })
    .collect()
}

/// Mean harmonic rhythm of an analysis, in ticks per chord — the quantity the
/// fixed window used to dictate and this now reports as an outcome.
pub fn harmonic_rhythm(segs: &[Segment]) -> f64 {
  if segs.is_empty() {
    return 0.0;
  }
  let mut changes = 1usize;
  for w in segs.windows(2) {
    if w[0].chord != w[1].chord {
      changes += 1;
    }
  }
  (segs.last().unwrap().end - segs[0].start) as f64 / changes as f64
}

/// The §14 note-accounting, over a Viterbi analysis instead of a fixed window.
pub fn report_viterbi(voices: &[Voice], beat: i64, lambda: f64) -> Report {
  let segs = analyse_viterbi(voices, beat, lambda);
  let mut r = Report::default();
  if segs.is_empty() {
    return r;
  }
  r.mean_fit = segs.iter().map(|s| s.fit).sum::<f64>() / segs.len() as f64;
  for v in voices {
    for (i, n) in v.notes.iter().enumerate() {
      let Some(seg) = segs.iter().find(|s| s.start <= n.onset && n.onset < s.end) else { continue };
      match classify(v, i, seg) {
        None => r.chord_tones += 1,
        Some(Nct::Suspension) => r.suspension += 1,
        Some(Nct::Passing) => r.passing += 1,
        Some(Nct::Neighbour) => r.neighbour += 1,
        Some(Nct::Appoggiatura) => r.appoggiatura += 1,
        Some(Nct::Escape) => r.escape += 1,
        Some(Nct::Untreated) => r.untreated += 1,
      }
    }
  }
  r
}

/// The fraction of chord *changes* that are standard functional progressions.
///
/// This is the test the modal control actually wants. Renaissance polyphony is
/// genuinely more triadic than Bach, so a chord-fit comparison flatters it
/// whatever the analyser does — that is a fact about the music. What should
/// separate tonal from modal is not whether chords can be *named* but whether
/// they *succeed each other functionally*. It is also the first thing to
/// exercise `progression_ok`, written in §14 and untested until now.
pub fn functional_rate(segs: &[Segment]) -> (usize, usize) {
  let mut chords: Vec<Chord> = vec![];
  for s in segs {
    if let Some(c) = s.chord {
      if chords.last() != Some(&c) {
        chords.push(c);
      }
    }
  }
  let mut ok = 0;
  for w in chords.windows(2) {
    if progression_ok(w[0].root, w[1].root) {
      ok += 1;
    }
  }
  (ok, chords.len().saturating_sub(1))
}
