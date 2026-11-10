//! Hash tables that keep the same order from one run to the next.
//!
//! The tables of the standard library are seeded at random on every
//! start, so going through one (all fighters, all walking actors) went in
//! a different order each time, and the game with it: the same test
//! commands could end differently. These hash the same way every time.

use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;

pub type Fixed = BuildHasherDefault<DefaultHasher>;
pub type HashMap<K, V> = std::collections::HashMap<K, V, Fixed>;
pub type HashSet<K> = std::collections::HashSet<K, Fixed>;
