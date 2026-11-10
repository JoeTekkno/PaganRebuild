//! Test commands for sound effects, speech and music.

use super::*;

pub(super) fn run(game: &mut Game, surf: &mut Surface, opts: &mut ViewOptions, st: &mut State, name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    match COMMANDS.iter().find(|(n, _)| *n == name) {
        Some((_, command)) => command(game, surf, opts, st, name, arg, cmd),
        None => Ok(Flow::Unknown),
    }
}

/// The commands of this group by name.
const COMMANDS: [(&str, super::Group); 6] = [
    ("sfxwav", cmd_sfxwav),
    ("sfxinfo", cmd_sfxinfo),
    ("speak", cmd_speak),
    ("sounds", cmd_sounds),
    ("musicwav", cmd_musicwav),
    ("albumwav", cmd_musicwav),
];

/// sfxwav:<effect>,<0|1>,<file.wav>  a sound effect at 44.1 kHz: 0 as the mixer would
///   stretch the original (in straight lines), 1 made anew (option "better sound")
fn cmd_sfxwav(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    let num = |s: &str| number(s, cmd);
    let parts: Vec<&str> = arg.splitn(3, ',').collect();
    let [n, how, file] = parts[..] else { return Err(format!("'{cmd}' expects effect,0|1,file")) };
    let better = num(how)? != 0;
    let Some((rate, samples)) = game.audio.effect_samples(num(n)?, better) else {
        println!("sfxwav: no effect {n}");
        return Ok(Flow::Done);
    };
    let samples: Vec<i16> = if rate == 44_100 {
        samples
    } else {
        let step = rate as f64 / 44_100.0;
        let len = (samples.len() as f64 / step) as usize;
        (0..len)
            .map(|i| {
                let p = i as f64 * step;
                let (k, t) = (p.floor() as usize, p.fract());
                let a = samples[k.min(samples.len() - 1)] as f64;
                let b = samples[(k + 1).min(samples.len() - 1)] as f64;
                (a + (b - a) * t) as i16
            })
            .collect()
    };
    let mut wav = Vec::with_capacity(44 + samples.len() * 2);
    let data_len = (samples.len() * 2) as u32;
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&44_100u32.to_le_bytes());
    wav.extend_from_slice(&(44_100u32 * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for v in &samples {
        wav.extend_from_slice(&v.to_le_bytes());
    }
    std::fs::write(file.trim(), wav).map_err(|e| e.to_string())?;
    println!("sfxwav: effect {n} ({} Hz originally), {} samples -> {}", rate, samples.len(), file.trim());
    Ok(Flow::Done)
}

/// decode every effect and report how plausible the result is
fn cmd_sfxinfo(_game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, _cmd: &str) -> Result<Flow, String> {
    let flex = flex::Flex::open(&std::path::PathBuf::from(arg)).map_err(|e| e.to_string())?;
    let (mut ok, mut bad, mut rough) = (0, 0, 0);
    for i in 1..flex.len() {
        let Some(d) = flex.get(i) else { continue };
        match audio::decode_sonarc(d) {
            Some((rate, pcm)) => {
                let diff: u64 = pcm.windows(2).map(|w| (w[0] as i32 - w[1] as i32).unsigned_abs() as u64).sum();
                let avg = diff as f32 / pcm.len().max(1) as f32;
                let want = flex::u32le(d, 0) as usize;
                ok += 1;
                if avg > 40.0 || pcm.len() != want {
                    rough += 1;
                    println!("entry {i}: rate {rate}, {} of {want} samples, roughness {avg:.1}", pcm.len());
                }
            }
            None => bad += 1,
        }
    }
    println!("sfxinfo: {ok} decoded ({rough} suspicious), {bad} failed");
    Ok(Flow::Done)
}

/// speak the first two recorded lines of a character
fn cmd_speak(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    let num = |s: &str| number(s, cmd);
    let shape = num(arg)? as u32;
    let text = game.audio.speech_sample_text(shape, 2);
    println!("speak {shape}: \"{}\" -> {:?} ms", text.chars().take(90).collect::<String>(), game.audio.speak(shape, &text));
    Ok(Flow::Done)
}

/// sounds  the playing effects of objects: where they come from (spatial sound)
fn cmd_sounds(game: &mut Game, surf: &mut Surface, opts: &mut ViewOptions, _st: &mut State, _name: &str, _arg: &str, _cmd: &str) -> Result<Flow, String> {
    game.options.spatial_sound = true;
    game.draw(surf, opts, false);
    game.place_sounds();
    for (o, pan, gain) in game.audio.spots() {
        println!("sounds: object {o} pan {pan:+.2} level {gain:.2}");
    }
    Ok(Flow::Done)
}

/// musicwav:<track>,<seconds>,<file.wav>  a piece rendered with the SoundFont
/// albumwav:<track>,<from>,<seconds>,<file.wav>  the album's piece as it is
///   played (with its loop), from second <from> on
fn cmd_musicwav(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    let num = |s: &str| number(s, cmd);
    let album = name == "albumwav";
    let parts: Vec<&str> = arg.splitn(if album { 4 } else { 3 }, ',').collect();
    let (track, from, secs, file) = match parts[..] {
        [t, s, f] if !album => (num(t)?, 0, num(s)?, f.trim()),
        [t, a, s, f] if album => (num(t)?, num(a)?, num(s)?, f.trim()),
        _ => return Err(format!("{name} needs <track>,{}<seconds>,<file> in '{cmd}'", if album { "<from>," } else { "" })),
    };
    let pcm = if album {
        let path = game.audio.album_file(track).ok_or(format!("no album piece for {track}"))?.clone();
        let mut pcm = album::render(&path, album::loops(track), (from + secs) as f32);
        pcm.drain(..(from as usize * 44_100 * 2).min(pcm.len()));
        pcm
    } else {
        let events = game.audio.music_events(track).ok_or(format!("no piece {track}"))?;
        audio::render_music(events, secs as f32).ok_or("no SoundFont found")?
    };
    let peak = pcm.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    let rms = (pcm.iter().map(|v| v * v).sum::<f32>() / pcm.len().max(1) as f32).sqrt();
    let mut wav = Vec::with_capacity(44 + pcm.len() * 2);
    let data_len = (pcm.len() * 2) as u32;
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&44_100u32.to_le_bytes());
    wav.extend_from_slice(&(44_100u32 * 4).to_le_bytes());
    wav.extend_from_slice(&4u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for v in &pcm {
        wav.extend_from_slice(&((v.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
    }
    std::fs::write(file, wav).map_err(|e| e.to_string())?;
    println!("{name}: piece {track}, {secs} s, peak {peak:.3}, rms {rms:.3} -> {file}");
    Ok(Flow::Done)
}
