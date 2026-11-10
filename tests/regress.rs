//! Regression tests: the game is started without a window for every case
//! in tests/regress/*.case and its output is checked.
//!
//! A case file has lines
//!   `# ...`         a comment (the first one names the case)
//!   `args: ...`     command line options before --do (map, position ...)
//!   `do: ...`       test commands (see src/testcmd); several lines are joined with ";"
//!   `expect: ...`   this text must turn up in the output
//!   `reject: ...`   this text must not
//!   `needs: path`   the case is only run if this file or folder exists
//!                   (relative to the game folder, e.g. for the album music)
//!
//! The original game files are needed: they are looked for in the folders
//! above this one (as the game itself does). Without them the tests are
//! skipped. Savegames go to a folder of their own, never to the player's.
//!
//! Run with `cargo test --release` (the scripts are slow without optimising).
//! One case: `cargo test --release -- --nocapture` and `PAGAN_CASE=<name>`.

use std::path::{Path, PathBuf};
use std::process::Command;

struct Case {
    name: String,
    args: Vec<String>,
    commands: String,
    expect: Vec<String>,
    reject: Vec<String>,
    needs: Vec<String>,
}

fn parse(path: &Path) -> Case {
    let text = std::fs::read_to_string(path).unwrap();
    let mut case = Case {
        name: path.file_stem().unwrap().to_string_lossy().into_owned(),
        args: Vec::new(),
        commands: String::new(),
        expect: Vec::new(),
        reject: Vec::new(),
        needs: Vec::new(),
    };
    for line in text.lines() {
        let line = line.trim_end();
        if let Some(a) = line.strip_prefix("args:") {
            case.args.extend(a.split_whitespace().map(String::from));
        } else if let Some(d) = line.strip_prefix("do:") {
            if !case.commands.is_empty() {
                case.commands.push(';');
            }
            case.commands.push_str(d.trim());
        } else if let Some(e) = line.strip_prefix("expect:") {
            case.expect.push(e.trim().to_string());
        } else if let Some(r) = line.strip_prefix("reject:") {
            case.reject.push(r.trim().to_string());
        } else if let Some(n) = line.strip_prefix("needs:") {
            case.needs.push(n.trim().to_string());
        }
    }
    case
}

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

#[test]
fn regression_cases() {
    let Some(game) = game_dir() else {
        eprintln!("regress: the original game files were not found, skipping");
        return;
    };
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("regress");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "case")).collect();
    files.sort();
    let only = std::env::var("PAGAN_CASE").ok();
    let work = std::env::temp_dir().join(format!("pagan-regress-{}", std::process::id()));
    let mut failed = Vec::new();
    let mut ran = 0;
    for file in files {
        let case = parse(&file);
        if only.as_ref().is_some_and(|o| *o != case.name) {
            continue;
        }
        if let Some(missing) = case.needs.iter().find(|n| !game.join(n).exists()) {
            eprintln!("regress {}: skipped ({missing} is not there)", case.name);
            continue;
        }
        // every case starts from the game's own beginning, with a fresh savegame folder
        let saves = work.join(&case.name);
        let _ = std::fs::remove_dir_all(&saves);
        std::fs::create_dir_all(&saves).unwrap();
        let shot = work.join(format!("{}.png", case.name));
        let out = Command::new(env!("CARGO_BIN_EXE_PaganRebuild"))
            .current_dir(&game)
            .env("PAGAN_SAVE_DIR", &saves)
            // the settings too: never the player's own
            .env("APPDATA", &saves)
            .env("XDG_DATA_HOME", &saves)
            .arg("--no-sound")
            .args(&case.args)
            .arg("--screenshot")
            .arg(&shot)
            .arg("--do")
            .arg(&case.commands)
            .output()
            .expect("the game did not start");
        let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        ran += 1;
        let mut problems: Vec<String> = Vec::new();
        if !out.status.success() {
            problems.push(format!("ended with {}", out.status));
        }
        problems.extend(case.expect.iter().filter(|e| !text.contains(e.as_str())).map(|e| format!("missing \"{e}\"")));
        problems.extend(case.reject.iter().filter(|r| text.contains(r.as_str())).map(|r| format!("unwanted \"{r}\"")));
        if problems.is_empty() {
            eprintln!("regress {}: ok", case.name);
        } else {
            eprintln!("regress {}: FAILED: {}", case.name, problems.join(", "));
            let log = work.join(format!("{}.log", case.name));
            let _ = std::fs::write(&log, &text);
            eprintln!("  output in {}", log.display());
            failed.push(case.name);
        }
    }
    assert!(ran > 0 || only.is_some(), "no cases found in {}", dir.display());
    assert!(failed.is_empty(), "failed: {}", failed.join(", "));
}
