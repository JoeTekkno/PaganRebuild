//! The savegame book, drawn as in the original: on each double page six
//! entries ("Entry 1" .. "Entry 6", in the book's blue handwriting), each
//! with the name the player wrote under it and, beyond the original, the
//! time of day of the game kept there. The dog-ears at the top corners turn
//! the pages. The original has two double pages (12 entries); here the
//! first page also holds the quick save and the autosave, and the book
//! grows: there is always a free entry after the last one used. Saving
//! opens the book at that free entry, loading at the last entry used. The book
//! also appears when the Avatar has died, with "New game" below it.

use crate::game::Game;
use crate::menu::draw_stud;
use crate::savegame::{SlotInfo, AUTO_SLOT, QUICK_SLOT};
use crate::shape::{Frame, TRANSPARENT};
use crate::view::Surface;

const BOOK_SHAPE: u32 = 35;
/// The dog-ears: frame 0 on the left page, 1 on the right one.
const CORNER_SHAPE: u32 = 34;
/// "Entry" (frame 0), "Eintragung" (frame 2).
const ENTRY_SHAPE: u32 = 46;
/// The numbers 1 to 12 in the same handwriting.
const DIGITS_SHAPE: u32 = 36;
/// Where the dog-ears sit on the book: on the page, a little in from its
/// outer top corner.
const CORNERS: [(i32, i32); 2] = [(40, 3), (243, 3)];
/// Left edge of the entries on the two pages, top of the first entry,
/// and the room one entry takes.
const PAGE_X: [i32; 2] = [36, 162];
const TOP: i32 = 20;
const ENTRY_HEIGHT: i32 = 40;
/// Room for the name under an entry (with the cursor while it is written).
const TEXT_WIDTH: i32 = 104;
/// Places on a double page, and at least in the book (as in the original).
const PER_SPREAD: usize = 6;
const MIN_PLACES: usize = 12;
/// The quick save and the autosave come first.
const SPECIAL: [usize; 2] = [QUICK_SLOT, AUTO_SLOT];
/// The main menu's words ("1. Intro" ..., in English, French and German),
/// and the letters taken from them: (letter, frame, first and last column).
const MENU_WORDS_SHAPE: u32 = 37;
const LETTERS: [(char, usize, i32, i32); 20] = [
    ('Q', 10, 12, 21), ('u', 10, 22, 29), ('i', 10, 30, 33), ('t', 10, 34, 39), // Quit
    ('A', 42, 13, 26), // Abbrechen
    ('k', 40, 59, 66), // Mitwirkende
    ('v', 18, 59, 67), ('e', 18, 74, 80), // Lire Livre
    ('o', 6, 38, 44), ('s', 6, 52, 57), // Options
    ('S', 36, 13, 20), ('c', 36, 21, 26), ('h', 36, 27, 33), ('r', 36, 34, 39), // Schreiben
    ('n', 32, 22, 28), ('l', 32, 28, 33), // Einleitung
    ('p', 46, 40, 48), // Endspiel
    ('R', 2, 12, 20), ('a', 2, 27, 33), ('d', 2, 34, 43), // Read Diary
];
/// The font of the names (as in the original), and the dark blue one of
/// words that cannot be put together in handwriting.
const NAME_FONT: usize = 4;
const HEADING_FONT: usize = 9;

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Save,
    Load,
    /// The Avatar is dead: load a game or start over.
    Death,
}

pub struct SlotMenu {
    pub mode: Mode,
    /// The chosen place; one past the last: "New game" (after a death).
    pub selected: usize,
    /// The double page open.
    spread: usize,
    /// The slot of each place in the book, double page after double page.
    slots: Vec<usize>,
    /// Saving: the name being written for the selected slot (as in the
    /// original's diary, where the entry is given a description).
    pub editing: Option<String>,
}

/// The slot of entry `n` (slot 9 is the autosave).
fn entry_slot(n: usize) -> usize {
    if n <= 8 { n } else { n + 1 }
}

/// The entry number of a slot; None for the quick save and the autosave.
pub fn slot_entry(slot: usize) -> Option<usize> {
    match slot {
        QUICK_SLOT | AUTO_SLOT => None,
        s if s <= 8 => Some(s),
        s => Some(s - 1),
    }
}

impl SlotMenu {
    fn spreads(&self) -> usize {
        self.slots.len().div_ceil(PER_SPREAD)
    }

    /// Which double page is open, and how many there are (for tests).
    pub fn page(&self) -> (usize, usize) {
        (self.spread, self.spreads())
    }

    /// "New game" is there (after a death), as the last choice.
    fn choices(&self) -> usize {
        self.slots.len() + usize::from(self.mode == Mode::Death)
    }

    /// Can place `i` be chosen? (The autosave is not saved to by hand.)
    fn selectable(&self, i: usize) -> bool {
        match self.slots.get(i) {
            Some(&s) => !(self.mode == Mode::Save && s == AUTO_SLOT),
            None => i < self.choices(),
        }
    }

    fn select(&mut self, i: usize) {
        if self.selected != i {
            // another place: a name begun for the last one is dropped
            self.editing = None;
        }
        self.selected = i;
        if i < self.slots.len() {
            self.spread = i / PER_SPREAD;
        }
    }
}

impl Game {
    pub fn open_slot_menu(&mut self, mode: Mode) {
        let last = self.used_slots().into_iter().filter_map(slot_entry).max().unwrap_or(0);
        // saving: room for one more after the last used entry
        let wanted = SPECIAL.len() + if mode == Mode::Save { last + 1 } else { last };
        let places = wanted.max(MIN_PLACES).div_ceil(PER_SPREAD) * PER_SPREAD;
        let slots: Vec<usize> = SPECIAL.into_iter().chain((1..=places - SPECIAL.len()).map(entry_slot)).collect();
        let index_of = |s: usize| slots.iter().position(|&x| x == s);
        let selected = match mode {
            Mode::Save => index_of(entry_slot(last + 1)),
            // the last page: the last entry written down (the quick save
            // and the autosave on the first page are often newer)
            Mode::Load if last > 0 => index_of(entry_slot(last)),
            _ => {
                // the newest game there is
                let newest = slots.iter().enumerate().filter(|(_, &s)| self.slot_info(s).is_some()).filter_map(|(i, &s)| Some((self.slot_time(s)?, i))).max();
                // after a death without any: "New game"
                newest.map(|(_, i)| i).or((mode == Mode::Death).then_some(slots.len()))
            }
        };
        let mut m = SlotMenu { mode, selected: 0, spread: 0, slots, editing: None };
        m.select(selected.unwrap_or(0));
        self.ui.menu = None;
        self.ui.slot_menu = Some(m);
    }

    /// Up / down: the place before or after (over the page if need be).
    pub fn slot_menu_key(&mut self, dy: i32) {
        let Some(m) = &mut self.ui.slot_menu else { return };
        let n = m.choices() as i32;
        if dy == 0 || n == 0 {
            return;
        }
        let mut i = m.selected as i32;
        for _ in 0..n {
            i = (i + dy.signum()).rem_euclid(n);
            if m.selectable(i as usize) {
                break;
            }
        }
        m.editing = None;
        m.select(i as usize);
    }

    /// Left / right, or a click on a dog-ear: turns a page.
    pub fn slot_menu_turn(&mut self, dir: i32) {
        let Some(m) = &mut self.ui.slot_menu else { return };
        let spread = m.spread as i32 + dir.signum();
        if spread < 0 || spread >= m.spreads() as i32 {
            return;
        }
        let start = spread as usize * PER_SPREAD;
        let first = (start..start + PER_SPREAD).find(|&i| m.selectable(i)).unwrap_or(start);
        m.editing = None;
        m.select(first);
    }

    /// Esc: back to the game, except from the death list; while a name
    /// is written, only that stops.
    pub fn slot_menu_close(&mut self) {
        if let Some(m) = self.ui.slot_menu.as_mut().filter(|m| m.editing.is_some()) {
            m.editing = None;
            return;
        }
        if self.ui.slot_menu.as_ref().is_some_and(|m| m.mode != Mode::Death) {
            self.ui.slot_menu = None;
        }
    }

    /// Selects entry `n` of the book (0: the quick save; tests).
    pub fn slot_menu_select_entry(&mut self, n: usize) {
        if let Some(m) = &mut self.ui.slot_menu {
            let slot = if n == 0 { QUICK_SLOT } else { entry_slot(n) };
            if let Some(i) = m.slots.iter().position(|&s| s == slot) {
                m.select(i);
            }
        }
    }

    /// Enter or a click on the selected place.
    pub fn slot_menu_activate(&mut self) {
        let Some(m) = &self.ui.slot_menu else { return };
        if !m.selectable(m.selected) {
            return;
        }
        let (mode, slot) = (m.mode, m.slots.get(m.selected).copied());
        let Some(slot) = slot else {
            // "New game"
            self.ui.slot_menu = None;
            self.restart = true;
            return;
        };
        if mode == Mode::Save {
            // first the name is written, then it is saved
            let Some(name) = self.ui.slot_menu.as_ref().and_then(|m| m.editing.clone()) else {
                let old = self.slot_name(slot).unwrap_or_default();
                if let Some(m) = &mut self.ui.slot_menu {
                    m.editing = Some(old);
                }
                return;
            };
            self.ui.slot_menu = None;
            let saved = self.save_to(slot);
            if saved.is_ok() {
                self.set_slot_name(slot, &name);
            }
            let msg = match (saved, self.language) {
                (Ok(_), 'G') => "Spiel gespeichert.".to_string(),
                (Ok(_), 'F') => "Partie sauvegard\u{82}e.".to_string(),
                (Ok(_), _) => "Game saved.".to_string(),
                (Err(e), _) => e,
            };
            self.notify(&msg);
            return;
        }
        if self.slot_info(slot).is_none() {
            return; // an empty slot: nothing happens
        }
        match self.load_from(slot) {
            Ok(()) => {
                self.ui.slot_menu = None;
                let msg = self.tr(["Spiel geladen.", "Partie charg\u{82}e.", "Game loaded."]);
                self.notify(msg);
            }
            Err(e) => self.notify(&e),
        }
    }

    fn book_pos(&mut self, surf_w: i32, surf_h: i32) -> (i32, i32) {
        let (w, h) = self.data.gump_shapes.get(BOOK_SHAPE).and_then(|s| s.frames.first().map(|f| (f.width, f.height))).unwrap_or((307, 158));
        ((surf_w - w) / 2, (surf_h - h) / 2)
    }

    /// Top left corner of place `i` (on its double page) in the view.
    fn entry_pos(&mut self, i: usize, surf_w: i32, surf_h: i32) -> (i32, i32) {
        let (bx, by) = self.book_pos(surf_w, surf_h);
        let k = i % PER_SPREAD;
        (bx + PAGE_X[k / 3], by + TOP + (k % 3) as i32 * ENTRY_HEIGHT)
    }

    /// "New game" below the book: where it is written, and its text.
    fn new_game_pos(&mut self, surf_w: i32, surf_h: i32) -> Option<(i32, i32, String)> {
        if self.ui.slot_menu.as_ref()?.mode != Mode::Death {
            return None;
        }
        let text = self.tr(["Neues Spiel", "Nouvelle partie", "New game"]);
        let (_, by) = self.book_pos(surf_w, surf_h);
        let width = self.data.font.as_ref().map_or(60, |f| f.width(text));
        let book_h = self.data.gump_shapes.get(BOOK_SHAPE).and_then(|s| s.frames.first().map(|f| f.height)).unwrap_or(158);
        Some(((surf_w - width) / 2, by + book_h + 8, text.to_string()))
    }

    fn gump_frame(&mut self, shape: u32, frame: usize) -> Option<Frame> {
        self.data.gump_shapes.get(shape)?.frames.get(frame).cloned()
    }

    /// The dog-ears shown: (frame, left edge, top) in the view, with the
    /// way they turn.
    fn corners(&mut self, surf_w: i32, surf_h: i32) -> Vec<(Frame, i32, i32, i32)> {
        let Some((spread, spreads)) = self.ui.slot_menu.as_ref().map(|m| m.page()) else { return Vec::new() };
        let (bx, by) = self.book_pos(surf_w, surf_h);
        let mut out = Vec::new();
        for (side, dir) in [(0usize, -1i32), (1, 1)] {
            let wanted = if dir < 0 { spread > 0 } else { spread + 1 < spreads };
            if let (true, Some(f)) = (wanted, self.gump_frame(CORNER_SHAPE, side)) {
                out.push((f, bx + CORNERS[side].0, by + CORNERS[side].1, dir));
            }
        }
        out
    }

    fn name_font(&self) -> Option<&crate::font::Font> {
        self.data.font_or_plain(NAME_FONT)
    }

    /// Typed characters (in the game font's codes) and backspace while a
    /// name is written; it stays within the page's width.
    pub fn slot_menu_type(&mut self, chars: &[char], backspace: bool) {
        let Some(mut text) = self.ui.slot_menu.as_ref().and_then(|m| m.editing.clone()) else { return };
        if backspace {
            text.pop();
        }
        for &c in chars {
            let longer = format!("{text}{c}_");
            if self.name_font().is_none_or(|f| f.width(&longer) <= TEXT_WIDTH) {
                text.push(c);
            }
        }
        if let Some(m) = &mut self.ui.slot_menu {
            m.editing = Some(text);
        }
    }

    /// Mouse: a dog-ear turns the page, a click on a place selects and
    /// activates it.
    pub fn slot_menu_click(&mut self, x: i32, y: i32, surf_w: i32, surf_h: i32) {
        for (f, cx, cy, dir) in self.corners(surf_w, surf_h) {
            if x >= cx && x < cx + f.width && y >= cy && y < cy + f.height {
                self.slot_menu_turn(dir);
                return;
            }
        }
        if let Some((nx, ny, text)) = self.new_game_pos(surf_w, surf_h) {
            let width = self.data.font.as_ref().map_or(60, |f| f.width(&text));
            if x >= nx - 8 && x < nx + width + 8 && y >= ny - 2 && y < ny + 12 {
                if let Some(m) = &mut self.ui.slot_menu {
                    let i = m.slots.len();
                    m.select(i);
                }
                self.slot_menu_activate();
                return;
            }
        }
        let Some((spread, count)) = self.ui.slot_menu.as_ref().map(|m| (m.spread, m.slots.len())) else { return };
        let start = spread * PER_SPREAD;
        for i in start..(start + PER_SPREAD).min(count) {
            let (ex, ey) = self.entry_pos(i, surf_w, surf_h);
            if x >= ex - 8 && x < ex + TEXT_WIDTH + 8 && y >= ey - 2 && y < ey + ENTRY_HEIGHT - 2 {
                let Some(m) = &mut self.ui.slot_menu else { return };
                if !m.selectable(i) {
                    return;
                }
                m.select(i);
                self.slot_menu_activate();
                return;
            }
        }
    }

    /// The heading of the quick save's and the autosave's place.
    fn special_label(&self, slot: usize) -> &'static str {
        match (slot, self.language) {
            (QUICK_SLOT, 'G') => "Schnellspeicher",
            (QUICK_SLOT, 'F') => "Rapide",
            (QUICK_SLOT, _) => "Quicksave",
            (_, 'G') => "Autospeicher",
            (_, 'F') => "Auto",
            _ => "Autosave",
        }
    }

    fn slot_details(&self, info: Option<SlotInfo>) -> String {
        match (info, self.language) {
            (None, _) => String::new(),
            (Some(i), lang) => {
                // an hour of game time has 900 game seconds
                let (day, hour, minute) = (i.game_seconds / 900 / 24 + 1, i.game_seconds / 900 % 24, i.game_seconds % 900 / 15);
                let d = match lang {
                    'G' => "Tag",
                    'F' => "Jour",
                    _ => "Day",
                };
                format!("{d} {day}, {hour}:{minute:02}")
            }
        }
    }

    /// The pictures of a number in the book's handwriting (1 to 12 are
    /// there; larger ones are put together, the 0 taken from the 10).
    fn number_frames(&mut self, n: usize) -> Vec<Frame> {
        if (1..=12).contains(&n) {
            return self.gump_frame(DIGITS_SHAPE, n - 1).into_iter().collect();
        }
        let zero = self.gump_frame(DIGITS_SHAPE, 9).map(|ten| {
            // the columns after the gap behind the 1
            let opaque = |x: i32| (0..ten.height).any(|y| ten.pixels[(y * ten.width + x) as usize] != TRANSPARENT);
            let one_end = (0..ten.width).find(|&x| opaque(x)).unwrap_or(0);
            let gap = (one_end..ten.width).find(|&x| !opaque(x)).unwrap_or(0);
            let start = (gap..ten.width).find(|&x| opaque(x)).unwrap_or(gap);
            let width = ten.width - start;
            let pixels = (0..ten.height).flat_map(|y| (start..ten.width).map(move |x| (y, x))).map(|(y, x)| ten.pixels[(y * ten.width + x) as usize]).collect();
            Frame { width, height: ten.height, xoff: 0, yoff: 0, pixels }
        });
        n.to_string()
            .chars()
            .filter_map(|c| match c.to_digit(10)? as usize {
                0 => zero.clone(),
                d => self.gump_frame(DIGITS_SHAPE, d - 1),
            })
            .collect()
    }

    /// A word the book has no picture for, put together from letters cut
    /// out of the main menu's words (written in the same hand). None if a
    /// letter is missing.
    fn handwriting(&mut self, text: &str) -> Option<Frame> {
        let mut parts = Vec::new();
        for c in text.chars() {
            let &(_, frame, x0, x1) = LETTERS.iter().find(|l| l.0 == c)?;
            parts.push((self.gump_frame(MENU_WORDS_SHAPE, frame)?, x0, x1));
        }
        let width: i32 = parts.iter().map(|(_, x0, x1)| x1 - x0 + 1).sum();
        let height = parts.iter().map(|(f, _, _)| f.height).max()?;
        let mut pixels = vec![TRANSPARENT; (width * height) as usize];
        let mut left = 0;
        for (f, x0, x1) in parts {
            // all the words stand on the same line (their row 9)
            for y in 0..f.height {
                for x in x0..=x1.min(f.width - 1) {
                    let c = f.pixels[(y * f.width + x) as usize];
                    if c != TRANSPARENT {
                        pixels[(y * width + left + x - x0) as usize] = c;
                    }
                }
            }
            left += x1 - x0 + 1;
        }
        Some(Frame { width, height, xoff: 0, yoff: 0, pixels })
    }

    pub fn draw_slot_menu(&mut self, surf: &mut Surface) {
        let Some((mode, selected, spread, slots, editing)) =
            self.ui.slot_menu.as_ref().map(|m| (m.mode, m.selected, m.spread, m.slots.clone(), m.editing.clone()))
        else {
            return;
        };
        // the world dims behind the book (black after a death)
        if mode == Mode::Death {
            surf.pixels.fill(0);
        } else {
            surf.dim();
        }
        let (w, h) = (surf.width, surf.height);
        let (bx, by) = self.book_pos(w, h);
        if let Some(f) = self.gump_frame(BOOK_SHAPE, 0) {
            surf.paint(&f, bx + f.xoff, by + f.yoff, false, false, &self.data.pal);
        }
        for (f, cx, cy, _) in self.corners(w, h) {
            surf.paint(&f, cx + f.xoff, cy + f.yoff, false, false, &self.data.pal);
        }
        // what the book is for, above it
        let title = match (mode, self.language) {
            (Mode::Save, 'G') => "Spiel speichern",
            (Mode::Save, 'F') => "Sauvegarder",
            (Mode::Save, _) => "Save game",
            (Mode::Load, 'G') => "Spiel laden",
            (Mode::Load, 'F') => "Charger",
            (Mode::Load, _) => "Load game",
            (Mode::Death, 'G') => "Ihr seid tot.",
            (Mode::Death, 'F') => "Vous \u{88}tes mort.",
            (Mode::Death, _) => "You have died.",
        };
        if let Some(font) = &self.data.font {
            font.draw(surf, title, (w - font.width(title)) / 2, by - font.height - 6, &self.data.pal);
        }
        if let Some((nx, ny, text)) = self.new_game_pos(w, h) {
            if let Some(font) = &self.data.font {
                font.draw(surf, &text, nx, ny, &self.data.pal);
            }
            if selected == slots.len() {
                draw_stud(surf, nx - 7, ny + 5, true, true);
            }
        }
        let entry_word = self.gump_frame(ENTRY_SHAPE, if self.language == 'G' { 2 } else { 0 });
        let start = spread * PER_SPREAD;
        for i in start..(start + PER_SPREAD).min(slots.len()) {
            let s = slots[i];
            let (ex, ey) = self.entry_pos(i, w, h);
            // the heading: "Entry n" in the book's handwriting
            match slot_entry(s) {
                Some(n) => {
                    let mut x = ex;
                    if let Some(f) = &entry_word {
                        surf.paint(f, x + f.xoff, ey + f.yoff, false, false, &self.data.pal);
                        x += f.width + 3;
                    }
                    for f in self.number_frames(n) {
                        // the numbers stand on the same line as the word
                        surf.paint(&f, x + f.xoff, ey + 2 + f.yoff, false, false, &self.data.pal);
                        x += f.width + 1;
                    }
                }
                None => {
                    let label = self.special_label(s);
                    if let Some(f) = self.handwriting(label) {
                        surf.paint(&f, ex, ey + 1, false, false, &self.data.pal);
                    } else if let Some(font) = self.data.font_or_plain(HEADING_FONT) {
                        font.draw(surf, label, ex, ey + 2, &self.data.pal);
                    }
                }
            }
            if i == selected {
                draw_stud(surf, ex - 7, ey + 7, true, true);
            }
            // under it the name (being written: with a cursor) and the time
            let name = match &editing {
                Some(text) if i == selected => format!("{text}_"),
                _ => self.slot_name(s).unwrap_or_default(),
            };
            let details = self.slot_details(self.slot_info(s));
            if let Some(font) = self.name_font() {
                // what is too long for the page is cut short
                let fit = |text: &str| -> String {
                    let mut t = text.to_string();
                    while font.width(&t) > TEXT_WIDTH && t.pop().is_some() {}
                    t
                };
                // (without a name the time moves up)
                for (row, text) in [name, details].iter().filter(|t| !t.is_empty()).enumerate() {
                    font.draw(surf, &fit(text), ex, ey + 17 + row as i32 * font.height, &self.data.pal);
                }
            }
        }
    }
}
