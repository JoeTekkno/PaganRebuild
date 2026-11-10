//! What the player sees and handles on top of the world: the windows of
//! containers and books, dragging, the menus, the savegame book, the
//! title, messages and what the pointer is over. (The code for each lives
//! in its own file: gumps.rs, drag.rs, menu.rs, slots.rs, title.rs, hover.rs ...)

use crate::hash::HashMap;

/// The windows, menus, books and other things on the screen for the player (ui.rs).
#[derive(Default)]
pub struct Ui {
    /// Where the enemy health bars were drawn, down to the creature's
    /// feet: (actor, x0, y0, x1, y1). A click there means the creature.
    pub bar_areas: Vec<(u16, i32, i32, i32, i32)>,
    /// Where the minimap was drawn (x0, y0, x1, y1), to keep clicks off the world.
    pub minimap_area: Option<(i32, i32, i32, i32)>,
    /// The page each slot window shows (option "better inventory").
    pub gump_pages: HashMap<u16, usize>,
    pub gumps: Vec<crate::gumps::Gump>,
    pub reader: Option<crate::gumps::Reader>,
    pub drag: Option<crate::drag::Drag>,
    /// A script waits for the player to click on a target (its process).
    pub targeting: Option<u16>,
    /// The target being picked is for the peace spell (its process).
    pub peace_target: Option<u16>,
    /// A script waits for the player to enter an amount.
    pub slider: Option<crate::magic::SliderInput>,
    /// A short message for the player and when it was put up.
    pub notice: Option<(String, std::time::Instant)>,
    /// The volume menu, while it is open.
    pub menu: Option<crate::menu::Menu>,
    /// The savegame book, while it is open.
    pub slot_menu: Option<crate::slots::SlotMenu>,
    pub hover: Option<u16>,
    /// What looking at the item under the pointer would say (hover.rs).
    pub hover_name: Option<String>,
    /// Where the mouse pointer is in the view (none outside the window).
    pub pointer: Option<(i32, i32)>,
    /// The key is held that lights up everything worth pointing out.
    pub show_all: bool,
    /// The main menu, the name entry, a movie or the credits.
    pub title: Option<crate::title::Title>,
    /// The large map with the list of places (option "guide", key O).
    pub worldmap: Option<crate::worldmap::WorldMap>,
}
