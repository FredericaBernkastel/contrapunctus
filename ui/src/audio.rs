//! The sound card — spec 6.2's other half, and the only untested file here.
//!
//! Everything that can be wrong is next door: [`crate::schedule`] turns a fugue
//! into samples and [`crate::synth`] turns samples into sound, and both are pure
//! and both have tests. What is left is opening a device and handing it a
//! buffer, which no test in this repository can exercise because a build machine
//! has no speakers. Keeping that boundary sharp is the point of the split.
//!
//! **The callback's own sample count is the position** — spec 6.1. There is no
//! timer anywhere in this program, so the playhead cannot drift from the sound:
//! it is not synchronised with it, it is read off it.
//!
//! The stream is built on the first press of Play and not before. That is what
//! makes this work in a browser as well, where an audio context may only be
//! created in response to a gesture — the gesture and the lazy build are the
//! same event, so nothing had to be arranged for it.

use std::sync::{
  atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
  mpsc::{self, Receiver, Sender},
  Arc,
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::{
  schedule::Score,
  synth::{Mix, Synth},
};

enum Cmd {
  Load(Arc<Score>),
  Seek(u64),
}

pub struct Player {
  rate: u32,
  pos: Arc<AtomicU64>,
  playing: Arc<AtomicBool>,
  /// A bit per voice, set to silence it. An atomic rather than a command,
  /// because a change to what is audible should take effect on the next buffer
  /// and never queue behind one.
  mute: Arc<AtomicU32>,
  /// The register tilt, as `f32` bits — same reasoning, same lifetime.
  tilt: Arc<AtomicU32>,
  tx: Sender<Cmd>,
  /// Dropping this stops the sound, so it is held even though nothing reads it.
  _stream: cpal::Stream,
}

impl Player {
  /// Open the default output. `Err` carries something a person can act on:
  /// spec 6.3's rule is that absence is stated, never silent.
  pub fn open() -> Result<Player, String> {
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or("no audio output device")?;
    let config = device.default_output_config().map_err(|e| format!("no output configuration: {e}"))?;
    let rate = config.sample_rate();
    let channels = config.channels() as usize;

    let pos = Arc::new(AtomicU64::new(0));
    let playing = Arc::new(AtomicBool::new(false));
    let mute = Arc::new(AtomicU32::new(0));
    let tilt = Arc::new(AtomicU32::new(Mix::default().tilt.to_bits()));
    let (tx, rx) = mpsc::channel::<Cmd>();

    let engine = Engine {
      synth: Synth::new(),
      score: Arc::new(Score::default()),
      rx,
      pos: pos.clone(),
      playing: playing.clone(),
      mute: mute.clone(),
      tilt: tilt.clone(),
    };
    let stream = build(&device, &config, channels, engine)?;
    stream.play().map_err(|e| format!("the stream would not start: {e}"))?;

    Ok(Player { rate, pos, playing, mute, tilt, tx, _stream: stream })
  }

  pub fn rate(&self) -> u32 {
    self.rate
  }

  /// Hand the callback a new piece. The position is left alone, so replacing the
  /// music under a playing head continues from where the ear already is — which
  /// is what an edit during playback should do.
  pub fn load(&self, score: Arc<Score>) {
    let _ = self.tx.send(Cmd::Load(score));
  }

  pub fn seek(&self, sample: u64) {
    let _ = self.tx.send(Cmd::Seek(sample));
  }

  pub fn set_playing(&self, on: bool) {
    self.playing.store(on, Ordering::Relaxed);
  }

  /// What to silence and how to balance the register.
  ///
  /// Nothing here changes the piece, so the playhead and the score still
  /// describe what is written rather than what is audible — which is the point
  /// of listening to two voices of three.
  pub fn set_mix(&self, mix: Mix) {
    self.mute.store(mix.mute, Ordering::Relaxed);
    self.tilt.store(mix.tilt.to_bits(), Ordering::Relaxed);
  }

  pub fn is_playing(&self) -> bool {
    self.playing.load(Ordering::Relaxed)
  }

  pub fn position(&self) -> u64 {
    self.pos.load(Ordering::Relaxed)
  }
}

/// One closure, three sample formats.
///
/// `f32` is what the synth produces and what most devices want; the integer
/// formats are a conversion at the very last step rather than a second synth.
fn build(
  device: &cpal::Device,
  config: &cpal::SupportedStreamConfig,
  channels: usize,
  mut engine: Engine,
) -> Result<cpal::Stream, String> {
  let cfg = config.config();
  let oops = |e| eprintln!("audio stream error: {e}");

  let stream = match config.sample_format() {
    cpal::SampleFormat::F32 => device.build_output_stream(
      cfg,
      move |data: &mut [f32], _: &cpal::OutputCallbackInfo| engine.fill(data, channels),
      oops,
      None,
    ),
    cpal::SampleFormat::I16 => {
      let mut buf: Vec<f32> = vec![];
      device.build_output_stream(
        cfg,
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
          buf.resize(data.len(), 0.0);
          engine.fill(&mut buf, channels);
          for (o, s) in data.iter_mut().zip(&buf) {
            *o = (s * i16::MAX as f32) as i16;
          }
        },
        oops,
        None,
      )
    }
    cpal::SampleFormat::U16 => {
      let mut buf: Vec<f32> = vec![];
      device.build_output_stream(
        cfg,
        move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
          buf.resize(data.len(), 0.0);
          engine.fill(&mut buf, channels);
          for (o, s) in data.iter_mut().zip(&buf) {
            *o = ((s * 0.5 + 0.5) * u16::MAX as f32) as u16;
          }
        },
        oops,
        None,
      )
    }
    other => return Err(format!("this device wants {other:?} samples, which is not handled")),
  };
  stream.map_err(|e| format!("the device refused a stream: {e}"))
}

/// What the audio thread owns.
///
/// Nothing here locks. Commands arrive down a channel and are drained at the top
/// of each callback; the position is an atomic the interface only reads. A mutex
/// around the score would work almost always and then, under a repaint that
/// happened to hold it, drop a buffer and click.
struct Engine {
  synth: Synth,
  score: Arc<Score>,
  rx: Receiver<Cmd>,
  pos: Arc<AtomicU64>,
  playing: Arc<AtomicBool>,
  mute: Arc<AtomicU32>,
  tilt: Arc<AtomicU32>,
}

impl Engine {
  fn fill(&mut self, data: &mut [f32], channels: usize) {
    while let Ok(cmd) = self.rx.try_recv() {
      match cmd {
        Cmd::Load(s) => {
          self.score = s;
          let at = self.pos.load(Ordering::Relaxed);
          self.synth.seek(&self.score, at);
        }
        Cmd::Seek(to) => {
          self.pos.store(to, Ordering::Relaxed);
          self.synth.seek(&self.score, to);
        }
      }
    }

    let at = self.pos.load(Ordering::Relaxed);
    if !self.playing.load(Ordering::Relaxed) || self.score.is_empty() {
      data.fill(0.0);
      return;
    }
    if at >= self.score.samples {
      // the end is a stop, not a wrap: a fugue that looped would be a different
      // claim about where it ends than the one the piece makes
      self.playing.store(false, Ordering::Relaxed);
      data.fill(0.0);
      return;
    }
    let mix = Mix {
      mute: self.mute.load(Ordering::Relaxed),
      tilt: f32::from_bits(self.tilt.load(Ordering::Relaxed)),
    };
    let next = self.synth.render(&self.score, at, data, channels, mix);
    self.pos.store(next, Ordering::Relaxed);
  }
}
