//! Smoothing for pixel art (optional): the xBR filter by Hyllian at twice
//! the size. It looks at the 5x5 pixels around each pixel, finds edges
//! running through it and fills the four new pixels so that the steps
//! along slanted edges become smooth lines, while straight edges and
//! flat areas stay as sharp as they were.
//!
//! For speed, what several pixels need is worked out once per picture:
//! the colour differences between neighbours, and for every square of
//! 2 x 2 pixels how strongly the colours change along each of its two
//! diagonals (the four pixels of a square all decide by these sums).

/// The colour in Y, U, V, for comparing colours as the eye sees them.
fn yuv(c: u32) -> [i32; 3] {
    let (r, g, b) = (((c >> 16) & 0xFF) as i32, ((c >> 8) & 0xFF) as i32, (c & 0xFF) as i32);
    [
        (299 * r + 587 * g + 114 * b) / 1000,
        (-169 * r - 331 * g + 500 * b) / 1000,
        (500 * r - 419 * g - 81 * b) / 1000,
    ]
}

/// How different two colours look.
fn df(a: [i32; 3], b: [i32; 3]) -> i32 {
    48 * (a[0] - b[0]).abs() + 7 * (a[1] - b[1]).abs() + 6 * (a[2] - b[2]).abs()
}

/// Do two colours look the same?
fn eq(a: [i32; 3], b: [i32; 3]) -> bool {
    (a[0] - b[0]).abs() <= 48 && (a[1] - b[1]).abs() <= 7 && (a[2] - b[2]).abs() <= 6
}

/// `dst` moved towards `src` by `k`/256.
fn blend(dst: u32, src: u32, k: u32) -> u32 {
    let ch = |s: u32| {
        let (d, c) = ((dst >> s) & 0xFF, (src >> s) & 0xFF);
        ((d * (256 - k) + c * k) >> 8) << s
    };
    ch(16) | ch(8) | ch(0)
}

/// The picture with a border of two pixels all round (the edge repeated,
/// which spares checking for the edge), and what is worked out in
/// advance. Positions are indices into it; `pw` is its width.
struct Grid<'a> {
    cols: &'a [u32],
    yuvs: &'a [[i32; 3]],
    /// Difference to the pixel to the right, below, below right, below left.
    right: &'a [i32],
    down: &'a [i32],
    down_right: &'a [i32],
    down_left: &'a [i32],
    /// Per square (by its top left pixel): the change along its "/" and
    /// along its "\" diagonal, with the parallel neighbours around it.
    slash: &'a [i32],
    backslash: &'a [i32],
    pw: usize,
}

impl Grid<'_> {
    /// How different the pixels at two positions look.
    #[inline(always)]
    fn d(&self, a: usize, b: usize) -> i32 {
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        let o = hi - lo;
        if o == 1 {
            self.right[lo]
        } else if o == self.pw {
            self.down[lo]
        } else if o == self.pw + 1 {
            self.down_right[lo]
        } else if o == self.pw - 1 {
            self.down_left[lo]
        } else {
            df(self.yuvs[a], self.yuvs[b])
        }
    }

    #[inline(always)]
    fn eq(&self, a: usize, b: usize) -> bool {
        eq(self.yuvs[a], self.yuvs[b])
    }
}

/// One corner of the four new pixels (the bottom right one as written;
/// the others by passing the neighbours turned). The neighbours are
/// positions in the grid, named as in this picture:
/// ```text
///        A1 B1 C1
///     A0 A  B  C  C4
///     D0 D  E  F  F4
///     G0 G  H  I  I4
///        G5 H5 I5
/// ```
/// `ee` and `ii` are the changes across and along the diagonal E-I of the
/// square E F H I, `ef` and `eh` the differences E-F and E-H. `out` holds
/// the four new pixels, `n1`..`n3` are those this corner can change.
#[inline(always)]
fn corner(
    gr: &Grid, ee: i32, ii: i32, ef: i32, eh: i32, e: usize, i: usize, h: usize, f: usize, g: usize, c: usize, d: usize, b: usize,
    i4: usize, i5: usize, out: &mut [u32; 4], n1: usize, n2: usize, n3: usize,
) {
    let col = gr.cols;
    if ee > ii || col[e] == col[h] || col[e] == col[f] {
        return;
    }
    let px = if ef <= eh { col[f] } else { col[h] };
    let shape = || {
        (!gr.eq(f, b) && !gr.eq(h, d)) || (gr.eq(e, i) && !gr.eq(f, i4) && !gr.eq(h, i5)) || gr.eq(e, g) || gr.eq(e, c)
    };
    if ee < ii && shape() {
        let (ke, ki) = (gr.d(f, g), gr.d(h, c));
        let ex2 = col[e] != col[c] && col[b] != col[c];
        let ex3 = col[e] != col[g] && col[d] != col[g];
        let shallow = (ke << 1) <= ki && ex3;
        let steep = ke >= (ki << 1) && ex2;
        if shallow && steep {
            out[n3] = blend(out[n3], px, 224);
            out[n2] = blend(out[n2], px, 64);
            out[n1] = out[n2];
        } else if shallow {
            out[n3] = blend(out[n3], px, 192);
            out[n2] = blend(out[n2], px, 64);
        } else if steep {
            out[n3] = blend(out[n3], px, 192);
            out[n1] = blend(out[n1], px, 64);
        } else {
            out[n3] = blend(out[n3], px, 128);
        }
    } else {
        out[n3] = blend(out[n3], px, 64);
    }
}

/// Buffers kept from picture to picture.
#[derive(Default)]
struct Buffers {
    cols: Vec<u32>,
    yuvs: Vec<[i32; 3]>,
    right: Vec<i32>,
    down: Vec<i32>,
    down_right: Vec<i32>,
    down_left: Vec<i32>,
    slash: Vec<i32>,
    backslash: Vec<i32>,
}

thread_local! {
    static BUFFERS: std::cell::RefCell<Buffers> = Default::default();
}

/// Runs `job(first index, parts)` on bands of the slices (the same band of
/// each), one band per processor core.
fn in_bands<T: Send>(slices: Vec<&mut [T]>, chunk: usize, job: impl Fn(usize, &mut [&mut [T]]) + Sync) {
    let mut parts: Vec<Vec<&mut [T]>> = Vec::new();
    for s in slices {
        for (n, c) in s.chunks_mut(chunk).enumerate() {
            if parts.len() <= n {
                parts.push(Vec::new());
            }
            parts[n].push(c);
        }
    }
    crate::pool::each(parts, |n, mut p| job(n * chunk, &mut p));
}

/// Smooths `src` (`w` x `h`) into `dst` at twice the size.
#[cfg(test)]
pub fn scale2x(src: &[u32], w: usize, h: usize, dst: &mut Vec<u32>) {
    scale2x_keeping(src, w, h, dst, &[]);
}

/// One row of the bordered picture (the edge pixels repeated twice on
/// each side) from a row `line` of the picture, and its YUV; `last` keeps
/// the colour converted last, as neighbours are often alike.
fn bordered_row(row: &mut [u32], yrow: &mut [[i32; 3]], line: &[u32], last: &mut (u32, [i32; 3])) {
    let w = line.len();
    row[2..w + 2].copy_from_slice(line);
    row[0] = line[0];
    row[1] = line[0];
    row[w + 2] = line[w - 1];
    row[w + 3] = line[w - 1];
    for (c, v) in row.iter().zip(yrow.iter_mut()) {
        if *c != last.0 {
            *last = (*c, yuv(*c));
        }
        *v = last.1;
    }
}

impl Grid<'_> {
    /// The four new pixels of pixel `e` (top left, top right, bottom left,
    /// bottom right); `sharp`: only doubled.
    fn four_pixels(&self, e: usize, sharp: bool) -> [u32; 4] {
        let (pw, col) = (self.pw, self.cols);
        let (sl, bs) = (self.slash, self.backslash);
        let ce = col[e];
        let (b, d, f, hh) = (e - pw, e - 1, e + 1, e + pw);
        let mut out = [ce; 4];
        // Every corner first asks whether the pixel differs from
        // both its neighbours on that side; often it does not.
        let (cb, cd, cf, ch) = (col[b], col[d], col[f], col[hh]);
        let flat = (ce == ch || ce == cf) && (ce == cf || ce == cb) && (ce == cb || ce == cd) && (ce == cd || ce == ch);
        if flat || sharp {
            return out;
        }
        let (a, c, g, i) = (b - 1, b + 1, hh - 1, hh + 1);
        let (a0, c1, g5, i4) = (a - 1, c - pw, g + pw, i + 1);
        let (a1, c4, g0, i5) = (a - pw, c + 1, g - 1, i + pw);
        // the differences to the four neighbours
        let (er, el, eu, ed) = (self.right[e], self.right[d], self.down[b], self.down[e]);
        // 0 top left, 1 top right, 2 bottom left, 3 bottom right;
        // the squares around E by their top left pixel
        corner(self, sl[e], bs[e], er, ed, e, i, hh, f, g, c, d, b, i4, i5, &mut out, 1, 2, 3);
        corner(self, bs[b], sl[b], eu, er, e, c, f, b, i, a, hh, d, c1, c4, &mut out, 0, 3, 1);
        corner(self, sl[a], bs[a], el, eu, e, a, b, d, c, g, f, hh, a0, a1, &mut out, 2, 1, 0);
        corner(self, bs[d], sl[d], ed, el, e, g, d, hh, a, i, b, f, g5, g0, &mut out, 3, 0, 2);
        out
    }
}

/// `scale2x`, but the pixels marked in `sharp` (if it is not empty) are
/// only doubled, not smoothed: fine patterns that would blur.
pub fn scale2x_keeping(src: &[u32], w: usize, h: usize, dst: &mut Vec<u32>, sharp: &[bool]) {
    dst.resize(4 * w * h, 0);
    let cores = crate::pool::cores();
    BUFFERS.with(|buffers| {
        let bufs = &mut *buffers.borrow_mut();
        let pw = w + 4;
        let n = pw * (h + 4);
        let chunk = (h + 4).div_ceil(cores).max(1) * pw;
        bufs.cols.resize(n, 0);
        bufs.yuvs.resize(n, [0; 3]);
        for v in [&mut bufs.right, &mut bufs.down, &mut bufs.down_right, &mut bufs.down_left, &mut bufs.slash, &mut bufs.backslash] {
            v.resize(n, 0);
        }

        // 1. the bordered picture and its YUV
        {
            let Buffers { cols, yuvs, .. } = bufs;
            let bands: Vec<_> = cols.chunks_mut(chunk).zip(yuvs.chunks_mut(chunk)).collect();
            crate::pool::each(bands, |bi, (cols, yuvs)| {
                let mut last = (u32::MAX, [0; 3]);
                for (r, (row, yrow)) in cols.chunks_mut(pw).zip(yuvs.chunks_mut(pw)).enumerate() {
                    let sy = (bi * chunk / pw + r).saturating_sub(2).min(h - 1);
                    bordered_row(row, yrow, &src[sy * w..(sy + 1) * w], &mut last);
                }
            });
        }

        // 2. the differences between neighbours
        {
            let Buffers { yuvs, right, down, down_right, down_left, .. } = bufs;
            let yuvs = &yuvs[..];
            in_bands(vec![&mut right[..], &mut down[..], &mut down_right[..], &mut down_left[..]], chunk, |base, p| {
                for j in 0..p[0].len() {
                    let k = base + j;
                    let to = |o: usize| yuvs.get(k + o).map_or(0, |&v| df(yuvs[k], v));
                    p[0][j] = to(1);
                    p[1][j] = to(pw);
                    p[2][j] = to(pw + 1);
                    p[3][j] = to(pw - 1);
                }
            });
        }

        // 3. per square of 2 x 2 (E F H I, by E): the change along its two
        // diagonals, with the parallel neighbours (see `corner`)
        {
            let Buffers { cols, yuvs, right, down, down_right, down_left, slash, backslash } = bufs;
            let gr = Grid { cols, yuvs, right, down, down_right, down_left, slash: &[], backslash: &[], pw };
            let gr = &gr;
            in_bands(vec![&mut slash[..], &mut backslash[..]], chunk, |base, p| {
                for j in 0..p[0].len() {
                    let e = base + j;
                    // only where all the pixels involved are inside the grid
                    if e < 2 * pw + 1 || e + 2 * pw + 2 >= n {
                        continue;
                    }
                    let (f, hh) = (e + 1, e + pw);
                    let (i, b, d) = (hh + 1, e - pw, e - 1);
                    let (c, g) = (b + 1, hh - 1);
                    let (h5, f4, i4, i5) = (hh + pw, f + 1, i + 1, i + pw);
                    p[0][j] = gr.d(e, c) + gr.d(e, g) + gr.d(i, h5) + gr.d(i, f4) + (gr.d(hh, f) << 2);
                    p[1][j] = gr.d(hh, d) + gr.d(hh, i5) + gr.d(f, i4) + gr.d(f, b) + (gr.d(e, i) << 2);
                }
            });
        }

        // 4. the new pixels
        let gr = Grid {
            cols: &bufs.cols,
            yuvs: &bufs.yuvs,
            right: &bufs.right,
            down: &bufs.down,
            down_right: &bufs.down_right,
            down_left: &bufs.down_left,
            slash: &bufs.slash,
            backslash: &bufs.backslash,
            pw,
        };
        let gr = &gr;
        let band = h.div_ceil(cores).max(1);
        crate::pool::each(dst.chunks_mut(band * 4 * w).collect(), |bi, rows: &mut [u32]| {
            let y0 = bi * band;
            let row = 2 * w;
            for y in y0..(y0 + band).min(h) {
                let oy = 2 * (y - y0);
                for x in 0..w {
                    let out = gr.four_pixels((y + 2) * pw + x + 2, sharp.get(y * w + x) == Some(&true));
                    let ox = 2 * x;
                    rows[oy * row + ox] = out[0];
                    rows[oy * row + ox + 1] = out[1];
                    rows[(oy + 1) * row + ox] = out[2];
                    rows[(oy + 1) * row + ox + 1] = out[3];
                }
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_stays_flat_and_steps_get_smoothed() {
        let mut out = Vec::new();
        scale2x(&vec![0x336699; 16 * 10], 16, 10, &mut out);
        assert!(out.iter().all(|&p| p == 0x336699));
        // a diagonal staircase: black above, white below
        let (w, h) = (16, 16);
        let src: Vec<u32> = (0..w * h).map(|i| if i % w > i / w { 0 } else { 0xFFFFFF }).collect();
        scale2x(&src, w, h, &mut out);
        let mixed = out.iter().filter(|&&p| p != 0 && p != 0xFFFFFF).count();
        assert!(mixed > 0, "the steps should be smoothed");
    }
}
