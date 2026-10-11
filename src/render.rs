//! Sunset beach and the three fighters. The beach and the poses are
//! photographs compiled into the binary. The scoreboard stays drawn in code.

use crate::sim::{
    fighter_name, jacket, spec, Action, Fighter, Game, Mode, MoveId, SCREEN_H, SCREEN_W, GROUND,
};

const W: i32 = SCREEN_W;
const H: i32 = SCREEN_H;

const IDLE: usize = 0;
const WALK_A: usize = 1;
const WALK_B: usize = 2;
const BLOCK: usize = 3;
const PUNCH: usize = 4;
const CROUCH: usize = 5;
const SWEEP: usize = 6;
const KICK: usize = 7;
const HIGH: usize = 8;
const FLYING: usize = 9;
const HEAD: usize = 10;
const SPLITS: usize = 11;
const FLIP: usize = 12;
const DOWN: usize = 13;
const JUMP: usize = 14;

pub struct Frame {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u32>,
}

impl Frame {
    pub fn new() -> Self {
        Self { w: W, h: H, px: vec![0; (W * H) as usize] }
    }
}

struct Sprite {
    w: i32,
    h: i32,
    /// Body center, and the lowest opaque row. That point sits on the feet.
    ax: i32,
    ay: i32,
    /// Extra lift for poses that are already off the ground in the photo.
    lift: i32,
    /// White, red, and blue gi. Each pixel is 0xAARRGGBB.
    px: [Vec<u32>; 3],
}

pub struct Renderer {
    beach: Vec<u32>,
    bg: Vec<u32>,
    key: i32,
    sprites: Vec<Sprite>,
}

impl Renderer {
    pub fn new() -> Self {
        let beach = decode_beach();
        let mut bg = beach.clone();
        grade_beach(&beach, &mut bg, 2);
        Self {
            beach,
            bg,
            key: 2,
            sprites: load_sprites(),
        }
    }

    pub fn draw(&mut self, game: &Game, frame: &mut Frame) {
        let key = game.sky_round();
        if self.key != key {
            grade_beach(&self.beach, &mut self.bg, key);
            self.key = key;
        }
        frame.px.copy_from_slice(&self.bg);
        let mag = game.shake;
        let ox = (mag * (game.time * 47.0).sin()) as i32;
        let oy = (mag * 0.4 * (game.time * 63.0).cos()) as i32;
        let show_bonus = game.mode == Mode::Bonus || (game.mode == Mode::Pause && game.paused_mode == Mode::Bonus);
        if show_bonus {
            for ball in &game.bonus.balls {
                if ball.alive {
                    ball_shadow(frame, ball.x + ox as f32, ball.y, ox, oy);
                }
            }
        }
        let mut order = [0usize, 1, 2];
        order.sort_by(|&a, &b| {
            game.fighters[a].x.partial_cmp(&game.fighters[b].x).unwrap_or(std::cmp::Ordering::Equal)
        });
        for &i in &order {
            shadow(frame, &game.fighters[i], ox, oy);
        }
        for &i in &order {
            draw_fighter(frame, &self.sprites, &game.fighters[i], game, ox, oy);
        }
        if show_bonus {
            draw_shield(frame, &game.fighters[game.bonus.player], ox, oy);
            for ball in &game.bonus.balls {
                if ball.alive {
                    draw_ball(frame, ball.x + ox as f32, ball.y + oy as f32);
                }
            }
        }
        for p in &game.particles {
            let a = (p.life / p.max).clamp(0.0, 1.0);
            blob(frame, p.x + ox as f32, p.y + oy as f32, p.r * (0.5 + a), p.color, a);
        }
        for f in &game.floaters {
            let tw = text_width(&f.text, f.scale);
            text(frame, (f.x as i32) - tw / 2 + 1, f.y as i32 + 1, &f.text, 0x1A120C, f.scale);
            text(frame, (f.x as i32) - tw / 2, f.y as i32, &f.text, f.color, f.scale);
        }
        draw_ui(frame, game);
    }
}

fn decode_beach() -> Vec<u32> {
    let img = image::load_from_memory(include_bytes!("../assets/beach.jpg"))
        .expect("beach photograph")
        .to_rgb8();
    assert_eq!(img.width() as i32, W, "beach width");
    assert_eq!(img.height() as i32, H, "beach height");
    img.pixels()
        .map(|p| ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | (p[2] as u32))
        .collect()
}

fn grade_beach(src: &[u32], dst: &mut [u32], round: i32) {
    let t = ((round.max(1) - 1) as f32 / 7.0).clamp(0.0, 1.0);
    let shade = 1.04 - t * 0.38;
    for (s, d) in src.iter().zip(dst.iter_mut()) {
        let r = ((((*s >> 16) & 255) as f32) * shade + t * 16.0).clamp(0.0, 255.0) as u32;
        let g = ((((*s >> 8) & 255) as f32) * (shade - t * 0.05)).clamp(0.0, 255.0) as u32;
        let b = (((*s & 255) as f32) * shade - t * 20.0).clamp(0.0, 255.0) as u32;
        *d = (r << 16) | (g << 8) | b;
    }
}

struct Sheet {
    bytes: &'static [u8],
    w: i32,
    h: i32,
    ax: i32,
    ay: i32,
    lift: i32,
}

fn load_sprites() -> Vec<Sprite> {
    let sheets = [
        Sheet { bytes: include_bytes!("../assets/idle.png"), w: 166, h: 318, ax: 83, ay: 316, lift: 0 },
        Sheet { bytes: include_bytes!("../assets/walk_a.png"), w: 194, h: 313, ax: 93, ay: 311, lift: 0 },
        Sheet { bytes: include_bytes!("../assets/walk_b.png"), w: 213, h: 297, ax: 72, ay: 295, lift: 0 },
        Sheet { bytes: include_bytes!("../assets/block.png"), w: 182, h: 326, ax: 90, ay: 324, lift: 0 },
        Sheet { bytes: include_bytes!("../assets/punch.png"), w: 201, h: 328, ax: 89, ay: 326, lift: 0 },
        Sheet { bytes: include_bytes!("../assets/crouch.png"), w: 233, h: 343, ax: 85, ay: 341, lift: 0 },
        Sheet { bytes: include_bytes!("../assets/sweep.png"), w: 209, h: 292, ax: 82, ay: 290, lift: 0 },
        Sheet { bytes: include_bytes!("../assets/kick.png"), w: 136, h: 342, ax: 71, ay: 340, lift: 0 },
        Sheet { bytes: include_bytes!("../assets/high.png"), w: 234, h: 329, ax: 94, ay: 327, lift: 0 },
        Sheet { bytes: include_bytes!("../assets/flying.png"), w: 261, h: 234, ax: 95, ay: 232, lift: 40 },
        Sheet { bytes: include_bytes!("../assets/head.png"), w: 208, h: 293, ax: 123, ay: 291, lift: 0 },
        Sheet { bytes: include_bytes!("../assets/splits.png"), w: 269, h: 193, ax: 133, ay: 191, lift: 54 },
        Sheet { bytes: include_bytes!("../assets/flip.png"), w: 127, h: 185, ax: 66, ay: 183, lift: 20 },
        Sheet { bytes: include_bytes!("../assets/down.png"), w: 231, h: 101, ax: 110, ay: 99, lift: 0 },
        Sheet { bytes: include_bytes!("../assets/jump.png"), w: 145, h: 302, ax: 68, ay: 300, lift: 0 },
    ];
    sheets.iter().map(decode_sprite).collect()
}

fn decode_sprite(sheet: &Sheet) -> Sprite {
    let img = image::load_from_memory(sheet.bytes).expect("fighter sprite").to_rgba8();
    assert_eq!(img.width() as i32, sheet.w, "sprite width");
    assert_eq!(img.height() as i32, sheet.h, "sprite height");
    let raw = img.into_raw();
    let n = (sheet.w * sheet.h) as usize;
    let mut white = Vec::with_capacity(n);
    let mut red = Vec::with_capacity(n);
    let mut blue = Vec::with_capacity(n);
    for px in raw.chunks_exact(4) {
        let (r, g, b, a) = (px[0], px[1], px[2], px[3]);
        white.push(pack(r, g, b, a));
        let (rr, rg, rb) = tint_gi(r, g, b, a, 0xC4, 0x2B, 0x34);
        red.push(pack(rr, rg, rb, a));
        let (br, bg, bb) = tint_gi(r, g, b, a, 0x2A, 0x4C, 0x86);
        blue.push(pack(br, bg, bb, a));
    }
    Sprite {
        w: sheet.w,
        h: sheet.h,
        ax: sheet.ax,
        ay: sheet.ay,
        lift: sheet.lift,
        px: [white, red, blue],
    }
}

fn pack(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Recolor the white gi. Skin, hair, and the face stay as photographed.
fn tint_gi(r: u8, g: u8, b: u8, a: u8, tr: u8, tg: u8, tb: u8) -> (u8, u8, u8) {
    if a < 16 {
        return (r, g, b);
    }
    let rf = r as f32;
    let gf = g as f32;
    let bf = b as f32;
    let mx = rf.max(gf).max(bf);
    let mn = rf.min(gf).min(bf);
    let sat = if mx <= 0.0 { 0.0 } else { (mx - mn) / mx };
    let skin = rf > gf + 12.0 && rf > bf + 12.0 && sat > 0.10;
    if skin || sat >= 0.42 || mx <= 60.0 {
        return (r, g, b);
    }
    let lum = (0.30 * rf + 0.59 * gf + 0.11 * bf) / 255.0;
    let k = (lum / 0.90).clamp(0.0, 1.35);
    let nr = (tr as f32 * k).clamp(0.0, 255.0);
    let ng = (tg as f32 * k).clamp(0.0, 255.0);
    let nb = (tb as f32 * k).clamp(0.0, 255.0);
    let strength = ((0.42 - sat) / 0.42).clamp(0.0, 1.0) * ((mx - 60.0) / 50.0).clamp(0.0, 1.0);
    let mixc = |src: f32, dst: f32| (src + (dst - src) * strength) as u8;
    (mixc(rf, nr), mixc(gf, ng), mixc(bf, nb))
}

fn pose_index(f: &Fighter, game: &Game) -> usize {
    if game.bow() > 0.45 {
        return HEAD;
    }
    match f.action {
        Action::Idle => IDLE,
        Action::Walk => {
            if f.phase.sin() >= 0.0 {
                WALK_A
            } else {
                WALK_B
            }
        }
        Action::Block => BLOCK,
        Action::Jump => JUMP,
        Action::Backflip => FLIP,
        Action::Fall | Action::Down => DOWN,
        Action::Rise => {
            if f.t < 0.18 {
                DOWN
            } else {
                CROUCH
            }
        }
        Action::Attack(id) => {
            let u = (f.t / spec(id).dur).clamp(0.0, 1.0);
            if !(0.18..0.80).contains(&u) {
                IDLE
            } else {
                strike_index(id)
            }
        }
    }
}

fn strike_index(id: MoveId) -> usize {
    match id {
        MoveId::FrontPunch | MoveId::ReversePunch => PUNCH,
        MoveId::ShinKick | MoveId::StomachKick => KICK,
        MoveId::Sweep | MoveId::ReverseSweep => SWEEP,
        MoveId::CrouchPunch => CROUCH,
        MoveId::FlyingKick => FLYING,
        MoveId::Headbutt => HEAD,
        MoveId::FaceKick | MoveId::BackStepKick => HIGH,
        MoveId::DoubleKick => SPLITS,
    }
}

fn shadow(frame: &mut Frame, f: &Fighter, ox: i32, oy: i32) {
    let air = (-f.y).max(0.0);
    let shrink = 1.0 - (air / 220.0).min(0.6);
    let alpha = 0.32 * shrink;
    ellipse(
        frame,
        f.x + ox as f32,
        GROUND + oy as f32 + 4.0,
        42.0 * shrink,
        11.0 * shrink,
        0x1A120C,
        alpha,
    );
}

fn draw_fighter(frame: &mut Frame, sprites: &[Sprite], f: &Fighter, game: &Game, ox: i32, oy: i32) {
    let sprite = &sprites[pose_index(f, game)];
    let flip = f.facing < 0.0;
    let ax = if flip { sprite.w - 1 - sprite.ax } else { sprite.ax };
    let bob = if matches!(f.action, Action::Idle) {
        ((game.time * 2.2 + f.phase).sin() * 1.4) as i32
    } else {
        0
    };
    let left = f.x as i32 + ox - ax;
    let top = (GROUND + f.y) as i32 + oy - sprite.ay - sprite.lift + bob;
    let gi = f.index.min(2);
    let flash = (f.flash * 4.0).clamp(0.0, 0.65);
    blit(frame, sprite, gi, left, top, flip, flash);
    if matches!(f.action, Action::Down) {
        for k in 0..3 {
            let a = game.time * 6.0 + k as f32 * 2.09;
            disc(
                frame,
                f.x + ox as f32 + a.cos() * 22.0,
                GROUND + f.y + oy as f32 - 78.0 + a.sin() * 6.0,
                3.2,
                0xFFE08A,
            );
        }
    }
}

fn blit(frame: &mut Frame, sprite: &Sprite, gi: usize, left: i32, top: i32, flip: bool, flash: f32) {
    let src = &sprite.px[gi];
    let sw = sprite.w;
    let sh = sprite.h;
    for y in 0..sh {
        let dy = top + y;
        if dy < 0 || dy >= H {
            continue;
        }
        let row = (dy * W) as usize;
        for x in 0..sw {
            let dx = left + x;
            if dx < 0 || dx >= W {
                continue;
            }
            let sx = if flip { sw - 1 - x } else { x };
            let p = src[(y * sw + sx) as usize];
            let a = p >> 24;
            if a < 8 {
                continue;
            }
            let mut r = (p >> 16) & 255;
            let mut g = (p >> 8) & 255;
            let mut b = p & 255;
            if flash > 0.0 {
                r = (r as f32 + (255.0 - r as f32) * flash) as u32;
                g = (g as f32 + (255.0 - g as f32) * flash) as u32;
                b = (b as f32 + (255.0 - b as f32) * flash) as u32;
            }
            blend_px(&mut frame.px[row + dx as usize], r, g, b, a);
        }
    }
}

fn blend_px(dst: &mut u32, r: u32, g: u32, b: u32, a: u32) {
    if a >= 250 {
        *dst = (r << 16) | (g << 8) | b;
        return;
    }
    let inv = 255 - a;
    let d = *dst;
    let nr = (r * a + ((d >> 16) & 255) * inv) / 255;
    let ng = (g * a + ((d >> 8) & 255) * inv) / 255;
    let nb = (b * a + (d & 255) * inv) / 255;
    *dst = (nr << 16) | (ng << 8) | nb;
}

fn draw_shield(frame: &mut Frame, f: &Fighter, ox: i32, oy: i32) {
    let x = f.x + ox as f32;
    let y = GROUND - 150.0 + oy as f32;
    capsule(frame, x - 62.0, y, x + 62.0, y, 16.0, 0x3A2414);
    capsule(frame, x - 62.0, y, x + 62.0, y, 13.0, 0xB4322E);
    capsule(frame, x - 62.0, y - 2.0, x + 62.0, y - 2.0, 4.0, 0xF0D78A);
}

fn draw_ball(frame: &mut Frame, x: f32, y: f32) {
    disc(frame, x, y, 15.0, 0x2A120E);
    disc(frame, x, y, 13.2, 0xF4F7FB);
    disc(frame, x - 3.0, y - 1.0, 12.0, 0xD5DEE8);
    blob(frame, x - 4.0, y - 5.0, 4.0, 0xFFFFFF, 0.9);
}

fn ball_shadow(frame: &mut Frame, x: f32, y: f32, ox: i32, oy: i32) {
    let h = (GROUND - y).max(0.0);
    let r = 15.0 * (1.0 - (h / 420.0).min(0.65));
    ellipse(frame, x, GROUND + oy as f32 - 2.0, r, r * 0.35, 0x1A120C, 0.35);
    let _ = ox;
}

fn draw_ui(frame: &mut Frame, game: &Game) {
    match game.mode {
        Mode::Title => draw_title(frame, game),
        Mode::Intro => {
            draw_hud_scores(frame, game);
            card(frame, 390, 86, 500, 118);
            let r = format!("ROUND {}", game.round);
            center(frame, 145, &r, 0xF6E2A8, 4);
            center(frame, 188, &game.belt, 0xFFF8EC, 2);
        }
        Mode::Fight | Mode::Pause => {
            if game.mode == Mode::Pause && game.paused_mode == Mode::Bonus {
                draw_bonus_ui(frame, game);
            } else if game.mode != Mode::Pause || game.paused_mode != Mode::Title {
                draw_hud_scores(frame, game);
                if game.round == 1 && game.mode == Mode::Fight && game.timer > 24.0 {
                    center(frame, 690, "ARROWS MOVE    SPACE ATTACK    F1 MOVES", 0x2A1C14, 2);
                }
            }
            if game.mode == Mode::Pause {
                shade(frame, 0, 0, W, H, 0x07060A, 0.45);
                center(frame, 330, "PAUSED", 0xFFF8EC, 5);
            }
        }
        Mode::Resolve => {
            draw_hud_scores(frame, game);
            card(frame, 360, 150, 560, 300);
            center_at(frame, 640, 172, "ROUND OVER", 0xF6E2A8, 3);
            for (place, &i) in game.ranks.iter().enumerate() {
                let y = 230 + place as i32 * 52;
                let name = fighter_name(i);
                let line = format!("{}   {}    {} PIPS    {}", place + 1, name, game.fighters[i].pips, game.fighters[i].round_score);
                text(frame, 400, y, &line, if place == 2 { 0xE7A0A0 } else { 0xFFF8EC }, 2);
            }
            if game.time_bonus > 0 {
                let b = format!("TIME BONUS  {}", game.time_bonus);
                center_at(frame, 640, 400, &b, 0xFFE08A, 2);
            }
            center_at(frame, 640, 424, &game.note, 0xF6E2A8, 2);
        }
        Mode::Bonus => draw_bonus_ui(frame, game),
        Mode::GameOver => {
            shade(frame, 0, 0, W, H, 0x07060A, 0.35);
            card(frame, 340, 180, 600, 280);
            center_at(frame, 640, 210, "ELIMINATED", 0xF0C0B0, 4);
            center_at(frame, 640, 270, &game.note, 0xFFF8EC, 2);
            let mut y = 320;
            for f in &game.fighters {
                if f.human {
                    let line = format!("{}   {}", fighter_name(f.index), f.total_score);
                    center_at(frame, 640, y, &line, 0xFFE08A, 3);
                    y += 48;
                }
            }
            center_at(frame, 640, 420, "ENTER  REMATCH", 0xF6E2A8, 2);
        }
    }
    if game.help {
        draw_help(frame);
    }
}

fn draw_bonus_ui(frame: &mut Frame, game: &Game) {
    draw_hud_scores(frame, game);
    center(frame, 96, "BONUS", 0xF6E2A8, 3);
    let line = format!("CAUGHT {}    MISSED {}", game.bonus.deflected, game.bonus.missed);
    center(frame, 132, &line, 0xFFF8EC, 2);
    if !game.note.is_empty() {
        center(frame, 168, &game.note, 0xFFE08A, 2);
    }
}

fn draw_hud_scores(frame: &mut Frame, game: &Game) {
    for i in 0..3 {
        let x = 36 + i as i32 * 430;
        let (gi, _, _, _) = jacket(i);
        disc(frame, x as f32 + 8.0, 30.0, 9.0, 0x1A120C);
        disc(frame, x as f32 + 8.0, 30.0, 7.0, gi);
        let mut label = fighter_name(i).to_string();
        if game.fighters[i].human {
            if game.players > 1 {
                label = format!("{label}  P{}", i + 1);
            } else {
                label = format!("{label}  YOU");
            }
        }
        text(frame, x + 24, 20, &label, 0x1A120C, 2);
        text(frame, x + 22, 18, &label, 0xFFF8EC, 2);
        for k in 0..6 {
            let cx = x as f32 + 10.0 + k as f32 * 22.0;
            disc(frame, cx, 52.0, 8.0, 0x1A120C);
            if k < game.fighters[i].pips {
                disc(frame, cx, 52.0, 6.2, if k == 5 { 0xFFE08A } else { gi });
            } else {
                disc(frame, cx, 52.0, 6.2, 0x2A241C);
            }
        }
        let score = format!("{}", game.fighters[i].total_score);
        text(frame, x + 1, 67, &score, 0x1A120C, 2);
        text(frame, x, 66, &score, 0xFFE08A, 2);
    }
    let secs = game.timer.ceil().clamp(0.0, 30.0) as i32;
    shade(frame, 590, 8, 100, 62, 0x140E0C, 0.45);
    draw_digit(frame, 604, 16, secs / 10, 0xFFF8EC);
    draw_digit(frame, 642, 16, secs % 10, 0xFFF8EC);
    let belt = if game.belt.len() > 22 { "BLACK BELT".to_string() } else { game.belt.clone() };
    center(frame, 78, &belt, 0x1A120C, 2);
    center(frame, 76, &belt, 0xFFF8EC, 2);
    let sp = format!("SPEED {}", game.speed);
    text(frame, 1150, 692, &sp, 0x3A2A20, 2);
}

fn draw_title(frame: &mut Frame, game: &Game) {
    draw_logo(frame, 640.0, 132.0);
    center(frame, 210, "INTERNATIONAL KARATE", 0x2A1C14, 2);
    center(frame, 208, "INTERNATIONAL KARATE", 0xFFF6E4, 2);
    let items = ["ONE FIGHTER", "TWO FIGHTERS"];
    for (i, item) in items.iter().enumerate() {
        let y = 250 + i as i32 * 36;
        let on = game.menu == i as i32;
        let label = if on { format!(">  {item}") } else { format!("   {item}") };
        let col = if on { 0xF6E2A8 } else { 0xF4E6D4 };
        center(frame, y, &label, 0x2A1C14, 3);
        center(frame, y - 1, &label, col, 3);
    }
    center(frame, 676, "ENTER START     UP OR DOWN TO CHOOSE", 0x2A1C14, 2);
    center(frame, 674, "ENTER START     UP OR DOWN TO CHOOSE", 0xFFF6E4, 2);
    center(frame, 702, "P1  ARROWS + SPACE      P2  WASD + F      F1 MOVES", 0x2A1C14, 2);
    center(frame, 700, "P1  ARROWS + SPACE      P2  WASD + F      F1 MOVES", 0xFFF6E4, 2);
}

fn draw_logo(frame: &mut Frame, cx: f32, cy: f32) {
    // Shadow, gold, then a thin core.
    logo_strokes(frame, cx + 5.0, cy + 7.0, 12.0, 0x4A2E16);
    logo_strokes(frame, cx, cy, 11.0, 0xF3D48A);
    logo_strokes(frame, cx - 1.0, cy - 2.0, 4.5, 0xFFF9EC);
}

fn logo_strokes(frame: &mut Frame, cx: f32, cy: f32, r: f32, color: u32) {
    capsule(frame, cx - 168.0, cy - 52.0, cx - 168.0, cy + 52.0, r, color);
    let kx = cx - 78.0;
    capsule(frame, kx, cy - 52.0, kx, cy + 52.0, r, color);
    capsule(frame, kx, cy, kx + 86.0, cy - 54.0, r, color);
    capsule(frame, kx, cy, kx + 86.0, cy + 54.0, r, color);
    let px = cx + 150.0;
    capsule(frame, px, cy - 34.0, px, cy + 34.0, r, color);
    capsule(frame, px - 34.0, cy, px + 34.0, cy, r, color);
}

fn draw_help(frame: &mut Frame) {
    shade(frame, 0, 0, W, H, 0x07060A, 0.78);
    card(frame, 70, 36, 1140, 650);
    center_at(frame, 640, 52, "MOVES", 0xF6E2A8, 3);
    center_at(frame, 640, 92, "DIRECTIONS ARE RELATIVE TO THE WAY YOU FACE", 0xF4E6D4, 2);
    let left = [
        "NO ATTACK BUTTON",
        "UP            JUMP",
        "UP FORWARD    FRONT PUNCH",
        "FORWARD       WALK",
        "DOWN FORWARD  SHIN KICK",
        "DOWN          FOOT SWEEP",
        "DOWN BACK     STOMACH PUNCH",
        "BACK          BLOCK",
        "UP BACK       REVERSE PUNCH",
    ];
    let right = [
        "HOLD SPACE",
        "UP            FLYING KICK",
        "UP FORWARD    HEADBUTT",
        "FORWARD       STOMACH KICK",
        "DOWN FORWARD  FACE KICK",
        "DOWN          REVERSE SWEEP",
        "DOWN BACK     BACK KICK",
        "BACK          BACKFLIP",
        "UP BACK       DOUBLE KICK",
    ];
    for (i, line) in left.iter().enumerate() {
        text(frame, 110, 140 + i as i32 * 32, line, 0xFFF8EC, 2);
    }
    for (i, line) in right.iter().enumerate() {
        text(frame, 680, 140 + i as i32 * 32, line, 0xFFE08A, 2);
    }
    center_at(frame, 640, 460, "FRONT HIT IS 2 PIPS AND FULL SCORE. A HIT IN THE BACK IS HALF.", 0xF4E6D4, 2);
    center_at(frame, 640, 492, "FIRST TO 6, OR THE LEADER AT 30 SECONDS. LAST PLACE IS OUT.", 0xF4E6D4, 2);
    center_at(frame, 640, 524, "THE COMPUTERS FIGHT EACH OTHER. EVERY TWO ROUNDS, STOP THE BALLS.", 0xF4E6D4, 2);
    center_at(frame, 640, 580, "P PAUSE    M MUTE    - AND = SPEED    F1 CLOSE", 0xF6E2A8, 2);
}

fn card(frame: &mut Frame, x: i32, y: i32, w: i32, h: i32) {
    shade(frame, x, y, w, h, 0x140E0C, 0.78);
    // Gold hairline.
    shade(frame, x, y, w, 2, 0xF3D48A, 0.9);
}

fn center(frame: &mut Frame, y: i32, s: &str, color: u32, scale: i32) {
    center_at(frame, W / 2, y, s, color, scale);
}

fn center_at(frame: &mut Frame, cx: i32, y: i32, s: &str, color: u32, scale: i32) {
    let tw = text_width(s, scale);
    text(frame, cx - tw / 2, y, s, color, scale);
}

fn draw_digit(frame: &mut Frame, x: i32, y: i32, d: i32, color: u32) {
    // Segments: 0 top, 1 ul, 2 ur, 3 mid, 4 ll, 5 lr, 6 bot.
    const ON: [u8; 10] = [
        0b1110111, 0b0100100, 0b1011101, 0b1101101, 0b0101110, 0b1101011, 0b1111011, 0b0100101, 0b1111111, 0b1101111,
    ];
    let mask = ON[d.clamp(0, 9) as usize];
    let w = 28.0;
    let h = 22.0;
    let x = x as f32;
    let y = y as f32;
    let mut seg = |bit: u8, x0, y0, x1, y1| {
        if mask & (1 << bit) != 0 {
            capsule(frame, x0, y0, x1, y1, 2.6, color);
        }
    };
    seg(0, x + 3.0, y, x + w - 3.0, y);
    seg(1, x, y + 3.0, x, y + h - 2.0);
    seg(2, x + w, y + 3.0, x + w, y + h - 2.0);
    seg(3, x + 3.0, y + h, x + w - 3.0, y + h);
    seg(4, x, y + h + 2.0, x, y + h * 2.0 - 3.0);
    seg(5, x + w, y + h + 2.0, x + w, y + h * 2.0 - 3.0);
    seg(6, x + 3.0, y + h * 2.0, x + w - 3.0, y + h * 2.0);
}

fn text_width(s: &str, scale: i32) -> i32 {
    s.chars().count() as i32 * 6 * scale
}

fn text(frame: &mut Frame, mut x: i32, y: i32, s: &str, color: u32, scale: i32) {
    for ch in s.chars() {
        if let Some(rows) = glyph(ch) {
            for (row, bits) in rows.iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) != 0 {
                        let px = x + col * scale;
                        let py = y + row as i32 * scale;
                        fill_rect(frame, px, py, scale, scale, color);
                    }
                }
            }
        }
        x += 6 * scale;
    }
}

fn glyph(c: char) -> Option<[u8; 7]> {
    let c = if c.is_ascii_lowercase() { c.to_ascii_uppercase() } else { c };
    Some(match c {
        ' ' => [0, 0, 0, 0, 0, 0, 0],
        'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        'D' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E],
        'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0E],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x1F],
        'J' => [0x01, 0x01, 0x01, 0x01, 0x11, 0x11, 0x0E],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'M' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'Q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0A],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x06, 0x08, 0x10, 0x1F],
        '3' => [0x1E, 0x01, 0x01, 0x0E, 0x01, 0x01, 0x1E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        '6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C],
        '+' => [0x00, 0x04, 0x04, 0x1F, 0x04, 0x04, 0x00],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        ':' => [0x00, 0x0C, 0x0C, 0x00, 0x0C, 0x0C, 0x00],
        '\'' => [0x04, 0x04, 0x08, 0x00, 0x00, 0x00, 0x00],
        '!' => [0x04, 0x04, 0x04, 0x04, 0x04, 0x00, 0x04],
        '?' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x00, 0x04],
        '/' => [0x01, 0x02, 0x04, 0x04, 0x08, 0x10, 0x10],
        '>' => [0x10, 0x08, 0x04, 0x02, 0x04, 0x08, 0x10],
        _ => return None,
    })
}

fn shade(frame: &mut Frame, x: i32, y: i32, w: i32, h: i32, color: u32, a: f32) {
    let x1 = (x + w).min(W);
    let y1 = (y + h).min(H);
    for yy in y.max(0)..y1 {
        let row = (yy * W) as usize;
        for xx in x.max(0)..x1 {
            blend(&mut frame.px[row + xx as usize], color, a);
        }
    }
}

fn fill_rect(frame: &mut Frame, x: i32, y: i32, w: i32, h: i32, color: u32) {
    let x1 = (x + w).min(W);
    let y1 = (y + h).min(H);
    for yy in y.max(0)..y1 {
        let row = (yy * W) as usize;
        for xx in x.max(0)..x1 {
            frame.px[row + xx as usize] = color;
        }
    }
}

fn disc(frame: &mut Frame, cx: f32, cy: f32, r: f32, color: u32) {
    if r <= 0.2 {
        return;
    }
    let y0 = (cy - r).floor() as i32;
    let y1 = (cy + r).ceil() as i32;
    for y in y0.max(0)..=y1.min(H - 1) {
        let dy = y as f32 + 0.5 - cy;
        let t = r * r - dy * dy;
        if t <= 0.0 {
            continue;
        }
        let h = t.sqrt();
        hspan(frame, y, cx - h, cx + h, color);
    }
}

fn blob(frame: &mut Frame, cx: f32, cy: f32, r: f32, color: u32, alpha: f32) {
    if r <= 0.2 || alpha <= 0.0 {
        return;
    }
    let y0 = (cy - r).floor() as i32;
    let y1 = (cy + r).ceil() as i32;
    for y in y0.max(0)..=y1.min(H - 1) {
        let dy = y as f32 + 0.5 - cy;
        let row = (y * W) as usize;
        let x0 = (cx - r).floor() as i32;
        let x1 = (cx + r).ceil() as i32;
        for x in x0.max(0)..=x1.min(W - 1) {
            let dx = x as f32 + 0.5 - cx;
            let d = (dx * dx + dy * dy).sqrt();
            let a = ((r - d) / 1.2).clamp(0.0, 1.0) * alpha;
            if a > 0.0 {
                blend(&mut frame.px[row + x as usize], color, a);
            }
        }
    }
}

fn ellipse(frame: &mut Frame, cx: f32, cy: f32, rx: f32, ry: f32, color: u32, alpha: f32) {
    if rx <= 0.2 || ry <= 0.2 {
        return;
    }
    let y0 = (cy - ry).floor() as i32;
    let y1 = (cy + ry).ceil() as i32;
    for y in y0.max(0)..=y1.min(H - 1) {
        let dy = (y as f32 + 0.5 - cy) / ry;
        if dy * dy >= 1.0 {
            continue;
        }
        let hx = rx * (1.0 - dy * dy).sqrt();
        hspan_a(frame, y, cx - hx, cx + hx, color, alpha);
    }
}

fn capsule(frame: &mut Frame, x0: f32, y0: f32, x1: f32, y1: f32, r: f32, color: u32) {
    if r <= 0.2 {
        return;
    }
    let minx = (x0.min(x1) - r).floor() as i32;
    let maxx = (x0.max(x1) + r).ceil() as i32;
    let miny = (y0.min(y1) - r).floor() as i32;
    let maxy = (y0.max(y1) + r).ceil() as i32;
    for y in miny.max(0)..=maxy.min(H - 1) {
        let row = (y * W) as usize;
        let py = y as f32 + 0.5;
        for x in minx.max(0)..=maxx.min(W - 1) {
            let d = dist_seg(x as f32 + 0.5, py, x0, y0, x1, y1);
            if d <= r - 0.8 {
                frame.px[row + x as usize] = color;
            } else if d < r + 0.6 {
                let a = (r + 0.6 - d).clamp(0.0, 1.0);
                blend(&mut frame.px[row + x as usize], color, a);
            }
        }
    }
}

fn hspan(frame: &mut Frame, y: i32, x0: f32, x1: f32, color: u32) {
    if y < 0 || y >= H || x1 < x0 {
        return;
    }
    let row = (y * W) as usize;
    let start = x0.floor() as i32;
    let end = x1.ceil() as i32;
    for x in start..=end {
        if x < 0 || x >= W {
            continue;
        }
        let px = x as f32 + 0.5;
        let out = (x0 - px).max(px - x1).max(0.0);
        let a = (1.0 - out).clamp(0.0, 1.0);
        if a >= 0.99 {
            frame.px[row + x as usize] = color;
        } else if a > 0.0 {
            blend(&mut frame.px[row + x as usize], color, a);
        }
    }
}

fn hspan_a(frame: &mut Frame, y: i32, x0: f32, x1: f32, color: u32, alpha: f32) {
    if y < 0 || y >= H || !(alpha > 0.0) {
        return;
    }
    let row = (y * W) as usize;
    let start = x0.floor() as i32;
    let end = x1.ceil() as i32;
    for x in start..=end {
        if x < 0 || x >= W {
            continue;
        }
        let px = x as f32 + 0.5;
        let out = (x0 - px).max(px - x1).max(0.0);
        let a = (1.0 - out).clamp(0.0, 1.0) * alpha;
        if a > 0.0 {
            blend(&mut frame.px[row + x as usize], color, a);
        }
    }
}

fn dist_seg(px: f32, py: f32, x0: f32, y0: f32, x1: f32, y1: f32) -> f32 {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len2 = dx * dx + dy * dy;
    let t = if len2 < 0.0001 {
        0.0
    } else {
        (((px - x0) * dx + (py - y0) * dy) / len2).clamp(0.0, 1.0)
    };
    let ex = px - (x0 + dx * t);
    let ey = py - (y0 + dy * t);
    (ex * ex + ey * ey).sqrt()
}

fn blend(dst: &mut u32, src: u32, a: f32) {
    if a >= 0.99 {
        *dst = src;
        return;
    }
    if a <= 0.0 {
        return;
    }
    let d = *dst;
    let inv = 1.0 - a;
    let ch = |shift: u32| {
        let dv = ((d >> shift) & 255) as f32;
        let sv = ((src >> shift) & 255) as f32;
        (dv * inv + sv * a) as u32
    };
    *dst = (ch(16) << 16) | (ch(8) << 8) | ch(0);
}

