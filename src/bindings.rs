//! The player's own keys and gamepad buttons.
//!
//! Each thing the player can do from the keyboard has the key the game
//! knows it by (its "own" key, the one in the help page). A key chosen
//! for it instead is passed on to the game as that own key (see
//! `Window::set_key_map`), and the own key then does nothing unless
//! something else was given it. The gamepad's buttons press the own keys
//! the same way, or the mouse buttons.
//!
//! The choices are kept in keys.ini beside settings.ini.

use crate::hash::HashMap;
use crate::platform::{Key, Pad};

/// What a binding does in the game.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Does {
    Key(Key),
    /// A mouse button: 0 left, 1 right.
    Mouse(usize),
}

pub struct Action {
    /// The name in keys.ini.
    pub id: &'static str,
    /// German, French, English.
    pub names: [&'static str; 3],
    pub does: Does,
    /// The gamepad button it has unless the player chose another.
    pub pad: Option<Pad>,
}

const fn act(id: &'static str, names: [&'static str; 3], key: Key, pad: Option<Pad>) -> Action {
    Action { id, names, does: Does::Key(key), pad }
}

pub const ACTIONS: [Action; 34] = [
    act("up", ["Gehen: hoch", "Marcher: haut", "Walk up"], Key::W, None),
    act("down", ["Gehen: runter", "Marcher: bas", "Walk down"], Key::S, None),
    act("left", ["Gehen: links", "Marcher: gauche", "Walk left"], Key::A, None),
    act("right", ["Gehen: rechts", "Marcher: droite", "Walk right"], Key::D, None),
    act("run", ["Rennen", "Courir", "Run"], Key::LeftShift, Some(Pad::LStick)),
    act("use", ["Benutzen", "Utiliser", "Use"], Key::E, Some(Pad::A)),
    act("jump", ["Springen", "Sauter", "Jump"], Key::J, Some(Pad::B)),
    act("attack", ["Angreifen", "Attaquer", "Attack"], Key::F, Some(Pad::X)),
    act("kick", ["Treten", "Coup de pied", "Kick"], Key::Q, Some(Pad::Left)),
    // K uses the key ring, as in the original
    act("keyring", ["Schl\u{81}sselring", "Trousseau de cl\u{82}s", "Key ring"], Key::K, None),
    act("block", ["Abwehren", "Parer", "Block"], Key::B, Some(Pad::RB)),
    act("combat", ["Kampfmodus", "Mode combat", "Combat mode"], Key::Tab, Some(Pad::LB)),
    act("inventory", ["Rucksack", "Sac", "Backpack"], Key::I, Some(Pad::Y)),
    act("close", ["Fenster schlie\u{e1}en", "Fermer les fen\u{88}tres", "Close windows"], Key::C, Some(Pad::Right)),
    act("menu", ["Men\u{81}", "Menu", "Menu"], Key::V, Some(Pad::Start)),
    act("quicksave", ["Schnell speichern", "Sauvegarde rapide", "Quick save"], Key::F5, None),
    act("quickload", ["Schnell laden", "Chargement rapide", "Quick load"], Key::F9, None),
    act("journal", ["Tagebuch", "Journal", "Journal"], Key::T, Some(Pad::Back)),
    act("highlight", ["Alles hervorheben", "Tout montrer", "Highlight all"], Key::H, Some(Pad::RStick)),
    act("zoomin", ["Zoom +", "Zoom +", "Zoom in"], Key::Equal, Some(Pad::Up)),
    act("zoomout", ["Zoom -", "Zoom -", "Zoom out"], Key::Minus, Some(Pad::Down)),
    act("turnleft", ["Ansicht drehen links", "Tourner la vue \u{85} gauche", "Turn view left"], Key::Comma, None),
    act("turnright", ["Ansicht drehen rechts", "Tourner la vue \u{85} droite", "Turn view right"], Key::Period, None),
    act("lighting", ["Beleuchtung", "Eclairage", "Lighting"], Key::L, None),
    act("minimap", ["Minikarte", "Mini-carte", "Minimap"], Key::M, None),
    act("worldmap", ["Karte und Wegweiser", "Carte et chemin", "Map and guide"], Key::O, None),
    act("roofs", ["D\u{84}cher", "Toits", "Roofs"], Key::R, None),
    // the potions keep their number keys as own keys: the keys chosen count
    // in the world only, so in a conversation the numbers still answer
    act("potion1", ["Trank gelb", "Potion jaune", "Yellow potion"], Key::Key1, None),
    act("potion2", ["Trank rot", "Potion rouge", "Red potion"], Key::Key2, None),
    act("potion3", ["Trank orange", "Potion orange", "Orange potion"], Key::Key3, None),
    act("potion4", ["Trank lila", "Potion violette", "Purple potion"], Key::Key4, None),
    act("potion5", ["Trank schwarz", "Potion noire", "Black potion"], Key::Key5, None),
    Action { id: "click", names: ["Linke Maustaste", "Bouton gauche", "Left button"], does: Does::Mouse(0), pad: Some(Pad::RT) },
    Action { id: "rclick", names: ["Rechte Maustaste", "Bouton droit", "Right button"], does: Does::Mouse(1), pad: Some(Pad::LT) },
];

/// The keys and buttons chosen, one each per action (None: none).
#[derive(Clone, PartialEq, Debug)]
pub struct Bindings {
    pub keys: Vec<Option<Key>>,
    pub pads: Vec<Option<Pad>>,
}

impl Default for Bindings {
    fn default() -> Bindings {
        Bindings {
            keys: ACTIONS.iter().map(own_key).collect(),
            pads: ACTIONS.iter().map(|a| a.pad).collect(),
        }
    }
}

fn own_key(a: &Action) -> Option<Key> {
    match a.does {
        Does::Key(k) => Some(k),
        Does::Mouse(_) => None,
    }
}

/// Keys that cannot be given to an action: Esc stops the choosing.
pub fn can_bind(k: Key) -> bool {
    !matches!(k, Key::Escape | Key::LeftAlt | Key::RightAlt)
}

fn path() -> Option<std::path::PathBuf> {
    Some(crate::menu::settings_path()?.with_file_name("keys.ini"))
}

/// Names of keys in keys.ini and on screen.
pub fn key_name(k: Key) -> String {
    match k {
        Key::Key0 | Key::Key1 | Key::Key2 | Key::Key3 | Key::Key4 | Key::Key5 | Key::Key6 | Key::Key7 | Key::Key8 | Key::Key9 => {
            format!("{k:?}")[3..].to_string()
        }
        Key::Equal => "Plus".into(),
        Key::Minus => "-".into(),
        Key::Comma => ",".into(),
        Key::Period => ".".into(),
        Key::Apostrophe => "'".into(),
        Key::LeftShift => "Shift".into(),
        Key::RightShift => "RShift".into(),
        Key::LeftCtrl => "Ctrl".into(),
        Key::RightCtrl => "RCtrl".into(),
        _ => format!("{k:?}"),
    }
}

const ALL_KEYS: [Key; 87] = {
    use Key::*;
    [
        A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
        Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
        F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
        Up, Down, Left, Right, PageUp, PageDown, Home, End, Insert, Delete,
        Escape, Enter, Space, Tab, Backspace,
        LeftShift, RightShift, LeftCtrl, RightCtrl, LeftAlt, RightAlt,
        Comma, Period, Minus, Equal, Apostrophe,
        NumPad0, NumPad1, NumPad2, NumPad3, NumPad4, NumPad5, NumPad6, NumPad7, NumPad8, NumPad9,
        NumPadMinus, NumPadPlus, NumPadEnter,
    ]
};

fn key_by_name(s: &str) -> Option<Key> {
    ALL_KEYS.iter().copied().find(|&k| key_name(k).eq_ignore_ascii_case(s))
}

pub fn pad_name(p: Pad) -> &'static str {
    match p {
        Pad::A => "A",
        Pad::B => "B",
        Pad::X => "X",
        Pad::Y => "Y",
        Pad::LB => "LB",
        Pad::RB => "RB",
        Pad::LT => "LT",
        Pad::RT => "RT",
        Pad::Back => "Back",
        Pad::Start => "Start",
        Pad::LStick => "L3",
        Pad::RStick => "R3",
        Pad::Up => "DUp",
        Pad::Down => "DDown",
        Pad::Left => "DLeft",
        Pad::Right => "DRight",
    }
}

fn pad_by_name(s: &str) -> Option<Pad> {
    Pad::ALL.iter().copied().find(|&p| pad_name(p).eq_ignore_ascii_case(s))
}

impl Bindings {
    pub fn load() -> Bindings {
        let mut b = Bindings::default();
        let Some(text) = path().and_then(|p| std::fs::read_to_string(p).ok()) else { return b };
        b.read(&text);
        b
    }

    fn read(&mut self, text: &str) {
        // a file from before the key ring had its key: kicking was on K then
        let before_keyring = !text.lines().any(|l| l.split_once('=').is_some_and(|(n, _)| n.trim() == "keyring"));
        for line in text.lines() {
            let Some((name, val)) = line.split_once('=') else { continue };
            let (name, val) = (name.trim(), val.trim());
            let (id, pad) = match name.strip_suffix(".pad") {
                Some(id) => (id, true),
                None => (name, false),
            };
            let Some(i) = ACTIONS.iter().position(|a| a.id == id) else { continue };
            if pad {
                self.pads[i] = pad_by_name(val);
            } else if own_key(&ACTIONS[i]).is_some() {
                self.keys[i] = key_by_name(val).filter(|&k| can_bind(k));
            }
        }
        if before_keyring {
            let at = |id: &str| ACTIONS.iter().position(|a| a.id == id).unwrap_or(0);
            let (kick, ring) = (at("kick"), at("keyring"));
            if self.keys[kick] == Some(Key::K) {
                // kicking goes to Q (if nothing else has it), K to the key ring
                let q_free = !self.keys.contains(&Some(Key::Q));
                self.keys[kick] = if q_free { Some(Key::Q) } else { None };
                self.keys[ring] = Some(Key::K);
            }
        }
        // a key or button twice (a file written by hand): the later one loses it
        for i in 0..ACTIONS.len() {
            for j in 0..i {
                if self.keys[i].is_some() && self.keys[i] == self.keys[j] {
                    self.keys[i] = None;
                }
                if self.pads[i].is_some() && self.pads[i] == self.pads[j] {
                    self.pads[i] = None;
                }
            }
        }
    }

    fn text(&self) -> String {
        let mut s = String::new();
        for (i, a) in ACTIONS.iter().enumerate() {
            if own_key(a).is_some() {
                s += &format!("{}={}\n", a.id, self.keys[i].map(key_name).unwrap_or_default());
            }
            s += &format!("{}.pad={}\n", a.id, self.pads[i].map(pad_name).unwrap_or_default());
        }
        s
    }

    pub fn save(&self) {
        if let Some(p) = path() {
            if let Some(dir) = p.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(p, self.text());
        }
    }

    /// Gives action `i` the key `k`; an action that had it gets the key
    /// `i` had (the two swap).
    pub fn set_key(&mut self, i: usize, k: Option<Key>) {
        if own_key(&ACTIONS[i]).is_none() {
            return;
        }
        if let Some(j) = (0..ACTIONS.len()).find(|&j| j != i && k.is_some() && self.keys[j] == k) {
            self.keys[j] = self.keys[i];
        }
        self.keys[i] = k;
    }

    /// The same for the gamepad.
    pub fn set_pad(&mut self, i: usize, p: Option<Pad>) {
        if let Some(j) = (0..ACTIONS.len()).find(|&j| j != i && p.is_some() && self.pads[j] == p) {
            self.pads[j] = self.pads[i];
        }
        self.pads[i] = p;
    }

    /// For the window: which key is taken as which, and which own keys
    /// now do nothing.
    pub fn key_map(&self) -> HashMap<Key, Option<Key>> {
        let mut map = HashMap::default();
        for (i, a) in ACTIONS.iter().enumerate() {
            let Some(own) = own_key(a) else { continue };
            if let Some(k) = self.keys[i].filter(|&k| k != own) {
                map.insert(k, Some(own));
            }
        }
        for (i, a) in ACTIONS.iter().enumerate() {
            let Some(own) = own_key(a) else { continue };
            if self.keys[i] != Some(own) {
                map.entry(own).or_insert(None);
            }
        }
        map
    }

    /// The key the player gave to potion `n` (0 yellow .. 4 black).
    pub fn potion_key(&self, n: usize) -> Option<Key> {
        let id = ["potion1", "potion2", "potion3", "potion4", "potion5"].get(n)?;
        self.keys[ACTIONS.iter().position(|a| a.id == *id)?]
    }

    /// What the pad's button does in the game, if anything.
    pub fn pad_does(&self, p: Pad) -> Option<Does> {
        let i = self.pads.iter().position(|&q| q == Some(p))?;
        Some(ACTIONS[i].does)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_and_map() {
        let mut b = Bindings::default();
        assert!(b.key_map().is_empty());
        let jump = ACTIONS.iter().position(|a| a.id == "jump").unwrap();
        let use_ = ACTIONS.iter().position(|a| a.id == "use").unwrap();
        // jump on Z: Z is J, J does nothing
        b.set_key(jump, Some(Key::Z));
        let m = b.key_map();
        assert_eq!(m.get(&Key::Z), Some(&Some(Key::J)));
        assert_eq!(m.get(&Key::J), Some(&None));
        // use on Z: jump gets E (the key use had), so the two swap
        b.set_key(use_, Some(Key::Z));
        assert_eq!(b.keys[jump], Some(Key::E));
        let m = b.key_map();
        assert_eq!(m.get(&Key::Z), Some(&Some(Key::E)));
        assert_eq!(m.get(&Key::E), Some(&Some(Key::J)));
        // saved and read back
        let mut c = Bindings::default();
        c.read(&b.text());
        assert_eq!(b, c);
        // pad swap
        b.set_pad(jump, Some(Pad::A));
        assert_eq!(b.pads[use_], Some(Pad::B));
        assert_eq!(b.pad_does(Pad::A), Some(Does::Key(Key::J)));
    }

    #[test]
    fn potion_keys() {
        let mut b = Bindings::default();
        assert_eq!(b.potion_key(0), Some(Key::Key1));
        assert_eq!(b.potion_key(4), Some(Key::Key5));
        // yellow on Z: Z drinks it (as 1), 1 does nothing in the world
        let yellow = ACTIONS.iter().position(|a| a.id == "potion1").unwrap();
        b.set_key(yellow, Some(Key::Z));
        assert_eq!(b.potion_key(0), Some(Key::Z));
        let m = b.key_map();
        assert_eq!(m.get(&Key::Z), Some(&Some(Key::Key1)));
        assert_eq!(m.get(&Key::Key1), Some(&None));
    }

    #[test]
    fn old_file_kick_on_k() {
        // a keys.ini from before the key ring: kicking on K
        let kick = ACTIONS.iter().position(|a| a.id == "kick").unwrap();
        let ring = ACTIONS.iter().position(|a| a.id == "keyring").unwrap();
        let mut b = Bindings::default();
        b.read("jump=J\nkick=K\n");
        assert_eq!((b.keys[kick], b.keys[ring]), (Some(Key::Q), Some(Key::K)));
        assert!(b.key_map().is_empty());
        // kicking given another key by the player stays there
        let mut b = Bindings::default();
        b.read("kick=X\n");
        assert_eq!((b.keys[kick], b.keys[ring]), (Some(Key::X), Some(Key::K)));
    }
}

/// The menu page where keys and buttons are chosen.
#[derive(Clone, Copy, Default)]
pub struct KeysPage {
    /// The row (an action, then "reset all" and "back") and the column
    /// (0 keyboard, 1 gamepad) the keyboard is on.
    pub row: usize,
    pub col: usize,
    /// Waiting for the key or button to be given to the row.
    pub waiting: bool,
    /// The first row shown (the list scrolls when the view is small).
    pub top: usize,
    mouse_was_down: bool,
}

const RESET_ROW: usize = ACTIONS.len();
const BACK_ROW: usize = ACTIONS.len() + 1;
const PAGE_ROWS: usize = ACTIONS.len() + 2;
/// Where the columns begin and how wide the page is.
const KEY_X: i32 = 150;
const PAD_X: i32 = 230;
const PAGE_W: i32 = 300;

use crate::game::Game;
use crate::view::Surface;

impl Game {
    /// Where the game's language is in the [German, French, English] lists.
    fn lang(&self) -> usize {
        self.tr([0, 1, 2])
    }

    pub fn open_keys_page(&mut self) {
        if let Some(m) = &mut self.ui.menu {
            m.keys = Some(KeysPage { mouse_was_down: true, ..Default::default() });
        }
    }

    fn keys_page(&mut self) -> Option<&mut KeysPage> {
        self.ui.menu.as_mut()?.keys.as_mut()
    }

    pub fn keys_page_open(&self) -> bool {
        self.ui.menu.as_ref().is_some_and(|m| m.keys.is_some())
    }

    /// Waiting for a key or (`pad`) a gamepad button.
    pub fn keys_waiting(&self, pad: bool) -> bool {
        self.ui.menu.as_ref().and_then(|m| m.keys).is_some_and(|k| k.waiting && (k.col == 1) == pad)
    }

    /// Esc: stops the waiting, or leaves the page. False: the page was not open.
    pub fn keys_back(&mut self) -> bool {
        let Some(m) = &mut self.ui.menu else { return false };
        match &mut m.keys {
            Some(k) if k.waiting => k.waiting = false,
            Some(_) => m.keys = None,
            None => return false,
        }
        true
    }

    /// Arrow keys on the page.
    pub fn keys_move(&mut self, dx: i32, dy: i32) {
        let Some(k) = self.keys_page() else { return };
        if k.waiting {
            return;
        }
        k.row = (k.row as i32 + dy).rem_euclid(PAGE_ROWS as i32) as usize;
        if dx != 0 {
            k.col = 1 - k.col;
        }
    }

    /// Enter on the page.
    pub fn keys_activate(&mut self) {
        let Some(k) = self.keys_page() else { return };
        if k.waiting {
            return;
        }
        match k.row {
            RESET_ROW => {
                self.bindings = Bindings::default();
                self.bindings.save();
            }
            BACK_ROW => {
                self.keys_back();
            }
            r => {
                // the mouse buttons have no key of their own: only a pad button
                if own_key(&ACTIONS[r]).is_none() {
                    k.col = 1;
                }
                k.waiting = true;
            }
        }
    }

    /// While waiting for a key: the key pressed. Delete and backspace
    /// take the key (or the pad button) away.
    pub fn keys_take_key(&mut self, key: Key) {
        let Some(k) = self.keys_page().copied() else { return };
        if !k.waiting || !can_bind(key) {
            return;
        }
        let clear = matches!(key, Key::Delete | Key::Backspace);
        match k.col {
            0 => self.bindings.set_key(k.row, (!clear).then_some(key)),
            _ if clear => self.bindings.set_pad(k.row, None),
            _ => return,
        }
        self.bindings.save();
        if let Some(k) = self.keys_page() {
            k.waiting = false;
        }
    }

    /// While waiting for a pad button: the one pressed.
    pub fn keys_take_pad(&mut self, p: Pad) {
        let Some(k) = self.keys_page().copied() else { return };
        if !k.waiting || k.col != 1 {
            return;
        }
        self.bindings.set_pad(k.row, Some(p));
        self.bindings.save();
        if let Some(k) = self.keys_page() {
            k.waiting = false;
        }
    }

    /// Left edge, first row's top, row height and how many rows fit.
    fn keys_layout(&self, surf_w: i32, surf_h: i32) -> (i32, i32, i32, usize) {
        let fh = self.data.font.as_ref().map_or(10, |f| f.height);
        let line = fh + 2;
        // title and column heads above, the hint below
        let first = 4 + 2 * line + 4;
        let room = surf_h - first - line - 8;
        let fit = ((room / line).max(3) as usize).min(PAGE_ROWS);
        let x0 = ((surf_w - PAGE_W) / 2).max(2);
        (x0, first, line, fit)
    }

    /// The mouse on the page: a click on a row's key or button starts the
    /// choosing; on the bottom rows it does what they say.
    pub fn keys_mouse(&mut self, x: i32, y: i32, down: bool, surf_w: i32, surf_h: i32) {
        let (x0, first, line, fit) = self.keys_layout(surf_w, surf_h);
        let Some(k) = self.keys_page() else { return };
        let pressed = down && !k.mouse_was_down;
        k.mouse_was_down = down;
        if !pressed || k.waiting || y < first {
            return;
        }
        let shown = ((y - first) / line) as usize;
        let row = k.top + shown;
        if shown >= fit || row >= PAGE_ROWS {
            return;
        }
        k.row = row;
        k.col = if x >= x0 + PAD_X - 3 { 1 } else { 0 };
        self.keys_activate();
    }

    pub fn draw_keys_page(&mut self, surf: &mut Surface) {
        let (x0, first, line, fit) = self.keys_layout(surf.width, surf.height);
        let Some(mut k) = self.ui.menu.as_ref().and_then(|m| m.keys) else { return };
        // the row the keyboard is on stays in sight
        if k.row < k.top {
            k.top = k.row;
        } else if k.row >= k.top + fit {
            k.top = k.row + 1 - fit;
        }
        if let Some(p) = self.keys_page() {
            p.top = k.top;
        }
        let l = self.lang();
        let title = ["Tasten umbelegen", "Changer les touches", "Change keys"][l];
        let heads = [["Tastatur", "Clavier", "Keyboard"][l], "Gamepad"];
        let bottom = [["Alles zur\u{81}cksetzen", "Tout r\u{82}tablir", "Reset all"][l], ["Zur\u{81}ck", "Retour", "Back"][l]];
        let hint = match (k.waiting, k.col) {
            (true, 0) => ["Taste dr\u{81}cken  (Esc: abbrechen, Entf: keine)", "Appuyez sur une touche  (Esc: annuler, Suppr: aucune)", "Press a key  (Esc: cancel, Del: none)"][l],
            (true, _) => ["Knopf am Gamepad dr\u{81}cken  (Esc: abbrechen, Entf: keiner)", "Appuyez sur un bouton  (Esc: annuler, Suppr: aucun)", "Press a pad button  (Esc: cancel, Del: none)"][l],
            _ => ["Enter: \u{84}ndern   Links/Rechts: Spalte   Esc: zur\u{81}ck", "Entr\u{82}e: changer   Gauche/Droite: colonne   Esc: retour", "Enter: change   Left/Right: column   Esc: back"][l],
        };
        let bindings = self.bindings.clone();
        let pal = &self.data.pal;
        let Some(font) = &self.data.font else { return };
        font.draw(surf, title, (surf.width - font.width(title)) / 2, 4, pal);
        let head_y = first - line - 2;
        font.draw(surf, heads[0], x0 + KEY_X, head_y, pal);
        font.draw(surf, heads[1], x0 + PAD_X, head_y, pal);
        for (i, row) in (k.top..PAGE_ROWS.min(k.top + fit)).enumerate() {
            let y = first + i as i32 * line;
            if row == k.row {
                crate::menu::tint(surf, x0, y - 1, PAGE_W, line);
                // the cell chosen a little brighter
                if row < ACTIONS.len() {
                    let (cx, cw) = if k.col == 0 { (x0 + KEY_X - 3, PAD_X - KEY_X - 2) } else { (x0 + PAD_X - 3, PAGE_W - PAD_X + 3) };
                    crate::menu::tint(surf, cx, y - 1, cw, line);
                }
            }
            if row >= ACTIONS.len() {
                font.draw(surf, bottom[row - ACTIONS.len()], x0 + 4, y, pal);
                continue;
            }
            let a = &ACTIONS[row];
            font.draw(surf, a.names[l], x0 + 4, y, pal);
            let waiting = k.waiting && row == k.row;
            let key = match (waiting && k.col == 0, own_key(a), bindings.keys[row]) {
                (true, ..) => "...".to_string(),
                (_, None, _) => String::new(),
                (_, _, Some(key)) => key_name(key),
                (_, _, None) => "-".to_string(),
            };
            let pad = match (waiting && k.col == 1, bindings.pads[row]) {
                (true, _) => "...".to_string(),
                (_, Some(p)) => pad_name(p).to_string(),
                (_, None) => "-".to_string(),
            };
            font.draw(surf, &key, x0 + KEY_X, y, pal);
            font.draw(surf, &pad, x0 + PAD_X, y, pal);
        }
        let hint_y = surf.height - line - 2;
        font.draw(surf, hint, ((surf.width - font.width(hint)) / 2).max(0), hint_y, pal);
    }
}
