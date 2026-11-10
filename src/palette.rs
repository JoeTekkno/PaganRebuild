//! Game palette (U8PAL.PAL) and the translucency table.

use std::fs;
use std::io;
use std::path::Path;

pub struct Palette {
    /// 0x00RRGGBB, with the current colour transform applied
    pub colors: [u32; 256],
    /// The colours as they are in the palette file.
    base: [u32; 256],
    /// Translucent colours: (r, g, b, alpha), alpha 0 = opaque palette colour.
    pub xform: [(u32, u32, u32, u32); 256],
    /// The colour transform now in use (see `transform_matrix`).
    matrix: [i32; 12],
    /// A running fade to another transform: (start, goal, ticks done, ticks).
    fading: Option<([i32; 12], [i32; 12], i32, i32)>,
}

/// Palette indices 8..=14 are drawn blended over the background
/// (smoke, glass, water ...). Values as used by ScummVM.
const XFORM: [(u32, u32, u32, u32); 7] = [
    (48, 48, 48, 80),
    (24, 24, 24, 128),
    (64, 64, 24, 64),
    (80, 80, 80, 80),
    (180, 90, 0, 80),
    (0, 0, 252, 40),
    (0, 0, 104, 40),
];

impl Palette {
    /// A colour as it is in the palette file (without the current transform).
    pub fn base_color(&self, i: usize) -> u32 {
        self.base[i]
    }

    pub fn load(path: &Path) -> io::Result<Palette> {
        let d = fs::read(path)?;
        if d.len() < 4 + 768 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "palette file too short"));
        }
        let mut colors = [0u32; 256];
        for (i, c) in colors.iter_mut().enumerate() {
            // 6 bit VGA values
            let v = |k: usize| {
                let x = (d[4 + i * 3 + k] & 0x3F) as u32;
                (x << 2) | (x >> 4)
            };
            *c = (v(0) << 16) | (v(1) << 8) | v(2);
        }
        let mut xform = [(0, 0, 0, 0); 256];
        xform[8..15].copy_from_slice(&XFORM);
        Ok(Palette { colors, base: colors, xform, matrix: transform_matrix(0), fading: None })
    }
}

/// The gentler rain storm of the option "storm colours".
pub const STORM_SOFT: u8 = 8;

/// The transform shown for one a script asks for: the rain storm (3) as
/// the option "storm colours" has it (0 original, 1 gentler, 2 none).
pub fn shown_transform(kind: u8, storm: usize) -> u8 {
    match (kind, storm) {
        (3, 1) => STORM_SOFT,
        (3, 2) => 0,
        _ => kind,
    }
}

/// Colour transforms the scripts can switch on: each output channel is a
/// mix of the input channels plus a constant, in 1/2048.  From ScummVM.
fn transform_matrix(kind: u8) -> [i32; 12] {
    const GREY: [i32; 3] = [0x300, 0x400, 0x100];
    let mut m = [0x800, 0, 0, 0, 0, 0x800, 0, 0, 0, 0, 0x800, 0];
    match kind {
        // greyscale
        1 => {
            for i in 0..3 {
                m[i * 4..i * 4 + 3].copy_from_slice(&GREY);
                m[i * 4 + 3] = 0;
            }
        }
        // no red
        2 => m[0] = 0,
        // rain storm: washed out and a little brighter
        3 => {
            for i in 0..3 {
                for j in 0..3 {
                    m[i * 4 + j] = (GREY[j] * 0x200) >> 11;
                }
                m[i * 4 + i] += 0x200;
                m[i * 4 + 3] = 0x180;
            }
        }
        // fire storm: everything glows red
        4 => {
            m = [((0x300 * 0x400) >> 11) + 0x400, (0x400 * 0x400) >> 11, (0x100 * 0x400) >> 11, 0x180,
                 (0x300 * 0x200) >> 11, ((0x400 * 0x200) >> 11) + 0x400, (0x100 * 0x200) >> 11, 0,
                 0, 0, 0x400, 0];
        }
        // saturated
        5 => {
            for i in 0..3 {
                for j in 0..3 {
                    m[i * 4 + j] = -GREY[j];
                }
                m[i * 4 + i] += 0x1000;
                m[i * 4 + 3] = 0;
            }
        }
        // channels rotated
        6 => m = [0, 0x800, 0, 0, 0, 0, 0x800, 0, 0x800, 0, 0, 0],
        // rain storm, gentler (option "storm colours"): a little darker and
        // paler, with a cool blue cast, instead of washed out to grey
        STORM_SOFT => {
            const DARK: i32 = 0x680; // 0.8
            const KEEP: i32 = 0x5C0; // 0.72 of the colour stays
            for i in 0..3 {
                for j in 0..3 {
                    m[i * 4 + j] = (DARK * (0x800 - KEEP) / 0x800) * GREY[j] / 0x800;
                }
                m[i * 4 + i] += DARK * KEEP / 0x800;
                m[i * 4 + 3] = 0;
            }
            // red a little weaker, blue a little stronger
            for j in 0..3 {
                m[j] = m[j] * 15 / 16;
                m[8 + j] = m[8 + j] * 9 / 8;
            }
            m[11] = 0x30;
        }
        7 => m = [0, 0, 0x800, 0, 0x800, 0, 0, 0, 0, 0x800, 0, 0],
        _ => {}
    }
    m
}

impl Palette {
    /// Switches the colour transform (0 = the normal colours).
    pub fn set_transform(&mut self, kind: u8) {
        self.fading = None;
        self.apply(transform_matrix(kind));
    }

    /// Blends from the colours on screen now to a transform over `ticks`
    /// ticks, as ScummVM's palette fader does.
    pub fn fade_to_transform(&mut self, kind: u8, ticks: i32) {
        self.fading = Some((self.matrix, transform_matrix(kind), 0, ticks.max(1)));
    }

    /// Advances a running palette fade by one tick.
    pub fn tick(&mut self) {
        let Some((from, to, done, total)) = self.fading else { return };
        let done = done + 1;
        let mut m = [0; 12];
        for i in 0..12 {
            m[i] = from[i] + (to[i] - from[i]) * done / total;
        }
        self.fading = if done >= total { None } else { Some((from, to, done, total)) };
        self.apply(m);
    }

    fn apply(&mut self, m: [i32; 12]) {
        self.matrix = m;
        for (out, &c) in self.colors.iter_mut().zip(self.base.iter()) {
            let (r, g, b) = (((c >> 16) & 0xFF) as i32, ((c >> 8) & 0xFF) as i32, (c & 0xFF) as i32);
            let ch = |row: usize| ((r * m[row * 4] + g * m[row * 4 + 1] + b * m[row * 4 + 2] + 255 * m[row * 4 + 3]) / 2048).clamp(0, 255) as u32;
            *out = (ch(0) << 16) | (ch(1) << 8) | ch(2);
        }
    }
}
