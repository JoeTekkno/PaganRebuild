//! Cleaner sound effects and speech (option "better sound").
//!
//! The game's samples are 8 bit, mostly at 11 kHz. Played as they are, the
//! sound card's (or the mixer's) simple conversion to 44.1 kHz repeats the
//! sound's spectrum above 5.5 kHz as a harsh, metallic buzz, and the 8 bit
//! steps hiss in every quiet tail. Here each sample is made anew at
//! 44.1 kHz before it is played:
//! - the steady offset some samples have is taken out, and the first and
//!   last milliseconds are faded, so that nothing clicks;
//! - very quiet stretches (only the hiss of the 8 bit steps) are faded
//!   down gently, the sound itself is left alone;
//! - the conversion to 44.1 kHz uses a windowed sinc filter that lets
//!   through what the recording holds and nothing above it.

/// The rate everything is made for.
pub const RATE: u32 = 44_100;

/// Half the length of the conversion filter, in source samples.
const TAPS: i32 = 24;
/// What the filter lets through, as a share of the source's highest frequency.
const CUTOFF: f32 = 0.92;

/// The 8 bit sample (unsigned, at `rate`) made anew at 44.1 kHz.
pub fn enhance(rate: u32, pcm: &[u8]) -> Vec<i16> {
    if pcm.is_empty() || rate == 0 {
        return Vec::new();
    }
    let mut x: Vec<f32> = pcm.iter().map(|&b| (b as f32 - 128.0) / 128.0).collect();
    remove_offset(&mut x);
    fade_ends(&mut x, rate);
    quiet_down_hiss(&mut x, rate);
    resample(&x, rate)
        .into_iter()
        .map(|v| (v * 32767.0).round().clamp(-32768.0, 32767.0) as i16)
        .collect()
}

/// A steady offset (the middle of the sample away from 128) makes a click
/// when the sound starts and stops.
fn remove_offset(x: &mut [f32]) {
    let mean = x.iter().sum::<f32>() / x.len() as f32;
    if mean.abs() > 0.5 / 128.0 {
        for v in x.iter_mut() {
            *v -= mean;
        }
    }
}

/// The first and last three milliseconds come in and go out smoothly.
fn fade_ends(x: &mut [f32], rate: u32) {
    let n = ((rate as usize * 3) / 1000).min(x.len() / 4).max(1);
    for i in 0..n {
        let g = i as f32 / n as f32;
        x[i] *= g;
        let j = x.len() - 1 - i;
        x[j] *= g;
    }
}

/// Where the sound is no louder than the 8 bit steps themselves, it is
/// faded down: an envelope that rises fast and falls slowly opens and
/// closes a soft gate, so that the sound's own decay stays.
fn quiet_down_hiss(x: &mut [f32], rate: u32) {
    // below about -40 dB closed, above about -30 dB fully open
    const CLOSED: f32 = 1.2 / 128.0;
    const OPEN: f32 = 4.0 / 128.0;
    let rate = rate as f32;
    let attack = 1.0 - (-1.0 / (0.001 * rate)).exp();
    let release = 1.0 - (-1.0 / (0.080 * rate)).exp();
    let smooth = 1.0 - (-1.0 / (0.010 * rate)).exp();
    let (mut env, mut gain) = (0.0f32, 1.0f32);
    for v in x.iter_mut() {
        let a = v.abs();
        env += (a - env) * if a > env { attack } else { release };
        let t = ((env - CLOSED) / (OPEN - CLOSED)).clamp(0.0, 1.0);
        let want = t * t * (3.0 - 2.0 * t);
        gain += (want - gain) * smooth;
        *v *= gain;
    }
}

/// sin(pi t) / (pi t)
fn sinc(t: f32) -> f32 {
    if t.abs() < 1e-6 {
        1.0
    } else {
        let p = std::f32::consts::PI * t;
        p.sin() / p
    }
}

/// Blackman window over -1..1.
fn window(t: f32) -> f32 {
    if t.abs() >= 1.0 {
        return 0.0;
    }
    let p = std::f32::consts::PI * (t + 1.0);
    0.42 - 0.5 * p.cos() + 0.08 * (2.0 * p).cos()
}

/// Steps between two source samples the filter is worked out for (the
/// position of an output sample is rounded to the nearest of them).
const PHASES: usize = 512;

/// From `rate` to 44.1 kHz with a windowed sinc low pass. The filter's
/// values are worked out once for every phase, then each output sample is
/// a sum of products.
fn resample(x: &[f32], rate: u32) -> Vec<f32> {
    let step = rate as f64 / RATE as f64; // source samples per output sample
    // when the source is faster than the output, the filter follows the output's limit
    let fc = CUTOFF * (1.0f64.min(1.0 / step)) as f32;
    let taps = (TAPS as f32 / fc).ceil() as i32;
    let width = (2 * taps) as usize;
    // table[phase][k]: weight of source sample centre - taps + 1 + k
    let mut table = vec![0.0f32; (PHASES + 1) * width];
    for ph in 0..=PHASES {
        let frac = ph as f32 / PHASES as f32;
        let row = &mut table[ph * width..(ph + 1) * width];
        let mut norm = 0.0;
        for (k, w) in row.iter_mut().enumerate() {
            let t = (k as i32 - taps + 1) as f32 - frac;
            *w = fc * sinc(fc * t) * window(t / taps as f32);
            norm += *w;
        }
        for w in row.iter_mut() {
            *w /= norm;
        }
    }
    let n_out = ((x.len() as f64) / step).floor() as usize;
    let mut out = Vec::with_capacity(n_out);
    for i in 0..n_out {
        let pos = i as f64 * step;
        let centre = pos.floor() as i64;
        let ph = ((pos - centre as f64) * PHASES as f64).round() as usize;
        let row = &table[ph * width..(ph + 1) * width];
        let first = centre - taps as i64 + 1;
        let mut acc = 0.0f32;
        if first >= 0 && (first as usize + width) <= x.len() {
            // the usual case: all of the filter lies within the sample
            for (w, v) in row.iter().zip(&x[first as usize..first as usize + width]) {
                acc += w * v;
            }
        } else {
            for (k, w) in row.iter().enumerate() {
                let j = first + k as i64;
                if j >= 0 && (j as usize) < x.len() {
                    acc += w * x[j as usize];
                }
            }
        }
        out.push(acc);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_and_level() {
        // a sine at 1 kHz, 11025 Hz, 8 bit
        let pcm: Vec<u8> = (0..11025).map(|i| (128.0 + 100.0 * (i as f32 * 2.0 * std::f32::consts::PI * 1000.0 / 11025.0).sin()) as u8).collect();
        let out = enhance(11025, &pcm);
        assert!((out.len() as i64 - 44100).abs() < 8, "{}", out.len());
        // the level stays (about 100/128 of full scale) in the middle
        let peak = out[10000..30000].iter().map(|v| v.unsigned_abs()).max().unwrap() as f32 / 32767.0;
        assert!((peak - 100.0 / 128.0).abs() < 0.05, "{peak}");
    }

    #[test]
    fn silence_stays_silent() {
        // the hiss of one 8 bit step goes away
        let pcm: Vec<u8> = (0..5000).map(|i| if i % 3 == 0 { 129 } else { 128 }).collect();
        let out = enhance(11025, &pcm);
        assert!(out[4000..16000].iter().all(|v| v.abs() < 40), "{:?}", &out[8000..8010]);
    }
}
