//! Shapes (sprites) from U8SHAPES.FLX and the per-shape type flags.

use crate::flex::{u16le, u32le, Flex};
use std::fs;
use std::io;
use std::path::Path;
use std::rc::Rc;

/// Marks a transparent pixel in a decoded frame.
pub const TRANSPARENT: u16 = 0xFFFF;

#[derive(Clone)]
pub struct Frame {
    pub width: i32,
    pub height: i32,
    pub xoff: i32,
    pub yoff: i32,
    /// Palette indices, `TRANSPARENT` where nothing is drawn.
    pub pixels: Vec<u16>,
}

pub struct Shape {
    pub frames: Vec<Frame>,
}

pub fn decode_frame(d: &[u8]) -> Option<Frame> {
    if d.len() < 18 {
        return None;
    }
    let compressed = d[8] != 0;
    let width = u16le(d, 10) as i16 as i32;
    let height = u16le(d, 12) as i16 as i32;
    let xoff = u16le(d, 14) as i16 as i32;
    let yoff = u16le(d, 16) as i16 as i32;
    if width <= 0 || height <= 0 || width > 4096 || height > 4096 {
        return Some(Frame { width: 0, height: 0, xoff, yoff, pixels: Vec::new() });
    }
    let (w, h) = (width as usize, height as usize);
    let rle = 18 + h * 2;
    let mut pixels = vec![TRANSPARENT; w * h];
    for y in 0..h {
        // line offsets are relative to their own position in the table
        let rel = u16le(d, 18 + y * 2) as usize;
        let mut p = (rle + rel).checked_sub((h - y) * 2)?;
        let mut x = 0usize;
        loop {
            x += *d.get(p)? as usize;
            p += 1;
            if x >= w {
                break;
            }
            let mut dlen = *d.get(p)? as usize;
            p += 1;
            let mut run = false;
            if compressed {
                run = dlen & 1 != 0;
                dlen >>= 1;
            }
            let end = (x + dlen).min(w);
            if run {
                let c = *d.get(p)? as u16;
                p += 1;
                pixels[y * w + x..y * w + end].fill(c);
            } else {
                for i in x..end {
                    pixels[y * w + i] = *d.get(p + (i - x))? as u16;
                }
                p += dlen;
            }
            x += dlen;
            if x >= w {
                break;
            }
        }
    }
    Some(Frame { width, height, xoff, yoff, pixels })
}

pub fn decode_shape(d: &[u8]) -> Option<Shape> {
    if d.len() < 6 {
        return None;
    }
    let count = u16le(d, 4) as usize;
    let mut frames = Vec::with_capacity(count);
    for i in 0..count {
        let e = 6 + i * 6;
        if e + 6 > d.len() {
            break;
        }
        let off = (u32le(d, e) & 0xFF_FFFF) as usize;
        let frame = d.get(off..).and_then(decode_frame);
        frames.push(frame.unwrap_or(Frame { width: 0, height: 0, xoff: 0, yoff: 0, pixels: Vec::new() }));
    }
    Some(Shape { frames })
}

/// Shape archive with lazily decoded shapes.
pub struct Shapes {
    flex: Flex,
    cache: Vec<Option<Option<Rc<Shape>>>>,
    /// Shapes handed out instead of the originals (the Avatar in armour).
    replaced: Vec<(u32, Rc<Shape>)>,
}

impl Shapes {
    pub fn open(path: &Path) -> io::Result<Shapes> {
        let flex = Flex::open(path)?;
        let n = flex.len();
        Ok(Shapes { flex, cache: (0..n).map(|_| None).collect(), replaced: Vec::new() })
    }

    /// From now on `get` hands out `shape` instead of shape `num` (None:
    /// the original again).
    pub fn replace(&mut self, num: u32, shape: Option<Rc<Shape>>) {
        self.replaced.retain(|r| r.0 != num);
        if let Some(s) = shape {
            self.replaced.push((num, s));
        }
    }

    pub fn get(&mut self, num: u32) -> Option<Rc<Shape>> {
        if let Some(r) = self.replaced.iter().find(|r| r.0 == num) {
            return Some(r.1.clone());
        }
        self.original(num)
    }

    /// The shape as in the game files, also if it is replaced.
    pub fn original(&mut self, num: u32) -> Option<Rc<Shape>> {
        let i = num as usize;
        if i >= self.cache.len() {
            return None;
        }
        if self.cache[i].is_none() {
            let shape = self.flex.get(i).and_then(decode_shape).map(Rc::new);
            self.cache[i] = Some(shape);
        }
        self.cache[i].as_ref().unwrap().clone()
    }

    /// A shape that `get` has already loaded (for drawing without `mut`).
    pub fn peek(&self, num: u32) -> Option<Rc<Shape>> {
        if let Some(r) = self.replaced.iter().find(|r| r.0 == num) {
            return Some(r.1.clone());
        }
        self.cache.get(num as usize)?.as_ref()?.clone()
    }
}

pub const FAMILY_GLOBEGG: u8 = 3;
pub const FAMILY_UNKEGG: u8 = 4;
pub const FAMILY_CONTAINER: u8 = 6;
pub const FAMILY_TELEPORTEGG: u8 = 8;

/// Per-shape properties from TYPEFLAG.DAT.
#[derive(Clone, Copy, Default)]
pub struct ShapeInfo {
    pub fixed: bool,
    pub solid: bool,
    /// Water.
    pub sea: bool,
    /// Hurts whoever touches it (lava, fire).
    pub damaging: bool,
    pub land: bool,
    pub occl: bool,
    pub draw: bool,
    pub roof: bool,
    pub translucent: bool,
    pub editor: bool,
    pub family: u8,
    /// Where the item is worn: 0 nowhere, 1 shield, 2 arm, 3 head, 4 body, 5 legs, 6 weapon.
    pub equip_type: u8,
    /// Footpad in tiles (x, y: 32 world units, z: 8 world units).
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub anim_type: u8,
    pub anim_data: u8,
    pub anim_speed: u8,
    /// The raw record, for scripts that test single bits.
    pub raw: [u8; 8],
    /// A creature shape (it has animations).
    pub creature: bool,
    /// Frames in which the creature lies on the ground (the end of dying,
    /// the start of getting up); `NO_FRAME` fills the rest.
    pub lying: [u16; 16],
}

pub const NO_FRAME: u16 = u16::MAX;

impl ShapeInfo {
    /// Does the creature lie on the ground in this frame?
    pub fn lies_in(&self, frame: u32) -> bool {
        self.creature && self.lying.contains(&(frame as u16))
    }

    /// Bounding box size in world units.
    pub fn footpad(&self, flipped: bool) -> (i32, i32, i32) {
        if flipped {
            (self.y * 32, self.x * 32, self.z * 8)
        } else {
            (self.x * 32, self.y * 32, self.z * 8)
        }
    }
}

pub fn load_type_flags(path: &Path) -> io::Result<Vec<ShapeInfo>> {
    let d = fs::read(path)?;
    let mut infos: Vec<ShapeInfo> = d
        .chunks_exact(8)
        .map(|b| ShapeInfo {
            fixed: b[0] & 0x01 != 0,
            solid: b[0] & 0x02 != 0,
            sea: b[0] & 0x04 != 0,
            damaging: b[0] & 0x40 != 0,
            land: b[0] & 0x08 != 0,
            occl: b[0] & 0x10 != 0,
            draw: b[1] & 0x01 != 0,
            roof: b[1] & 0x04 != 0,
            translucent: b[1] & 0x08 != 0,
            family: b[1] >> 4,
            equip_type: b[2] & 0x0F,
            x: (b[2] >> 4) as i32,
            y: (b[3] & 0x0F) as i32,
            z: (b[3] >> 4) as i32,
            anim_type: b[4] & 0x0F,
            anim_data: b[4] >> 4,
            anim_speed: b[5] & 0x0F,
            editor: b[5] & 0x10 != 0,
            raw: [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]],
            creature: false,
            lying: [NO_FRAME; 16],
        })
        .collect();
    // The "moss curtains" in the catacombs are wrongly marked solid in the
    // original data.
    for i in 459..=464 {
        if let Some(si) = infos.get_mut(i) {
            si.solid = false;
        }
    }
    Ok(infos)
}
