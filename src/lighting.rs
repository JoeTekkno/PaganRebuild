//! Lighting (optional): the world is drawn a little darker, and around
//! lamps, fires, candles and lava it is lit up in their colour. This is
//! done on the finished picture of the world, so it works with the
//! turned view as well.

use crate::game::Game;
use crate::view::Surface;

/// The general light where no lamp shines (1.0 = as in the original).
pub const AMBIENT: f32 = 0.6;
/// Height of a lamp post below its lamp.
const POST: i32 = 80;
/// How much brighter than the original a lit spot may get.
const MOST: f32 = 1.3;

/// A light on screen.
pub struct Light {
    pub x: f32,
    pub y: f32,
    /// Reach in pixels (sideways; half as far up and down, as the ground
    /// is seen at a slant).
    pub radius: f32,
    /// Colour and strength.
    pub rgb: [f32; 3],
}

/// What kind of light a thing gives: (radius, colour, flickers).
fn light_of(shape: u32, frame: u32) -> Option<(f32, [f32; 3], bool)> {
    const LAMP: [f32; 3] = [1.5, 1.25, 0.8];
    const FIRE: [f32; 3] = [1.1, 0.6, 0.25];
    const CANDLE: [f32; 3] = [1.0, 0.8, 0.45];
    const LAVA: [f32; 3] = [1.0, 0.35, 0.1];
    Some(match shape {
        // lamp posts (the lamp at the top), wall lamps that burn, the lamp at the end
        690 | 691 => (90.0, LAMP, false),
        314 | 317 | 640 => (95.0, LAMP, false),
        216 => (60.0, LAMP, false),
        // camp fire, fireplaces, fire pit, fire field
        267 => (140.0, FIRE, true),
        276 | 277 => (120.0, FIRE, true),
        318 => (150.0, FIRE, true),
        400 => (70.0, FIRE, true),
        // candles and candlesticks, when lit
        454 if frame & 3 != 0 => (55.0, CANDLE, true),
        452 if frame & 3 != 0 => (75.0, CANDLE, true),
        // lava
        111 | 409 | 479 => (90.0, LAVA, false),
        603 => (60.0, LAVA, true),
        _ => return None,
    })
}

impl Game {
    /// The lights in and around the view, as it is about to be drawn.
    pub fn lights(&mut self, xf: &crate::turnview::Xform) -> Vec<Light> {
        let (cx, cy, _) = xf.cam;
        let (w, h) = (xf.size.0 as f32, xf.size.1 as f32);
        let reach = 2 * xf.size.0 + 4 * xf.size.1 + 1024;
        let tick = self.tick as f32;
        let mut out = Vec::new();
        for i in self.map().items_near(cx, cy, reach) {
            let it = &self.map().items[i];
            if !it.in_world() {
                continue;
            }
            let Some((radius, rgb, flickers)) = light_of(it.shape, it.frame) else { continue };
            let Some(si) = self.data.infos.get(it.shape as usize) else { continue };
            if self.covered(it, si) {
                continue;
            }
            let (xd, yd, zd) = si.footpad(it.flags & crate::map::FLG_FLIPPED != 0);
            // from the middle of the thing
            let (sx, sy) = xf.to_screen(it.x - xd / 2, it.y - yd / 2, it.z + zd / 2);
            let (sx, sy) = (sx as f32, sy as f32);
            if sx < -radius || sx > w + radius || sy < -radius || sy > h + radius {
                continue;
            }
            // fire flickers, every one in its own rhythm
            let strength = if flickers {
                let p = it.id as f32 * 1.7;
                0.85 + 0.1 * (tick * 0.31 + p).sin() + 0.05 * (tick * 0.83 + 2.0 * p).sin()
            } else {
                1.0
            };
            out.push(Light { x: sx, y: sy, radius: radius * (0.95 + 0.05 * strength), rgb: rgb.map(|c| c * strength) });
            // a lamp on a post lights the ground below it
            if matches!(it.shape, 690 | 691) {
                let (gx, gy) = xf.to_screen(it.x - xd / 2, it.y - yd / 2, (it.z - POST).max(0));
                out.push(Light { x: gx as f32, y: gy as f32, radius: 150.0, rgb: rgb.map(|c| c * 0.9) });
            }
        }
        out
    }
}

impl Game {
    /// Is a light covered by floor right above it (lava under a corridor
    /// that has not fallen in)? Then its glow does not show.
    fn covered(&self, it: &crate::map::Item, si: &crate::shape::ShapeInfo) -> bool {
        // only what lies flat on the ground can be covered so
        if si.z != 0 {
            return false;
        }
        let me = crate::map::WorldBox::of(it, si);
        let (cx, cy) = (it.x - me.xd / 2, it.y - me.yd / 2);
        self.map().items_near(it.x, it.y, 512).into_iter().any(|i| {
            let o = &self.map().items[i];
            let Some(oi) = self.data.infos.get(o.shape as usize) else { return false };
            let b = crate::map::WorldBox::of(o, oi);
            o.in_world()
                && o.z > it.z
                && o.z <= it.z + 48
                && oi.land
                && light_of(o.shape, o.frame).is_none()
                && cx < b.x
                && cx >= b.x - b.xd
                && cy < b.y
                && cy >= b.y - b.yd
        })
    }
}

/// For every pixel (from the buffer of item numbers + 1): is it part of
/// something that gives light? Those keep their own colours: lit by
/// themselves and their neighbours, a stretch of lava turned a flat yellow.
pub fn glowing_pixels(ids: &[u32], items: &[crate::map::Item]) -> Vec<bool> {
    ids.iter()
        .map(|&id| id.checked_sub(1).and_then(|i| items.get(i as usize)).is_some_and(|it| light_of(it.shape, it.frame).is_some()))
        .collect()
}

/// Darkens the picture to the general light `ambient` and lights it up
/// around the lights. `glows` (may be empty): pixels left as they are.
pub fn apply(surf: &mut Surface, lights: &[Light], ambient: f32, glows: &[bool]) {
    let (w, h) = (surf.width, surf.height);
    let lamp = (1.0 - AMBIENT) * 1.2;
    // in bands of rows, one per processor core
    let cores = crate::pool::cores() as i32;
    let band = ((h + cores - 1) / cores).max(1);
    let parts: Vec<_> = surf
        .pixels
        .chunks_mut((band * w) as usize)
        .enumerate()
        .map(|(b, pixels)| {
            let start = b * (band * w) as usize;
            let glows = glows.get(start..start + pixels.len()).unwrap_or(&[]);
            (b as i32 * band, pixels, glows)
        })
        .collect();
    crate::pool::each(parts, |_, (y0, pixels, glows)| {
        let rows = pixels.len() as i32 / w;
        let mut light: Vec<[f32; 3]> = vec![[ambient; 3]; (rows * w) as usize];
        for l in lights {
            let half = l.radius / 2.0;
            let (x0, x1) = (((l.x - l.radius) as i32).max(0), ((l.x + l.radius) as i32 + 1).min(w));
            let (ly0, ly1) = (((l.y - half) as i32).max(y0), ((l.y + half) as i32 + 1).min(y0 + rows));
            for y in ly0..ly1 {
                let dy = 2.0 * (y as f32 - l.y);
                for x in x0..x1 {
                    let dx = x as f32 - l.x;
                    let d2 = (dx * dx + dy * dy) / (l.radius * l.radius);
                    if d2 >= 1.0 {
                        continue;
                    }
                    // soft towards the edge
                    let f = (1.0 - d2) * (1.0 - d2) * lamp;
                    let p = &mut light[((y - y0) * w + x) as usize];
                    for c in 0..3 {
                        p[c] += f * l.rgb[c];
                    }
                }
            }
        }
        for (i, (px, l)) in pixels.iter_mut().zip(&light).enumerate() {
            if glows.get(i).copied().unwrap_or(false) {
                continue;
            }
            let ch = |v: u32, k: f32| ((v as f32 * k.min(MOST)) as u32).min(255);
            let (r, g, b) = ((*px >> 16) & 0xFF, (*px >> 8) & 0xFF, *px & 0xFF);
            *px = (ch(r, l[0]) << 16) | (ch(g, l[1]) << 8) | ch(b, l[2]);
        }
    });
}
