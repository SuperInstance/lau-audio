//! `lau-audio` — procedural audio generation for games.
//!
//! No audio files. All synthesized from math. Deterministic.
//! Same parameters → same audio buffer, always.

use std::f64::consts::{PI, TAU};

// ---------------------------------------------------------------------------
// AudioFormat
// ---------------------------------------------------------------------------

/// Describes the digital format of an audio buffer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioFormat {
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: u16,
}

impl Default for AudioFormat {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 1,
            bit_depth: 16,
        }
    }
}

// ---------------------------------------------------------------------------
// AudioSample
// ---------------------------------------------------------------------------

/// A single floating-point audio sample in [-1.0, 1.0].
pub type AudioSample = f64;

// ---------------------------------------------------------------------------
// Waveform
// ---------------------------------------------------------------------------

/// Supported oscillator waveform shapes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Noise,
}

impl Waveform {
    /// Compute a sample at the given phase in radians [0, 2π].
    pub fn sample(&self, phase: f64) -> f64 {
        match self {
            Waveform::Sine => phase.sin(),
            Waveform::Square => {
                if phase.sin() >= 0.0 { 1.0 } else { -1.0 }
            }
            Waveform::Sawtooth => {
                let p = phase % TAU;
                (p / PI) - 1.0
            }
            Waveform::Triangle => {
                let p = phase % TAU;
                if p < PI {
                    -1.0 + 2.0 * (p / PI)
                } else {
                    3.0 - 2.0 * (p / PI)
                }
            }
            Waveform::Noise => {
                let bits = phase.to_bits();
                let hash = bits.wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let raw = (hash >> 32) as u32;
                (raw as f64 / u32::MAX as f64) * 2.0 - 1.0
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Oscillator
// ---------------------------------------------------------------------------

/// A simple oscillator that generates periodic waveforms.
#[derive(Debug, Clone)]
pub struct Oscillator {
    pub frequency: f64,
    pub amplitude: f64,
    pub phase: f64,
    pub waveform: Waveform,
}

impl Oscillator {
    /// Create a new oscillator with an initial phase of 0.
    pub fn new(frequency: f64, amplitude: f64) -> Self {
        Self {
            frequency,
            amplitude,
            phase: 0.0,
            waveform: Waveform::Sine,
        }
    }

    /// Advance the oscillator by one sample and return the next sample value.
    pub fn tick(&mut self, sample_rate: f64) -> AudioSample {
        let sample = self.amplitude * self.waveform.sample(self.phase);
        self.phase += TAU * self.frequency / sample_rate;
        self.phase %= TAU;
        sample
    }

    pub fn set_frequency(&mut self, freq: f64) {
        self.frequency = freq;
    }

    pub fn set_amplitude(&mut self, amp: f64) {
        self.amplitude = amp;
    }
}

// ---------------------------------------------------------------------------
// Envelope
// ---------------------------------------------------------------------------

/// The current state of an ADSR envelope.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// A standard ADSR envelope generator.
#[derive(Debug, Clone)]
pub struct Envelope {
    pub attack: f64,
    pub decay: f64,
    pub sustain: f64,
    pub release: f64,
    pub state: EnvelopeState,
    elapsed: f64,
    level: f64,
    release_start_level: f64,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.3,
            state: EnvelopeState::Idle,
            elapsed: 0.0,
            level: 0.0,
            release_start_level: 0.0,
        }
    }
}

impl Envelope {
    pub fn new(attack: f64, decay: f64, sustain: f64, release: f64) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,
            state: EnvelopeState::Idle,
            elapsed: 0.0,
            level: 0.0,
            release_start_level: 0.0,
        }
    }

    pub fn note_on(&mut self) {
        self.state = EnvelopeState::Attack;
        self.elapsed = 0.0;
        self.level = 0.0;
    }

    pub fn note_off(&mut self) {
        if self.state != EnvelopeState::Idle && self.state != EnvelopeState::Release {
            self.release_start_level = self.level;
            self.state = EnvelopeState::Release;
            self.elapsed = 0.0;
        }
    }

    /// Advance the envelope by `dt` seconds and return the amplitude multiplier in [0, 1].
    pub fn tick(&mut self, dt: f64) -> f64 {
        self.elapsed += dt;
        match self.state {
            EnvelopeState::Idle => 0.0,
            EnvelopeState::Attack => {
                if self.elapsed >= self.attack {
                    self.level = 1.0;
                    self.state = EnvelopeState::Decay;
                    self.elapsed = 0.0;
                    1.0
                } else {
                    self.level = self.elapsed / self.attack;
                    self.level
                }
            }
            EnvelopeState::Decay => {
                if self.elapsed >= self.decay {
                    self.level = self.sustain;
                    self.state = EnvelopeState::Sustain;
                    self.elapsed = 0.0;
                    self.sustain
                } else {
                    let t = self.elapsed / self.decay;
                    self.level = 1.0 - (1.0 - self.sustain) * t;
                    self.level
                }
            }
            EnvelopeState::Sustain => self.sustain,
            EnvelopeState::Release => {
                if self.elapsed >= self.release {
                    self.level = 0.0;
                    self.state = EnvelopeState::Idle;
                    0.0
                } else {
                    let t = self.elapsed / self.release;
                    self.level = self.release_start_level * (1.0 - t);
                    self.level
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Mixer
// ---------------------------------------------------------------------------

/// A single channel in the audio mixer.
#[derive(Debug, Clone, Copy)]
pub struct MixerChannel {
    pub volume: f64,
    pub pan: f64,
    pub muted: bool,
}

impl Default for MixerChannel {
    fn default() -> Self {
        Self {
            volume: 1.0,
            pan: 0.0,
            muted: false,
        }
    }
}

/// A multi-channel audio mixer with panning support.
#[derive(Debug, Clone)]
pub struct AudioMixer {
    pub channels: Vec<MixerChannel>,
    pub master_volume: f64,
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self {
            channels: Vec::new(),
            master_volume: 1.0,
        }
    }
}

impl AudioMixer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_channel(&mut self) -> usize {
        let idx = self.channels.len();
        self.channels.push(MixerChannel::default());
        idx
    }

    /// Mix inputs from each channel. Uses a constant-power pan law.
    /// For mono output, both left and right gains are summed.
    pub fn mix(&self, inputs: &[AudioSample]) -> AudioSample {
        let mut sum = 0.0;
        for (i, &input) in inputs.iter().enumerate() {
            if let Some(ch) = self.channels.get(i) {
                if ch.muted { continue; }
                let left_gain = (PI / 4.0 * (1.0 - ch.pan)).cos();
                let right_gain = (PI / 4.0 * (1.0 - ch.pan)).sin();
                sum += input * ch.volume * (left_gain + right_gain);
            }
        }
        sum * self.master_volume
    }
}

// ---------------------------------------------------------------------------
// AudioBuffer
// ---------------------------------------------------------------------------

/// A buffer of audio samples with a known format.
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub samples: Vec<AudioSample>,
    pub format: AudioFormat,
}

impl AudioBuffer {
    pub fn new(format: AudioFormat, duration_secs: f64) -> Self {
        let len = (duration_secs * format.sample_rate as f64).ceil() as usize;
        Self { samples: vec![0.0; len], format }
    }

    pub fn set_sample(&mut self, index: usize, sample: AudioSample) {
        if let Some(s) = self.samples.get_mut(index) {
            *s = sample.clamp(-1.0, 1.0);
        }
    }

    pub fn get_sample(&self, index: usize) -> AudioSample {
        self.samples.get(index).copied().unwrap_or(0.0)
    }

    pub fn len(&self) -> usize { self.samples.len() }
    pub fn is_empty(&self) -> bool { self.samples.is_empty() }

    pub fn duration(&self) -> f64 {
        self.samples.len() as f64 / self.format.sample_rate as f64
    }

    pub fn to_i16_samples(&self) -> Vec<i16> {
        self.samples.iter().map(|&s| {
            let clamped = s.clamp(-1.0, 1.0);
            (clamped * i16::MAX as f64) as i16
        }).collect()
    }
}

// ---------------------------------------------------------------------------
// SoundGenerator
// ---------------------------------------------------------------------------

/// Utility to generate playable sounds from oscillators and envelopes.
pub struct SoundGenerator;

impl SoundGenerator {
    pub fn generate_tone(freq: f64, duration: f64, format: &AudioFormat, waveform: Waveform) -> AudioBuffer {
        let mut buf = AudioBuffer::new(*format, duration);
        let sr = format.sample_rate as f64;
        let mut osc = Oscillator::new(freq, 1.0);
        osc.waveform = waveform;
        let mut env = Envelope::new(0.005, 0.05, 0.8, 0.05);
        env.note_on();
        let dt = 1.0 / sr;
        for i in 0..buf.len() {
            buf.set_sample(i, osc.tick(sr) * env.tick(dt));
        }
        buf
    }

    pub fn generate_chord(freqs: &[f64], duration: f64, format: &AudioFormat) -> AudioBuffer {
        if freqs.is_empty() { return AudioBuffer::new(*format, duration); }
        let sr = format.sample_rate as f64;
        let mut buf = AudioBuffer::new(*format, duration);
        let dt = 1.0 / sr;
        let mut oscs: Vec<Oscillator> = freqs.iter().map(|&f| {
            let mut o = Oscillator::new(f, 1.0 / freqs.len() as f64);
            o.waveform = Waveform::Sine;
            o
        }).collect();
        let mut env = Envelope::new(0.005, 0.1, 0.7, 0.1);
        env.note_on();
        for i in 0..buf.len() {
            let mut sum = 0.0;
            for osc in oscs.iter_mut() { sum += osc.tick(sr); }
            buf.set_sample(i, sum * env.tick(dt));
        }
        buf
    }

    pub fn generate_sweep(start_freq: f64, end_freq: f64, duration: f64, format: &AudioFormat) -> AudioBuffer {
        let sr = format.sample_rate as f64;
        let mut buf = AudioBuffer::new(*format, duration);
        let mut osc = Oscillator::new(start_freq, 0.5);
        osc.waveform = Waveform::Sine;
        let total = buf.len() as f64;
        let mut env = Envelope::new(0.01, 0.02, 0.9, 0.05);
        env.note_on();
        let dt = 1.0 / sr;
        for i in 0..buf.len() {
            let t = i as f64 / total;
            osc.set_frequency(start_freq + (end_freq - start_freq) * t);
            buf.set_sample(i, osc.tick(sr) * env.tick(dt));
        }
        buf
    }
}

// ---------------------------------------------------------------------------
// MusicScale
// ---------------------------------------------------------------------------

/// Common musical scales.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MusicScale {
    Major,
    Minor,
    Pentatonic,
    Blues,
    Chromatic,
}

impl MusicScale {
    pub fn frequencies(&self, root: f64, octaves: u32) -> Vec<f64> {
        let intervals: &[u32] = match self {
            MusicScale::Major => &[0, 2, 4, 5, 7, 9, 11],
            MusicScale::Minor => &[0, 2, 3, 5, 7, 8, 10],
            MusicScale::Pentatonic => &[0, 2, 4, 7, 9],
            MusicScale::Blues => &[0, 3, 5, 6, 7, 10],
            MusicScale::Chromatic => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        };
        let mut result = Vec::new();
        for octave in 0..octaves {
            for &interval in intervals {
                let semitones = (octave * 12 + interval) as f64;
                result.push(root * 2.0_f64.powf(semitones / 12.0));
            }
        }
        result
    }
}

// ---------------------------------------------------------------------------
// VibeToMusic
// ---------------------------------------------------------------------------

/// Maps a vibe value (a scalar from -1 to 1) to musical parameters.
pub struct VibeToMusic;

impl VibeToMusic {
    /// Map vibe [-1, 1] to BPM in [60, 180].
    pub fn tempo_from_vibe(vibe: f64) -> f64 {
        let v = vibe.clamp(-1.0, 1.0);
        60.0 + ((v + 1.0) / 2.0) * 120.0
    }

    /// Negative → Minor, neutral → Pentatonic, positive → Major.
    pub fn scale_from_vibe(vibe: f64) -> MusicScale {
        if vibe < -0.33 { MusicScale::Minor }
        else if vibe > 0.33 { MusicScale::Major }
        else { MusicScale::Pentatonic }
    }

    /// Map |vibe| to volume in [0.2, 1.0].
    pub fn volume_from_vibe(vibe: f64) -> f64 {
        0.2 + vibe.clamp(-1.0, 1.0).abs() * 0.8
    }

    /// Map vibe to a frequency in the scale rooted at `root`.
    pub fn frequency_from_vibe(vibe: f64, root: f64) -> f64 {
        let scale = Self::scale_from_vibe(vibe);
        let freqs = scale.frequencies(root, 1);
        if freqs.is_empty() { return root; }
        let t = (vibe.clamp(-1.0, 1.0) + 1.0) / 2.0;
        let idx = (t * (freqs.len() - 1) as f64).round() as usize;
        freqs[idx.min(freqs.len() - 1)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- AudioFormat --
    #[test]
    fn test_audio_format_default() {
        let fmt = AudioFormat::default();
        assert_eq!(fmt.sample_rate, 44100);
        assert_eq!(fmt.channels, 1);
        assert_eq!(fmt.bit_depth, 16);
    }

    // -- Waveform --
    #[test]
    fn test_waveform_sine_known_points() {
        assert!((Waveform::Sine.sample(0.0)).abs() < 1e-15);
        assert!((Waveform::Sine.sample(PI / 2.0) - 1.0).abs() < 1e-15);
        assert!((Waveform::Sine.sample(PI)).abs() < 1e-15);
    }

    #[test]
    fn test_waveform_square_edges() {
        assert!((Waveform::Square.sample(0.1) - 1.0).abs() < 1e-15);
        assert!((Waveform::Square.sample(PI + 0.1) - (-1.0)).abs() < 1e-15);
    }

    #[test]
    fn test_waveform_sawtooth_range() {
        for i in 0..100 {
            let p = (i as f64 / 100.0) * TAU;
            let s = Waveform::Sawtooth.sample(p);
            assert!(s >= -1.0 && s <= 1.0);
        }
    }

    #[test]
    fn test_waveform_triangle_range() {
        for i in 0..100 {
            let p = (i as f64 / 100.0) * TAU;
            let s = Waveform::Triangle.sample(p);
            assert!(s >= -1.0 && s <= 1.0);
        }
    }

    #[test]
    fn test_waveform_noise_deterministic() {
        assert!((Waveform::Noise.sample(0.5) - Waveform::Noise.sample(0.5)).abs() < 1e-15);
    }

    #[test]
    fn test_waveform_noise_range() {
        for i in 0..100 {
            let s = Waveform::Noise.sample(i as f64 * 123.45);
            assert!(s >= -1.0 && s <= 1.0);
        }
    }

    #[test]
    fn test_waveform_noise_not_constant() {
        assert!((Waveform::Noise.sample(1.0) - Waveform::Noise.sample(2.0)).abs() > 1e-10);
    }

    #[test]
    fn test_waveform_phase_wraps() {
        assert!((Waveform::Sine.sample(0.0) - Waveform::Sine.sample(TAU)).abs() < 1e-15);
    }

    // -- Oscillator --
    #[test]
    fn test_oscillator_new() {
        let o = Oscillator::new(440.0, 0.5);
        assert!((o.frequency - 440.0).abs() < 1e-10);
        assert!((o.amplitude - 0.5).abs() < 1e-10);
        assert!(o.phase.abs() < 1e-10);
        assert_eq!(o.waveform, Waveform::Sine);
    }

    #[test]
    fn test_oscillator_tick_advances_phase() {
        let mut o = Oscillator::new(440.0, 0.5);
        let _ = o.tick(44100.0);
        assert!(o.phase > 0.0);
    }

    #[test]
    fn test_oscillator_setters() {
        let mut o = Oscillator::new(440.0, 0.5);
        o.set_frequency(880.0);
        o.set_amplitude(0.8);
        assert!((o.frequency - 880.0).abs() < 1e-10);
        assert!((o.amplitude - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_oscillator_phase_wraps() {
        let mut o = Oscillator::new(44100.0, 1.0);
        for _ in 0..1000 { o.tick(44100.0); }
        assert!(o.phase < TAU);
    }

    // -- Envelope --
    #[test]
    fn test_envelope_adsr_cycle() {
        let mut e = Envelope::new(0.1, 0.1, 0.5, 0.2);
        assert_eq!(e.state, EnvelopeState::Idle);
        assert!((e.tick(0.01) - 0.0).abs() < 1e-10);
        e.note_on();
        assert_eq!(e.state, EnvelopeState::Attack);
        let _ = e.tick(0.1);
        assert_eq!(e.state, EnvelopeState::Decay);
        let _ = e.tick(0.1);
        assert_eq!(e.state, EnvelopeState::Sustain);
        assert!((e.tick(0.01) - 0.5).abs() < 0.001);
        e.note_off();
        assert_eq!(e.state, EnvelopeState::Release);
        let _ = e.tick(0.2);
        assert_eq!(e.state, EnvelopeState::Idle);
    }

    #[test]
    fn test_envelope_attack_range() {
        let mut e = Envelope::new(0.1, 0.0, 1.0, 0.0);
        e.note_on();
        let half = e.tick(0.05);
        assert!(half > 0.4 && half < 0.6);
        assert!((e.tick(0.05) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_envelope_release_decays() {
        let mut e = Envelope::new(0.001, 0.001, 1.0, 0.1);
        e.note_on();
        e.tick(0.01);
        e.note_off();
        let half = e.tick(0.05);
        assert!(half > 0.4 && half < 0.6);
    }

    #[test]
    fn test_envelope_note_off_idle_noop() {
        let mut e = Envelope::default();
        e.note_off();
        assert_eq!(e.state, EnvelopeState::Idle);
    }

    // -- Mixer --
    #[test]
    fn test_mixer_add_channel() {
        let mut m = AudioMixer::new();
        assert_eq!(m.add_channel(), 0);
        assert_eq!(m.add_channel(), 1);
    }

    #[test]
    fn test_mixer_mute() {
        let mut m = AudioMixer::new();
        m.add_channel();
        m.add_channel();
        m.channels[0].muted = true;
        // Channel 0 muted, channel 1 center pan
        m.channels[1].volume = 1.0;
        m.channels[1].pan = 0.0;
        let expected = (PI / 4.0).cos() + (PI / 4.0).sin();
        let r = m.mix(&[1.0, 1.0]);
        assert!((r - expected).abs() < 0.001);
    }

    #[test]
    fn test_mixer_pan_center() {
        let mut m = AudioMixer::new();
        m.add_channel();
        let expected = (PI / 4.0).cos() + (PI / 4.0).sin();
        assert!((m.mix(&[1.0]) - expected).abs() < 0.001);
    }

    #[test]
    fn test_mixer_pan_hard_left() {
        let mut m = AudioMixer::new();
        m.add_channel();
        m.channels[0].pan = -1.0;
        assert!((m.mix(&[1.0]) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_mixer_pan_hard_right() {
        let mut m = AudioMixer::new();
        m.add_channel();
        m.channels[0].pan = 1.0;
        assert!((m.mix(&[1.0]) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_mixer_master_volume() {
        let mut m = AudioMixer::new();
        m.add_channel();
        m.master_volume = 0.5;
        let unattenuated = (PI / 4.0).cos() + (PI / 4.0).sin();
        assert!((m.mix(&[1.0]) - unattenuated * 0.5).abs() < 0.001);
    }

    #[test]
    fn test_mixer_empty_no_channels() {
        assert!((AudioMixer::new().mix(&[]) - 0.0).abs() < 1e-10);
    }

    // -- AudioBuffer --
    #[test]
    fn test_buffer_new() {
        let buf = AudioBuffer::new(AudioFormat::default(), 1.0);
        assert_eq!(buf.len(), 44100);
        assert!((buf.duration() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_buffer_get_set() {
        let mut buf = AudioBuffer::new(AudioFormat::default(), 0.001);
        buf.set_sample(0, 0.5);
        assert!((buf.get_sample(0) - 0.5).abs() < 1e-10);
        assert!((buf.get_sample(9999) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_buffer_clamp() {
        let mut buf = AudioBuffer::new(AudioFormat::default(), 0.001);
        buf.set_sample(0, 2.0);
        assert!((buf.get_sample(0) - 1.0).abs() < 1e-10);
        buf.set_sample(0, -2.0);
        assert!((buf.get_sample(0) + 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_buffer_to_i16() {
        let mut buf = AudioBuffer::new(AudioFormat::default(), 0.001);
        buf.set_sample(0, 1.0);
        buf.set_sample(1, -1.0);
        buf.set_sample(2, 0.0);
        let v = buf.to_i16_samples();
        assert_eq!(v[0], i16::MAX);
        assert_eq!(v[1], -i16::MAX);  // -32767, not i16::MIN
        assert_eq!(v[2], 0);
    }

    #[test]
    fn test_buffer_is_empty() {
        let fmt = AudioFormat::default();
        assert!(AudioBuffer::new(fmt, 0.0).is_empty());
        assert!(!AudioBuffer::new(fmt, 0.001).is_empty());
    }

    // -- SoundGenerator --
    #[test]
    fn test_generate_tone_not_empty() {
        let fmt = AudioFormat::default();
        let buf = SoundGenerator::generate_tone(440.0, 0.1, &fmt, Waveform::Sine);
        assert!(buf.len() > 0);
        assert!(buf.samples.iter().any(|&s| s != 0.0));
    }

    #[test]
    fn test_generate_tone_deterministic() {
        let fmt = AudioFormat::default();
        let a = SoundGenerator::generate_tone(440.0, 0.05, &fmt, Waveform::Sine);
        let b = SoundGenerator::generate_tone(440.0, 0.05, &fmt, Waveform::Sine);
        assert_eq!(a.samples, b.samples);
    }

    #[test]
    fn test_generate_chord_not_empty() {
        let fmt = AudioFormat::default();
        let buf = SoundGenerator::generate_chord(&[261.63, 329.63, 392.0], 0.1, &fmt);
        assert!(buf.samples.iter().any(|&s| s != 0.0));
    }

    #[test]
    fn test_generate_chord_empty_freqs() {
        let buf = SoundGenerator::generate_chord(&[], 0.1, &AudioFormat::default());
        assert!(buf.samples.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_generate_sweep_not_empty() {
        let fmt = AudioFormat::default();
        let buf = SoundGenerator::generate_sweep(200.0, 2000.0, 0.1, &fmt);
        assert!(buf.samples.iter().any(|&s| s != 0.0));
    }

    #[test]
    fn test_generate_sweep_deterministic() {
        let fmt = AudioFormat::default();
        let a = SoundGenerator::generate_sweep(200.0, 2000.0, 0.05, &fmt);
        let b = SoundGenerator::generate_sweep(200.0, 2000.0, 0.05, &fmt);
        assert_eq!(a.samples, b.samples);
    }

    // -- MusicScale --
    #[test]
    fn test_scale_major_c() {
        let f = MusicScale::Major.frequencies(261.63, 1);
        assert_eq!(f.len(), 7);
        assert!((f[0] - 261.63).abs() < 1.0);
        assert!((f[4] - 392.0).abs() < 2.0);
    }

    #[test]
    fn test_scale_minor_a() {
        let f = MusicScale::Minor.frequencies(220.0, 1);
        assert_eq!(f.len(), 7);
        assert!((f[2] - 261.63).abs() < 2.0);
    }

    #[test]
    fn test_scale_pentatonic_count() { assert_eq!(MusicScale::Pentatonic.frequencies(261.63, 1).len(), 5); }
    #[test]
    fn test_scale_blues_count() { assert_eq!(MusicScale::Blues.frequencies(261.63, 1).len(), 6); }
    #[test]
    fn test_scale_chromatic_count() { assert_eq!(MusicScale::Chromatic.frequencies(261.63, 1).len(), 12); }

    #[test]
    fn test_scale_multi_octave() {
        let f = MusicScale::Major.frequencies(261.63, 2);
        assert_eq!(f.len(), 14);
        assert!(f[7] > f[6]);
    }

    // -- VibeToMusic --
    #[test]
    fn test_tempo_min() { assert!((VibeToMusic::tempo_from_vibe(-1.0) - 60.0).abs() < 0.001); }
    #[test]
    fn test_tempo_max() { assert!((VibeToMusic::tempo_from_vibe(1.0) - 180.0).abs() < 0.001); }
    #[test]
    fn test_tempo_mid() { assert!((VibeToMusic::tempo_from_vibe(0.0) - 120.0).abs() < 0.001); }
    #[test]
    fn test_tempo_clamp() { assert!((VibeToMusic::tempo_from_vibe(-2.0) - 60.0).abs() < 0.001); }

    #[test]
    fn test_scale_from_vibe() {
        assert_eq!(VibeToMusic::scale_from_vibe(-0.5), MusicScale::Minor);
        assert_eq!(VibeToMusic::scale_from_vibe(0.0), MusicScale::Pentatonic);
        assert_eq!(VibeToMusic::scale_from_vibe(0.5), MusicScale::Major);
    }

    #[test]
    fn test_volume_from_vibe() {
        assert!((VibeToMusic::volume_from_vibe(-1.0) - 1.0).abs() < 0.001);
        assert!((VibeToMusic::volume_from_vibe(0.0) - 0.2).abs() < 0.001);
        assert!((VibeToMusic::volume_from_vibe(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_frequency_from_vibe() {
        let f = VibeToMusic::frequency_from_vibe(0.0, 261.63);
        assert!(f > 0.0);
    }

    #[test]
    fn test_frequency_from_vibe_positive() {
        let f = VibeToMusic::frequency_from_vibe(1.0, 261.63);
        assert!(f > 261.63, "positive vibe should give higher freq, got {}", f);
    }

    #[test]
    fn test_frequency_from_vibe_negative() {
        let f = VibeToMusic::frequency_from_vibe(-1.0, 261.63);
        assert!((f - 261.63).abs() < 1.0, "negative vibe should give root or nearby, got {}", f);
    }

    // -- Determinism across all generators --
    #[test]
    fn test_full_determinism() {
        let fmt = AudioFormat::default();
        let a_tone = SoundGenerator::generate_tone(440.0, 0.1, &fmt, Waveform::Sine);
        let b_tone = SoundGenerator::generate_tone(440.0, 0.1, &fmt, Waveform::Sine);
        assert_eq!(a_tone.samples, b_tone.samples);

        let a_chord = SoundGenerator::generate_chord(&[261.63, 329.63], 0.1, &fmt);
        let b_chord = SoundGenerator::generate_chord(&[261.63, 329.63], 0.1, &fmt);
        assert_eq!(a_chord.samples, b_chord.samples);

        let a_sweep = SoundGenerator::generate_sweep(200.0, 800.0, 0.1, &fmt);
        let b_sweep = SoundGenerator::generate_sweep(200.0, 800.0, 0.1, &fmt);
        assert_eq!(a_sweep.samples, b_sweep.samples);
    }

    // -- Scale frequencies are ascending --
    #[test]
    fn test_scale_frequencies_ascending() {
        for scale in &[MusicScale::Major, MusicScale::Minor, MusicScale::Pentatonic, MusicScale::Blues, MusicScale::Chromatic] {
            let f = scale.frequencies(261.63, 2);
            for w in f.windows(2) {
                assert!(w[1] > w[0], "scale frequencies not ascending: {:?} at {:?}", scale, w);
            }
        }
    }
}
