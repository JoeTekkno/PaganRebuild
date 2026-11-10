//! Original savegame archives (U8SAVE.000 holds the state of a new game).

use crate::flex::u32le;
use crate::hash::HashMap;
use std::fs;
use std::io;
use std::path::Path;

const MAGIC: &[u8] = b"Ultima 8 SaveGame File.";

/// Reads all files stored in a savegame, keyed by upper-case name.
pub fn read_savegame(path: &Path) -> io::Result<HashMap<String, Vec<u8>>> {
    let d = fs::read(path)?;
    let bad = || io::Error::new(io::ErrorKind::InvalidData, format!("{}: not an Ultima 8 savegame", path.display()));
    if !d.starts_with(MAGIC) || d.len() < 0x1A {
        return Err(bad());
    }
    let count = u16::from_le_bytes([d[0x18], d[0x19]]) as usize;
    let mut files = HashMap::default();
    let mut p = 0x1A;
    for _ in 0..count {
        if p + 4 > d.len() {
            return Err(bad());
        }
        let namelen = u32le(&d, p) as usize;
        p += 4;
        let name = d.get(p..p + namelen).ok_or_else(bad)?;
        let name = String::from_utf8_lossy(name).trim_end_matches('\0').to_uppercase();
        p += namelen;
        if p + 4 > d.len() {
            return Err(bad());
        }
        let size = u32le(&d, p) as usize;
        p += 4;
        let data = d.get(p..p + size).ok_or_else(bad)?;
        files.insert(name, data.to_vec());
        p += size;
    }
    Ok(files)
}
