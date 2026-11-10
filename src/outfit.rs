//! Optional (option "show armour"): the Avatar's red tunic takes the
//! colours of the body armour he wears. The original draws him always the
//! same; his pictures (shape 1, all animations) are recoloured once per
//! armour and handed out instead of the originals.
//!
//! Clothes (shirts, robes ...) give the tunic their own colours, read from
//! the garment's picture.
//!
//! Only the tunic can be told apart by its colours (the red ramp of the
//! palette); legs and head share the grey ramp, so helmets and leggings
//! do not show.

use crate::palette::Palette;
use crate::shape::{Frame, Shape, TRANSPARENT};

/// The Avatar's pictures.
pub const AVATAR_SHAPE: u32 = 1;

/// The red ramp of the palette (light to dark): the tunic.
const TUNIC: std::ops::RangeInclusive<u16> = 32..=47;
/// Ramps to recolour it with, light to dark.
const GREY: std::ops::RangeInclusive<u16> = 16..=31;
const BROWN: std::ops::RangeInclusive<u16> = 208..=223;
const STEEL: std::ops::RangeInclusive<u16> = 240..=254;
const BRONZE: std::ops::RangeInclusive<u16> = 162..=174;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Look {
    /// No armour: as in the original.
    Plain,
    /// One of the clothes (its number among them, 0..=12).
    Cloth(u8),
    Leather,
    Chain,
    Plate,
    /// The magic armour that looks like silver mail.
    SilverMail,
    /// The magic armour of bronze plates.
    Bronze,
}

/// The look of a body armour (shape, frame), as the game's items show it.
pub fn look_of(shape: u32, frame: u32) -> Look {
    match (shape, frame / 2) {
        (523, 0) => Look::Plate,
        (523, 1) => Look::Chain,
        (523, 2) => Look::Leather,
        (64, _) => Look::SilverMail,
        (841, _) => Look::Bronze,
        (CLOTHES_SHAPE, n) => Look::Cloth(n as u8),
        _ => Look::Plain,
    }
}

/// Looks with a fine pattern (rings), which the smoothing would blur:
/// those pixels are left out of it (see `Game::mark_sharp`).
pub fn patterned(look: Look) -> bool {
    matches!(look, Look::Chain | Look::SilverMail)
}

/// Is this pixel of the Avatar's original pictures part of the tunic?
pub fn is_tunic(p: u16) -> bool {
    TUNIC.contains(&p)
}

/// The clothes: shirts, robes and the like, two frames each (lying on the
/// ground, worn in the equipment window).
pub const CLOTHES_SHAPE: u32 = 823;

/// The colours of a garment, from its picture in the equipment window:
/// those that make up at least a fiftieth of it (not the black outline).
pub fn cloth_colours(clothes: &Shape, n: u8) -> Vec<u16> {
    let Some(f) = clothes.frames.get(n as usize * 2 + 1).or_else(|| clothes.frames.get(n as usize * 2)) else { return Vec::new() };
    let mut count = [0usize; 256];
    for &p in &f.pixels {
        if p != TRANSPARENT && p != 0 {
            count[p as usize] += 1;
        }
    }
    let total: usize = count.iter().sum();
    (0..256u16).filter(|&i| count[i as usize] * 50 >= total.max(1)).collect()
}

/// Brightness of a palette colour (the strongest channel).
fn value(c: u32) -> i32 {
    (((c >> 16) & 0xFF).max((c >> 8) & 0xFF).max(c & 0xFF)) as i32
}

/// For every colour of the tunic: the one of `colours` of about the same
/// brightness (times `k`).
fn ramp_map(pal: &Palette, colours: &[u16], k: f32) -> [u16; 16] {
    let mut map = [0u16; 16];
    for (n, i) in TUNIC.enumerate() {
        let want = (value(pal.base_color(i as usize)) as f32 * k) as i32;
        map[n] = colours.iter().copied().min_by_key(|&j| (value(pal.base_color(j as usize)) - want).abs()).unwrap_or(i);
    }
    map
}

/// One step lighter (`d` < 0) or darker within a ramp.
fn step(i: u16, d: i32, ramp: &std::ops::RangeInclusive<u16>) -> u16 {
    (i as i32 + d).clamp(*ramp.start() as i32, *ramp.end() as i32) as u16
}

/// The Avatar's pictures with the tunic in the colours of `look`;
/// `cloth`: the colours of the garment (`cloth_colours`) for clothes;
/// `glow`: the magic pieces that glow.
pub fn dress(orig: &Shape, look: Look, helm: Helm, glow: Glow, pal: &Palette, cloth: &[u16]) -> Shape {
    // the tunic as it is: no armour (or clothes without colours)
    let plain = look == Look::Plain || (matches!(look, Look::Cloth(_)) && cloth.is_empty());
    let (ramp, k, rings) = match look {
        _ if plain => (TUNIC, 1.0, false),
        // (the ramp is only used for patterns, which clothes do not have)
        Look::Plain | Look::Cloth(_) => (TUNIC, 1.0, false),
        Look::Leather => (BROWN, 0.8, false),
        Look::Chain => (GREY, 0.85, true),
        Look::Plate => (STEEL, 1.0, false),
        Look::SilverMail => (STEEL, 1.1, true),
        Look::Bronze => (BRONZE, 0.9, false),
    };
    let colours: Vec<u16> = match look {
        Look::Cloth(_) => cloth.to_vec(),
        _ => ramp.clone().collect(),
    };
    let map = ramp_map(pal, &colours, k);
    let frames = orig
        .frames
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let mut out = clone_frame(f);
            put_on_helm(&mut out, f, i as u32, helm, pal);
            if plain {
                return out;
            }
            for row in 0..f.height {
                for col in 0..f.width {
                    let p = &mut out.pixels[(row * f.width + col) as usize];
                    if *p == TRANSPARENT || !TUNIC.contains(p) {
                        continue;
                    }
                    let mut c = map[(*p - TUNIC.start()) as usize];
                    if rings {
                        // a pattern of small rings: a dark dot every few pixels,
                        // a light one beside it, offset from row to row
                        match (col + 2 * row).rem_euclid(4) {
                            0 => c = step(c, 2, &ramp),
                            1 => c = step(c, -1, &ramp),
                            _ => {}
                        }
                    } else if look == Look::Plate || look == Look::Bronze {
                        // polished: the light parts a little brighter still
                        if c <= *ramp.start() + 5 {
                            c = step(c, -1, &ramp);
                        }
                    }
                    *p = c;
                }
            }
            out
        })
        .collect::<Vec<Frame>>();
    let frames = frames.into_iter().zip(&orig.frames).enumerate().map(|(i, (out, f))| glow_outline(out, f, i as u32, glow));
    Shape { frames: frames.collect() }
}

/// The magic pieces the Avatar wears (option "magic glow"): they glow at
/// their outline as their pictures as things do.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Glow {
    pub head: bool,
    pub body: bool,
    pub arms: bool,
    pub legs: bool,
}

impl Glow {
    pub fn any(&self) -> bool {
        self.head || self.body || self.arms || self.legs
    }
}

/// Which magic piece a worn thing is, if it is one: the winged helmet,
/// the silver mail and the bronze armour, the arm and the leg guards of
/// the same kind (their pictures have the blue outline).
pub fn add_glow(glow: &mut Glow, shape: u32) {
    match shape {
        843 => glow.head = true,
        64 | 841 => glow.body = true,
        844 => glow.arms = true,
        845 => glow.legs = true,
        _ => {}
    }
}

/// Lets the magic pieces in `out` (a copy of the original picture `f`)
/// glow: a blue line one pixel outside the figure (its black outline
/// stays), wherever a piece lies up to two pixels away. Outside the figure
/// it has no gaps (the outline is not black everywhere), and only the
/// outside edge glows, not the lines between arm and body. The picture
/// grows by a pixel on every side.
/// The arms are the grey and gold above the hem of the tunic, the legs
/// the grey from there down.
fn glow_outline(out: Frame, f: &Frame, frame: u32, glow: Glow) -> Frame {
    if !glow.any() {
        return out;
    }
    let (w, h) = (f.width, f.height);
    let head: crate::hash::HashSet<usize> = head_pixels(f, frame).unwrap_or_default().into_iter().collect();
    let hem = (0..h).rev().find(|&y| (0..w).any(|x| TUNIC.contains(&f.pixels[(y * w + x) as usize]))).map(|y| y - 2);
    let pixel = |x: i32, y: i32| (x >= 0 && y >= 0 && x < w && y < h).then(|| f.pixels[(y * w + x) as usize]);
    let solid = |x: i32, y: i32| pixel(x, y).is_some_and(|c| c != TRANSPARENT);
    // does the pixel belong to a glowing piece?
    let glows = |x: i32, y: i32| {
        let Some(c) = pixel(x, y) else { return false };
        if head.contains(&((y * w + x) as usize)) {
            return glow.head;
        }
        if TUNIC.contains(&c) {
            return glow.body;
        }
        let Some(hem) = hem else { return false };
        let low = y >= hem;
        (glow.legs && low && GREY.contains(&c)) || (glow.arms && !low && (GREY.contains(&c) || LIMB_GOLD.contains(&c)))
    };
    let near_piece = |x: i32, y: i32| (-2..=2).any(|dy| (-2..=2).any(|dx| glows(x + dx, y + dy)));
    let touches = |x: i32, y: i32| [(-1, 0), (1, 0), (0, -1), (0, 1)].iter().any(|&(dx, dy)| solid(x + dx, y + dy));

    let (nw, nh) = (w + 2, h + 2);
    let mut pixels = vec![TRANSPARENT; (nw * nh) as usize];
    for y in -1..=h {
        for x in -1..=w {
            let at = ((y + 1) * nw + x + 1) as usize;
            if solid(x, y) {
                pixels[at] = out.pixels[(y * w + x) as usize];
            } else if touches(x, y) && near_piece(x, y) {
                pixels[at] = GLOW;
            }
        }
    }
    Frame { width: nw, height: nh, xoff: out.xoff + 1, yoff: out.yoff + 1, pixels }
}

/// The blue of the glow around magic things (their pictures as things
/// have it at the outline).
const GLOW: u16 = 138;

/// The gold of the shoulder pads and arm guards.
const LIMB_GOLD: std::ops::RangeInclusive<u16> = 162..=174;

fn clone_frame(f: &Frame) -> Frame {
    Frame { width: f.width, height: f.height, xoff: f.xoff, yoff: f.yoff, pixels: f.pixels.clone() }
}

/// The helmet the Avatar wears. He is always drawn with a closed iron
/// helmet (the game's great helm); others change its colours.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Helm {
    /// No helmet, or the great helm: as drawn.
    Plain,
    /// The mail coif: light grey rings.
    Coif,
    /// The open steel cap: bright, bluish steel.
    Cap,
    /// The dark helmet: blackened iron.
    Dark,
    /// The dark helmet with an orange crest along its top.
    Crest,
    /// The magic helmet: bright steel with the blue glow of magic things.
    Winged,
}

/// The look of a helmet (shape, frame).
pub fn helm_of(shape: u32, frame: u32) -> Helm {
    match (shape, frame / 2) {
        (531, 1) => Helm::Coif,
        (531, 2) => Helm::Cap,
        (531, 3) => Helm::Dark,
        (531, 4) => Helm::Crest,
        (843, _) => Helm::Winged,
        _ => Helm::Plain,
    }
}

/// Does the helmet have a fine pattern the smoothing would blur?
pub fn helm_patterned(helm: Helm) -> bool {
    helm == Helm::Coif
}

/// Recolours the head in `out` (a copy of the original picture `f`).
fn put_on_helm(out: &mut Frame, f: &Frame, frame: u32, helm: Helm, pal: &Palette) {
    if helm == Helm::Plain {
        return;
    }
    let Some(head) = head_pixels(f, frame) else { return };
    let w = f.width as usize;
    let near = |ramp: std::ops::RangeInclusive<u16>, i: u16, k: f32| -> u16 {
        let want = (value(pal.base_color(i as usize)) as f32 * k) as i32;
        ramp.min_by_key(|&j| (value(pal.base_color(j as usize)) - want).abs()).unwrap_or(i)
    };
    // the topmost head pixel of every column, for the crest
    let mut col_top = vec![usize::MAX; w];
    for &p in &head {
        col_top[p % w] = col_top[p % w].min(p / w);
    }
    for &p in &head {
        let (x, y) = (p % w, p / w);
        let i = f.pixels[p];
        out.pixels[p] = match helm {
            Helm::Plain => i,
            Helm::Coif => {
                let c = near(GREY, i, 1.6);
                match (x + 2 * y) % 4 {
                    0 => step(c, 2, &GREY),
                    1 => step(c, -1, &GREY),
                    _ => c,
                }
            }
            Helm::Cap => near(STEEL, i, 1.7),
            Helm::Dark => near(GREY, i, 0.6),
            Helm::Crest if y < col_top[x] + 2 => near(CREST, i, 3.0),
            Helm::Crest => near(GREY, i, 0.6),
            Helm::Winged => near(STEEL, i, 1.6),
        };
    }
}

/// Orange for the crest, light to dark.
const CREST: std::ops::RangeInclusive<u16> = 229..=234;

/// The grey ramp: the Avatar's helmet, but also his legs.
const HEAD: std::ops::RangeInclusive<u16> = 16..=31;

/// The pixels of the Avatar's head (his closed helmet) in one of his
/// pictures, as indices into its pixels. The head is the grey patch
/// highest up, as far as it lies above the tunic; the legs are grey too,
/// but below it. None where that is not clear (lying, falling ...): there the
/// helmet keeps its own colours.
///
/// Where that finds nothing, a second look: bent forward (climbing, a
/// blow) the head lies lower than the top of the tunic, beside it; with
/// the arms raised the grey of the arms runs into the head, which is then
/// the bottom end of that patch. Without it the helmet lost its colours
/// (and the winged one its glow) in those pictures.
///
/// Not so where he falls, lies or gets up (`frame`, see note_fallen_frames):
/// there a leg stretched out looks like a head.
pub fn head_pixels(f: &Frame, frame: u32) -> Option<Vec<usize>> {
    let fallen = FALLEN.get().is_some_and(|s| s.contains(&frame));
    find_head(f, false).or_else(|| if fallen { None } else { find_head(f, true) })
}

/// The Avatar's pictures of falling, lying and getting up.
static FALLEN: std::sync::OnceLock<std::collections::BTreeSet<u32>> = std::sync::OnceLock::new();

/// Notes the Avatar's pictures of falling, lying and getting up (made by
/// `frames` the first time).
pub fn note_fallen_frames(frames: impl FnOnce() -> std::collections::BTreeSet<u32>) {
    FALLEN.get_or_init(frames);
}

fn find_head(f: &Frame, second_look: bool) -> Option<Vec<usize>> {
    let (w, h) = (f.width as usize, f.height as usize);
    let at = |x: usize, y: usize| f.pixels[y * w + x];
    let tunic_top = (0..h).find(|&y| (0..w).any(|x| TUNIC.contains(&at(x, y))))?;
    let tunic_bottom = (0..h).rev().find(|&y| (0..w).any(|x| TUNIC.contains(&at(x, y))))?;
    // the first tunic pixel from the top in every column
    let col_tunic: Vec<usize> = (0..w).map(|x| (0..h).find(|&y| TUNIC.contains(&at(x, y))).unwrap_or(h)).collect();
    let mut seen = vec![false; w * h];
    // the grey patches, as far as they lie above the tunic
    let mut pieces: Vec<Vec<usize>> = Vec::new();
    for start in 0..w * h {
        if seen[start] || !HEAD.contains(&f.pixels[start]) {
            continue;
        }
        let mut patch = Vec::new();
        let mut todo = vec![start];
        seen[start] = true;
        while let Some(p) = todo.pop() {
            patch.push(p);
            let (x, y) = ((p % w) as i32, (p / w) as i32);
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let q = ny as usize * w + nx as usize;
                if !seen[q] && HEAD.contains(&f.pixels[q]) {
                    seen[q] = true;
                    todo.push(q);
                }
            }
        }
        // seen from the front the helmet joins the grey of the arms over the
        // neck: only what lies above the tunic can be the head (at the
        // second look: no tunic above it in its column, and not lower than
        // the tunic reaches - the legs are below it)
        if second_look {
            patch.retain(|p| p / w < col_tunic[p % w] && p / w <= tunic_bottom);
        } else {
            patch.retain(|p| p / w < tunic_top);
        }
        if !patch.is_empty() {
            pieces.push(patch);
        }
    }
    let bbox = |v: &[usize]| {
        let xs = v.iter().map(|p| p % w);
        let ys = v.iter().map(|p| p / w);
        (xs.clone().min().unwrap_or(0), ys.clone().min().unwrap_or(0), xs.max().unwrap_or(0), ys.max().unwrap_or(0))
    };
    // pieces close to each other belong together (an arm in front of the
    // helmet cuts it in two)
    let mut group: Vec<usize> = (0..pieces.len()).collect();
    let boxes: Vec<_> = pieces.iter().map(|p| bbox(p)).collect();
    loop {
        let mut changed = false;
        for a in 0..pieces.len() {
            for b in 0..pieces.len() {
                let (ba, bb) = (boxes[a], boxes[b]);
                let near = ba.0 <= bb.2 + 2 && bb.0 <= ba.2 + 2 && ba.1 <= bb.3 + 2 && bb.1 <= ba.3 + 2;
                if near && group[a] != group[b] {
                    let g = group[a].min(group[b]);
                    group[a] = g;
                    group[b] = g;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    // a head is compact: about as wide as high and well filled (a leg
    // stretched upwards in a fall is narrow and long, so is an arm beside
    // the helmet)
    let head_like = |head: &[usize]| {
        let (x0, y0, x1, y1) = bbox(head);
        let (bw, bh) = (x1 - x0 + 1, y1 - y0 + 1);
        // (at the second look a little wider - a leg stretched out sideways
        // beside the tunic is a narrow patch too - but less filled: a head
        // seen from behind while climbing is much shaded)
        let (least, fill) = if second_look { (7, 45) } else { (5, 50) };
        (25..=220).contains(&head.len()) && bw >= least && bh >= least && bw.max(bh) <= 16 && bw.max(bh) <= 2 * bw.min(bh)
            && head.len() * 100 >= fill * bw * bh
    };
    let mut best: Option<(usize, Vec<usize>)> = None;
    for g in 0..pieces.len() {
        let members: Vec<usize> = (0..pieces.len()).filter(|&k| group[k] == g).collect();
        if members.is_empty() {
            continue;
        }
        let whole: Vec<usize> = members.iter().flat_map(|&k| pieces[k].iter().copied()).collect();
        // the pieces together, or else the biggest of them alone
        let head = if head_like(&whole) {
            whole
        } else {
            let Some(&big) = members.iter().max_by_key(|&&k| pieces[k].len()) else { continue };
            if head_like(&pieces[big]) {
                pieces[big].clone()
            } else if second_look {
                // the arms raised over the head: the bottom end of a piece;
                // an arm hanging down from the head (climbing): the top end
                // (of any piece, the biggest first: that may be a leg)
                let mut by_size = members.clone();
                by_size.sort_by_key(|&k| std::cmp::Reverse(pieces[k].len()));
                let ends = by_size.iter().flat_map(|&k| {
                    let (_, top, _, bottom) = bbox(&pieces[k]);
                    let low: Vec<usize> = pieces[k].iter().copied().filter(|p| p / w + 12 > bottom).collect();
                    let high: Vec<usize> = pieces[k].iter().copied().filter(|p| p / w < top + 12).collect();
                    [low, high]
                });
                let Some(end) = ends.into_iter().find(|e| head_like(e)) else { continue };
                end
            } else {
                continue;
            }
        };
        let top = bbox(&head).1;
        if best.as_ref().is_none_or(|b| top < b.0) {
            best = Some((top, head));
        }
    }
    best.map(|b| b.1)
}
