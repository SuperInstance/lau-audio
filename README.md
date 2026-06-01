# lau-audio

> Procedural audio generation for games — no audio files, all synthesized from math. Deterministic.

## What This Does

Procedural audio generation for games — no audio files, all synthesized from math. Deterministic.. Part of the PLATO/LAU ecosystem — a mathematically rigorous framework for building educational agents that learn, teach, and evolve.

## The Key Idea

This crate implements the core abstractions needed for its domain, with a focus on correctness, composability, and conservation guarantees. Every public type is serializable (serde), every algorithm is tested, and every invariant is verified.

## Install

```bash
cargo add lau-audio
```

## Quick Start

See the API Reference below for complete usage. Key entry points:

```rust
use lau_audio::*;
// See types and methods below for complete usage
```

## API Reference

```rust
pub struct AudioFormat 
pub enum Waveform 
    pub fn sample(&self, phase: f64) -> f64 
pub struct Oscillator 
    pub fn new(frequency: f64, amplitude: f64) -> Self 
    pub fn tick(&mut self, sample_rate: f64) -> AudioSample 
    pub fn set_frequency(&mut self, freq: f64) 
    pub fn set_amplitude(&mut self, amp: f64) 
pub enum EnvelopeState 
pub struct Envelope 
    pub fn new(attack: f64, decay: f64, sustain: f64, release: f64) -> Self 
    pub fn note_on(&mut self) 
    pub fn note_off(&mut self) 
    pub fn tick(&mut self, dt: f64) -> f64 
pub struct MixerChannel 
pub struct AudioMixer 
    pub fn new() -> Self 
    pub fn add_channel(&mut self) -> usize 
    pub fn mix(&self, inputs: &[AudioSample]) -> AudioSample 
pub struct AudioBuffer 
    pub fn new(format: AudioFormat, duration_secs: f64) -> Self 
    pub fn set_sample(&mut self, index: usize, sample: AudioSample) 
    pub fn get_sample(&self, index: usize) -> AudioSample 
    pub fn len(&self) -> usize  self.samples.len() }
    pub fn is_empty(&self) -> bool  self.samples.is_empty() }
    pub fn duration(&self) -> f64 
    pub fn to_i16_samples(&self) -> Vec<i16> 
pub struct SoundGenerator;
    pub fn generate_tone(freq: f64, duration: f64, format: &AudioFormat, waveform: Waveform) -> AudioBuffer 
    pub fn generate_chord(freqs: &[f64], duration: f64, format: &AudioFormat) -> AudioBuffer 
    pub fn generate_sweep(start_freq: f64, end_freq: f64, duration: f64, format: &AudioFormat) -> AudioBuffer 
pub enum MusicScale 
    pub fn frequencies(&self, root: f64, octaves: u32) -> Vec<f64> 
pub struct VibeToMusic;
    pub fn tempo_from_vibe(vibe: f64) -> f64 
    pub fn scale_from_vibe(vibe: f64) -> MusicScale 
    pub fn volume_from_vibe(vibe: f64) -> f64 
    pub fn frequency_from_vibe(vibe: f64, root: f64) -> f64 
```

## How It Works

Read the source in `src/` for full implementation details. All algorithms are documented with inline comments explaining the mathematical foundations.

## The Math

This crate implements formal mathematical constructs. See the source documentation for theorem statements and proofs of correctness.

## Testing

**52 tests** covering construction, serialization, correctness properties, edge cases, and composability with other lau-* crates.

## License

MIT
