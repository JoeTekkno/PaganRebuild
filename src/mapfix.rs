//! Corrections to the game's own map data, applied when a map is built
//! and when a savegame is loaded (savegames carry their maps).
//!
//! Upper Catacombs (map 50): stretches of corridor over lava are laid out
//! from pattern chunks when they come into view: before the earthquake a
//! whole floor, after it a floor whose middle has fallen in. In the
//! patterns of two of them the side floor lies 40 too high (48 instead of
//! 8): laid out, it hung in the air, the lava lay open beside the walkway
//! (with a black gap along its edge) and the floor in the air was painted
//! over the Avatar's legs. Here the side floor goes to its height, also in
//! a corridor that is laid out at the moment, and comes back where an
//! earlier version of this correction had taken it away.
//!
//! Caves under the volcano (map 62): beside the lava lake five pieces of
//! the corridor floor are missing (a hole into the void the Avatar fell
//! through), and one piece lies 32 off. Six pieces of the same floor lie
//! at height 0 instead, unseen under the corridor - left there by mistake.
//! The missing pieces are put in.
//!
//! Upper Catacombs (map 50), once the dead have been talked to: an egg
//! clears a region and lays new floor in small pieces. The clearing takes
//! the large floor plates of the room beside it too (they reach into the
//! region), but the new floor begins a row later: a strip without floor
//! stays (black, the Avatar stood in it). That row is laid as well.

use crate::eggs::FLG_FAST_ONLY;
use crate::map::{Item, MapData};

const CATACOMBS: usize = 50;
const FLOOR: u32 = 556;
/// Chunks are 512 square.
const CHUNK: i32 = 0x7E00;
/// Height of the side floor in the patterns: wrong and right.
const WRONG: i32 = 48;
const RIGHT: i32 = 8;
/// What the corridors are laid out higher than their patterns.
const LAID_OUT_ABOVE: i32 = 40;

/// The corridors (chunk) with their patterns before and after the earthquake.
const CORRIDORS: [((i32, i32), [(i32, i32); 2]); 2] =
    [((17920, 5120), [(29696, 20480), (29696, 19456)]), ((17920, 5632), [(29696, 22528), (29696, 21504)])];

fn chunk(it: &Item) -> (i32, i32) {
    (it.x & CHUNK, it.y & CHUNK)
}

const CAVES: usize = 62;
/// Cave floor.
const CAVE_FLOOR: u32 = 301;
/// Floor pieces of the caves that are missing: x, y, z, frame.
const CAVE_MISSING: [(i32, i32, i32, u32); 5] =
    [(15743, 6015, 48, 2), (15871, 6015, 48, 6), (15871, 6143, 48, 2), (15999, 6143, 48, 0), (16127, 6143, 48, 1)];
/// A floor piece that lies off: from, to.
const CAVE_MOVED: [((i32, i32, i32), (i32, i32, i32)); 1] = [((15967, 6015, 48), (15999, 6015, 48))];

fn fix_caves(map: &mut MapData) {
    let at = |map: &MapData, (x, y, z): (i32, i32, i32)| {
        map.items.iter().position(|it| it.in_world() && it.shape == CAVE_FLOOR && (it.x, it.y, it.z) == (x, y, z))
    };
    for (from, to) in CAVE_MOVED {
        if let (Some(i), None) = (at(map, from), at(map, to)) {
            map.place(i, to.0, to.1, to.2);
        }
    }
    for (x, y, z, frame) in CAVE_MISSING {
        if at(map, (x, y, z)).is_none() {
            map.add(Item { shape: CAVE_FLOOR, frame, x, y, z, mapnum: CAVES as u16, ..Item::default() });
        }
    }
}

/// The floor the clearing egg lays, the first piece of it (x, y, z), and
/// the row before it that it leaves bare: x of each piece, y, z.
const WIPE_FLOOR: u32 = 349;
const WIPE_FIRST: (i32, i32, i32) = (16992, 1504, 48);
const WIPE_GAP: ([i32; 4], i32, i32) = ([17023, 17151, 17279, 17407], 1407, 48);

/// After the clearing egg in the catacombs has laid its floor: the row it
/// leaves bare gets floor too. Cheap enough to call often (it looks only
/// at the few things there).
pub fn fix_wiped(n: usize, map: &mut MapData) {
    if n != CATACOMBS {
        return;
    }
    let near = |map: &MapData, (x, y, z): (i32, i32, i32)| {
        map.items_near(x, y, 4).into_iter().any(|i| {
            let it = &map.items[i];
            it.in_world() && it.shape == WIPE_FLOOR && (it.x, it.y, it.z) == (x, y, z)
        })
    };
    if !near(map, WIPE_FIRST) {
        return;
    }
    let (xs, y, z) = WIPE_GAP;
    for (k, x) in xs.into_iter().enumerate() {
        if !near(map, (x, y, z)) {
            map.add(Item { shape: WIPE_FLOOR, frame: 12 + k as u32 % 4, x, y, z, mapnum: CATACOMBS as u16, ..Item::default() });
        }
    }
    // the big floor tiles at its western edge, which a version that
    // searched the egg's area a little wider cleared away too (savegames
    // from then had a black hole there)
    for (x, y, z, frame) in WIPE_EDGE {
        let there = map.items_near(x, y, 4).into_iter().any(|i| {
            let it = &map.items[i];
            it.in_world() && it.shape == EDGE_FLOOR && (it.x, it.y, it.z) == (x, y, z)
        });
        if !there {
            map.add(Item { shape: EDGE_FLOOR, frame, x, y, z, mapnum: CATACOMBS as u16, ..Item::default() });
        }
    }
}

/// The floor tiles at the western edge of the clearing egg's area.
const EDGE_FLOOR: u32 = 352;
const WIPE_EDGE: [(i32, i32, i32, u32); 3] = [(16895, 1535, 48, 6), (16895, 1791, 48, 2), (16895, 2047, 48, 4)];

/// Applies the corrections for map `n`; `original` builds the map as the
/// game data have it (only called for maps that need it).
pub fn fix_map(n: usize, map: &mut MapData, original: impl FnOnce() -> MapData) {
    if n == CAVES {
        fix_caves(map);
    }
    fix_wiped(n, map);
    if n != CATACOMBS {
        return;
    }
    let original = original();
    let floor_at = |map: &MapData, x: i32, y: i32, z: i32, copy: bool| {
        map.items.iter().position(|it| {
            it.in_world() && it.shape == FLOOR && (it.x, it.y, it.z) == (x, y, z) && (it.flags & FLG_FAST_ONLY != 0) == copy
        })
    };
    for (corridor, patterns) in CORRIDORS {
        // the side floor of the patterns, as the game data have it
        let sides: Vec<Item> = original
            .items
            .iter()
            .filter(|it| it.in_world() && it.shape == FLOOR && it.z == WRONG && patterns.contains(&chunk(it)))
            .copied()
            .collect();
        for side in &sides {
            match floor_at(map, side.x, side.y, WRONG, false) {
                Some(i) => map.items[i].z = RIGHT,
                None if floor_at(map, side.x, side.y, RIGHT, false).is_none() => {
                    map.add(Item { id: 0, z: RIGHT, ..*side });
                }
                None => {}
            }
        }
        // a corridor laid out at the moment (its copies are in the savegame)
        let laid_out = map.items.iter().any(|it| it.in_world() && it.flags & FLG_FAST_ONLY != 0 && chunk(it) == corridor);
        if !laid_out {
            continue;
        }
        let pattern = patterns[0];
        for side in sides.iter().filter(|s| chunk(s) == pattern) {
            let (x, y) = (corridor.0 | (side.x & 511), corridor.1 | (side.y & 511));
            let (wrong, right) = (WRONG + LAID_OUT_ABOVE, RIGHT + LAID_OUT_ABOVE);
            match floor_at(map, x, y, wrong, true) {
                Some(i) => map.items[i].z = right,
                None if floor_at(map, x, y, right, true).is_none() => {
                    map.add(Item { id: 0, x, y, z: right, flags: FLG_FAST_ONLY, fast: true, ..*side });
                }
                None => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> MapData {
        let globs = crate::flex::Flex::from_bytes(vec![0; 0x80]).unwrap();
        MapData::build(&[], &[], &globs, &[])
    }

    /// The map as the game data have it: just the pattern's side floor.
    fn original(side: Item) -> MapData {
        let mut map = empty();
        map.add(side);
        map
    }

    #[test]
    fn side_floor_goes_down_to_its_height() {
        let side = Item { shape: FLOOR, frame: 36, x: 29823, y: 20607, z: WRONG, ..Item::default() };
        // a savegame: pattern floor still too high, corridor laid out with the copy in the air
        let mut map = empty();
        let pat = map.add(side);
        let copy = map.add(Item { shape: FLOOR, frame: 36, x: 18047, y: 5247, z: 88, flags: FLG_FAST_ONLY, ..Item::default() });
        let wall_top = map.add(Item { shape: FLOOR, x: 18431, y: 5247, z: 136, flags: FLG_FAST_ONLY, ..Item::default() });
        fix_map(CATACOMBS, &mut map, || original(side));
        assert_eq!(map.get(pat).map(|it| it.z), Some(RIGHT));
        assert_eq!(map.get(copy).map(|it| it.z), Some(48));
        assert_eq!(map.get(wall_top).map(|it| it.z), Some(136));
    }

    #[test]
    fn side_floor_taken_away_comes_back() {
        let side = Item { shape: FLOOR, frame: 36, x: 29823, y: 20607, z: WRONG, ..Item::default() };
        // a savegame where the side floor (pattern and copy) was taken away
        let mut map = empty();
        map.add(Item { shape: 409, x: 18047, y: 5247, z: 40, flags: FLG_FAST_ONLY, ..Item::default() });
        fix_map(CATACOMBS, &mut map, || original(side));
        let at = |x, y, z| map.items.iter().any(|it| it.in_world() && it.shape == FLOOR && (it.x, it.y, it.z) == (x, y, z));
        assert!(at(29823, 20607, RIGHT) && at(18047, 5247, 48));
        // applied twice it changes nothing more
        let count = map.items.iter().filter(|it| it.in_world()).count();
        fix_map(CATACOMBS, &mut map, || original(side));
        assert_eq!(map.items.iter().filter(|it| it.in_world()).count(), count);
    }
}
