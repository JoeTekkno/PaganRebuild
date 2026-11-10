//! Stretches the picture to the window size when the zoom makes the ratio
//! uneven (for example 800 pixels onto 1280).
//!
//! The window would stretch it by repeating pixels: some come out one
//! window pixel wide, others two. When the view scrolls, that pattern
//! wanders over the landscape and the picture judders. Here every window
//! pixel takes the colours of the view pixels it covers, weighted by how
//! much of it each one covers. The pixels stay sharp (only the borders
//! between them are mixed) and all of them keep the same width.
//!
//! In the game the graphics card does this (gpuscale.rs, with the shares
//! from `taps`); `Upscale` is the same on the processor, for comparing.

/// Which source pixels make one output pixel, and with what share (of 256).
type Taps = Vec<(usize, u32)>;

/// For every output pixel: the source pixels it covers and their shares.
pub fn taps(src: usize, dst: usize) -> Vec<Taps> {
    let step = src as f64 / dst as f64;
    (0..dst)
        .map(|o| {
            let (a, b) = (o as f64 * step, (o + 1) as f64 * step);
            let mut t: Taps = Vec::new();
            let mut i = a.floor() as usize;
            while (i as f64) < b && i < src {
                let cover = (b.min(i as f64 + 1.0) - a.max(i as f64)) / step;
                let w = (cover * 256.0).round() as u32;
                if w > 0 {
                    t.push((i, w));
                }
                i += 1;
            }
            // the shares add up to exactly 256
            let sum: u32 = t.iter().map(|&(_, w)| w).sum();
            if let Some(last) = t.last_mut() {
                last.1 = (last.1 + 256).saturating_sub(sum).max(1);
            } else {
                t.push(((a as usize).min(src - 1), 256));
            }
            t
        })
        .collect()
}

#[derive(Default)]
pub struct Upscale {
    key: (usize, usize, usize, usize),
    xs: Vec<Taps>,
    ys: Vec<Taps>,
    /// After the horizontal pass: output width, source height.
    wide: Vec<u32>,
    pub out: Vec<u32>,
}

fn mix(acc: &mut [u32; 3], p: u32, w: u32) {
    acc[0] += (p >> 16 & 0xFF) * w;
    acc[1] += (p >> 8 & 0xFF) * w;
    acc[2] += (p & 0xFF) * w;
}

fn pack(acc: [u32; 3]) -> u32 {
    ((acc[0] + 128) >> 8) << 16 | ((acc[1] + 128) >> 8) << 8 | (acc[2] + 128) >> 8
}

/// One output pixel from the source pixels `t` covers (`get` reads one).
fn blend(t: &[(usize, u32)], get: impl Fn(usize) -> u32) -> u32 {
    if let [(i, _)] = t {
        return get(*i);
    }
    let mut acc = [0; 3];
    for &(i, w) in t {
        mix(&mut acc, get(i), w);
    }
    pack(acc)
}

impl Upscale {
    /// Scales `pixels` (`src` in size) to `dst`; the result is in `out`.
    pub fn run(&mut self, pixels: &[u32], src: (usize, usize), dst: (usize, usize)) {
        let key = (src.0, src.1, dst.0, dst.1);
        if self.key != key {
            self.key = key;
            self.xs = taps(src.0, dst.0);
            self.ys = taps(src.1, dst.1);
            self.wide = vec![0; dst.0 * src.1];
            self.out = vec![0; dst.0 * dst.1];
        }
        // both passes in bands of rows, one per processor core
        let cores = crate::pool::cores();
        let xs = &self.xs;
        let band = src.1.div_ceil(cores).max(1);
        crate::pool::each(self.wide.chunks_mut(band * dst.0).collect(), |bi, wide: &mut [u32]| {
            for (r, wide) in wide.chunks_mut(dst.0).enumerate() {
                let y = bi * band + r;
                let row = &pixels[y * src.0..(y + 1) * src.0];
                for (o, t) in xs.iter().enumerate() {
                    wide[o] = blend(t, |i| row[i]);
                }
            }
        });
        let (ys, wide) = (&self.ys, &self.wide);
        let band = dst.1.div_ceil(cores).max(1);
        crate::pool::each(self.out.chunks_mut(band * dst.0).collect(), |bi, out: &mut [u32]| {
            for (r, out) in out.chunks_mut(dst.0).enumerate() {
                let t = &ys[bi * band + r];
                if let [(i, _)] = t.as_slice() {
                    out.copy_from_slice(&wide[i * dst.0..(i + 1) * dst.0]);
                    continue;
                }
                for (x, px) in out.iter_mut().enumerate() {
                    *px = blend(t, |i| wide[i * dst.0 + x]);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shares_add_up_and_colours_stay() {
        for (s, d) in [(480, 1280), (800, 1280), (960, 1280), (800, 640), (500, 800)] {
            for t in taps(s, d) {
                assert_eq!(t.iter().map(|&(_, w)| w).sum::<u32>(), 256, "{s}->{d}");
                assert!(t.iter().all(|&(i, _)| i < s));
            }
        }
        let mut up = Upscale::default();
        up.run(&vec![0x123456; 800 * 500], (800, 500), (1280, 800));
        assert!(up.out.iter().all(|&p| p == 0x123456));
    }
}
