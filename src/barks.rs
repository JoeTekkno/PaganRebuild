//! Where a thing's question is shown: the recall teleporter asks for the
//! destination above itself in the original, while the answers come above
//! the Avatar. Here the question is shown above the Avatar as well, as the
//! beds do when asking until which time to sleep.

use crate::game::{Game, AVATAR};

/// Shape of the recall teleporter.
const RECALL: u32 = 833;

/// Its question for the destination, in every language.
fn is_destination_question(text: &str) -> bool {
    let t = text.trim();
    t.starts_with("What is your destination") || t.starts_with("Was ist Euer Reiseziel") || t.starts_with("Quelle est ta destination")
}

impl Game {
    /// A bark of the recall teleporter: its question goes above the
    /// Avatar. None: leave it where it is.
    pub fn question_speaker(&self, item: u16, text: &str) -> Option<u16> {
        let recall = self.map().get(item).is_some_and(|it| it.shape == RECALL);
        (recall && is_destination_question(text)).then_some(AVATAR)
    }
}
