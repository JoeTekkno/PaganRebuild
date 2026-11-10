//! Music from the remastered soundtrack album instead of the MIDI pieces
//! (option "album music"). The player brings the album's files; they are
//! found by their titles, in a folder next to the game or the program, or in
//! %APPDATA%\pagan\music.
//!
//! The pieces are streamed from the files (WAV or Ogg Vorbis). A piece that
//! loops goes back to its beginning by blending its last seconds into its
//! first ones, as the composer suggested; a change of piece blends the old
//! one out while the new one comes in.

use crate::hash::HashMap;
use rodio::Source;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender, TryRecvError};
use std::sync::Arc;
use std::time::Duration;

/// Output rate; files at other rates are converted.
const RATE: u32 = 44_100;
/// Length of the blend from a piece's end into its beginning.
const LOOP_BLEND: f32 = 5.0;
/// How long the old piece takes to fade out when another one comes.
const FADE_OUT: f32 = 2.0;
/// How long a new piece takes to come in.
const FADE_IN: f32 = 0.3;
/// Frames per block from the reading thread to the output.
const BLOCK: usize = 4096;
/// Below this a sample counts as silence (about -60 dB): where a file
/// really begins and ends.
const QUIET: f32 = 0.001;
/// Level of the album against the MIDI music, so that switching the
/// option does not change the loudness much.
const LEVEL: f32 = 0.5;

/// The game's pieces (numbers as in MUSIC.FLX) and the album titles that
/// replace them. Pieces without a title stay MIDI (short ones like the
/// fanfare after a kill). Several numbers can share a title: the intro
/// movie asks for "intro" and then "fall", the album has both in one piece.
const TITLES: &[(i32, &str)] = &[
    (44, "game over"),
    (DEATH_WORLD, "death"),
    (48, "the east road (discrete version)"),
    (49, "banished to pagan (intro cinematic)"),
    (50, "banished to pagan (intro cinematic)"),
    (51, "theme to ultima viii - pagan"),
    (52, "the docks"),
    (53, "toran's execution"),
    (54, "toran's execution"),
    (55, "toran's execution"),
    (56, "tenebrae"),
    (57, "fisherman's reef (discrete version)"),
    (97, "argentrock isle (the path of air)"),
    (98, "pagan combat"),
    (99, "shrine of the ancient ones"),
    (102, "carthax lake (the path of water)"),
    (103, "hydros - the titan of water"),
    (105, "the ethereal plane"),
    (106, "running into disaster"),
    (109, "victory fanfare"),
    (107, "daemon's crag (the path of fire)"),
    (108, "pyros - the titan of fire"),
    (110, "running"),
    (111, "britannia burns (endgame cinematic)"),
    (112, "conventicle of the dead"),
    (114, "the upper catacombs"),
    (115, "stratos - the titan of air"),
    (116, "mythran's spell"),
    (117, "lithos - the titan of earth"),
    (119, "the cemetery"),
    (120, "the zealan gods"),
    (121, "the lower catacombs (the path of earth)"),
];

/// Pieces that are played once and not looped (a movie's, the death
/// music, the fanfare).
const ONCE: &[i32] = &[44, DEATH_WORLD, 49, 50, 109];

/// The death piece when no enemy killed the Avatar (drowned, burnt in lava,
/// a trap): "Death" from the album. Killed by an enemy, the game's own
/// death piece (44) plays, "Game Over" as in the original. Without the
/// album both are the MIDI piece 44.
pub const DEATH_WORLD: i32 = 1044;

/// Title of a file: its name without the track number in front
/// ("03 - The Docks.wav" -> "the docks").
fn title_of(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?.to_lowercase().replace('\u{2019}', "'");
    let rest = stem.trim_start_matches(|c: char| c.is_ascii_digit());
    let rest = if rest.len() < stem.len() { rest.trim_start().strip_prefix('-').unwrap_or(rest) } else { rest };
    Some(rest.trim().to_string())
}

/// The album's files in a folder: piece number -> file.
fn files_in(dir: &Path) -> HashMap<i32, PathBuf> {
    let mut found = HashMap::default();
    let Ok(entries) = std::fs::read_dir(dir) else { return found };
    for e in entries.flatten() {
        let path = e.path();
        let Some(wav) = path.extension().and_then(|x| x.to_str()).map(|x| x.to_ascii_lowercase()).and_then(|x| match x.as_str() {
            "wav" => Some(true),
            "ogg" => Some(false),
            _ => None,
        }) else {
            continue;
        };
        let Some(title) = title_of(&path) else { continue };
        for &(track, t) in TITLES {
            if t == title {
                // both there: the WAV (the better one) wins
                if wav || !found.contains_key(&track) {
                    found.insert(track, path.clone());
                }
            }
        }
    }
    found
}

/// Looks for the album: in %APPDATA%\pagan\music, then in the folders of
/// the game folder; the folder with the most pieces wins.
pub fn find(game_dir: &Path) -> HashMap<i32, PathBuf> {
    let mut dirs = Vec::new();
    if let Some(dir) = crate::user_dir() {
        dirs.push(dir.join("music"));
    }
    // folders in the game folder, and next to the program (when it has a
    // folder of its own inside the game folder or elsewhere)
    let exe = crate::exe_dir();
    for base in std::iter::once(game_dir.to_path_buf()).chain(exe) {
        if let Ok(entries) = std::fs::read_dir(&base) {
            dirs.extend(entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()));
        }
    }
    dirs.iter().map(|d| files_in(d)).max_by_key(|f| f.len()).unwrap_or_default()
}

/// Pieces the album has inside the one before, and how far in they
/// begin (seconds after the sound begins). The intro movie asks for the
/// fall (50) after the intro (49); in "Banished to Pagan" the cymbal crash
/// at 41.6 s belongs to the Avatar hitting the water, which comes 6 s
/// after the movie asks for the fall.
const CUES: &[(i32, f32)] = &[(50, 35.4)];

/// How far into the album's piece before it a piece begins.
pub fn cue(track: i32) -> Option<f32> {
    CUES.iter().find(|c| c.0 == track).map(|c| c.1)
}

/// Whether a piece loops.
pub fn loops(track: i32) -> bool {
    !ONCE.contains(&track)
}

// ---- reading the files ---------------------------------------------------------

/// A WAV file, read frame by frame as stereo.
struct Wav {
    file: BufReader<File>,
    data: u64,
    frames: u64,
    channels: u16,
    rate: u32,
    /// 1 = integer PCM, 3 = floating point.
    format: u16,
    bits: u16,
    /// The next frame to be read.
    at: u64,
}

impl Wav {
    fn open(path: &Path) -> Option<Wav> {
        let mut file = BufReader::with_capacity(1 << 16, File::open(path).ok()?);
        let mut head = [0u8; 12];
        file.read_exact(&mut head).ok()?;
        if &head[0..4] != b"RIFF" || &head[8..12] != b"WAVE" {
            return None;
        }
        let mut fmt = None;
        loop {
            let mut chunk = [0u8; 8];
            file.read_exact(&mut chunk).ok()?;
            let size = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]) as u64;
            match &chunk[0..4] {
                b"fmt " => {
                    let mut b = vec![0u8; size as usize];
                    file.read_exact(&mut b).ok()?;
                    let u16_at = |i: usize| u16::from_le_bytes([b[i], b[i + 1]]);
                    let mut format = u16_at(0);
                    if format == 0xFFFE && b.len() >= 26 {
                        format = u16_at(24); // WAVE_FORMAT_EXTENSIBLE: the real format
                    }
                    fmt = Some((format, u16_at(2), u32::from_le_bytes([b[4], b[5], b[6], b[7]]), u16_at(14)));
                    if size & 1 == 1 {
                        file.seek(SeekFrom::Current(1)).ok()?;
                    }
                }
                b"data" => {
                    let (format, channels, rate, bits) = fmt?;
                    let ok = matches!((format, bits), (1, 16 | 24 | 32) | (3, 32));
                    if !ok || channels == 0 || rate == 0 {
                        return None;
                    }
                    let data = file.stream_position().ok()?;
                    let frames = size / (channels as u64 * bits as u64 / 8);
                    return Some(Wav { file, data, frames, channels, rate, format, bits, at: 0 });
                }
                _ => {
                    file.seek(SeekFrom::Current((size + (size & 1)) as i64)).ok()?;
                }
            }
        }
    }

    fn frame_bytes(&self) -> u64 {
        self.channels as u64 * self.bits as u64 / 8
    }

    fn seek(&mut self, frame: u64) {
        self.at = frame.min(self.frames);
        let _ = self.file.seek(SeekFrom::Start(self.data + self.at * self.frame_bytes()));
    }

    /// Up to `n` frames from where it stands, appended to `out` as stereo.
    fn read(&mut self, n: usize, out: &mut Vec<[f32; 2]>) {
        let n = (n as u64).min(self.frames - self.at) as usize;
        let mut raw = vec![0u8; n * self.frame_bytes() as usize];
        if self.file.read_exact(&mut raw).is_err() {
            self.at = self.frames;
            return;
        }
        self.at += n as u64;
        let width = self.bits as usize / 8;
        let sample = |b: &[u8]| -> f32 {
            match (self.format, width) {
                (3, _) => f32::from_le_bytes([b[0], b[1], b[2], b[3]]),
                (_, 2) => i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0,
                (_, 3) => (i32::from_le_bytes([0, b[0], b[1], b[2]]) >> 8) as f32 / 8_388_608.0,
                _ => i32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f32 / 2_147_483_648.0,
            }
        };
        for f in raw.chunks_exact(self.frame_bytes() as usize) {
            let l = sample(&f[0..width]);
            let r = if self.channels > 1 { sample(&f[width..2 * width]) } else { l };
            out.push([l, r]);
        }
    }

    /// Where the sound really begins and ends (frames), without the
    /// silence around it.
    fn sounding(&mut self) -> (u64, u64) {
        let step = self.rate as u64; // one second at a time
        let loud = |f: &[f32; 2]| f[0].abs() > QUIET || f[1].abs() > QUIET;
        let mut buf = Vec::new();
        let mut start = 0;
        'start: while start < self.frames {
            self.seek(start);
            buf.clear();
            self.read(step as usize, &mut buf);
            if let Some(i) = buf.iter().position(loud) {
                start += i as u64;
                break 'start;
            }
            start += step;
        }
        let mut end = self.frames;
        while end > start {
            let from = end.saturating_sub(step).max(start);
            self.seek(from);
            buf.clear();
            self.read((end - from) as usize, &mut buf);
            if let Some(i) = buf.iter().rposition(loud) {
                end = from + i as u64 + 1;
                break;
            }
            end = from;
        }
        (start, end.max(start))
    }
}

/// An Ogg Vorbis file, decoded frame by frame as stereo. Going back means
/// starting again from the beginning, which the loop only does for the
/// first seconds.
struct Ogg {
    path: PathBuf,
    dec: Vorbis,
    channels: usize,
    rate: u32,
    /// Length by the position of the last page.
    frames: u64,
    /// The next frame to be read.
    at: u64,
    /// The decoded packet (interleaved) and how much of it is handed out.
    packet: Vec<f32>,
    used: usize,
}

type Vorbis = lewton::inside_ogg::OggStreamReader<BufReader<File>>;

impl Ogg {
    fn decoder(path: &Path) -> Option<Vorbis> {
        Vorbis::new(BufReader::with_capacity(1 << 16, File::open(path).ok()?)).ok()
    }

    fn open(path: &Path) -> Option<Ogg> {
        let dec = Ogg::decoder(path)?;
        let channels = dec.ident_hdr.audio_channels as usize;
        let rate = dec.ident_hdr.audio_sample_rate;
        let frames = last_granule(path)?;
        Some(Ogg { path: path.to_path_buf(), dec, channels, rate, frames, at: 0, packet: Vec::new(), used: 0 })
    }

    /// The next packet; false at the end.
    fn next_packet(&mut self) -> bool {
        loop {
            match self.dec.read_dec_packet_generic::<lewton::samples::InterleavedSamples<f32>>() {
                Ok(Some(p)) if p.samples.is_empty() => continue,
                Ok(Some(p)) => {
                    self.packet = p.samples;
                    self.used = 0;
                    return true;
                }
                _ => return false,
            }
        }
    }

    fn read(&mut self, n: usize, out: &mut Vec<[f32; 2]>) {
        let ch = self.channels;
        let mut got = 0;
        while got < n {
            if self.used >= self.packet.len() && !self.next_packet() {
                self.frames = self.at;
                break;
            }
            let take = ((self.packet.len() - self.used) / ch).min(n - got);
            for f in self.packet[self.used..self.used + take * ch].chunks_exact(ch) {
                out.push([f[0], if ch > 1 { f[1] } else { f[0] }]);
            }
            self.used += take * ch;
            got += take;
            self.at += take as u64;
        }
    }

    fn seek(&mut self, frame: u64) {
        if frame < self.at {
            let Some(d) = Ogg::decoder(&self.path) else { return };
            self.dec = d;
            self.at = 0;
            self.packet.clear();
            self.used = 0;
        }
        let mut skip = Vec::with_capacity(BLOCK);
        while self.at < frame {
            skip.clear();
            self.read(BLOCK.min((frame - self.at) as usize), &mut skip);
            if skip.is_empty() {
                break;
            }
        }
    }

    /// Where the sound really begins and ends (frames). The end is found
    /// in the last seconds alone: decoded from a page before them up to
    /// the end, they are counted back from the length.
    fn sounding(&mut self) -> (u64, u64) {
        let loud = |f: &[f32; 2]| f[0].abs() > QUIET || f[1].abs() > QUIET;
        let mut buf = Vec::new();
        self.seek(0);
        let mut start = 0;
        loop {
            buf.clear();
            self.read(self.rate as usize, &mut buf);
            if buf.is_empty() {
                break;
            }
            if let Some(i) = buf.iter().position(loud) {
                start += i as u64;
                break;
            }
            start += buf.len() as u64;
        }
        let mut end = self.frames;
        let tail = 30 * self.rate as u64;
        if self.frames < 2 * tail {
            // a short piece is simply read on to its end
            let mut at = start + buf.len() as u64;
            end = start;
            loop {
                if let Some(i) = buf.iter().rposition(loud) {
                    end = at - buf.len() as u64 + i as u64 + 1;
                }
                buf.clear();
                self.read(self.rate as usize, &mut buf);
                if buf.is_empty() {
                    break;
                }
                at += buf.len() as u64;
            }
        } else if let Some(e) = self.sound_end_in_tail(tail, loud) {
            end = e;
        }
        (start, end.clamp(start, self.frames))
    }

    /// Where the sound ends within the last `tail` frames: decoded from a
    /// page before them up to the end and counted back from the length.
    /// None if the file cannot be read there.
    fn sound_end_in_tail(&self, tail: u64, loud: impl Fn(&[f32; 2]) -> bool) -> Option<u64> {
        let mut dec = Ogg::decoder(&self.path)?;
        dec.seek_absgp_pg(self.frames - tail).ok()?;
        let (mut n, mut last) = (0u64, None);
        while let Ok(Some(p)) = dec.read_dec_packet_generic::<lewton::samples::InterleavedSamples<f32>>() {
            for f in p.samples.chunks_exact(self.channels) {
                let fr = [f[0], if self.channels > 1 { f[1] } else { f[0] }];
                if loud(&fr) {
                    last = Some(n);
                }
                n += 1;
            }
        }
        let first = self.frames.saturating_sub(n);
        Some(last.map_or(first, |i| first + i + 1))
    }
}

/// The position of the last page of an Ogg file: its length in frames.
fn last_granule(path: &Path) -> Option<u64> {
    let mut file = File::open(path).ok()?;
    let size = file.seek(SeekFrom::End(0)).ok()?;
    let from = size.saturating_sub(70_000);
    file.seek(SeekFrom::Start(from)).ok()?;
    let mut tail = Vec::new();
    file.read_to_end(&mut tail).ok()?;
    let at = tail.windows(4).rposition(|w| w == b"OggS")?;
    let g = tail.get(at + 6..at + 14)?;
    Some(u64::from_le_bytes(g.try_into().ok()?))
}

/// A file of the album, WAV or Ogg Vorbis.
enum Piece {
    Wav(Wav),
    Ogg(Box<Ogg>),
}

impl Piece {
    fn open(path: &Path) -> Option<Piece> {
        if path.extension().is_some_and(|x| x.eq_ignore_ascii_case("ogg")) {
            Ogg::open(path).map(|o| Piece::Ogg(Box::new(o)))
        } else {
            Wav::open(path).map(Piece::Wav)
        }
    }

    fn rate(&self) -> u32 {
        match self {
            Piece::Wav(w) => w.rate,
            Piece::Ogg(o) => o.rate,
        }
    }

    fn at(&self) -> u64 {
        match self {
            Piece::Wav(w) => w.at,
            Piece::Ogg(o) => o.at,
        }
    }

    fn seek(&mut self, frame: u64) {
        match self {
            Piece::Wav(w) => w.seek(frame),
            Piece::Ogg(o) => o.seek(frame),
        }
    }

    fn read(&mut self, n: usize, out: &mut Vec<[f32; 2]>) {
        match self {
            Piece::Wav(w) => w.read(n, out),
            Piece::Ogg(o) => o.read(n, out),
        }
    }

    fn sounding(&mut self) -> (u64, u64) {
        match self {
            Piece::Wav(w) => w.sounding(),
            Piece::Ogg(o) => o.sounding(),
        }
    }
}

/// Reads a file in a thread of its own and hands it on in blocks of
/// stereo samples at `RATE`. A looping piece blends its end into its
/// beginning, over and over; otherwise the thread ends with the piece.
fn read_piece(path: PathBuf, looped: bool, tx: SyncSender<Vec<f32>>) {
    let Some(mut a) = Piece::open(&path) else {
        eprintln!("album: cannot read {}", path.display());
        return;
    };
    let Some(mut b) = Piece::open(&path) else { return };
    let (start, end) = a.sounding();
    let blend = (LOOP_BLEND * a.rate() as f32) as u64;
    // too short to blend: it simply starts over
    let blend = if end - start > 3 * blend { blend } else { 0 };
    let mut conv = Converter::new(a.rate());
    a.seek(start);
    let send = |frames: &[[f32; 2]], conv: &mut Converter| -> bool {
        let out = conv.convert(frames);
        out.is_empty() || tx.send(out).is_ok()
    };
    let mut frames = Vec::with_capacity(BLOCK);
    loop {
        // the body of the piece, up to the blend
        let until = if looped { end - blend } else { end };
        while a.at() < until {
            frames.clear();
            a.read(BLOCK.min((until - a.at()) as usize), &mut frames);
            if frames.is_empty() || !send(&frames, &mut conv) {
                return;
            }
        }
        if !looped {
            return;
        }
        // the end fades out while the beginning comes in
        b.seek(start);
        let mut done = 0;
        while done < blend {
            let n = BLOCK.min((blend - done) as usize);
            let (mut x, mut y) = (Vec::with_capacity(n), Vec::with_capacity(n));
            a.read(n, &mut x);
            b.read(n, &mut y);
            frames.clear();
            for i in 0..x.len().min(y.len()) {
                let t = (done + i as u64) as f32 / blend as f32 * std::f32::consts::FRAC_PI_2;
                let (out, inn) = (t.cos(), t.sin());
                frames.push([x[i][0] * out + y[i][0] * inn, x[i][1] * out + y[i][1] * inn]);
            }
            if frames.is_empty() || !send(&frames, &mut conv) {
                return;
            }
            done += n as u64;
        }
        // on from where the beginning has got to
        std::mem::swap(&mut a, &mut b);
    }
}

/// Converts the sample rate (linear), for files not at `RATE`.
struct Converter {
    step: f64,
    pos: f64,
    last: [f32; 2],
}

impl Converter {
    fn new(rate: u32) -> Converter {
        Converter { step: rate as f64 / RATE as f64, pos: 0.0, last: [0.0; 2] }
    }

    fn convert(&mut self, frames: &[[f32; 2]]) -> Vec<f32> {
        let mut out = Vec::with_capacity(frames.len() * 2 + 4);
        if self.step == 1.0 {
            for f in frames {
                out.extend_from_slice(f);
            }
            return out;
        }
        // positions are counted from the frame before this block (`last`)
        while self.pos < frames.len() as f64 {
            let i = self.pos.floor() as usize;
            let t = (self.pos - i as f64) as f32;
            let a = if i == 0 { self.last } else { frames[i - 1] };
            let b = frames[i];
            out.push(a[0] + (b[0] - a[0]) * t);
            out.push(a[1] + (b[1] - a[1]) * t);
            self.pos += self.step;
        }
        self.pos -= frames.len() as f64;
        if let Some(&f) = frames.last() {
            self.last = f;
        }
        out
    }
}

// ---- playing -------------------------------------------------------------

/// A piece being played.
struct Voice {
    rx: Receiver<Vec<f32>>,
    buf: Vec<f32>,
    pos: usize,
    gain: f32,
    /// Change of the gain per sample (negative: fading out).
    step: f32,
    /// The piece has ended (or faded out) and goes away.
    gone: bool,
}

impl Voice {
    fn next(&mut self) -> f32 {
        if self.pos >= self.buf.len() {
            match self.rx.try_recv() {
                Ok(b) => {
                    self.buf = b;
                    self.pos = 0;
                }
                Err(TryRecvError::Empty) => return 0.0, // the file is late: a moment of silence
                Err(TryRecvError::Disconnected) => {
                    self.gone = true;
                    return 0.0;
                }
            }
        }
        let s = self.buf[self.pos] * self.gain;
        self.pos += 1;
        self.gain += self.step;
        if self.gain >= 1.0 {
            self.gain = 1.0;
            self.step = 0.0;
        } else if self.gain <= 0.0 && self.step < 0.0 {
            self.gone = true;
        }
        s
    }
}

enum Command {
    /// A new piece; the playing ones fade out.
    Play(Receiver<Vec<f32>>),
    /// All pieces fade out.
    Stop,
    /// A short piece over the playing one (the fanfare), which goes on.
    Overlay(Receiver<Vec<f32>>),
}

/// The output: mixes the pieces (the new one and the fading ones).
struct Mixer {
    commands: Receiver<Command>,
    voices: Vec<Voice>,
    volume: Arc<AtomicU32>,
    /// Samples until the commands are looked at again.
    countdown: u32,
}

impl Mixer {
    fn fade_all(&mut self) {
        let step = -1.0 / (FADE_OUT * RATE as f32 * 2.0);
        for v in &mut self.voices {
            v.step = v.step.min(step);
        }
    }
}

impl Iterator for Mixer {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.countdown == 0 {
            self.countdown = 1024; // only at the start of a frame pair (even count)
            while let Ok(c) = self.commands.try_recv() {
                let rx = match c {
                    Command::Overlay(rx) => rx,
                    Command::Play(rx) => {
                        self.fade_all();
                        rx
                    }
                    Command::Stop => {
                        self.fade_all();
                        continue;
                    }
                };
                let step = 1.0 / (FADE_IN * RATE as f32 * 2.0);
                self.voices.push(Voice { rx, buf: Vec::new(), pos: 0, gain: 0.0, step, gone: false });
            }
            self.voices.retain(|v| !v.gone);
        }
        self.countdown -= 1;
        let level = f32::from_bits(self.volume.load(Ordering::Relaxed));
        let mut s = 0.0;
        for v in &mut self.voices {
            s += v.next();
        }
        Some(s * level)
    }
}

impl Source for Mixer {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        2
    }
    fn sample_rate(&self) -> u32 {
        RATE
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Plays the album's pieces.
pub struct Player {
    commands: Sender<Command>,
    volume: Arc<AtomicU32>,
    _sink: rodio::Sink,
    /// The file playing (None: silence).
    pub playing: Option<PathBuf>,
}

impl Player {
    pub fn new(stream: &rodio::OutputStreamHandle, volume: u8) -> Option<Player> {
        let sink = rodio::Sink::try_new(stream).ok()?;
        let (commands, rx) = mpsc::channel();
        let level = Arc::new(AtomicU32::new(0));
        sink.append(Mixer { commands: rx, voices: Vec::new(), volume: level.clone(), countdown: 0 });
        let p = Player { commands, volume: level, _sink: sink, playing: None };
        p.set_volume(volume);
        Some(p)
    }

    /// Music level 0..=100.
    pub fn set_volume(&self, level: u8) {
        // squared, as the MIDI music answers to its volume
        let v = level.min(100) as f32 / 100.0;
        self.volume.store((v * v * LEVEL).to_bits(), Ordering::Relaxed);
    }

    /// Starts a piece (the one playing fades out); the same piece again
    /// goes on undisturbed.
    pub fn play(&mut self, path: &Path, looped: bool) {
        if self.playing.as_deref() == Some(path) {
            return;
        }
        // about 1.5 s read ahead
        let (tx, rx) = mpsc::sync_channel(16);
        let p = path.to_path_buf();
        std::thread::spawn(move || read_piece(p, looped, tx));
        let _ = self.commands.send(Command::Play(rx));
        self.playing = Some(path.to_path_buf());
    }

    /// Plays a piece once over the one playing, which goes on under it.
    pub fn overlay(&mut self, path: &Path) {
        let (tx, rx) = mpsc::sync_channel(16);
        let p = path.to_path_buf();
        std::thread::spawn(move || read_piece(p, false, tx));
        let _ = self.commands.send(Command::Overlay(rx));
    }

    pub fn stop(&mut self) {
        if self.playing.take().is_some() {
            let _ = self.commands.send(Command::Stop);
        }
    }
}

/// Renders a piece as it is played (with its loop), for tests: stereo
/// samples at `RATE`.
pub fn render(path: &Path, looped: bool, seconds: f32) -> Vec<f32> {
    let (tx, rx) = mpsc::sync_channel(16);
    let p = path.to_path_buf();
    std::thread::spawn(move || read_piece(p, looped, tx));
    let want = (seconds * RATE as f32) as usize * 2;
    let mut out = Vec::with_capacity(want);
    while out.len() < want {
        match rx.recv() {
            Ok(b) => out.extend(b),
            Err(_) => break,
        }
    }
    out.truncate(want);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles_from_file_names() {
        assert_eq!(title_of(Path::new("03 - The Docks.wav")).as_deref(), Some("the docks"));
        assert_eq!(title_of(Path::new("21 - Mythran\u{2019}s Spell.wav")).as_deref(), Some("mythran's spell"));
        assert_eq!(title_of(Path::new("The Docks.wav")).as_deref(), Some("the docks"));
    }

    /// The Ogg files against the WAV files they were made from: same
    /// length and sounding part, the decoded length as the pages say.
    /// `PAGAN_ALBUM_WAV` and `PAGAN_ALBUM_OGG` name the two folders.
    #[test]
    #[ignore]
    fn ogg_matches_wav() {
        let (Some(w), Some(o)) = (std::env::var_os("PAGAN_ALBUM_WAV"), std::env::var_os("PAGAN_ALBUM_OGG")) else { return };
        let mut names: Vec<_> = std::fs::read_dir(&w).unwrap().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "wav")).collect();
        names.sort();
        for wp in names {
            let op = Path::new(&o).join(wp.with_extension("ogg").file_name().unwrap());
            let mut wav = Wav::open(&wp).unwrap();
            let mut ogg = Ogg::open(&op).unwrap();
            let (ws, we) = wav.sounding();
            let (os, oe) = ogg.sounding();
            let pages = ogg.frames;
            ogg.seek(0);
            let mut n = 0u64;
            let mut buf = Vec::new();
            loop {
                buf.clear();
                ogg.read(1 << 16, &mut buf);
                if buf.is_empty() {
                    break;
                }
                n += buf.len() as u64;
            }
            println!("{}: wav {} {ws}..{we}, ogg {pages} (decoded {n}) {os}..{oe}", wp.file_name().unwrap().to_string_lossy(), wav.frames);
            assert_eq!(n, pages, "decoded length");
            // half a second: the end of a fade-out lies near the limit of silence
            assert!(ws.abs_diff(os) < 22_050 && we.abs_diff(oe) < 22_050, "sounding part");
            // played with its loop (a short piece, past the blend): the
            // loudness goes alike in tenths of a second (no gap, no jump)
            if wav.frames < 60 * RATE as u64 && wav.frames > 3 * (LOOP_BLEND * RATE as f32) as u64 {
                let secs = wav.frames as f32 / RATE as f32 + 8.0;
                let (x, y) = (render(&wp, true, secs), render(&op, true, secs));
                assert_eq!(x.len(), y.len());
                let level = |v: &[f32]| v.chunks(RATE as usize / 5).map(|c| (c.iter().map(|s| s * s).sum::<f32>() / c.len() as f32).sqrt()).collect::<Vec<_>>();
                let (lx, ly) = (level(&x), level(&y));
                let worst = lx.iter().zip(&ly).map(|(a, b)| (a - b).abs() / a.max(*b).max(0.01)).fold(0.0f32, f32::max);
                println!("  looped: largest change of loudness {:.1} %", worst * 100.0);
                assert!(worst < 0.3, "loop sounds different");
            }
        }
    }

    #[test]
    fn rate_conversion_keeps_length() {
        let mut c = Converter::new(22_050);
        let frames = vec![[0.5f32, -0.5]; 1000];
        let n: usize = (0..10).map(|_| c.convert(&frames).len()).sum();
        assert!((n as i64 - 20_000 * 2).abs() <= 4, "{n}");
    }
}
