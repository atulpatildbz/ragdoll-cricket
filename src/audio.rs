//! Procedurally synthesised sound effects. Everything is generated at startup
//! into in-memory WAV files, so the game ships with no audio assets.

use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use std::f32::consts::TAU;

const RATE: u32 = 44_100;

pub struct Sfx {
    pub thud: Sound,
    pub bump: Sound,
    pub chime1: Sound,
    pub chime2: Sound,
    pub chime4: Sound,
    pub chime6: Sound,
    pub stumps: Sound,
    pub appeal: Sound,
}

impl Sfx {
    pub async fn load() -> Self {
        Self {
            thud: sound(bat_tock()).await,
            bump: sound(ground_bump()).await,
            chime1: sound(chime(&[1047.0])).await,
            chime2: sound(chime(&[880.0, 1175.0])).await,
            chime4: sound(chime(&[784.0, 988.0, 1175.0, 1568.0])).await,
            chime6: sound(fanfare()).await,
            stumps: sound(stumps_clatter()).await,
            appeal: sound(crowd_appeal()).await,
        }
    }

    pub fn play(&self, s: &Sound, volume: f32) {
        play_sound(
            s,
            PlaySoundParams {
                looped: false,
                volume,
            },
        );
    }
}

async fn sound(samples: Vec<f32>) -> Sound {
    load_sound_from_bytes(&wav(&samples))
        .await
        .expect("synthesised WAV should always decode")
}

fn wav(samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32_000.0) as i16).to_le_bytes());
    }
    out
}

fn buffer(seconds: f32) -> Vec<f32> {
    vec![0.0; (seconds * RATE as f32) as usize]
}

fn t(i: usize) -> f32 {
    i as f32 / RATE as f32
}

/// Small deterministic noise source.
struct Noise(u32);
impl Noise {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// Willow on leather: a hollow wooden knock with a sharp click.
fn bat_tock() -> Vec<f32> {
    let mut b = buffer(0.25);
    let mut n = Noise(0x1234_5678);
    for (i, s) in b.iter_mut().enumerate() {
        let t = t(i);
        let body = (TAU * 520.0 * t).sin() * 0.6 + (TAU * 1310.0 * t).sin() * 0.3;
        let click = n.next() * (-t * 400.0).exp();
        *s = (body * (-t * 28.0).exp() + click * 0.5) * 0.9;
    }
    b
}

/// Ball thumping into turf: a short low pitch drop.
fn ground_bump() -> Vec<f32> {
    let mut b = buffer(0.18);
    let mut n = Noise(0xBEEF);
    let mut phase = 0.0;
    let mut lp = 0.0;
    for (i, s) in b.iter_mut().enumerate() {
        let t = t(i);
        phase += TAU * (140.0 - 300.0 * t) / RATE as f32;
        lp += (n.next() - lp) * 0.05;
        *s = (phase.sin() * 0.7 + lp * 0.8) * (-t * 30.0).exp() * 0.6;
    }
    b
}

fn bell(b: &mut [f32], start: f32, freq: f32, amp: f32) {
    let first = (start * RATE as f32) as usize;
    for (i, s) in b.iter_mut().enumerate().skip(first) {
        let t = t(i - first);
        let tone = (TAU * freq * t).sin()
            + 0.4 * (TAU * freq * 2.01 * t).sin()
            + 0.2 * (TAU * freq * 3.98 * t).sin();
        *s += tone * amp * (-t * 5.0).exp() * (1.0 - (-t * 400.0).exp());
    }
}

/// Rising bell notes, one per entry.
fn chime(notes: &[f32]) -> Vec<f32> {
    let mut b = buffer(0.5 + notes.len() as f32 * 0.09);
    for (k, &f) in notes.iter().enumerate() {
        bell(&mut b, k as f32 * 0.09, f, 0.3);
    }
    b
}

/// A bright little brass-ish fanfare for a six.
fn fanfare() -> Vec<f32> {
    let notes = [
        (0.0, 523.0),
        (0.1, 659.0),
        (0.2, 784.0),
        (0.32, 1047.0),
        (0.32, 1319.0),
    ];
    let mut b = buffer(1.1);
    for (start, f) in notes {
        let first = (start * RATE as f32) as usize;
        for (i, s) in b.iter_mut().enumerate().skip(first) {
            let t = t(i - first);
            let saw = (0..6)
                .map(|h| (TAU * f * (h + 1) as f32 * t).sin() / (h + 1) as f32)
                .sum::<f32>();
            let env = (1.0 - (-t * 60.0).exp()) * (-t * 3.5).exp();
            *s += saw * env * 0.12;
        }
    }
    for &f in &[1568.0, 2093.0] {
        bell(&mut b, 0.32, f, 0.12);
    }
    b
}

/// Timber clattering: a few wooden knocks at random pitches.
fn stumps_clatter() -> Vec<f32> {
    let mut b = buffer(0.6);
    let mut n = Noise(0xC0FFEE);
    for (start, f) in [(0.0, 760.0), (0.045, 610.0), (0.11, 890.0), (0.2, 540.0)] {
        let first = (start * RATE as f32) as usize;
        for (i, s) in b.iter_mut().enumerate().skip(first) {
            let t = t(i - first);
            let knock = (TAU * f * t).sin() + 0.5 * (TAU * f * 2.7 * t).sin();
            *s += (knock * 0.4 + n.next() * 0.25) * (-t * 35.0).exp();
        }
    }
    b
}

/// A crowd roar that swells and falls: the stadium going up for the appeal.
fn crowd_appeal() -> Vec<f32> {
    let mut b = buffer(1.6);
    let mut n = Noise(0xA77EA1);
    let (mut lp1, mut lp2) = (0.0, 0.0);
    for (i, s) in b.iter_mut().enumerate() {
        let t = t(i);
        let x = n.next();
        lp1 += (x - lp1) * 0.12;
        lp2 += (lp1 - lp2) * 0.12;
        let band = lp1 - lp2 * 0.6;
        // Many voices: a wobbling "aah" vowel under the roar.
        let vowel = (0..5)
            .map(|v| {
                let f = 220.0 + v as f32 * 37.0 + (t * (3.0 + v as f32)).sin() * 6.0;
                (TAU * f * t).sin() * 0.08 + (TAU * f * 3.4 * t).sin() * 0.03
            })
            .sum::<f32>();
        let env = (t / 0.15).min(1.0) * (-((t - 0.25).max(0.0)) * 2.2).exp();
        *s = (band * 2.2 + vowel) * env * 0.7;
    }
    b
}
