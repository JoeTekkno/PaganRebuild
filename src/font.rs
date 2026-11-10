//! Text drawing with the game's bitmap fonts (U8FONTS.FLX: one shape per
//! font, one frame per character).

use crate::palette::Palette;
use crate::shape::{Shape, TRANSPARENT};
use crate::view::Surface;
use std::rc::Rc;

pub struct Font {
    shape: Rc<Shape>,
    pub height: i32,
    baseline: i32,
    /// Rows a line of text can cover (letters may reach below `height`).
    rows: i32,
}

impl Font {
    pub fn new(shape: Rc<Shape>) -> Font {
        let height = shape.frames.iter().map(|f| f.height).max().unwrap_or(8);
        let baseline = shape.frames.iter().map(|f| f.yoff).max().unwrap_or(0);
        let rows = shape.frames.iter().filter(|f| f.width > 0).map(|f| baseline - f.yoff + f.height).max().unwrap_or(height).max(height);
        Font { shape, height, baseline, rows }
    }

    fn char_width(&self, c: char) -> i32 {
        self.shape.frames.get(c as usize).map_or(0, |f| f.width - f.xoff)
    }

    pub fn width(&self, text: &str) -> i32 {
        text.chars().map(|c| self.char_width(c)).sum()
    }

    /// Breaks text into lines of at most `max_width` pixels.
    pub fn wrap(&self, text: &str, max_width: i32) -> Vec<String> {
        let mut lines = Vec::new();
        let mut line = String::new();
        for word in text.split_whitespace() {
            let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            if self.width(&candidate) > max_width && !line.is_empty() {
                lines.push(std::mem::replace(&mut line, word.to_string()));
            } else {
                line = candidate;
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
        lines
    }

    pub fn draw(&self, surf: &mut Surface, text: &str, mut x: i32, y: i32, pal: &Palette) {
        for c in text.chars() {
            if let Some(f) = self.shape.frames.get(c as usize) {
                if f.width > 0 {
                    surf.paint(f, x, y + self.baseline, false, false, pal);
                }
                x += f.width - f.xoff;
            }
        }
    }

    // Drawing at a size factor `s` (the option "text size"). At 1 this is
    // exactly the plain drawing; otherwise the line is drawn at its own
    // size and resampled: each pixel gets the colours of the font pixels
    // it covers, weighted by how much it covers them, so strokes stay even
    // at factors like 1.5 instead of some doubling and some not.

    pub fn height_at(&self, s: f32) -> i32 {
        (self.height as f32 * s).round() as i32
    }

    pub fn width_at(&self, text: &str, s: f32) -> i32 {
        (self.width(text) as f32 * s).round() as i32
    }

    pub fn wrap_at(&self, text: &str, max_width: i32, s: f32) -> Vec<String> {
        self.wrap(text, (max_width as f32 / s) as i32)
    }

    pub fn draw_at(&self, surf: &mut Surface, text: &str, x: i32, y: i32, pal: &Palette, s: f32) {
        if s == 1.0 {
            return self.draw(surf, text, x, y, pal);
        }
        let (w, h) = (self.width(text), self.rows);
        if w <= 0 || h <= 0 {
            return;
        }
        // the line at its own size: colours, None where nothing is drawn
        let mut line: Vec<Option<u32>> = vec![None; (w * h) as usize];
        let mut cx = 0;
        for c in text.chars() {
            let Some(f) = self.shape.frames.get(c as usize) else { continue };
            let top = self.baseline - f.yoff;
            for row in 0..f.height {
                let ly = top + row;
                if ly < 0 || ly >= h {
                    continue;
                }
                for col in 0..f.width {
                    let p = f.pixels[(row * f.width + col) as usize];
                    let lx = cx - f.xoff + col;
                    if p != TRANSPARENT && lx >= 0 && lx < w {
                        line[(ly * w + lx) as usize] = Some(pal.colors[p as usize]);
                    }
                }
            }
            cx += f.width - f.xoff;
        }
        let (dw, dh) = (((w as f32) * s).round() as i32, ((h as f32) * s).round() as i32);
        // for every target column (row): the source columns it covers and how much
        let spans = |n: i32, dn: i32| -> Vec<Vec<(i32, f32)>> {
            let k = n as f32 / dn as f32;
            (0..dn)
                .map(|d| {
                    let (a, b) = (d as f32 * k, (d + 1) as f32 * k);
                    let mut v = Vec::new();
                    let mut i = a.floor() as i32;
                    while (i as f32) < b && i < n {
                        let part = (b.min((i + 1) as f32) - a.max(i as f32)) / k;
                        if part > 0.001 {
                            v.push((i, part));
                        }
                        i += 1;
                    }
                    v
                })
                .collect()
        };
        let (cols, rows) = (spans(w, dw), spans(h, dh));
        for (dy, rspan) in rows.iter().enumerate() {
            let py = y + dy as i32;
            if py < 0 || py >= surf.height {
                continue;
            }
            for (dx, cspan) in cols.iter().enumerate() {
                let px = x + dx as i32;
                if px < 0 || px >= surf.width {
                    continue;
                }
                let (r, g, b, cover) = covered_colour(&line, w, rspan, cspan);
                if cover < 0.01 {
                    continue;
                }
                let dst = &mut surf.pixels[(py * surf.width + px) as usize];
                let a = cover.min(1.0);
                let mix = |src: f32, shift: u32| {
                    let d = ((*dst >> shift) & 0xFF) as f32;
                    ((src / cover) * a + d * (1.0 - a)).round().clamp(0.0, 255.0) as u32
                };
                *dst = (mix(r, 16) << 16) | (mix(g, 8) << 8) | mix(b, 0);
            }
        }
    }

    /// `draw_centered` at a size factor; `max_width` is in target pixels.
    pub fn draw_centered_at(&self, surf: &mut Surface, text: &str, cx: i32, bottom: i32, max_width: i32, pal: &Palette, s: f32) {
        let lines = self.wrap_at(text, max_width, s);
        let lh = self.height_at(s);
        let total = lines.len() as i32 * lh;
        let mut y = (bottom - total).clamp(2, (surf.height - total - 2).max(2));
        for line in &lines {
            let w = self.width_at(line, s);
            let x = (cx - w / 2).clamp(2, (surf.width - w - 2).max(2));
            self.draw_at(surf, line, x, y, pal, s);
            y += lh;
        }
    }
}

/// The weighted sum of red, green and blue of the source pixels a target
/// pixel covers (rows `rspan`, columns `cspan` with their shares), and how
/// much of it is covered by drawn pixels.
fn covered_colour(line: &[Option<u32>], w: i32, rspan: &[(i32, f32)], cspan: &[(i32, f32)]) -> (f32, f32, f32, f32) {
    let (mut r, mut g, mut b, mut cover) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    for &(sy, wy) in rspan {
        for &(sx, wx) in cspan {
            let Some(c) = line[(sy * w + sx) as usize] else { continue };
            let wt = wx * wy;
            r += ((c >> 16) & 0xFF) as f32 * wt;
            g += ((c >> 8) & 0xFF) as f32 * wt;
            b += (c & 0xFF) as f32 * wt;
            cover += wt;
        }
    }
    (r, g, b, cover)
}
