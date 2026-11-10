//! The spell book (option "spell_notes", key N): the spells the Avatar has
//! learned, with their words of power and reagents, shown in the game's
//! own book. A spell counts as learned once a text that teaches it has
//! been shown: a book, a scroll, or what a teacher says. Texts are
//! recognised by their words of power, which are the same in every
//! language of the game. Only the names of the learned spells are kept in
//! the savegame.

use crate::game::Game;
use crate::gumps::Look;

/// A spell that can be noted.
pub struct Spell {
    /// Name in the savegame (English, never changes).
    pub key: &'static str,
    /// Names as the game calls them.
    pub de: &'static str,
    pub en: &'static str,
    /// The words of power; empty for a spell that has none.
    pub words: &'static str,
    /// Other parts of a text that teach the spell (for a spell without
    /// words of power), in any language.
    pub marks: &'static [&'static str],
    /// Story flags (as in quests.rs) that show the spell must be known:
    /// for savegames from before the spell book. Empty: none.
    pub known_if: &'static str,
    /// Reagents and how the spell is used.
    pub reagents_de: &'static str,
    pub reagents_en: &'static str,
    pub use_de: &'static str,
    pub use_en: &'static str,
}

/// A school of magic with its spells in the game's order.
pub struct School {
    pub de: &'static str,
    pub en: &'static str,
    pub spells: &'static [Spell],
}

pub const SCHOOLS: &[School] = &[School {
    de: "Erdmagie (Nekromantie)",
    en: "Earth magic (Necromancy)",
    spells: &[
        Spell {
            key: "Death Speak",
            known_if: "deathSpeakCast | metNec1 | metNec2 | metNec3 | metNec4 | metNec5 | metNec6",
            de: "Seance",
            en: "Death Speak",
            words: "Kal Wis Corp",
            marks: &[],
            reagents_de: "Blut, Knochen",
            reagents_en: "Blood, Bone",
            use_de: "Auf einen Toten gerichtet: er spricht mit Euch.",
            use_en: "Cast upon a corpse: the dead will speak to you.",
        },
        Spell {
            key: "Mask of Death",
            known_if: "metNec1",
            de: "Todesmaske",
            en: "Mask of Death",
            words: "Quas Corp",
            marks: &[],
            reagents_de: "Holz, Henkersmaske",
            reagents_en: "Wood, Executioner's Hood",
            use_de: "Auf Euch selbst: Ihr erscheint allen als tot.",
            use_en: "Cast upon yourself: all who see you think you dead.",
        },
        Spell {
            key: "Rock Flesh",
            known_if: "metNec2",
            de: "Felsenhaut",
            en: "Rock Flesh",
            words: "Rel Sanct Ylem",
            marks: &[],
            reagents_de: "Holz, Erde",
            reagents_en: "Wood, Dirt",
            use_de: "Auf Euch selbst: Eure Haut wird hart wie Stein.",
            use_en: "Cast upon yourself: your skin becomes as hard as stone.",
        },
        Spell {
            key: "Summon Undead",
            known_if: "metNec3",
            de: "Beschw\u{f6}rung",
            en: "Summon Undead",
            words: "Kal Corp Xen",
            marks: &[],
            reagents_de: "Blut, Knochen, Holz",
            reagents_en: "Blood, Bone, Wood",
            use_de: "Auf den Boden gerichtet: Hilfe f\u{fc}r den Kampf erscheint.",
            use_en: "Focused upon the ground: help for the battle arrives.",
        },
        Spell {
            key: "Open Ground",
            known_if: "metNec1 | metNec2 | metNec3 | metNec4 | metNec5 | metNec6",
            de: "Offene Erde",
            en: "Open Ground",
            words: "Des Por Ylem",
            marks: &[],
            reagents_de: "Blut, Schwarzmoor",
            reagents_en: "Blood, Blackmoor",
            use_de: "\u{d6}ffnet die Erde, etwa eine Felswand, und l\u{e4}sst Euch eintreten.",
            use_en: "Opens the earth, a rock face for one, and lets you enter.",
        },
        Spell {
            key: "Create Golem",
            known_if: "metNec6",
            de: "Golem",
            en: "Create Golem",
            words: "In Ort Ylem Xen",
            marks: &[],
            reagents_de: "Blut, Knochen, Holz, Erde, Schwarzmoor",
            reagents_en: "Blood, Bone, Wood, Dirt, Blackmoor",
            use_de: "Auf eine gro\u{df}e Erdfl\u{e4}che gerichtet: ein Golem steigt auf.",
            use_en: "Focused upon a broad space of mud: a golem rises.",
        },
        Spell {
            key: "Withstand Death",
            known_if: "metNec5",
            de: "Wiedergeburt",
            en: "Withstand Death",
            words: "Vas An Corp",
            marks: &[],
            reagents_de: "Holz, Erde, Schwarzmoor",
            reagents_en: "Wood, Dirt, Blackmoor",
            use_de: "Auf Euch selbst: Ihr widersteht dem Tod.",
            use_en: "Cast upon yourself: you resist death.",
        },
        Spell {
            key: "Grant Peace",
            known_if: "metNec4",
            de: "Ewiger Frieden",
            en: "Grant Peace",
            words: "In Vas Corp",
            marks: &[],
            reagents_de: "Henkersmaske, Schwarzmoor",
            reagents_en: "Executioner's Hood, Blackmoor",
            use_de: "Auf den Boden gerichtet: schickt Feinde in den ewigen Frieden.",
            use_en: "Focused upon the ground: grants your foes eternal peace.",
        },
        Spell {
            key: "Call Quake",
            known_if: "",
            de: "Erdbeben",
            en: "Call Quake",
            words: "",
            // the end of Lothian's last words (English, French; German)
            marks: &["~Lothian, N", "Nekromantin Lothian"],
            reagents_de: "Knochen, Holz, Erde, Schwarzmoor",
            reagents_en: "Bone, Wood, Dirt, Blackmoor",
            use_de: "Die Reagenzien in einen Beutel geben und die Magie auf den Beutel richten: die Erde bebt.",
            use_en: "Put the reagents in a bag and focus the magic upon it: the earth shakes.",
        },
    ],
}];

/// The learned spells (by key), in the order they were learned.
#[derive(Default)]
pub struct SpellNotes {
    pub learned: Vec<String>,
}

/// Whether `words` stand in `text` as whole words.
fn has_words(text: &str, words: &str) -> bool {
    let letter = |c: Option<char>| c.is_some_and(|c| c.is_ascii_alphabetic());
    text.match_indices(words).any(|(i, _)| !letter(text[..i].chars().next_back()) && !letter(text[i + words.len()..].chars().next()))
}

/// Does the text teach the spell? The words alone, as said when the spell
/// is cast ("Kal Wis Corp!"), teach nothing.
fn teaches(text: &str, s: &Spell) -> bool {
    if !s.words.is_empty() {
        let bare = text.trim().trim_end_matches('!').trim();
        return bare != s.words && has_words(text, s.words);
    }
    s.marks.iter().any(|m| text.contains(m))
}

fn all_spells() -> impl Iterator<Item = (&'static School, &'static Spell)> {
    SCHOOLS.iter().flat_map(|school| school.spells.iter().map(move |s| (school, s)))
}

impl Game {
    /// A text was shown to the player: the spells it teaches are noted
    /// (and announced, with the option on).
    pub fn note_spells(&mut self, text: &str) {
        let new: Vec<&Spell> = all_spells().map(|(_, s)| s).filter(|s| teaches(text, s) && !self.spell_notes.learned.iter().any(|k| k == s.key)).collect();
        if new.is_empty() {
            return;
        }
        self.spell_notes.learned.extend(new.iter().map(|s| s.key.to_string()));
        if self.options.spell_notes {
            let de = self.language == 'G';
            let names: Vec<&str> = new.iter().map(|s| if de { s.de } else { s.en }).collect();
            let text = format!("{}: {}", if de { "Zauberbuch" } else { "Spell book" }, names.join(", "));
            self.notify(&crate::journal::dos(&text));
        }
    }

    /// After loading: adds the spells the story shows to be known (a
    /// savegame from before the spell book has none noted), quietly.
    pub fn spells_from_story(&mut self) {
        let known: Vec<&Spell> = all_spells().map(|(_, s)| s).filter(|s| self.holds(s.known_if) && !self.spell_notes.learned.iter().any(|k| k == s.key)).collect();
        self.spell_notes.learned.extend(known.iter().map(|s| s.key.to_string()));
    }

    /// The text of the spell book: per school the learned spells in the
    /// game's order.
    pub fn spell_notes_text(&self) -> String {
        let de = self.language == 'G';
        let mut text = String::new();
        for school in SCHOOLS {
            let known: Vec<&Spell> = school.spells.iter().filter(|s| self.spell_notes.learned.iter().any(|k| k == s.key)).collect();
            if known.is_empty() {
                continue;
            }
            let of = if de { "von" } else { "of" };
            text += &format!("{}*{} {of} {}**", if de { school.de } else { school.en }.to_uppercase(), known.len(), school.spells.len());
            for s in known {
                text += &format!("{}*", if de { s.de } else { s.en });
                if !s.words.is_empty() {
                    text += &format!("{}: {}*", if de { "Worte" } else { "Words" }, s.words);
                }
                text += &format!("{}: {}*", if de { "Reagenzien" } else { "Reagents" }, if de { s.reagents_de } else { s.reagents_en });
                text += if de { s.use_de } else { s.use_en };
                text += "**";
            }
        }
        if text.is_empty() {
            text = if de { "Noch keine Zauber gelernt." } else { "No spells learned yet." }.to_string();
        }
        crate::journal::dos(text.trim_end_matches('*'))
    }

    /// Opens the spell book in the game's book.
    pub fn open_spell_notes(&mut self) {
        let text = self.spell_notes_text();
        self.read_text(0, &text, Look::Book);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spell(key: &str) -> &'static Spell {
        all_spells().map(|(_, s)| s).find(|s| s.key == key).unwrap()
    }

    #[test]
    fn keys_are_unique() {
        let mut keys: Vec<&str> = all_spells().map(|(_, s)| s.key).collect();
        let n = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), n);
    }

    #[test]
    fn every_flag_is_known() {
        let Some(usecode) = crate::usecode::for_tests() else { return };
        for (_, s) in all_spells().filter(|(_, s)| !s.known_if.is_empty()) {
            for name in s.known_if.split(['|', '&']).map(|t| t.trim()) {
                assert!(usecode.flag_at(name).is_some(), "unknown flag {name}");
            }
        }
    }

    #[test]
    fn words_teach_only_in_a_text() {
        let s = spell("Death Speak");
        assert!(teaches("The reagents needed for Death Speak are but Blood and Bone. Use these reagents together with the words Kal Wis Corp and", s));
        assert!(!teaches("Kal Wis Corp!", s));
        assert!(!teaches("Kal Wis Corpse", s));
    }

    #[test]
    fn longer_words_do_not_teach_shorter_ones() {
        // In Vas Corp (Grant Peace) is no part of the words of other spells
        for (_, s) in all_spells() {
            for (_, other) in all_spells() {
                if s.key != other.key && !s.words.is_empty() && !other.words.is_empty() {
                    assert!(!has_words(other.words, s.words), "{} in {}", s.words, other.words);
                }
            }
        }
        assert!(!teaches("say In Vas Ort Corp and", spell("Grant Peace")));
    }
}
