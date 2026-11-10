//! Sleeping until a time of day (optional, switch "sleep until a time of
//! day"). The beds and the bedroll ask in the original "Rest for how many
//! periods of time?" above themselves and offer 0 to 5 (a period is four
//! game hours). With the switch on the Avatar asks instead, naming the
//! time of day as Pagan does (Bloodwatch, FirstEbb, DayTide, ThreeMoons,
//! LastEbb, EvenTide; the hourglass script uses the same names), and the
//! choices are the beginnings of the next five of them. The bed's script
//! does the rest as before (fading, healing); only the hour it sets is the
//! one chosen.

use crate::game::{Game, AVATAR};

/// The bedroll and the two kinds of bed (their scripts: BEDROLL, BEDNS, BEDEW).
const BEDS: [u32; 3] = [534, 175, 196];
/// A period of the day: four game hours.
const PERIOD_HOURS: u32 = 4;

/// The names of the six periods, as the choices show them.
fn names(lang: char) -> [&'static str; 6] {
    match lang {
        'G' => ["Blutwache", "Erste Flut", "Tageswende", "Drei Monde", "Letzte Flut", "Nachtwende"],
        'F' => ["Auburne", "Midurne", "Maturne", "Amidurne", "Minurne", "Vespurne"],
        _ => ["Bloodwatch", "FirstEbb", "DayTide", "ThreeMoons", "LastEbb", "EvenTide"],
    }
}

/// "It is DayTide. Rest until ..."
fn question(lang: char, period: usize) -> String {
    match lang {
        'G' => {
            let of = ["der Blutwache", "der Ersten Flut", "der Tageswende", "der Drei Monde", "der Letzten Flut", "der Nachtwende"];
            format!("Es ist die Zeit {}. Schlafen bis ...", of[period])
        }
        'F' => format!("Il est maintenant {}. Dormir jusqu'\u{85} ...", names(lang)[period]),
        _ => format!("It is {}. Rest until ...", names(lang)[period]),
    }
}

/// One choice: a period and the hour it begins.
fn label(lang: char, period: usize) -> String {
    let hour = period as u32 * PERIOD_HOURS;
    match lang {
        'G' => format!("{} ({hour} Uhr)", names(lang)[period]),
        'F' => format!("{} ({hour} h)", names(lang)[period]),
        _ => format!("{} ({hour} o'clock)", names(lang)[period]),
    }
}

fn stay_awake(lang: char) -> &'static str {
    match lang {
        'G' => "Nicht schlafen",
        'F' => "Ne pas dormir",
        _ => "Do not rest",
    }
}

/// The bed's question in the original, in every language.
fn is_rest_question(text: &str) -> bool {
    let t = text.trim();
    t.starts_with("Rest for how many periods") || t.starts_with("Schlafen? Wie viele Zeiten") || t.starts_with("Du repos? Pendant combien")
}

impl Game {
    fn sleep_bed(&self, item: u16) -> bool {
        self.options.sleep_until && self.map().get(item).is_some_and(|it| BEDS.contains(&it.shape))
    }

    /// Whether `item` is a bed and the script of a bed is still running.
    /// Each bed script notes whether the Avatar was held when it began and
    /// sets that again at its end; two at once leave him held forever.
    pub(crate) fn bed_busy(&self, item: u16) -> bool {
        let is_bed = |id: u16| id != 0 && self.map().get(id).is_some_and(|it| BEDS.contains(&it.shape));
        is_bed(item) && self.scripts.procs.iter().any(|p| !p.terminated && is_bed(p.item))
    }

    /// The period of the day now (0 Bloodwatch .. 5 EvenTide).
    fn period_now(&self) -> usize {
        (self.game_hours() % 24 / PERIOD_HOURS) as usize
    }

    /// A bark of a bed: its question about how long to rest becomes the
    /// Avatar's, with the time of day. None: leave it as it is.
    pub fn sleep_question(&self, item: u16, text: &str) -> Option<(u16, String)> {
        (self.sleep_bed(item) && is_rest_question(text)).then(|| (AVATAR, question(self.language, self.period_now())))
    }

    /// A bed offers 0 to 5 periods: the beginnings of the next five times
    /// of day instead, and not resting. None: leave the choices as they are.
    pub fn sleep_choices(&mut self, item: u16, choices: &[String]) -> Option<Vec<String>> {
        let original: Vec<String> = (0..=5).map(|n| n.to_string()).collect();
        if !self.sleep_bed(item) || choices.iter().map(|c| c.trim()).ne(original.iter().map(String::as_str)) {
            return None;
        }
        let hours = self.game_hours();
        let now = self.period_now();
        let day_start = hours - hours % 24;
        let mut targets = Vec::new();
        let mut labels = Vec::new();
        for k in 1..=5 {
            let p = now + k;
            targets.push(Some(day_start + p as u32 * PERIOD_HOURS));
            labels.push(label(self.language, p % 6));
        }
        targets.push(None);
        labels.push(stay_awake(self.language).to_string());
        self.sleep_menu = Some(targets);
        Some(labels)
    }

    /// The player picked choice `n` of the sleep question: what the bed's
    /// script is told ("0" stay awake, "1" rest; the hour it then sets is
    /// replaced by the chosen one). None: not a sleep question.
    pub fn sleep_answer(&mut self, n: usize) -> Option<String> {
        let targets = self.sleep_menu.take()?;
        let target = targets.get(n).copied().flatten();
        self.sleep_until = target;
        Some(if target.is_some() { "1" } else { "0" }.to_string())
    }

    /// The hour a bed's script sets after resting: the chosen one, if any.
    pub fn sleep_hour(&mut self, hour: u32) -> u32 {
        self.sleep_until.take().unwrap_or(hour)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn questions_are_known() {
        assert!(super::is_rest_question("Rest for how many periods of time? "));
        assert!(super::is_rest_question(" Schlafen? Wie viele Zeiten?"));
        assert!(super::is_rest_question(" Du repos? Pendant combien de laps de temps? "));
        assert!(!super::is_rest_question("bedroll "));
    }
}
