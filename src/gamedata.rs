//! The game's own data as read from its files when it starts: the shapes
//! and their type flags, the palette, animations, fonts, window graphics
//! and the tables for creatures and equipment.

use crate::anim::AnimDat;
use crate::flex::Flex;
use crate::font::Font;
use crate::hash::HashMap;
use crate::palette::Palette;
use crate::shape::{ShapeInfo, Shapes};

/// The game's own data from its files (gamedata.rs).
pub struct GameData {
    pub shapes: Shapes,
    pub infos: Vec<ShapeInfo>,
    pub pal: Palette,
    pub anims: AnimDat,
    pub font: Option<Font>,
    pub monsters: HashMap<u32, crate::combat::MonsterInfo>,
    /// Positions of the weapon in the Avatar's hand (WPNOVLAY.DAT).
    pub weapon_overlays: Flex,
    /// Weapon, armour and treasure tables.
    pub tables: crate::equip::Tables,
    /// Window graphics and, per gump shape, the area its contents lie in.
    pub gump_shapes: Shapes,
    pub gump_areas: Vec<[i32; 4]>,
    /// The fonts of the game by number (the main one is also in `font`).
    pub fonts: Vec<Option<Font>>,
    pub static_dir: std::path::PathBuf,
}

impl GameData {
    /// Font number `n`, or the usual one if the game has no such font.
    pub fn font_or_plain(&self, n: usize) -> Option<&Font> {
        self.fonts.get(n).and_then(|f| f.as_ref()).or(self.font.as_ref())
    }

    /// Notes for every creature shape the frames in which it lies on the
    /// ground, so the picture can put a body under whoever stands on it.
    pub fn mark_lying(&mut self) {
        use crate::anim::{DIE, STAND, STAND_UP};
        for shape in 0..self.infos.len() as u32 {
            if self.anims.action(shape, STAND).is_none() {
                continue;
            }
            let mut frames = Vec::new();
            if let Some(die) = self.anims.action(shape, DIE).filter(|a| a.size > 0) {
                frames.extend((0..8).map(|d| die.frame(d, die.size - 1).frame as u16));
            }
            if let Some(up) = self.anims.action(shape, STAND_UP).filter(|a| a.size > 0) {
                frames.extend((0..8).map(|d| up.frame(d, 0).frame as u16));
            }
            frames.sort_unstable();
            frames.dedup();
            let info = &mut self.infos[shape as usize];
            info.creature = true;
            for (slot, f) in info.lying.iter_mut().zip(frames) {
                *slot = f;
            }
        }
    }
}
