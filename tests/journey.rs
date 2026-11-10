//! The playthrough: the game from its beginning to its end, chapter by
//! chapter (tests/journey/*.case, in the order of their names). Each
//! chapter goes on from the savegame the one before it left, and it only
//! does what a player can (see src/testcmd/journey.rs).
//!
//! It takes long and is not run with the other tests, only when asked for:
//!   cargo test --release --test journey -- --ignored --nocapture
//! From a chapter on (the savegames of the last run are kept in
//! target/journey):  PAGAN_CHAPTER=<name> ...;  one chapter only:
//! PAGAN_CHAPTER=<name> PAGAN_ONLY=1 ...
//!
//! A chapter file has the lines of a regression case (`# `, `do:`,
//! `expect:`, `reject:`); `args:` only in the first chapter. The run of a
//! chapter ends by saving to slot 100 + its number.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The game folder: the first folder above this one with ENGLISH/STATIC.
fn game_dir() -> Option<PathBuf> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join("ENGLISH").join("STATIC").join("U8SHAPES.FLX").is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

struct Chapter {
    name: String,
    args: Vec<String>,
    commands: String,
    expect: Vec<String>,
    reject: Vec<String>,
}

fn parse(path: &Path) -> Chapter {
    let text = std::fs::read_to_string(path).unwrap();
    let mut c = Chapter { name: path.file_stem().unwrap().to_string_lossy().into_owned(), args: Vec::new(), commands: String::new(), expect: Vec::new(), reject: Vec::new() };
    for line in text.lines() {
        let line = line.trim_end();
        if let Some(a) = line.strip_prefix("args:") {
            c.args.extend(a.split_whitespace().map(String::from));
        } else if let Some(d) = line.strip_prefix("do:") {
            if !c.commands.is_empty() {
                c.commands.push(';');
            }
            c.commands.push_str(d.trim());
        } else if let Some(e) = line.strip_prefix("expect:") {
            c.expect.push(e.trim().to_string());
        } else if let Some(r) = line.strip_prefix("reject:") {
            c.reject.push(r.trim().to_string());
        }
    }
    c
}

#[test]
#[ignore = "the whole game, long: run with --ignored"]
fn playthrough() {
    let game = game_dir().expect("the original game files were not found");
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("journey");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "case")).collect();
    files.sort();
    let from = std::env::var("PAGAN_CHAPTER").ok();
    let only = std::env::var_os("PAGAN_ONLY").is_some();
    let saves = Path::new(env!("CARGO_MANIFEST_DIR")).join("target").join("journey");
    std::fs::create_dir_all(&saves).unwrap();
    let mut started = from.is_none();
    for (k, file) in files.iter().enumerate() {
        let ch = parse(file);
        if !started {
            started = from.as_deref() == Some(ch.name.as_str());
            if !started {
                continue;
            }
        }
        let slot = 100 + k;
        let mut commands = if k == 0 { String::new() } else { format!("loadslot:{};", slot - 1) };
        commands += &ch.commands;
        commands += &format!(";saveslot:{slot}");
        let t = std::time::Instant::now();
        let out = Command::new(env!("CARGO_BIN_EXE_pagan"))
            .current_dir(&game)
            .env("PAGAN_SAVE_DIR", &saves)
            .env("APPDATA", &saves)
            .env("XDG_DATA_HOME", &saves)
            .arg("--no-sound")
            .arg("--no-menu")
            .args(&ch.args)
            .arg("--screenshot")
            .arg(saves.join(format!("{}.png", ch.name)))
            .arg("--do")
            .arg(&commands)
            .output()
            .expect("the game did not start");
        let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        let log = saves.join(format!("{}.log", ch.name));
        let _ = std::fs::write(&log, &text);
        let mut problems: Vec<String> = Vec::new();
        if let Some(e) = text.lines().find(|l| l.starts_with("error:")) {
            problems.push(e.to_string());
        } else if !out.status.success() {
            problems.push(format!("ended with {}", out.status));
        }
        problems.extend(ch.expect.iter().filter(|e| !text.contains(e.as_str())).map(|e| format!("missing \"{e}\"")));
        problems.extend(ch.reject.iter().filter(|r| text.contains(r.as_str())).map(|r| format!("unwanted \"{r}\"")));
        assert!(problems.is_empty(), "chapter {} failed after {:.0} s: {}\n  output in {}", ch.name, t.elapsed().as_secs_f32(), problems.join(", "), log.display());
        eprintln!("chapter {}: ok ({:.0} s, saved in slot {slot})", ch.name, t.elapsed().as_secs_f32());
        if only {
            break;
        }
    }
    assert!(started, "no chapter called {}", from.unwrap_or_default());
}
