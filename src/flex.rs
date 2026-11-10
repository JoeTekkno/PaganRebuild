//! FLX archives: a 0x80 byte header (entry count at 0x54) followed by a
//! table of (offset, size) pairs.

use std::fs;
use std::io;
use std::path::Path;

#[derive(Clone)]
pub struct Flex {
    data: Vec<u8>,
    table: Vec<(u32, u32)>,
}

pub fn u16le(d: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([d[o], d[o + 1]])
}

pub fn u32le(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}

impl Flex {
    pub fn open(path: &Path) -> io::Result<Flex> {
        Flex::from_bytes(fs::read(path)?).map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))
    }

    pub fn from_bytes(data: Vec<u8>) -> io::Result<Flex> {
        if data.len() < 0x80 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "not a flex file"));
        }
        let count = u32le(&data, 0x54) as usize;
        if 0x80 + count * 8 > data.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "truncated flex table"));
        }
        let table = (0..count).map(|i| (u32le(&data, 0x80 + i * 8), u32le(&data, 0x84 + i * 8))).collect();
        Ok(Flex { data, table })
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Entry `i`, or `None` if it is empty or out of range.
    pub fn get(&self, i: usize) -> Option<&[u8]> {
        let &(off, size) = self.table.get(i)?;
        let (off, size) = (off as usize, size as usize);
        if size == 0 || off + size > self.data.len() {
            return None;
        }
        Some(&self.data[off..off + size])
    }
}
