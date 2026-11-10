//! The game's movies (.SKF files: the introduction and the end game).
//!
//! A movie is an archive: entry 0 lists what happens at which frame
//! (music, sounds, speech with subtitles, fades, waits), the other
//! entries are palettes, pictures (the first one whole, the following
//! ones only what changes), recorded speech and subtitle texts.  Ported
//! from ScummVM's Ultima 8 engine (gfx/skf_player.cpp; (C) the
//! ScummVM Team, GPL-3.0-or-later).

use crate::audio::Audio;
use crate::flex::{u16le, Flex};
use crate::font::Font;
use crate::palette::Palette;
use crate::shape::{decode_frame, TRANSPARENT};
use crate::view::Surface;
use std::path::Path;
use std::time::{Duration, Instant};

pub const WIDTH: i32 = 320;
pub const HEIGHT: i32 = 200;
/// Steps of a fade, and how long one takes.
const FADE_STEPS: i32 = 16;
const FADE_STEP: Duration = Duration::from_millis(33);

// what the event list can ask for
const PLAY_MUSIC: u16 = 3;
const SLOW_STOP_MUSIC: u16 = 4;
const PLAY_SFX: u16 = 5;
const STOP_SFX: u16 = 6;
const SET_SPEED: u16 = 7;
const FADE_OUT: u16 = 8;
const FADE_IN: u16 = 9;
const WAIT: u16 = 12;
const PLAY_SOUND: u16 = 14;
const FADE_WHITE: u16 = 15;
const CLEAR_SUBS: u16 = 18;

// kinds of archive entries
const OBJ_PALETTE: u16 = 1;
const OBJ_PICTURE: u16 = 2;

#[derive(Clone)]
pub struct Movie {
    flex: Flex,
    /// (frame, action, data), in the order they happen.
    events: Vec<(u16, u16, u16)>,
    cur_event: usize,
    cur_object: usize,
    cur_frame: u32,
    /// Frames still to wait.
    timer: u32,
    /// Frames per second.
    frame_rate: u32,
    next: Instant,
    colors: [u32; 256],
    /// The picture, as palette indices.
    image: Vec<u8>,
    /// How far the picture is faded away (0..=FADE_STEPS), to which
    /// colour, and whether a fade is going on (+1 out, -1 in).
    faded: i32,
    fade_colour: u32,
    fade_dir: i32,
    /// Subtitle and the line it starts at.
    subs: Option<(String, i32)>,
    pub done: bool,
    /// Movie time so far: the pauses between the steps added up.
    clock: Duration,
    /// When the movie asks for which music, worked out in advance.
    music_times: Vec<(Duration, i32)>,
    /// A piece that starts before the movie asks for it, so that it fits
    /// the pictures (album music), and whether that is worked out yet.
    early_music: Option<(Duration, i32)>,
    planned: bool,
}

/// What a movie asks of the sound: the game's audio, or a rehearsal that
/// only notes the times.
pub trait Sounds {
    fn music(&mut self, track: i32);
    fn sfx(&mut self, num: i32);
    fn stop_sfx(&mut self, num: i32);
    fn voice(&mut self, rate: u32, pcm: &[u8]);
    fn voice_playing(&self) -> bool;
    fn voice_remaining(&self) -> Option<Duration>;
    /// The music grows quiet and stops.
    fn fade_music(&mut self);
}

impl Sounds for Audio {
    fn music(&mut self, track: i32) {
        self.play_music(track);
    }
    fn sfx(&mut self, num: i32) {
        self.play_sfx(num, 0, false, 1.0);
    }
    fn stop_sfx(&mut self, num: i32) {
        Audio::stop_sfx(self, num, 0);
    }
    fn voice(&mut self, rate: u32, pcm: &[u8]) {
        self.play_voice(rate, pcm);
    }
    fn voice_playing(&self) -> bool {
        Audio::voice_playing(self)
    }
    fn voice_remaining(&self) -> Option<Duration> {
        Audio::voice_remaining(self)
    }
    fn fade_music(&mut self) {
        self.fade_out_music();
    }
}

/// A silent run through the movie on its own clock.
struct Rehearsal {
    now: Duration,
    voice_end: Duration,
    music: Vec<(Duration, i32)>,
}

impl Sounds for Rehearsal {
    fn music(&mut self, track: i32) {
        self.music.push((self.now, track));
    }
    fn sfx(&mut self, _: i32) {}
    fn stop_sfx(&mut self, _: i32) {}
    fn voice(&mut self, rate: u32, pcm: &[u8]) {
        self.voice_end = self.now + Duration::from_millis(pcm.len() as u64 * 1000 / rate.max(1) as u64);
    }
    fn voice_playing(&self) -> bool {
        self.now < self.voice_end
    }
    fn voice_remaining(&self) -> Option<Duration> {
        self.voice_end.checked_sub(self.now).filter(|d| !d.is_zero())
    }
    fn fade_music(&mut self) {}
}

impl Movie {
    pub fn open(path: &Path) -> Option<Movie> {
        let flex = Flex::open(path).ok()?;
        let list = flex.get(0)?;
        let mut events = Vec::new();
        for e in list.chunks_exact(6) {
            if u16le(e, 0) == 0xFFFF {
                break;
            }
            let event = (u16le(e, 0), u16le(e, 2), u16le(e, 4));
            // A last entry that repeats the one before it under picture 0
            // (the end movie: the Guardian's laughter once more) is left
            // out: waiting for the first to be spoken, the pictures stood
            // still for six seconds, then the laughter came again.
            if event.0 == 0 && events.last().is_some_and(|&(f, a, d)| f != 0 && (a, d) == (event.1, event.2)) {
                continue;
            }
            events.push(event);
        }
        Some(Movie {
            flex,
            events,
            cur_event: 0,
            cur_object: 0,
            cur_frame: 0,
            timer: 0,
            frame_rate: 15,
            next: Instant::now(),
            colors: [0; 256],
            image: vec![0; (WIDTH * HEIGHT) as usize],
            faded: 0,
            fade_colour: 0,
            fade_dir: 0,
            subs: None,
            done: false,
            clock: Duration::ZERO,
            music_times: Vec::new(),
            early_music: None,
            planned: false,
        })
        .map(|mut m| {
            m.music_times = m.rehearse();
            m
        })
    }

    /// Runs through a copy of the movie without sound and notes when it
    /// asks for music.
    fn rehearse(&self) -> Vec<(Duration, i32)> {
        let mut copy = self.clone();
        let mut r = Rehearsal { now: Duration::ZERO, voice_end: Duration::ZERO, music: Vec::new() };
        // bounded, should a broken list never end
        for _ in 0..100_000 {
            if copy.done {
                break;
            }
            r.now = copy.clock;
            copy.advance(&mut r);
        }
        r.music
    }

    /// For tests: runs through a copy without sound and lists where the
    /// pictures stand still because a sentence is still being spoken
    /// (picture, from when, how long), and how long the steps before it are.
    pub fn holds(&self) -> Vec<String> {
        let mut copy = self.clone();
        let mut r = Rehearsal { now: Duration::ZERO, voice_end: Duration::ZERO, music: Vec::new() };
        let mut out = Vec::new();
        let mut held: Option<(u32, Duration, String)> = None;
        let mut speaking = String::new();
        for _ in 0..100_000 {
            if copy.done {
                break;
            }
            r.now = copy.clock;
            let (frame, timer, fading) = (copy.cur_frame, copy.timer, copy.fade_dir != 0);
            let subs_before = copy.subs.clone();
            let pause = copy.advance(&mut r);
            if copy.subs != subs_before {
                if let Some((s, _)) = &copy.subs {
                    speaking = format!("'{s}' ({:.2} s from {:.2} s)", r.voice_end.saturating_sub(r.now).as_secs_f32(), r.now.as_secs_f32());
                }
            }
            let still = copy.cur_frame == frame && timer == 0 && !fading && copy.timer == 0 && copy.fade_dir == 0;
            match (still, &held) {
                (true, None) => {
                    let waiting = copy.events.get(copy.cur_event).map_or(String::new(), |e| format!("event {} data {} at picture {}", e.1, e.2, e.0));
                    held = Some((frame, r.now, format!("speaking {speaking}, waiting: {waiting}")));
                }
                (false, Some((f, since, why))) => {
                    out.push(format!("picture {f} stands still {:.2} s from {:.2} s (steps before {} ms); {why}", (r.now - *since).as_secs_f32(), since.as_secs_f32(), pause.as_millis()));
                    held = None;
                }
                _ => {}
            }
        }
        out
    }

    /// When the album has a piece the movie asks for later inside the one
    /// before (the fall of the intro inside "Banished to Pagan"), the
    /// piece before starts so early that the later part comes when the
    /// movie asks for it.
    fn plan_early_music(&self, audio: &Audio) -> Option<(Duration, i32)> {
        for pair in self.music_times.windows(2) {
            let ((first_at, first), (later_at, later)) = (pair[0], pair[1]);
            let Some(inside) = audio.album_cue(first, later) else { continue };
            let start = later_at.checked_sub(inside)?;
            return (start < first_at).then_some((start, first));
        }
        None
    }

    /// Lets the movie go on in real time.
    pub fn run(&mut self, audio: &mut Audio) {
        if !self.planned {
            self.planned = true;
            self.early_music = self.plan_early_music(audio);
        }
        let now = Instant::now();
        if self.done || now < self.next {
            return;
        }
        if let Some((at, track)) = self.early_music {
            if self.clock >= at {
                self.early_music = None;
                audio.play_music(track);
            }
        }
        let pause = self.advance(audio);
        // never try to catch up after a stall
        self.next = (self.next + pause).max(now);
    }

    /// One step, and the pause until the next one on the movie's clock.
    fn advance<S: Sounds>(&mut self, audio: &mut S) -> Duration {
        let pause = self.pause(audio);
        self.step(audio);
        self.clock += pause;
        pause
    }

    /// How long until the next step.
    fn pause<S: Sounds>(&self, audio: &S) -> Duration {
        let mut pause = Duration::from_millis(1000 / self.frame_rate.max(1) as u64);
        if self.fade_dir != 0 {
            // (the fade after the last picture slowly, into the credits)
            pause = if self.cur_object >= self.flex.len() { FADE_STEP * 3 } else { FADE_STEP };
        } else if let Some(rest) = audio.voice_remaining() {
            // The event list starts the sentences closer together than
            // they take to speak (it was timed for the machines of the
            // time).  Rather than stopping the pictures until a sentence
            // is over, they run a little slower all the way to the next.
            pause = pause.max(rest / self.steps_to_next_sentence()).min(pause * 4);
        }
        pause
    }

    /// Steps (pictures and waits) until the next sentence starts or the
    /// subtitle goes away; to the end of the movie if there is none.
    fn steps_to_next_sentence(&self) -> u32 {
        let mut steps = self.timer;
        for &(frame, action, data) in &self.events[self.cur_event.min(self.events.len())..] {
            // the last entries of a list can belong to frame 0: they happen at the very end
            if (action == PLAY_SOUND || action == CLEAR_SUBS) && frame as u32 >= self.cur_frame {
                return (steps + frame as u32 - self.cur_frame).max(1);
            }
            if action == WAIT {
                steps += data as u32;
            }
        }
        (steps + self.flex.len().saturating_sub(self.cur_object) as u32).max(1)
    }

    /// One step: of a fade, of a wait, or on to the next picture.
    pub fn step<S: Sounds>(&mut self, audio: &mut S) {
        if self.done {
            return;
        }
        if self.fade_dir != 0 {
            self.faded += self.fade_dir;
            if self.faded <= 0 || self.faded >= FADE_STEPS {
                self.faded = self.faded.clamp(0, FADE_STEPS);
                self.fade_dir = 0;
            }
            return;
        }
        if self.timer > 0 {
            self.timer -= 1;
            return;
        }
        if self.run_events(audio) {
            self.next_picture(audio);
        }
    }

    /// The events of the current picture. Returns false if one of them
    /// holds the movie here (a fade, a wait, a sentence still spoken).
    fn run_events<S: Sounds>(&mut self, audio: &mut S) -> bool {
        while let Some(&(frame, action, data)) = self.events.get(self.cur_event) {
            if frame as u32 > self.cur_frame {
                break;
            }
            // The pictures wait for a sentence to end before the next one
            // starts or its subtitle goes away: the events are timed for
            // the speed of the machines of the time, and nothing may be
            // cut off.
            if (action == PLAY_SOUND || action == CLEAR_SUBS) && audio.voice_playing() {
                return false;
            }
            self.cur_event += 1;
            match action {
                PLAY_MUSIC => audio.music(data as i32),
                SLOW_STOP_MUSIC => audio.music(0),
                PLAY_SFX => audio.sfx(data as i32),
                STOP_SFX => audio.stop_sfx(data as i32),
                SET_SPEED => self.frame_rate = (data as u32).max(1),
                FADE_OUT | FADE_WHITE => {
                    self.fade_colour = if action == FADE_WHITE { 0xFF_FFFF } else { 0 };
                    self.fade_dir = 1;
                    return false;
                }
                FADE_IN => {
                    self.fade_dir = -1;
                    return false;
                }
                WAIT => {
                    self.timer = data as u32;
                    return false;
                }
                PLAY_SOUND => self.play_sound(data as usize, audio),
                CLEAR_SUBS => self.subs = None,
                other => eprintln!("movie: unknown event {other} (data {data}) at frame {frame}"),
            }
        }
        true
    }

    /// On to the next picture; palettes on the way take effect.
    fn next_picture<S: Sounds>(&mut self, audio: &mut S) {
        self.cur_frame += 1;
        loop {
            self.cur_object += 1;
            if self.cur_object >= self.flex.len() {
                self.cur_object = self.flex.len();
                // the last picture fades away (the end movie, before the
                // credits: it stopped on its last picture)
                if self.faded < FADE_STEPS {
                    self.fade_colour = 0;
                    self.fade_dir = 1;
                    audio.fade_music();
                    return;
                }
                // the last words are spoken to the end
                self.done = !audio.voice_playing();
                return;
            }
            let Some(obj) = self.flex.get(self.cur_object).filter(|o| o.len() >= 2) else { continue };
            match u16le(obj, 0) {
                OBJ_PALETTE if obj.len() >= 2 + 768 => set_palette(&mut self.colors, &obj[2..]),
                OBJ_PICTURE => {
                    paint_picture(&mut self.image, &obj[2..]);
                    return;
                }
                _ => {}
            }
        }
    }

    /// Recorded speech; the entry before it holds the subtitle.
    fn play_sound<S: Sounds>(&mut self, object: usize, audio: &mut S) {
        if let Some(snd) = self.flex.get(object).filter(|s| s.len() > 34) {
            // signed 8 bit samples after a header with the sample rate
            let pcm: Vec<u8> = snd[34..].iter().map(|b| b ^ 0x80).collect();
            audio.voice(u16le(snd, 6) as u32, &pcm);
        }
        if let Some(text) = object.checked_sub(1).and_then(|o| self.flex.get(o)).filter(|t| t.len() > 7) {
            // the game fonts use the DOS character set: one byte, one character
            let line: String = text[6..].iter().take_while(|&&b| b != 0).map(|&b| if b < 0x20 { ' ' } else { b as char }).collect();
            self.subs = Some((line.trim().to_string(), u16le(text, 4) as i32));
        }
    }

    /// Draws the movie into the middle of the view, as large as it fits.
    pub fn draw(&self, surf: &mut Surface, font: Option<&Font>, pal: &Palette) {
        let mut pic = Surface::new(WIDTH, HEIGHT);
        let (fr, fg, fb) = ((self.fade_colour >> 16) & 0xFF, (self.fade_colour >> 8) & 0xFF, self.fade_colour & 0xFF);
        let (keep, gone) = ((FADE_STEPS - self.faded) as u32, self.faded as u32);
        let mut shown = [0u32; 256];
        for (out, &c) in shown.iter_mut().zip(self.colors.iter()) {
            let mix = |v: u32, to: u32| (v * keep + to * gone) / FADE_STEPS as u32;
            *out = (mix((c >> 16) & 0xFF, fr) << 16) | (mix((c >> 8) & 0xFF, fg) << 8) | mix(c & 0xFF, fb);
        }
        for (p, &i) in pic.pixels.iter_mut().zip(self.image.iter()) {
            *p = shown[i as usize];
        }
        if let (Some(font), Some((text, top))) = (font, &self.subs) {
            let mut y = *top;
            for line in font.wrap(text, 200) {
                font.draw(&mut pic, &line, (WIDTH - font.width(&line)) / 2, y, pal);
                y += font.height;
            }
        }

        let k = (surf.width / WIDTH).min(surf.height / HEIGHT).max(1);
        let (x0, y0) = ((surf.width - WIDTH * k) / 2, (surf.height - HEIGHT * k) / 2);
        surf.pixels.fill(0);
        for y in 0..HEIGHT * k {
            let dy = y0 + y;
            if dy < 0 || dy >= surf.height {
                continue;
            }
            for x in 0..WIDTH * k {
                let dx = x0 + x;
                if dx >= 0 && dx < surf.width {
                    surf.pixels[(dy * surf.width + dx) as usize] = pic.pixels[((y / k) * WIDTH + x / k) as usize];
                }
            }
        }
    }

    /// Where the movie is, for tests.
    pub fn describe(&self) -> String {
        format!("frame {} object {} event {}/{} faded {} subtitle {:?}{}", self.cur_frame, self.cur_object, self.cur_event,
                self.events.len(), self.faded, self.subs.as_ref().map(|s| s.0.as_str()), if self.done { " (over)" } else { "" })
    }
}

/// A palette of the movie: 256 colours of 6 bit VGA values.
fn set_palette(colors: &mut [u32; 256], data: &[u8]) {
    for (i, c) in colors.iter_mut().enumerate() {
        let v = |k: usize| {
            let x = (data[i * 3 + k] & 0x3F) as u32;
            (x << 2) | (x >> 4)
        };
        *c = (v(0) << 16) | (v(1) << 8) | v(2);
    }
}

/// Paints a picture of the movie over the last one: a shape frame
/// without the first part of its header.
fn paint_picture(image: &mut [u8], obj: &[u8]) {
    let mut data = vec![0u8; 8];
    data.extend_from_slice(obj);
    let Some(f) = decode_frame(&data) else { return };
    for row in 0..f.height {
        let y = row - f.yoff;
        if !(0..HEIGHT).contains(&y) {
            continue;
        }
        for col in 0..f.width {
            let (x, c) = (col - f.xoff, f.pixels[(row * f.width + col) as usize]);
            if c != TRANSPARENT && (0..WIDTH).contains(&x) {
                image[(y * WIDTH + x) as usize] = c as u8;
            }
        }
    }
}
