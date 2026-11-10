//! The places of the list on the large map (worldmap.rs): well-known
//! spots of the world, each on its map, and the people of the story,
//! who are found wherever they are at the time (their daily round).
//! Names are German, French and English (German with its umlauts, French
//! in the game's code page).

pub struct Place {
    pub map: usize,
    pub x: i32,
    pub y: i32,
    /// German, French, English.
    pub names: [&'static str; 3],
}

const fn place(map: usize, x: i32, y: i32, names: [&'static str; 3]) -> Place {
    Place { map, x, y, names }
}

pub const PLACES: &[Place] = &[
    place(3, 14527, 5887, ["Anlegestelle", "D\u{82}barcad\u{8a}re", "Landing"]),
    place(3, 11732, 5844, ["Hinrichtungssteg", "Ponton des ex\u{82}cutions", "Execution pier"]),
    place(41, 14896, 2880, ["Bibliothek", "Biblioth\u{8a}que", "Library"]),
    place(39, 15600, 12216, ["Taverne", "Taverne", "Tavern"]),
    place(40, 13230, 7988, ["Palast", "Palais", "Palace"]),
    place(8, 3071, 1791, ["Mythrans Haus", "Maison de Mythran", "Mythran's house"]),
    place(12, 10047, 15615, ["Obsidian-Festung", "Forteresse d'obsidienne", "Obsidian Fortress"]),
    place(6, 7839, 24831, ["Friedhof", "Cimeti\u{8a}re", "Cemetery"]),
    place(5, 5040, 22784, ["Nordtor", "Porte nord", "North gate"]),
    // the dungeons: where one arrives from outside, and a few marked spots
    place(50, 2948, 10612, ["Obere Katakomben", "Catacombes sup\u{82}rieures", "Upper catacombs"]),
    place(51, 28732, 4804, ["Lavahalle", "Salle de lave", "Lava hall"]),
    place(54, 15694, 14786, ["Grabgewölbe, Eingang", "Tombeau, entr\u{82}e", "Tomb, entrance"]),
    place(54, 10175, 2719, ["Grabkammer", "Chambre fun\u{82}raire", "Burial chamber"]),
    place(62, 1695, 14655, ["Grab des sechsten Nekromanten", "Tombe du sixi\u{8a}me n\u{82}cromancien", "Tomb of the sixth necromancer"]),
    place(49, 17631, 24735, ["Höhlen", "Grottes", "Caves"]),
    place(57, 30495, 28127, ["Höhlen", "Grottes", "Caves"]),
    place(21, 4342, 2138, ["Totenstadt", "N\u{82}cropole", "Necropolis"]),
    place(31, 27850, 23350, ["Untere Katakomben", "Catacombes inf\u{82}rieures", "Lower catacombs"]),
    place(31, 1768, 1776, ["Lithos", "Lithos", "Lithos"]),
    place(37, 26776, 26912, ["Plateau, Höhleneingang", "Plateau, entr\u{82}e de grotte", "Plateau, cave entrance"]),
    place(37, 10655, 17247, ["Schwebende Felsen, Stratos", "Rochers flottants, Stratos", "Floating rocks, Stratos"]),
    place(63, 16127, 18879, ["Mauer im Nichts, vom Hof", "Mur dans le n\u{82}ant, de la ferme", "Wall in the void, from the farm"]),
    place(14, 15935, 2303, ["Feuerhöhle unter der Festung", "Grotte de feu sous la forteresse", "Fire cave below the fortress"]),
    place(12, 4095, 18687, ["Säulenhöhle der Festung", "Grotte aux piliers de la forteresse", "Pillar cave of the fortress"]),
    place(16, 19999, 14367, ["Hydros", "Hydros", "Hydros"]),
    place(28, 14980, 8108, ["Höhle", "Grotte", "Cave"]),
    place(43, 16000, 16000, ["Pentagramm im Äther", "Pentagramme dans l'\u{82}ther", "Pentagram in the ether"]),
    place(44, 27295, 30367, ["Lavafeld", "Champ de lave", "Lava field"]),
    place(45, 21823, 14431, ["Dunkles Wasser", "Eaux sombres", "Dark water"]),
    place(46, 13951, 7455, ["Felsplateau", "Plateau rocheux", "Rock plateau"]),
    place(47, 25279, 13503, ["Schlucht", "Gorge", "Gorge"]),
];

/// The names of the maps one can get to (German, French, English): shown
/// on the large map. Taken from the places on them and the music that
/// plays there.
pub const MAP_NAMES: &[(usize, [&str; 3])] = &[
    (3, ["Hafen", "Le port", "The docks"]),
    (4, ["Fischerriff", "R\u{82}cif du p\u{88}cheur", "Fisherman's Reef"]),
    (5, ["Oststraße", "Route de l'est", "East road"]),
    (6, ["Friedhof", "Cimeti\u{8a}re", "Cemetery"]),
    (7, ["Hof und Felder", "Ferme et champs", "Farm and fields"]),
    (8, ["Mythrans Haus", "Maison de Mythran", "Mythran's house"]),
    (12, ["Obsidian-Festung", "Forteresse d'obsidienne", "Obsidian Fortress"]),
    (14, ["Feuerhöhle", "Grotte de feu", "Fire cave"]),
    (15, ["Gewölbe", "Caveau", "Vault"]),
    (16, ["Carthax-See", "Lac Carthax", "Carthax Lake"]),
    (21, ["Totenstadt", "N\u{82}cropole", "Necropolis"]),
    (24, ["Lavahöhlen", "Grottes de lave", "Lava caves"]),
    (25, ["Lavahöhlen", "Grottes de lave", "Lava caves"]),
    (26, ["Höhlen am Carthax-See", "Grottes du lac Carthax", "Carthax Lake caves"]),
    (28, ["Höhlen am Carthax-See", "Grottes du lac Carthax", "Carthax Lake caves"]),
    (31, ["Untere Katakomben", "Catacombes inf\u{82}rieures", "Lower catacombs"]),
    (37, ["Argentrock-Insel", "Ile d'Argentrock", "Argentrock Isle"]),
    (39, ["Tenebrae, Westen", "Tenebrae, ouest", "Tenebrae, west"]),
    (40, ["Tenebrae, Mitte", "Tenebrae, centre", "Tenebrae, centre"]),
    (41, ["Tenebrae, Osten", "Tenebrae, est", "Tenebrae, east"]),
    (43, ["Ätherebene", "Plan \u{82}th\u{82}r\u{82}", "Ethereal plane"]),
    (44, ["Lavafeld", "Champ de lave", "Lava field"]),
    (45, ["Dunkles Wasser", "Eaux sombres", "Dark water"]),
    (46, ["Felsplateau", "Plateau rocheux", "Rock plateau"]),
    (47, ["Schlucht", "Gorge", "Gorge"]),
    (49, ["Höhlen", "Grottes", "Caves"]),
    (50, ["Obere Katakomben", "Catacombes sup\u{82}rieures", "Upper catacombs"]),
    (51, ["Lavahalle", "Salle de lave", "Lava hall"]),
    (54, ["Grabgewölbe", "Tombeau", "Tomb"]),
    (57, ["Höhlen", "Grottes", "Caves"]),
    (62, ["Grab des sechsten Nekromanten", "Tombe du sixi\u{8a}me n\u{82}cromancien", "Tomb of the sixth necromancer"]),
    (63, ["Mauer im Nichts", "Mur dans le n\u{82}ant", "Wall in the void"]),
];

/// The people of the list: their number and name.
pub const PEOPLE: &[(u16, &str)] = &[
    (2, "Devon"),
    (3, "Salkind"),
    (5, "Darion"),
    (7, "Mordea"),
    (8, "Tarna"),
    (9, "Korick"),
    (10, "Orlok"),
    (12, "Bentic"),
    (13, "Aramina"),
    (14, "Gwillim"),
    (15, "Corinth"),
    (16, "Kilandra"),
    (18, "Rhian"),
    (19, "Shaana"),
    (21, "Vividos"),
    (22, "Cyrrus"),
    (23, "Stellos"),
    (24, "Jenna"),
    (25, "Xavier"),
    (26, "Torwin"),
    (27, "Malchir"),
    (28, "Vardion"),
    (29, "Bane"),
    (31, "Gorgrond"),
    (32, "Beren"),
    (33, "Mythran"),
    (52, "Cardas"),
    (53, "Mentar"),
    (54, "Daemos"),
    (55, "Tallon"),
    (56, "Kothius"),
    (60, "Emrichol"),
];
