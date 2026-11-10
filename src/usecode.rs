//! The usecode archive (EUSECODE.FLX ...): bytecode of the game scripts,
//! one class per shape / NPC / egg type.

use crate::flex::{u32le, Flex};
use crate::hash::HashMap;
use std::io;
use std::path::Path;
use std::rc::Rc;

/// Code offsets inside a class are relative to this file offset.
const BASE: usize = 0x0C;

pub struct Usecode {
    classes: Vec<Option<Rc<Vec<u8>>>>,
    /// Global flags, addressed by bit position.
    globals: Vec<u8>,
    /// Bit position and size of the named global flags.
    flags: HashMap<String, (usize, usize)>,
}

impl Usecode {
    pub fn load(path: &Path) -> io::Result<Usecode> {
        let flex = Flex::open(path)?;
        // entry 0: names of the globals, entry 1: class names
        let classes = (2..flex.len()).map(|i| flex.get(i).filter(|d| d.len() > BASE + 128).map(|d| Rc::new(d.to_vec()))).collect();
        let flags = read_flags(flex.get(0).unwrap_or_default());
        Ok(Usecode { classes, globals: vec![0; 0x2000], flags })
    }

    /// Bit position and size of a named global flag.
    pub fn flag_at(&self, name: &str) -> Option<(usize, usize)> {
        self.flags.get(name).copied()
    }

    /// The named global flags that differ between two copies of the globals
    /// (for tests): "flag name: old -> new", in the order of the flags.
    pub fn flag_changes(&self, a: &[u8], b: &[u8]) -> Vec<String> {
        let get = |g: &[u8], pos: usize, bits: usize| -> u32 {
            (0..bits.min(32)).filter(|i| g.get((pos + i) / 8).is_some_and(|v| v & (1 << ((pos + i) % 8)) != 0)).map(|i| 1 << i).sum()
        };
        let mut out: Vec<(usize, String)> = self
            .flags
            .iter()
            .filter_map(|(name, &(pos, bits))| {
                let (x, y) = (get(a, pos, bits), get(b, pos, bits));
                (x != y).then(|| (pos, format!("flag {name}: {x} -> {y}")))
            })
            .collect();
        out.sort();
        out.into_iter().map(|(_, s)| s).collect()
    }

    /// The bytecode of a class; offsets used by the scripts index into it.
    pub fn code(&self, class: u16) -> Option<Rc<Vec<u8>>> {
        self.classes.get(class as usize)?.clone()
    }

    /// Entry point of an event handler (0 = look, 1 = use ...).
    pub fn event(&self, class: u32, event: u32) -> Option<u16> {
        let d = self.classes.get(class as usize)?.as_ref()?;
        if event >= 32 {
            return None;
        }
        let off = u32le(d, BASE + event as usize * 4);
        (off != 0).then_some(off as u16)
    }

    /// The raw global flags, for savegames.
    pub fn globals(&self) -> &[u8] {
        &self.globals
    }

    pub fn set_globals(&mut self, data: &[u8]) {
        let n = data.len().min(self.globals.len());
        self.globals[..n].copy_from_slice(&data[..n]);
    }

    pub fn global(&self, pos: usize, bits: usize) -> u32 {
        let mut v = 0;
        for i in 0..bits.min(32) {
            let p = pos + i;
            if self.globals.get(p / 8).is_some_and(|b| b & (1 << (p % 8)) != 0) {
                v |= 1 << i;
            }
        }
        v
    }

    pub fn set_global(&mut self, pos: usize, bits: usize, val: u32) {
        for i in 0..bits.min(32) {
            let p = pos + i;
            if let Some(b) = self.globals.get_mut(p / 8) {
                if val & (1 << i) != 0 {
                    *b |= 1 << (p % 8);
                } else {
                    *b &= !(1 << (p % 8));
                }
            }
        }
    }
}

/// The table of named global flags: per flag its name (ending in a zero
/// byte), then bit position and bit size (two bytes each).
fn read_flags(data: &[u8]) -> HashMap<String, (usize, usize)> {
    let mut flags = HashMap::default();
    let mut rest = data;
    while let Some(end) = rest.iter().position(|&b| b == 0) {
        let Some(nums) = rest.get(end + 1..end + 5) else { break };
        let pos = u16::from_le_bytes([nums[0], nums[1]]) as usize;
        let bits = u16::from_le_bytes([nums[2], nums[3]]) as usize;
        if end > 0 {
            flags.insert(String::from_utf8_lossy(&rest[..end]).into_owned(), (pos, bits));
        }
        rest = &rest[end + 5..];
    }
    flags
}

/// The English usecode of the game this checkout lies in (for unit tests).
#[cfg(test)]
pub fn for_tests() -> Option<Usecode> {
    let game = Path::new(env!("CARGO_MANIFEST_DIR")).parent()?;
    Usecode::load(&game.join("ENGLISH").join("USECODE").join("EUSECODE.FLX")).ok()
}

/// Reads code relative to the class base.
pub fn code_byte(code: &[u8], ip: usize) -> Option<u8> {
    code.get(BASE + ip).copied()
}
