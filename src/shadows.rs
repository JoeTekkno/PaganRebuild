//! Shadows (optional): the figures (the Avatar, people, creatures) cast a
//! soft shadow on the ground, as the time of day has it (`Sun`).
//!
//! Done on the finished picture: every drawn pixel of a figure is laid down
//! onto the ground, the farther out the higher it is above the figure's
//! feet (its lowest pixel), and the ground there is darkened. Only where
//! the top of what is drawn (by the pixel ids) is at the height the shadow
//! falls to: so the shadow goes behind whatever stands in front of it and
//! never darkens the side of a wall or the figure itself. Where the ground
//! there is higher or lower (a step), the shadow lands farther in or out
//! on it.

use crate::map::{Item, MapData, FLG_INVISIBLE};
use crate::shape::{Frame, Shapes, ShapeInfo, TRANSPARENT};
use crate::view::{Painted, Surface};
use std::sync::atomic::{AtomicBool, Ordering};

/// How much darker the ground is in full shadow, in full daylight.
const DARK: f32 = 0.45;
/// One unit of height in units of x and y (the pictures are seen from 30
/// degrees above, see ego.rs).
const Z_SCALE: f32 = 3.266;

/// The maps under the open sky (the others are caves, dungeons, the
/// ethereal plane: there are no shadows there). Found by looking at every
/// map (test command mapsheet).
const OUTDOORS: [usize; 15] = [3, 4, 5, 6, 7, 8, 12, 29, 37, 39, 40, 41, 56, 200, 223];

/// Whether a map lies under the open sky.
pub fn outdoors(map: usize) -> bool {
    OUTDOORS.contains(&map)
}

/// How many of the eight directions around a spot lead into the void
/// (nothing there, or only the black filler plates the map makers put
/// there). Cellars and other rooms lie as islands in the void; out in
/// the open there is always ground (or sea) all around.
pub fn void_directions(map: &MapData, x: i32, y: i32) -> usize {
    // (looking only at 768 and 1280, the count in a big dungeon swung
    // between 0 and 4 from step to step: the Avatar's shadow came and went)
    const FAR: [i32; 4] = [768, 1024, 1280, 1792];
    const LOOK: i32 = 128;
    const DIRS: [(i32, i32); 8] = [(0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1)];
    // (only the plates count: beyond the edge of a map there is nothing either)
    let void = |px: i32, py: i32| {
        let near = map.items_near(px, py, LOOK);
        let mut there = near.iter().map(|&i| &map.items[i]).filter(|it| it.in_world());
        let first = there.next();
        first.is_some_and(void_plate) && there.all(void_plate)
    };
    DIRS.iter().filter(|&&(dx, dy)| FAR.iter().any(|&far| void(x + dx * far, y + dy * far))).count()
}
/// The black plate the void around rooms is filled with (shape 564 frame
/// 2; frame 0 is the black top of thick town walls).
fn void_plate(it: &Item) -> bool {
    it.shape == 564 && it.frame == 2
}

/// Whether the sun shines where the Avatar is: on a map under the open
/// sky and not in a room lying in the void (a cellar).
pub fn sunlit(map_no: usize, map: &MapData, x: i32, y: i32) -> bool {
    outdoors(map_no) && void_directions(map, x, y) < VOID_ROOM && !sees_stars(map, x, y)
}

/// The starry sky around the house that floats in the void (Mythran's).
const STARS: u32 = 351;

/// Whether the starry sky lies in some direction not far off: then the
/// spot is in the house in the void, where no sun shines (it is too big
/// for the void count to see the void from everywhere inside it).
pub fn sees_stars(map: &MapData, x: i32, y: i32) -> bool {
    const FAR: [i32; 4] = [512, 1024, 1536, 2048];
    const DIRS: [(i32, i32); 8] = [(0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1)];
    DIRS.iter().any(|&(dx, dy)| {
        FAR.iter().any(|&far| map.items_near(x + dx * far, y + dy * far, 128).iter().any(|&i| map.items[i].in_world() && map.items[i].shape == STARS))
    })
}
/// From this many empty directions on, the Avatar is in a room in the void.
/// (Measured with the test command sunlitscan: open land and towns have
/// none, at most 2 on cliffs at the edge of a map; caves 3 to 5, cellars 6.)
pub const VOID_ROOM: usize = 3;

/// Where the light comes from at a time of day. The original pictures are
/// lit from the top left of the screen, that is from the west: the sun
/// keeps near there so that the shadows fit them, from the south-west in
/// the morning over the west to the north-west in the evening, lower then
/// (longer shadows) and high at noon (short ones). At night a faint moon
/// stands in the west; at dawn and dusk the one goes over into the other
/// within an hour.
#[derive(Clone, Copy, Debug)]
pub struct Sun {
    /// Per pixel of height: how far a shadow lands across and down on screen.
    pub across: f32,
    pub down: f32,
    /// How dark the shadows are (0..1).
    pub dark: f32,
}

impl Sun {
    /// `hour`: the time of day, 0.0 .. 24.0.
    pub fn at(hour: f32) -> Sun {
        // the moon: from the west, 50 degrees high, faint
        let moon = (270.0f32, 60.0f32, 0.4f32);
        // the sun from 6 to 18 o'clock
        let day = |h: f32| {
            let t = ((h - 6.0) / 12.0).clamp(0.0, 1.0);
            let bearing = 225.0 + 90.0 * t;
            // low in the morning and evening, highest at noon
            let height = 45.0 + 27.0 * (t * std::f32::consts::PI).sin();
            (bearing, height, 1.0)
        };
        // dawn 5-6, dusk 18-19: from the one to the other
        let (a, b, k) = match hour {
            h if (6.0..18.0).contains(&h) => (day(h), day(h), 0.0),
            h if (5.0..6.0).contains(&h) => (moon, day(6.0), h - 5.0),
            h if (18.0..19.0).contains(&h) => (day(18.0), moon, h - 18.0),
            _ => (moon, moon, 0.0),
        };
        let k = k * k * (3.0 - 2.0 * k);
        let mix = |x: f32, y: f32| x + (y - x) * k;
        let (bearing, height, dark) = (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2));
        // the shadow points away from the sun (bearing 0 = north, -y; 90 = east, +x)
        let out = Z_SCALE / height.to_radians().tan();
        let (s, c) = bearing.to_radians().sin_cos();
        let (wx, wy) = (-s * out, c * out);
        Sun { across: (wx - wy) / 4.0, down: 1.0 + (wx + wy) / 8.0, dark: DARK * dark }
    }
}
/// The ground must have its top this close to the height the shadow falls to.
const NEAR_Z: i32 = 8;
/// No ground at a pixel (a figure, or nothing).
const NONE: i32 = i32::MIN;

/// Whether a thing casts a shadow: the figures that stand (not the dead).
fn casts(it: &Item) -> bool {
    it.in_world() && it.is_actor() && !it.dead && it.flags & FLG_INVISIBLE == 0
}

/// Whether a figure stands under a roof (in a house: no sun there). The
/// same test that hides the roofs while the Avatar is below one.
fn under_roof(map: &MapData, infos: &[ShapeInfo], it: &Item) -> bool {
    let probe = crate::map::WorldBox { x: it.x, y: it.y, z: it.z, xd: 64, yd: 64, zd: 16 };
    map.position_info(&probe, &probe, infos, it.id).roof_z.is_some()
}

/// A figure casting a shadow, ready for the cores.
struct Caster<'a> {
    frame: &'a Frame,
    /// Screen position of the picture's anchor.
    sx: i32,
    sy: i32,
    flipped: bool,
    /// The height it stands at.
    z: i32,
}

/// Lays the shadows of the figures drawn (`painted`) onto the picture;
/// `ids`: the item drawn at every pixel (index into the map's items + 1).
pub fn apply(surf: &mut Surface, ids: &[u32], painted: &[Painted], map: &MapData, infos: &[ShapeInfo], shapes: &Shapes, sun: Sun) {
    let (w, h) = (surf.width, surf.height);
    let n = (w * h) as usize;
    if ids.len() != n || n == 0 || sun.dark <= 0.01 {
        return;
    }
    let items = &map.items;

    // the figures (their pictures held while the cores work)
    let mut held = Vec::new();
    let mut list = Vec::new();
    for p in painted {
        let Some(it) = items.get(p.index) else { continue };
        if !casts(it) || under_roof(map, infos, it) {
            continue;
        }
        let Some(shape) = shapes.peek(p.shape) else { continue };
        if shape.frames.get(p.frame as usize).is_none_or(|f| f.width == 0) {
            continue;
        }
        list.push((held.len(), p.frame as usize, p.sx, p.sy, p.flipped, it.z));
        held.push(shape);
    }
    if list.is_empty() {
        return;
    }
    let casters: Vec<Caster> = list.into_iter().map(|(k, frame, sx, sy, flipped, z)| Caster { frame: &held[k].frames[frame], sx, sy, flipped, z }).collect();
    let bands = (h as usize).div_ceil(crate::pool::cores() * 2).max(8);

    // the top of what is drawn at every pixel (not the figures)
    let mut tops = vec![NONE; n];
    let parts: Vec<(usize, &mut [i32])> = tops.chunks_mut(bands * w as usize).enumerate().collect();
    crate::pool::each(parts, |_, (b, part)| {
        let start = b * bands * w as usize;
        for (k, top) in part.iter_mut().enumerate() {
            let id = ids[start + k];
            let Some(under) = (id as usize).checked_sub(1).and_then(|i| items.get(i)) else { continue };
            let Some(info) = infos.get(under.shape as usize) else { continue };
            if !under.is_actor() {
                *top = under.z + info.z * 8;
            }
        }
    });

    // the pixels in shadow
    let shade: Vec<AtomicBool> = (0..n).map(|_| AtomicBool::new(false)).collect();
    crate::pool::run(casters.len(), &|i| {
        let c = &casters[i];
        let f = c.frame;
        // the lowest row drawn: the feet, on the ground
        let Some(lowest) = (0..f.height).rev().find(|&r| f.pixels[(r * f.width) as usize..((r + 1) * f.width) as usize].iter().any(|&p| p != TRANSPARENT)) else { return };
        let top = c.sy - f.yoff;
        for row in 0..=lowest {
            let line = &f.pixels[(row * f.width) as usize..((row + 1) * f.width) as usize];
            let z = c.z + (lowest - row);
            for (col, &px) in line.iter().enumerate() {
                if px == TRANSPARENT {
                    continue;
                }
                let x = if c.flipped { c.sx + f.xoff - 1 - col as i32 } else { c.sx - f.xoff + col as i32 };
                let y = top + row;
                // where the shadow lands on the ground the figure stands on;
                // if the ground there is higher or lower, on that ground
                let mut g = c.z;
                for _ in 0..3 {
                    let above = z - g;
                    if above < 0 {
                        break;
                    }
                    let (qx, qy) = (x + (above as f32 * sun.across).round() as i32, y + (above as f32 * sun.down).round() as i32);
                    if qx < 0 || qy < 0 || qx >= w || qy >= h {
                        break;
                    }
                    let q = (qy * w + qx) as usize;
                    let t = tops[q];
                    if t == NONE {
                        break;
                    }
                    if (t - g).abs() <= NEAR_Z {
                        shade[q].store(true, Ordering::Relaxed);
                        break;
                    }
                    g = t;
                }
            }
        }
    });

    // darken the ground under the shadows, softly at their edges (the
    // share of the 3 x 3 pixels around that are in shadow): first the
    // count across, then down
    let lit = |i: usize| shade[i].load(Ordering::Relaxed) as u8;
    let mut across = vec![0u8; n];
    let parts: Vec<(usize, &mut [u8])> = across.chunks_mut(w as usize).enumerate().collect();
    crate::pool::each(parts, |y, (_, row)| {
        let base = y * w as usize;
        for x in 0..w as usize {
            let mut c = lit(base + x);
            if x > 0 {
                c += lit(base + x - 1);
            }
            if x + 1 < w as usize {
                c += lit(base + x + 1);
            }
            row[x] = c;
        }
    });
    let parts: Vec<(usize, &mut [u32])> = surf.pixels.chunks_mut(bands * w as usize).enumerate().collect();
    crate::pool::each(parts, |_, (b, part)| {
        let start = b * bands * w as usize;
        for (k, px) in part.iter_mut().enumerate() {
            let i = start + k;
            let y = (i / w as usize) as i32;
            let mut count = across[i] as u32;
            if y > 0 {
                count += across[i - w as usize] as u32;
            }
            if y + 1 < h {
                count += across[i + w as usize] as u32;
            }
            if count == 0 || tops[i] == NONE {
                continue;
            }
            let k = 1.0 - sun.dark * count as f32 / 9.0;
            let ch = |v: u32| (v as f32 * k) as u32;
            *px = (ch((*px >> 16) & 0xFF) << 16) | (ch((*px >> 8) & 0xFF) << 8) | ch(*px & 0xFF);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::Sun;

    #[test]
    fn the_sun_moves_smoothly() {
        let len = |s: Sun| (s.across * s.across + (s.down - 1.0) * (s.down - 1.0)).sqrt();
        // no jumps from one minute to the next, all day round
        let mut last = Sun::at(0.0);
        for m in 1..=24 * 60 {
            let s = Sun::at(m as f32 / 60.0 % 24.0);
            assert!((s.across - last.across).abs() < 0.1 && (s.down - last.down).abs() < 0.1 && (s.dark - last.dark).abs() < 0.02, "jump at minute {m}: {last:?} -> {s:?}");
            last = s;
        }
        // long in the morning and evening, short at noon, faint at night
        assert!(len(Sun::at(12.0)) < len(Sun::at(7.0)) && len(Sun::at(12.0)) < len(Sun::at(17.0)));
        assert!(Sun::at(23.0).dark < Sun::at(12.0).dark);
        // in the morning to the right and up (north-east), in the evening right and down (south-east)
        assert!(Sun::at(7.0).down < Sun::at(17.0).down);
    }
}
