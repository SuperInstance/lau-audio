# lau-audio

> Procedural audio from pure math. No audio files. No samples. Just sine waves, noise, and mathematics. Deterministic — same parameters always produce the same sound.

## What This Does

Procedural audio generation for the **Lau (Layered Agent-UI)** game platform. Every sound in the game world — footsteps, wind, combat, music — is generated from mathematical functions at runtime. No audio files on disk. No downloaded samples. Just code and math.

This means: smaller game size, infinite variation, and every player with the same seed hears the same world.

## The Key Idea

Most games ship gigabytes of audio files. Lau generates everything procedurally:
- A sword swing = filtered noise burst with exponential decay
- Wind = layered Perlin noise at different frequencies
- Footsteps = filtered impulses with randomized timing
- Music = sequences of synthesized waveforms

Same seed → same sound. Always. This is critical for determinism in a git-tracked game world.

## Install

```bash
cargo add lau-audio
```

## Quick Start

### Generate a Waveform

```rust
use lau_audio::{Oscillator, Waveform, AudioFormat, AudioBuffer};

let format = AudioFormat::cd_quality(); // 44100 Hz, 16-bit, mono

// Generate a 440Hz sine wave for 1 second
let buffer = Oscillator::new(Waveform::Sine, 440.0)
    .generate(&format, 1.0); // 1 second

// Save to WAV
buffer.to_wav_file("tone.wav")?;
```

### Available Waveforms

```rust
Waveform::Sine,       // Pure tone
Waveform::Square,     // Retro game sound
Waveform::Sawtooth,   // buzzy, rich harmonics
Waveform::Triangle,   // Soft, flute-like
Waveform::Noise,      // White noise
Waveform::Pulse { duty_cycle: 0.25 }, // NES-style
```

### Envelopes (ADSR)

```rust
use lau_audio::{Envelope, Oscillator, Waveform};

let env = Envelope::adsr()
    .attack(0.01)    // 10ms fade in
    .decay(0.1)      // 100ms decay
    .sustain(0.7)    // Hold at 70%
    .release(0.3);   // 300ms fade out

let sound = Oscillator::new(Waveform::Sawtooth, 220.0)
    .with_envelope(env)
    .generate(&format, 0.5);
```

### Mix Multiple Sounds

```rust
use lau_audio::Mixer;

let mut mixer = Mixer::new(&format);

// Layer sounds
mixer.add(Oscillator::new(Waveform::Sine, 440.0).generate(&format, 0.5));
mixer.add(Oscillator::new(Waveform::Sine, 554.0).generate(&format, 0.5)); // major third
mixer.add(Oscillator::new(Waveform::Sine, 660.0).generate(&format, 0.5)); // perfect fifth

let chord = mixer.mix(); // A major chord
```

### Filters

```rust
use lau_audio::Filter;

let bright = Oscillator::new(Waveform::Sawtooth, 440.0)
    .generate(&format, 1.0);

// Low-pass filter — muffle the sound
let muffled = Filter::low_pass(800.0) // cutoff at 800Hz
    .apply(&bright, &format);

// High-pass — thin out bass
let thin = Filter::high_pass(2000.0)
    .apply(&bright, &format);
```

### Game Sound Effects

```rust
use lau_audio::Sfx;

// Sword swing — noise burst with decay
let sword = Sfx::sword_swing(&format);

// Footstep — filtered click
let step = Sfx::footstep(&format, "stone"); // surface type matters

// Explosion — layered noise with resonance
let boom = Sfx::explosion(&format, 0.8); // intensity

// Door creak — frequency-modulated sine
let creak = Sfx::door_creak(&format);
```

## API Reference

### Oscillator

| Method | Description |
|--------|-------------|
| `Oscillator::new(waveform, freq)` | Create oscillator |
| `.with_envelope(env)` | Apply ADSR envelope |
| `.with_frequency_mod(lfo)` | Add vibrato/tremolo |
| `.generate(format, duration)` | Produce AudioBuffer |

### Waveform

| Variant | Sound Character |
|---------|----------------|
| `Sine` | Pure, clean |
| `Square` | Hollow, retro |
| `Sawtooth` | Bright, buzzy |
| `Triangle` | Soft, warm |
| `Noise` | Hiss, texture |
| `Pulse { duty_cycle }` | NES/chip-tune |

### Filter

| Method | Description |
|--------|-------------|
| `Filter::low_pass(cutoff)` | Remove high frequencies |
| `Filter::high_pass(cutoff)` | Remove low frequencies |
| `Filter::band_pass(low, high)` | Keep a frequency range |
| `Filter::notch(freq, q)` | Remove specific frequency |

### Sfx (Sound Effects)

| Method | Description |
|--------|-------------|
| `Sfx::sword_swing(format)` | Weapon sound |
| `Sfx::footstep(format, surface)` | Surface-dependent step |
| `Sfx::explosion(format, intensity)` | Layered boom |
| `Sfx::door_creak(format)` | Frequency-modulated creak |
| `Sfx::splash(format, size)` | Water impact |
| `Sfx::wind(format, speed)` | Continuous wind noise |

### AudioBuffer

| Method | Description |
|--------|-------------|
| `buffer.samples()` | Raw sample data |
| `buffer.duration()` | Length in seconds |
| `buffer.to_wav_file(path)` | Write WAV file |
| `buffer.normalize()` | Normalize to [-1, 1] |
| `buffer.amplify(gain)` | Volume adjustment |

## How It Works

Audio generation uses additive and subtractive synthesis:
1. **Oscillators** generate raw waveforms at specified frequencies
2. **Envelopes** shape amplitude over time (ADSR)
3. **Filters** sculpt the frequency content (biquad filters)
4. **Mixing** combines multiple sources with gain control
5. **SFX generators** compose these primitives into game sounds

All generation is deterministic: same parameters + same seed = identical output. No randomness unless explicitly seeded.

Sample rates: 22050, 44100, 48000, 96000 Hz supported. Default: CD quality (44100 Hz, 16-bit, mono).

## Testing

52 tests covering: all waveform generation, ADSR envelopes, filter frequency response, mixing gain, SFX generation, WAV serialization, sample rate conversion, edge cases (silence, clipping, DC offset).

## Part of the Lau Platform

- **lau-git-world** — Git-native game worlds
- **lau-quest** — Quest/mission system
- **lau-biome** — 10 ecological zones
- **lau-spatial** — Spatial indexing
- **lau-audio** — You are here
- **lau-scheduler** — Game loop
- **lau-memory-arena** — Entity allocator
- **lau-genealogy** — Lineage tracking
- **lau-recipe** — Crafting recipes

## License

MIT
