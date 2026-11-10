//! Test commands for items: creating them, the backpack and its windows, dragging, splitting.

use super::*;

pub(super) fn run(game: &mut Game, surf: &mut Surface, opts: &mut ViewOptions, st: &mut State, name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    match COMMANDS.iter().find(|(n, _)| *n == name) {
        Some((_, command)) => command(game, surf, opts, st, name, arg, cmd),
        None => Ok(Flow::Unknown),
    }
}

/// The commands of this group by name.
const COMMANDS: [(&str, super::Group); 26] = [
    ("dragpointer", cmd_dragpointer),
    ("hurl", cmd_hurl),
    ("drag", cmd_drag),
    ("give", cmd_give),
    ("pack", cmd_pack),
    ("place", cmd_place),
    ("putin", cmd_putin),
    ("stackables", cmd_stackables),
    ("potion", cmd_potion),
    ("keyring", cmd_keyring),
    ("wearables", cmd_wearables),
    ("helmcycle", cmd_helmcycle),
    ("armourcycle", cmd_armourcycle),
    ("worn", cmd_worn),
    ("wear", cmd_wear),
    ("packmask", cmd_packmask),
    ("sliderclick", cmd_sliderclick),
    ("asksplit", cmd_asksplit),
    ("split", cmd_split),
    ("movegump", cmd_movegump),
    ("take", cmd_take),
    ("inv", cmd_inv),
    ("dragpage", cmd_dragpage),
    ("piles", cmd_piles),
    ("dropsplit", cmd_dropsplit),
    ("gridclick", cmd_gridclick),
];

/// dragpointer:<id>:<x>,<y>  carrying the item: what letting go there would do (the pointer)
fn cmd_dragpointer(game: &mut Game, surf: &mut Surface, opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    let num = |s: &str| number(s, cmd);
    let point = |s: &str| point(s, cmd);
    let (id, to) = arg.split_once(':').ok_or(format!("'{cmd}' expects id:x,y"))?;
    let (x, y) = point(to)?;
    let id = num(id)? as u16;
    game.draw(surf, opts, false);
    game.ui.drag = Some(drag::Drag { item: id, x, y });
    let fb = game.drag_feedback(x, y);
    game.ui.drag = None;
    println!("dragpointer {x},{y}: {}", match fb {
        drag::DragFeedback::Drop => "put down",
        drag::DragFeedback::Throw => "throw",
        drag::DragFeedback::Nothing => "no",
        drag::DragFeedback::None => "-",
    });
    Ok(Flow::Done)
}

/// hurl:<id>,<vx>,<vy>,<vz>[,<gravity>]  throw an item with these speeds per tick (gravity 4)
fn cmd_hurl(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    let num = |s: &str| number(s, cmd);
    let v: Vec<i32> = arg.split(',').map(num).collect::<Result<_, _>>()?;
    if v.len() < 4 {
        return Err(format!("'{cmd}' expects id,vx,vy,vz[,gravity]"));
    }
    let hurl = crate::effects::Hurl { item: v[0] as u16, speed: (v[1], v[2], v[3]), gravity: v.get(4).copied().unwrap_or(4) };
    game.add_native(crate::vm::Kind::Hurl(hurl), hurl.item, crate::effects::PROC_HURL);
    println!("{cmd}: thrown");
    Ok(Flow::Done)
}

/// drag:<id>:<x>,<y>  move an item to a view position
fn cmd_drag(game: &mut Game, surf: &mut Surface, opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    let num = |s: &str| number(s, cmd);
    let point = |s: &str| point(s, cmd);
    let (id, to) = arg.split_once(':').ok_or(format!("'{cmd}' expects id:x,y"))?;
    let (x, y) = point(to)?;
    game.draw(surf, opts, false);
    let ok = game.begin_drag(num(id)? as u16, x, y) && game.drop_drag(x, y);
    game.ui.drag = None;
    println!("{cmd}: {}", if ok { "moved" } else { "not possible" });
    Ok(Flow::Done)
}

/// give:<shape>[,<frame>[,<amount>]]  create an item at the Avatar's feet
fn cmd_give(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, st: &mut State, _name: &str, arg: &str, _cmd: &str) -> Result<Flow, String> {
    let v: Vec<u32> = arg.split(',').filter_map(|s| s.trim().parse().ok()).collect();
    let av = game.avatar();
    let cur = game.cur_map as u16;
    let id = game.map_mut().add(map::Item {
        x: av.x + 96, y: av.y, z: av.z, shape: v.first().copied().unwrap_or(0), frame: v.get(1).copied().unwrap_or(0),
        quality: v.get(2).copied().unwrap_or(0) as u16, mapnum: cur, ..map::Item::default()
    });
    println!("give: item {id}");
    st.last = id as u32;
    Ok(Flow::Done)
}

/// pack:<shape>[,<frame>[,<amount>]]  create an item in the Avatar's backpack
fn cmd_pack(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, st: &mut State, _name: &str, arg: &str, _cmd: &str) -> Result<Flow, String> {
    let v: Vec<u32> = arg.split(',').filter_map(|s| s.trim().parse().ok()).collect();
    let cur = game.cur_map as u16;
    let id = game.map_mut().add(map::Item {
        shape: v.first().copied().unwrap_or(0), frame: v.get(1).copied().unwrap_or(0),
        quality: v.get(2).copied().unwrap_or(0) as u16, mapnum: cur, parent: map::ETHEREAL, ..map::Item::default()
    });
    let ok = game.backpack().is_some_and(|pack| game.put_in_container(id, pack, 0, 0));
    println!("pack: item {id} {}", if ok { "in the backpack" } else { "does not fit" });
    st.last = id as u32;
    Ok(Flow::Done)
}

/// place:<shape>,<frame>,<x>,<y>,<z>[,<quality>]  create an item at a world position
fn cmd_place(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, st: &mut State, _name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    let num = |s: &str| number(s, cmd);
    let v: Vec<i32> = arg.split(',').map(num).collect::<Result<_, _>>()?;
    if v.len() < 5 {
        return Err(format!("'{cmd}' expects shape,frame,x,y,z[,quality]"));
    }
    let cur = game.cur_map as u16;
    let id = game.map_mut().add(map::Item {
        x: v[2], y: v[3], z: v[4], shape: v[0] as u32, frame: v[1] as u32,
        quality: v.get(5).copied().unwrap_or(0) as u16, mapnum: cur, ..map::Item::default()
    });
    println!("place: item {id}");
    st.last = id as u32;
    Ok(Flow::Done)
}

/// putin:<container>,<shape>[,<frame>[,<amount>]]  create an item in a container
fn cmd_putin(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, _cmd: &str) -> Result<Flow, String> {
    let v: Vec<u32> = arg.split(',').filter_map(|s| s.trim().parse().ok()).collect();
    let cur = game.cur_map as u16;
    let into = v.first().copied().unwrap_or(0) as u16;
    let id = game.map_mut().add(map::Item {
        shape: v.get(1).copied().unwrap_or(0), frame: v.get(2).copied().unwrap_or(0),
        quality: v.get(3).copied().unwrap_or(0) as u16, mapnum: cur, parent: map::ETHEREAL, ..map::Item::default()
    });
    let ok = game.put_in_container(id, into, 0, 0);
    println!("putin: item {id} {}", if ok { "in it" } else { "does not fit" });
    Ok(Flow::Done)
}

/// stackables  shapes that come in amounts
fn cmd_stackables(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, _name: &str, _arg: &str, _cmd: &str) -> Result<Flow, String> {
    let list: Vec<usize> = game.data.infos.iter().enumerate().filter(|(_, si)| si.family == 2 || si.family == 9).map(|(n, _)| n).collect();
    println!("stackables: {list:?}");
    Ok(Flow::Done)
}

/// potion:<1|2|3>  as the number key: drink a yellow, red, orange potion
fn cmd_potion(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    let num = |s: &str| number(s, cmd);
    println!("potion: {}", if game.quick_potion((num(arg)? - 1).max(0) as usize) { "drunk" } else { "none" });
    Ok(Flow::Done)
}

/// keyring  as the key K: use the key ring the Avatar carries
fn cmd_keyring(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, _name: &str, _arg: &str, _cmd: &str) -> Result<Flow, String> {
    println!("keyring: {}", if game.use_keyring() { "used" } else { "none" });
    Ok(Flow::Done)
}

/// wearables  every shape that can be worn, by slot (from the type flags)
fn cmd_wearables(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, _name: &str, _arg: &str, _cmd: &str) -> Result<Flow, String> {
    for (n, si) in game.data.infos.iter().enumerate().filter(|(_, si)| (1..=5).contains(&si.equip_type)) {
        println!("wearables: slot {} shape {n}", si.equip_type);
    }
    Ok(Flow::Done)
}

/// helmcycle  as F9 with --cheat: the next helmet
fn cmd_helmcycle(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, _cmd: &str) -> Result<Flow, String> {
    game.cheat_cycle_helm(arg == "back");
    println!("helmcycle: {:?}", game.equipped(AVATAR, 3).map(|w| (w.shape, w.frame)));
    Ok(Flow::Done)
}

/// armourcycle  as F8 with --cheat: the next kind of body armour
fn cmd_armourcycle(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, _cmd: &str) -> Result<Flow, String> {
    game.cheat_cycle_armour(arg == "back");
    println!("armourcycle: {:?}", game.equipped(AVATAR, 4).map(|w| (w.shape, w.frame)));
    Ok(Flow::Done)
}

/// worn  what the Avatar wears, slot by slot
fn cmd_worn(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, _name: &str, _arg: &str, _cmd: &str) -> Result<Flow, String> {
    for slot in 1..=7 {
        if let Some(it) = game.equipped(crate::game::AVATAR, slot) {
            println!("worn {slot}: {}", game.describe(&it));
        }
    }
    Ok(Flow::Done)
}

fn cmd_wear(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    let num = |s: &str| number(s, cmd);
    println!("wear: {}", game.drop_on_avatar(num(arg)? as u16));
    Ok(Flow::Done)
}

/// packmask  around the Avatar: '#' where a double click opens the backpack, '+' the rest of him
fn cmd_packmask(game: &mut Game, surf: &mut Surface, opts: &mut ViewOptions, _st: &mut State, _name: &str, _arg: &str, _cmd: &str) -> Result<Flow, String> {
    game.draw(surf, opts, false);
    let (ax, ay) = game.shown_at(AVATAR).ok_or("the Avatar is not drawn")?;
    for y in (ay - 70..ay + 6).step_by(2) {
        let line: String = (ax - 30..ax + 30)
            .map(|x| if game.backpack_at(x, y) { '#' } else if game.pick_world(x, y, 0) == Some(AVATAR) { '+' } else { '.' })
            .collect();
        println!("packmask: {line}");
    }
    Ok(Flow::Done)
}

/// sliderclick:<x>,<y>  a click on the amount slider (at a view position)
fn cmd_sliderclick(game: &mut Game, surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    let point = |s: &str| point(s, cmd);
    let (x, y) = point(arg)?;
    game.slider_click(x, y, surf.width, surf.height);
    println!("sliderclick: {}", game.ui.slider.as_ref().map_or("closed".to_string(), |s| format!("value {}", s.value)));
    Ok(Flow::Done)
}

/// asksplit:<id>  ask how much of a stack to put down in front of the Avatar
fn cmd_asksplit(game: &mut Game, surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    let num = |s: &str| number(s, cmd);
    let id = num(arg)? as u16;
    let amount = match game.map().get(id).copied() {
        Some(it) if game.stackable(&it) => it.quality as i32,
        _ => game.pile_of(id).len() as i32,
    };
    game.start_split(id, amount, surf.width / 2 + 40, surf.height / 2);
    Ok(Flow::Done)
}

/// split:<id>:<amount>:<x>,<y>  put part of a stack at a view position
fn cmd_split(game: &mut Game, surf: &mut Surface, opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    let num = |s: &str| number(s, cmd);
    let point = |s: &str| point(s, cmd);
    let parts: Vec<&str> = arg.splitn(3, ':').collect();
    if parts.len() != 3 {
        return Err(format!("'{cmd}' expects id:amount:x,y"));
    }
    let (x, y) = point(parts[2])?;
    game.draw(surf, opts, false);
    game.drop_part(num(parts[0])? as u16, num(parts[1])?, x, y);
    Ok(Flow::Done)
}

/// movegump:<x>,<y>:<x2>,<y2>  take hold of a window and move it
fn cmd_movegump(game: &mut Game, surf: &mut Surface, opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    let point = |s: &str| point(s, cmd);
    let (from, to) = arg.split_once(':').ok_or(format!("'{cmd}' expects x,y:x2,y2"))?;
    let ((x, y), (x2, y2)) = (point(from)?, point(to)?);
    game.draw(surf, opts, false);
    match game.grab_gump(x, y) {
        Some(held) => println!("movegump: window of {} moved: {}", held.0, game.move_gump(held, x2, y2, (surf.width, surf.height))),
        None => println!("movegump: no window to hold there"),
    }
    Ok(Flow::Done)
}

/// take:<id>  as a right click (shift + right click): straight into the backpack
fn cmd_take(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, st: &mut State, _name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    let num = |s: &str| number(s, cmd);
    let id = if arg.is_empty() { st.last as u16 } else { num(arg)? as u16 };
    println!("take: {}", match game.take_into_backpack(id) {
        Some(true) => "taken",
        Some(false) => "refused",
        None => "not takeable",
    });
    Ok(Flow::Done)
}

fn cmd_inv(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, _name: &str, _arg: &str, _cmd: &str) -> Result<Flow, String> {
    game.open_inventory();
    Ok(Flow::Done)
}

/// dragpage:<id>:<pages>:<x>,<y>  take an item, turn the backpack's page (as when held over an arrow), drop it
fn cmd_dragpage(game: &mut Game, surf: &mut Surface, opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    let num = |s: &str| number(s, cmd);
    let point = |s: &str| point(s, cmd);
    let v: Vec<&str> = arg.splitn(3, ':').collect();
    let (id, by, (x, y)) = (num(v.first().ok_or("dragpage:<id>:<pages>:<x>,<y>")?)? as u16, num(v.get(1).ok_or("pages")?)?, point(v.get(2).ok_or("x,y")?)?);
    game.draw(surf, opts, false);
    let ok = game.begin_drag(id, x, y);
    if let (true, Some(pack)) = (ok, game.backpack()) {
        game.grid_turn(pack, by);
        println!("dragpage: page {}", game.grid_page(pack) + 1);
    }
    let ok = ok && game.drop_drag(x, y);
    game.ui.drag = None;
    println!("dragpage: {}", if ok { "moved" } else { "not possible" });
    Ok(Flow::Done)
}

/// piles  the backpack's slots with what lies on them (several: a pile)
fn cmd_piles(game: &mut Game, _surf: &mut Surface, _opts: &mut ViewOptions, _st: &mut State, _name: &str, _arg: &str, _cmd: &str) -> Result<Flow, String> {
    let pack = game.backpack().ok_or("no backpack")?;
    for (slot, first) in game.grid_layout(pack) {
        let pile = game.pile_of(first);
        let it = game.map().get(first).copied().unwrap_or_default();
        let fl: Vec<String> = pile.iter().filter_map(|&i| game.map().get(i)).map(|i| format!("{:#x}/{}", i.flags, i.x & 0xffff)).collect();
        println!("slot {slot}: shape {} frame {} q {} x{} {:?} {:?}", it.shape, it.frame, it.quality, pile.len(), pile, fl);
    }
    Ok(Flow::Done)
}

/// dropsplit:<id>:<x>,<y>  take an item and let go of it without shift (a pile asks how many)
fn cmd_dropsplit(game: &mut Game, surf: &mut Surface, opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    let num = |s: &str| number(s, cmd);
    let point = |s: &str| point(s, cmd);
    let (id, to) = arg.split_once(':').ok_or(format!("'{cmd}' expects id:x,y"))?;
    let (x, y) = point(to)?;
    game.draw(surf, opts, false);
    let ok = game.begin_drag(num(id)? as u16, x, y) && game.drop_drag_split(x, y);
    game.ui.drag = None;
    println!("{cmd}: {} {}", if ok { "ok" } else { "not possible" }, game.ui.slider.as_ref().map_or("".to_string(), |s| format!("asks, value {}", s.value)));
    Ok(Flow::Done)
}

/// gridclick:<x>,<y>  a click on a page arrow of a slot window
fn cmd_gridclick(game: &mut Game, surf: &mut Surface, opts: &mut ViewOptions, _st: &mut State, _name: &str, arg: &str, cmd: &str) -> Result<Flow, String> {
    let point = |s: &str| point(s, cmd);
    let (x, y) = point(arg)?;
    game.draw(surf, opts, false);
    println!("gridclick: {}", if game.grid_button(x, y) { "page arrow" } else { "no arrow there" });
    Ok(Flow::Done)
}
