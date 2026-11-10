//! Stacks of potions (option "better inventory"; in the original every
//! potion is a thing of its own).
//!
//! Potions of one kind (the frame says which) lie on one stack whose amount
//! is kept in the quality field, as for coins; a single potion keeps its
//! original quality 0. The potion scripts know nothing of this: drinking
//! from a stack first takes one potion off it, and only that one is handed
//! to the script, which turns it into an empty bottle as usual. Now and
//! then the empty bottles and newly bought potions in the backpack join
//! their stacks again. With the option switched off, every stack falls
//! apart into single potions, so that the original inventory sees them as
//! the original game would.

use crate::game::Game;
use crate::map::{Item, ETHEREAL};

pub const POTION: u32 = 766;

/// The potions worth having at hand, in the order of their quick keys
/// (1 to 5) and of their slots in the backpack: yellow (a little health),
/// red (much health, harms the healthy), orange (mana), purple
/// (protection), black (invisible). The others: 0 empty bottle, 4 green,
/// 6 blue (sleep).
pub const QUICK_FRAMES: [u32; 5] = [3, 1, 2, 5, 7];

/// How often (ticks) the stacks are looked after.
const TIDY_TICKS: u32 = 15;

impl Game {
    /// For drinking: if `id` is a stack of potions, one potion comes off
    /// it (into the same container) and is returned; otherwise `id`.
    pub fn one_potion_off(&mut self, id: u16) -> u16 {
        let Some(it) = self.map().get(id).copied() else { return id };
        if it.shape != POTION || it.quality <= 1 {
            return id;
        }
        let single = self.map_mut().add(Item { id: 0, quality: 0, parent: ETHEREAL, fast: false, ..it });
        self.with_item_pub(id, |i| i.quality -= 1);
        if it.parent == crate::map::IN_WORLD {
            // a stack lying about: the single one stands beside it
            if let Some(idx) = self.map().index(single) {
                self.map_mut().place(idx, it.x, it.y, it.z);
            }
        } else {
            self.copy_into(single, it.parent, it.y);
        }
        single
    }

    /// Puts a new item straight into a container (no room check: what
    /// comes off a stack took up that room already).
    fn copy_into(&mut self, id: u16, container: u16, gump_pos: i32) {
        let Some(idx) = self.map().index(id) else { return };
        let map = self.map_mut();
        map.take(idx, container);
        let item = &mut map.items[idx];
        item.x = 0;
        item.y = gump_pos;
        item.z = 0;
        item.flags |= crate::drag::FLG_CONTAINED;
    }

    /// Once in a while: with the option on, loose potions in the backpack
    /// join their stacks; with it off, stacks fall apart into single ones.
    pub fn tidy_potions(&mut self) {
        if !self.tick.is_multiple_of(TIDY_TICKS) {
            return;
        }
        if self.options.better_inventory {
            if let Some(pack) = self.backpack() {
                self.join_all_stacks(pack);
            }
            return;
        }
        let stacks: Vec<Item> = self.map().items.iter().filter(|it| !it.gone && it.shape == POTION && it.quality > 1).copied().collect();
        for st in stacks {
            for k in 1..st.quality {
                let single = self.map_mut().add(Item { id: 0, quality: 0, parent: ETHEREAL, fast: false, ..st });
                if st.parent == crate::map::IN_WORLD {
                    if let Some(idx) = self.map().index(single) {
                        self.map_mut().place(idx, st.x + 8 * (k as i32 % 4), st.y + 8 * (k as i32 / 4), st.z);
                    }
                } else {
                    // a little apart in the original window, not one on top of the other
                    let (gx, gy) = ((st.y & 0xFF) + 6 * (k as i32 % 5), ((st.y >> 8) & 0xFF) + 4 * (k as i32 / 5));
                    self.copy_into(single, st.parent, gx.clamp(0, 255) | gy.clamp(0, 255) << 8);
                }
            }
            self.with_item_pub(st.id, |i| i.quality = 0);
        }
    }
}
