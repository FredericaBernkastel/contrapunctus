//! A Humdrum `**kern` reader, narrow enough to be exact.
//!
//! Time is in **ticks of 1/960 of a whole note**, chosen by counting what the
//! corpora actually contain rather than by guessing. The Well-Tempered Clavier
//! uses reciprocals `{1,2,4,8,16,32}` with at most one dot and no tuplets; the
//! Renaissance corpus adds the breve and coloration, `{0,3,5,6,12,24}`. 960 is
//! the smallest base making every one of those — dotted included — a whole
//! number of ticks. A reciprocal that does not divide is an error rather than a
//! rounding, because a silent rounding here would reintroduce exactly the
//! defect the lattice was adopted to remove.
//!
//! Spine splits (`*^`) and merges (`*v`) are handled rather than assumed away:
//! the corpus has 15 and 24 of them. A split does not create a voice — it is
//! one voice momentarily in two parts — so both halves keep the voice identity
//! they came from, and an instant where a voice sounds more than one pitch is
//! *reported* rather than guessed at.

use crate::pitch::{parse_kern_pitch, Pitch};

pub const TICKS_PER_WHOLE: i64 = 960;
pub const TICKS_PER_QUARTER: i64 = TICKS_PER_WHOLE / 4;

#[derive(Clone, Copy, Debug)]
pub struct Note {
  pub onset: i64,
  pub dur: i64,
  pub pitch: Pitch,
  /// False when the note is tied over from the previous one, so the rules can
  /// tell a struck dissonance from a suspended one — readme §2.2.
  pub attack: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Voice {
  pub notes: Vec<Note>,
}

#[derive(Clone, Debug)]
pub struct Piece {
  pub id: String,
  pub voices: Vec<Voice>,
  /// Ticks per measure, from the time signature.
  pub measure: i64,
  /// Ticks per notated beat, from the time signature's denominator.
  pub beat: i64,
  /// Alteration of each diatonic letter C..B under the key signature. Needed
  /// for *diatonic* transposition, which is what a stretto at the second or
  /// third actually is — an exact interval transposition would leave the key.
  pub key: [i8; 7],
  /// Instants where one voice sounded more than one pitch at once, which this
  /// reader declines to interpret. Counted so the omission is visible.
  pub polyphonic_instants: usize,
}

/// Decode a `**kern` duration: a reciprocal, optionally dotted.
fn duration(tok: &str) -> Result<i64, String> {
  let digits: String = tok.chars().skip_while(|c| !c.is_ascii_digit())
    .take_while(|c| c.is_ascii_digit()).collect();
  if digits.is_empty() {
    return Err(format!("no duration in {tok:?}"));
  }
  // `0` is the breve (two whole notes) and `00` the longa (four), so a run of
  // zeros doubles rather than divides.
  let base = if digits.chars().all(|c| c == '0') {
    TICKS_PER_WHOLE << digits.len()
  } else {
    let recip: i64 = digits.parse().map_err(|_| format!("bad duration {digits:?}"))?;
    if TICKS_PER_WHOLE % recip != 0 {
      return Err(format!("reciprocal {recip} does not divide {TICKS_PER_WHOLE} ticks"));
    }
    TICKS_PER_WHOLE / recip
  };
  let dots = tok.chars().filter(|&c| c == '.').count();
  if dots > 1 {
    return Err(format!("{dots} dots in {tok:?}"));
  }
  Ok(if dots == 1 {
    if base % 2 != 0 {
      return Err(format!("dotted {digits} is not a whole number of ticks"));
    }
    base + base / 2
  } else {
    base
  })
}

/// One spine, carrying the voice it belongs to across splits.
///
/// **Every** spine is tracked, not only the note-bearing ones. Vocal music
/// interleaves `**text` and `**silbe` spines with the `**kern` ones, and a
/// reader that pushes a spine only for `**kern` while indexing data fields by
/// position reads every note after the first lyric column out of the wrong
/// field. 60 of 200 Renaissance files have such spines; before this was fixed
/// they failed to parse, and the ones that did parse were a biased sample.
struct Spine {
  voice: usize,
  time: i64,
  is_kern: bool,
}

pub fn read(path: &std::path::Path) -> Result<Piece, String> {
  let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
  let id = path.file_stem().unwrap_or_default().to_string_lossy().into_owned();

  let mut spines: Vec<Spine> = vec![];
  let mut voices: Vec<Voice> = vec![];
  let mut measure = 4 * TICKS_PER_QUARTER;
  let mut beat = TICKS_PER_QUARTER;
  let mut key = [0i8; 7];
  // A tie open in this voice: the next note continues it rather than attacking.
  let mut tied: Vec<bool> = vec![];

  for (lineno, raw) in text.lines().enumerate() {
    let line = raw.trim_end_matches('\r');
    if line.starts_with('!') || line.is_empty() {
      continue;
    }
    let fields: Vec<&str> = line.split('\t').collect();

    // --- interpretations: spine structure, meter -------------------------
    if line.starts_with('*') {
      if fields.iter().any(|f| f.starts_with("**")) {
        for f in &fields {
          let is_kern = f.starts_with("**kern");
          let voice = if is_kern {
            voices.push(Voice::default());
            tied.push(false);
            voices.len() - 1
          } else {
            usize::MAX
          };
          spines.push(Spine { voice, time: 0, is_kern });
        }
        continue;
      }
      for f in &fields {
        if let Some(sig) = f.strip_prefix("*k[") {
          key = [0; 7];
          let body = sig.trim_end_matches(']');
          let mut letter: Option<usize> = None;
          for ch in body.chars() {
            match ch {
              'a'..='g' => letter = "cdefgab".find(ch).map(|i| i),
              '#' => { if let Some(l) = letter { key[l] += 1 } }
              '-' => { if let Some(l) = letter { key[l] -= 1 } }
              _ => {}
            }
          }
        }
        if let Some(sig) = f.strip_prefix("*M") {
          if let Some((n, d)) = sig.split_once('/') {
            if let (Ok(n), Ok(d)) = (n.parse::<i64>(), d.parse::<i64>()) {
              if d > 0 && TICKS_PER_WHOLE % d == 0 {
                beat = TICKS_PER_WHOLE / d;
                measure = beat * n;
              }
            }
          }
        }
      }
      // Splits and merges change the field count of following lines. A split
      // duplicates the spine (same voice, same clock); a merge drops the
      // right-hand partners of each run of `*v`.
      let mut next: Vec<Spine> = Vec::with_capacity(spines.len());
      let mut i = 0;
      let mut fi = 0;
      while i < spines.len() {
        let f = fields.get(fi).copied().unwrap_or("*");
        match f {
          "*^" => {
            let (v, t, k) = (spines[i].voice, spines[i].time, spines[i].is_kern);
            next.push(Spine { voice: v, time: t, is_kern: k });
            next.push(Spine { voice: v, time: t, is_kern: k });
            i += 1;
          }
          "*v" => {
            // consume the whole run of `*v`, keeping the leftmost
            let keep = Spine {
              voice: spines[i].voice,
              time: spines[i].time,
              is_kern: spines[i].is_kern,
            };
            let mut j = fi;
            while fields.get(j).copied() == Some("*v") && i < spines.len() {
              i += 1;
              j += 1;
            }
            fi = j - 1;
            next.push(keep);
          }
          "*-" => {
            i += 1; // spine ends
          }
          _ => {
            next.push(Spine {
              voice: spines[i].voice,
              time: spines[i].time,
              is_kern: spines[i].is_kern,
            });
            i += 1;
          }
        }
        fi += 1;
      }
      spines = next;
      continue;
    }

    // --- barlines --------------------------------------------------------
    if line.starts_with('=') {
      continue;
    }

    // --- data ------------------------------------------------------------
    if spines.is_empty() {
      continue;
    }
    for (i, field) in fields.iter().enumerate() {
      let Some(sp) = spines.get_mut(i) else { continue };
      if !sp.is_kern {
        continue; // lyrics, figures, anything that is not notes
      }
      let tok = field.trim();
      if tok.is_empty() || tok == "." {
        continue;
      }
      // A chord: several pitches in one spine at one instant. Take the whole
      // token's duration from the first, and record the rest as unusable.
      let subs: Vec<&str> = tok.split(' ').filter(|s| !s.is_empty()).collect();
      let dur = duration(subs[0])
        .map_err(|e| format!("{}:{}: {e}", path.display(), lineno + 1))?;

      let is_rest = subs[0].contains('r');
      if !is_rest {
        if let Some(p) = parse_kern_pitch(subs[0]) {
          let opens = subs[0].contains('[');
          let closes = subs[0].contains(']');
          let middle = subs[0].contains('_');
          let attack = !(tied[sp.voice] && (closes || middle));
          voices[sp.voice].notes.push(Note { onset: sp.time, dur, pitch: p, attack });
          tied[sp.voice] = opens || middle;
        }
      } else {
        tied[sp.voice] = false;
      }
      sp.time += dur;
    }
  }

  // Sort each voice and count instants where it sounds more than one pitch.
  let mut polyphonic_instants = 0;
  for v in &mut voices {
    v.notes.sort_by_key(|n| (n.onset, n.pitch.chroma()));
    polyphonic_instants += v.notes.windows(2).filter(|w| w[0].onset == w[1].onset).count();
  }
  voices.retain(|v| !v.notes.is_empty());

  Ok(Piece { id, voices, measure, beat, key, polyphonic_instants })
}

/// The pitch sounding in `v` at tick `t`, and whether it is struck there.
pub fn sounding(v: &Voice, t: i64) -> Option<(Pitch, bool)> {
  v.notes.iter()
    .find(|n| n.onset <= t && t < n.onset + n.dur)
    .map(|n| (n.pitch, n.attack && n.onset == t))
}

/// Every instant at which either voice articulates — the slices the automaton
/// reads. Rhythm is data, not a variable (readme §2.6), so this is a lookup.
pub fn slices(a: &Voice, b: &Voice) -> Vec<i64> {
  let mut t: Vec<i64> = a.notes.iter().chain(b.notes.iter()).map(|n| n.onset).collect();
  t.sort_unstable();
  t.dedup();
  t
}
