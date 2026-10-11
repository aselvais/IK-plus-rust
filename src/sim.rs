//! Three-fighter karate rules. Facing is sticky, like the 1987 game:
//! walking backward blocks, and only a reverse attack turns you around.
//! A clean hit from the front is two pips. A hit in the back is one.
//! First to six, or the leader when the clock runs out, takes the round.
//! Last place is eliminated.

use std::cmp::Ordering;

pub const SCREEN_W: i32 = 1280;
pub const SCREEN_H: i32 = 720;
pub const GROUND: f32 = 640.0;
const LEFT: f32 = 86.0;
const RIGHT: f32 = 1194.0;
const ROUND_TIME: f32 = 30.0;
const WALK: f32 = 310.0;
const GRAVITY: f32 = 2100.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Title,
    Intro,
    Fight,
    Resolve,
    Bonus,
    GameOver,
    Pause,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveId {
    FrontPunch,
    ShinKick,
    Sweep,
    CrouchPunch,
    ReversePunch,
    FlyingKick,
    Headbutt,
    StomachKick,
    FaceKick,
    ReverseSweep,
    BackStepKick,
    DoubleKick,
}

impl MoveId {
    pub fn name(self) -> &'static str {
        match self {
            MoveId::FrontPunch => "FRONT PUNCH",
            MoveId::ShinKick => "SHIN KICK",
            MoveId::Sweep => "FOOT SWEEP",
            MoveId::CrouchPunch => "STOMACH PUNCH",
            MoveId::ReversePunch => "REVERSE PUNCH",
            MoveId::FlyingKick => "FLYING KICK",
            MoveId::Headbutt => "HEADBUTT",
            MoveId::StomachKick => "STOMACH KICK",
            MoveId::FaceKick => "FACE KICK",
            MoveId::ReverseSweep => "REVERSE SWEEP",
            MoveId::BackStepKick => "BACK KICK",
            MoveId::DoubleKick => "DOUBLE KICK",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Idle,
    Walk,
    Block,
    Jump,
    Attack(MoveId),
    Backflip,
    Fall,
    Down,
    Rise,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sfx {
    Punch,
    Kick,
    Sweep,
    Whoosh,
    Hit,
    Block,
    Fall,
    Cheer,
    Ui,
    Deflect,
    Miss,
    Kiai,
}

#[derive(Clone, Copy, Debug)]
pub struct Spec {
    pub dur: f32,
    pub hit0: f32,
    pub hit1: f32,
    pub min_d: f32,
    pub max_d: f32,
    pub height: u8,
    pub score: i32,
    pub blockable: bool,
    pub both: bool,
    pub turns: bool,
    pub hop: f32,
    pub speed: f32,
    pub sfx: Sfx,
}

pub fn spec(id: MoveId) -> Spec {
    use MoveId::*;
    let s = |dur, hit0, hit1, min_d, max_d, height, score, blockable, both, turns, hop, speed, sfx| Spec {
        dur,
        hit0,
        hit1,
        min_d,
        max_d,
        height,
        score,
        blockable,
        both,
        turns,
        hop,
        speed,
        sfx,
    };
    match id {
        FrontPunch => s(0.32, 0.08, 0.16, 42.0, 116.0, 2, 800, true, false, false, 0.0, 36.0, Sfx::Punch),
        ShinKick => s(0.36, 0.10, 0.18, 44.0, 122.0, 0, 400, false, false, false, 0.0, 24.0, Sfx::Kick),
        Sweep => s(0.40, 0.12, 0.22, 36.0, 132.0, 0, 400, false, false, false, 0.0, 18.0, Sfx::Sweep),
        CrouchPunch => s(0.34, 0.10, 0.18, 40.0, 110.0, 1, 400, true, false, false, 0.0, 20.0, Sfx::Punch),
        ReversePunch => s(0.34, 0.10, 0.18, 42.0, 116.0, 2, 800, true, false, true, 0.0, 28.0, Sfx::Punch),
        FlyingKick => s(0.58, 0.16, 0.34, 34.0, 128.0, 2, 800, true, false, false, 620.0, 390.0, Sfx::Kiai),
        Headbutt => s(0.30, 0.08, 0.16, 26.0, 82.0, 2, 1000, true, false, false, 0.0, 48.0, Sfx::Punch),
        StomachKick => s(0.34, 0.08, 0.16, 44.0, 118.0, 1, 200, true, false, false, 0.0, 20.0, Sfx::Kick),
        FaceKick => s(0.42, 0.14, 0.24, 50.0, 140.0, 2, 800, true, false, false, 0.0, 28.0, Sfx::Kick),
        ReverseSweep => s(0.42, 0.14, 0.24, 36.0, 132.0, 0, 400, false, false, true, 0.0, 20.0, Sfx::Sweep),
        BackStepKick => s(0.46, 0.16, 0.28, 48.0, 136.0, 2, 800, true, false, true, 90.0, 70.0, Sfx::Kick),
        DoubleKick => s(0.50, 0.16, 0.30, 36.0, 136.0, 2, 1000, false, true, false, 420.0, 0.0, Sfx::Kiai),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Plan {
    None,
    Walk,
    Block,
    Attack(MoveId),
    Flip,
}

#[derive(Clone, Copy, Debug)]
pub struct Fighter {
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub facing: f32,
    pub action: Action,
    pub t: f32,
    pub phase: f32,
    pub human: bool,
    pub pips: i32,
    pub round_score: i32,
    pub total_score: i32,
    pub flash: f32,
    pub i_frames: f32,
    pub hit_mask: u8,
    pub index: usize,
    knock: f32,
    jump_ok: bool,
    turn_latch: bool,
    dust: f32,
    plan: Plan,
    plan_t: f32,
    ai_rng: u32,
    last_hit_time: f32,
}

impl Fighter {
    fn spawn(index: usize) -> Self {
        Self {
            x: 640.0,
            y: 0.0,
            vy: 0.0,
            facing: 1.0,
            action: Action::Idle,
            t: 0.0,
            phase: index as f32,
            human: false,
            pips: 0,
            round_score: 0,
            total_score: 0,
            flash: 0.0,
            i_frames: 0.0,
            hit_mask: 0,
            index,
            knock: 0.0,
            jump_ok: true,
            turn_latch: false,
            dust: 0.0,
            plan: Plan::None,
            plan_t: 0.2 + index as f32 * 0.15,
            ai_rng: 0xA341_316C ^ (index as u32).wrapping_mul(0x9E37_79B9),
            last_hit_time: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max: f32,
    pub color: u32,
    pub r: f32,
}

#[derive(Clone, Debug)]
pub struct Floater {
    pub x: f32,
    pub y: f32,
    pub life: f32,
    pub text: String,
    pub color: u32,
    pub scale: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Ball {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub alive: bool,
}

#[derive(Clone, Debug)]
pub struct Bonus {
    pub balls: Vec<Ball>,
    pub remaining: i32,
    pub spawn_t: f32,
    pub deflected: i32,
    pub missed: i32,
    pub player: usize,
    pub clear: bool,
    pub clear_t: f32,
    pub awarded: bool,
    rng: u32,
}

impl Bonus {
    fn empty() -> Self {
        Self {
            balls: Vec::new(),
            remaining: 0,
            spawn_t: 0.4,
            deflected: 0,
            missed: 0,
            player: 0,
            clear: false,
            clear_t: 0.0,
            awarded: false,
            rng: 1,
        }
    }

    fn start(round: i32, player: usize, rng: u32) -> Self {
        Self {
            balls: Vec::new(),
            remaining: 8 + round.min(6),
            spawn_t: 0.35,
            deflected: 0,
            missed: 0,
            player,
            clear: false,
            clear_t: 0.0,
            awarded: false,
            rng,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Pad {
    pub x: i8,
    pub y: i8,
    pub fire: bool,
    pub start: bool,
    pub y_edge: i8,
}

#[derive(Clone, Copy, PartialEq)]
enum Intent {
    None,
    Walk,
    Block,
    Jump,
    Flip,
    Attack(MoveId),
}

pub struct Game {
    pub mode: Mode,
    pub fighters: [Fighter; 3],
    pub round: i32,
    pub timer: f32,
    pub speed: i32,
    pub time: f32,
    pub shake: f32,
    pub particles: Vec<Particle>,
    pub floaters: Vec<Floater>,
    pub ranks: [usize; 3],
    pub time_bonus: i32,
    pub bonus: Bonus,
    pub menu: i32,
    pub players: i32,
    pub help: bool,
    pub hitstop: f32,
    pub sounds: Vec<Sfx>,
    pub note: String,
    pub belt: String,
    pub ai_enabled: bool,
    resolve_t: f32,
    intro_t: f32,
    pub paused_mode: Mode,
    seed: u32,
}

impl Game {
    pub fn new() -> Self {
        let mut g = Self {
            mode: Mode::Title,
            fighters: [Fighter::spawn(0), Fighter::spawn(1), Fighter::spawn(2)],
            round: 1,
            timer: ROUND_TIME,
            speed: 5,
            time: 0.0,
            shake: 0.0,
            particles: Vec::new(),
            floaters: Vec::new(),
            ranks: [0, 1, 2],
            time_bonus: 0,
            bonus: Bonus::empty(),
            menu: 0,
            players: 1,
            help: false,
            hitstop: 0.0,
            sounds: Vec::new(),
            note: String::new(),
            belt: belt_name(1),
            ai_enabled: true,
            resolve_t: 0.0,
            intro_t: 0.0,
            paused_mode: Mode::Fight,
            seed: 0xC0FFEE,
        };
        g.place_title();
        g
    }

    #[allow(dead_code)]
    pub fn fresh_fight() -> Self {
        let mut g = Self::new();
        g.mode = Mode::Fight;
        g.ai_enabled = false;
        g.timer = ROUND_TIME;
        g.round = 1;
        g.belt = belt_name(1);
        g.park();
        g
    }

    pub fn speed_mul(&self) -> f32 {
        0.62 + (self.speed - 1) as f32 * 0.12
    }

    pub fn bow(&self) -> f32 {
        if self.mode != Mode::Intro {
            return 0.0;
        }
        let k = (self.intro_t / 1.55).clamp(0.0, 1.0);
        if k < 0.45 {
            k / 0.45
        } else if k < 0.7 {
            1.0
        } else {
            (1.0 - k) / 0.3
        }
    }

    pub fn sky_round(&self) -> i32 {
        match self.mode {
            Mode::Title | Mode::GameOver => 2,
            Mode::Bonus => self.round + 1,
            Mode::Pause => match self.paused_mode {
                Mode::Bonus => self.round + 1,
                Mode::Title => 2,
                _ => self.round,
            },
            _ => self.round,
        }
    }

    pub fn take_sounds(&mut self) -> Vec<Sfx> {
        std::mem::take(&mut self.sounds)
    }

    pub fn toggle_pause(&mut self) {
        if self.help {
            return;
        }
        if self.mode == Mode::Pause {
            self.mode = self.paused_mode;
        } else if matches!(self.mode, Mode::Fight | Mode::Intro | Mode::Bonus) {
            self.paused_mode = self.mode;
            self.mode = Mode::Pause;
        }
    }

    pub fn nudge_speed(&mut self, delta: i32) {
        self.speed = (self.speed + delta).clamp(1, 8);
    }

    pub fn update(&mut self, pads: [Pad; 2], dt: f32) {
        let dt = dt.clamp(0.0, 0.05);
        self.time += dt;
        self.shake = (self.shake - dt * 18.0).max(0.0);
        self.tick_fx(dt);
        if self.help {
            return;
        }
        if self.hitstop > 0.0 && self.mode == Mode::Fight {
            self.hitstop = (self.hitstop - dt).max(0.0);
            return;
        }
        let sim = dt * self.speed_mul();
        match self.mode {
            Mode::Title => self.update_title(pads, dt),
            Mode::Intro => self.update_intro(sim),
            Mode::Fight => self.update_fight(pads, sim),
            Mode::Resolve => self.update_resolve(pads, dt),
            Mode::Bonus => self.update_bonus(pads, sim),
            Mode::GameOver => self.update_over(pads),
            Mode::Pause => {}
        }
    }

    fn update_title(&mut self, pads: [Pad; 2], dt: f32) {
        self.place_title();
        for f in &mut self.fighters {
            f.action = Action::Idle;
            f.y = 0.0;
            f.vy = 0.0;
            f.phase += dt * 1.7;
            f.human = false;
        }
        if pads.iter().any(|p| p.y_edge < 0) {
            self.menu = 0;
            self.sound(Sfx::Ui);
        }
        if pads.iter().any(|p| p.y_edge > 0) {
            self.menu = 1;
            self.sound(Sfx::Ui);
        }
        if pads.iter().any(|p| p.start) {
            self.start_match(self.menu + 1);
        }
    }

    pub fn start_match(&mut self, players: i32) {
        let speed = self.speed;
        let seed = self.seed.wrapping_add(1);
        *self = Game::new();
        self.speed = speed;
        self.seed = seed;
        self.players = players.clamp(1, 2);
        self.fighters[0].human = true;
        self.fighters[1].human = self.players >= 2;
        self.round = 1;
        self.timer = ROUND_TIME;
        self.mode = Mode::Intro;
        self.intro_t = 0.0;
        self.belt = belt_name(1);
        self.note.clear();
        self.park();
        for f in &mut self.fighters {
            f.i_frames = 0.6;
            f.pips = 0;
            f.round_score = 0;
            f.total_score = 0;
        }
        self.sound(Sfx::Ui);
    }

    fn update_intro(&mut self, dt: f32) {
        self.intro_t += dt;
        for f in &mut self.fighters {
            f.phase += dt;
            f.action = Action::Idle;
            f.y = 0.0;
            f.vy = 0.0;
        }
        if self.intro_t >= 1.7 {
            self.mode = Mode::Fight;
            self.timer = ROUND_TIME;
            for f in &mut self.fighters {
                f.i_frames = 0.25;
                f.action = Action::Idle;
            }
        }
    }

    fn update_fight(&mut self, pads: [Pad; 2], dt: f32) {
        for i in 0..3 {
            if self.fighters[i].human {
                let pad = if i == 0 { pads[0] } else { pads[1] };
                self.control_human(i, pad, dt);
            } else if self.ai_enabled {
                self.control_ai(i, dt);
            }
            self.integrate(i, dt);
        }
        self.resolve_hits();
        self.expire_states();
        for f in &mut self.fighters {
            f.x = f.x.clamp(LEFT, RIGHT);
            if f.flash > 0.0 {
                f.flash -= dt;
            }
            if f.i_frames > 0.0 {
                f.i_frames -= dt;
            }
        }
        self.timer -= dt;
        if self.timer <= 0.0 || self.fighters.iter().any(|f| f.pips >= 6) {
            self.timer = self.timer.max(0.0);
            self.enter_resolve();
        }
    }

    fn control_human(&mut self, i: usize, pad: Pad, dt: f32) {
        if !interruptible(self.fighters[i].action) {
            return;
        }
        let facing = self.fighters[i].facing;
        let intent = intent_from_pad(pad, facing);
        match intent {
            Intent::None => {
                self.fighters[i].action = Action::Idle;
                self.fighters[i].turn_latch = false;
            }
            Intent::Walk => {
                let dust_x = {
                    let f = &mut self.fighters[i];
                    f.action = Action::Walk;
                    f.turn_latch = false;
                    f.phase += dt * 9.0;
                    f.x += f.facing * WALK * dt;
                    f.dust -= dt;
                    if f.dust <= 0.0 {
                        f.dust = 0.11;
                        Some(f.x)
                    } else {
                        None
                    }
                };
                if let Some(x) = dust_x {
                    self.puff(x, GROUND - 4.0, 0xE4C891, 3, 40.0);
                }
            }
            Intent::Block => {
                let f = &mut self.fighters[i];
                f.action = Action::Block;
                f.turn_latch = false;
                f.x -= f.facing * WALK * 0.7 * dt;
            }
            Intent::Jump => {
                self.fighters[i].turn_latch = false;
                if self.fighters[i].jump_ok {
                    self.begin_jump(i);
                    self.fighters[i].jump_ok = false;
                }
            }
            Intent::Flip => {
                if self.fighters[i].turn_latch {
                    self.fighters[i].action = Action::Block;
                } else {
                    self.begin_flip(i);
                    self.fighters[i].turn_latch = true;
                }
            }
            Intent::Attack(id) => {
                let turning = spec(id).turns;
                if turning && self.fighters[i].turn_latch {
                    self.fighters[i].action = Action::Idle;
                } else {
                    self.begin_attack(i, id);
                    if turning {
                        self.fighters[i].turn_latch = true;
                    }
                }
            }
        }
        if !matches!(intent, Intent::Jump) {
            self.fighters[i].jump_ok = true;
        }
    }

    fn control_ai(&mut self, i: usize, dt: f32) {
        if !interruptible(self.fighters[i].action) {
            return;
        }
        self.fighters[i].plan_t -= dt;
        if self.fighters[i].plan_t <= 0.0 || self.fighters[i].plan == Plan::None {
            let (plan, hold) = self.think(i);
            self.fighters[i].plan = plan;
            self.fighters[i].plan_t = hold;
        }
        match self.fighters[i].plan {
            Plan::None => self.fighters[i].action = Action::Idle,
            Plan::Walk => {
                let f = &mut self.fighters[i];
                f.action = Action::Walk;
                f.phase += dt * 9.0;
                f.x += f.facing * WALK * dt;
            }
            Plan::Block => {
                let f = &mut self.fighters[i];
                f.action = Action::Block;
                f.x -= f.facing * WALK * 0.55 * dt;
            }
            Plan::Flip => {
                self.begin_flip(i);
                self.fighters[i].plan = Plan::None;
            }
            Plan::Attack(id) => {
                self.begin_attack(i, id);
                self.fighters[i].plan = Plan::None;
                self.fighters[i].plan_t = 0.08;
            }
        }
    }

    fn think(&mut self, i: usize) -> (Plan, f32) {
        let skill = ((self.round - 1) as f32 * 0.11).min(1.0);
        let me_x = self.fighters[i].x;
        let me_f = self.fighters[i].facing;
        let mut live = [0usize; 2];
        let mut n = 0;
        for j in 0..3 {
            if j == i {
                continue;
            }
            if matches!(self.fighters[j].action, Action::Fall | Action::Down | Action::Rise) {
                continue;
            }
            live[n] = j;
            n += 1;
        }
        if n == 0 {
            return (Plan::None, 0.35);
        }
        for k in 0..n {
            let j = live[k];
            let threatened = self.threatens(j, i);
            let roll = self.luck(i);
            if threatened && roll < 0.22 + skill * 0.62 {
                let dx = self.fighters[j].x - me_x;
                let in_front = dx * me_f > 0.0;
                let block_roll = self.luck(i);
                if in_front && block_roll < 0.7 {
                    let hold = 0.26 + self.luck(i) * 0.16;
                    return (Plan::Block, hold);
                }
                let flip_roll = self.luck(i);
                if flip_roll < 0.55 {
                    return (Plan::Flip, 0.2);
                }
            }
        }
        let mut target = live[0];
        let mut best = f32::MAX;
        for k in 0..n {
            let j = live[k];
            let mut d = (self.fighters[j].x - me_x).abs();
            d -= self.fighters[j].pips as f32 * (12.0 + skill * 22.0);
            if self.fighters[j].human {
                d -= 18.0;
            }
            if d < best {
                best = d;
                target = j;
            }
        }
        let dx = self.fighters[target].x - me_x;
        let dist = dx.abs();
        let in_front = dx * me_f > 0.0;
        if !in_front {
            if dist < 140.0 && self.luck(i) < 0.4 {
                return (Plan::Attack(MoveId::ReverseSweep), 0.12);
            }
            return (Plan::Attack(MoveId::ReversePunch), 0.1);
        }
        if n == 2 {
            let a = self.fighters[live[0]].x - me_x;
            let b = self.fighters[live[1]].x - me_x;
            if a * b < 0.0 && a.abs() < 130.0 && b.abs() < 130.0 && self.luck(i) < 0.12 + skill * 0.5 {
                return (Plan::Attack(MoveId::DoubleKick), 0.16);
            }
        }
        if dist > 200.0 {
            if dist < 360.0 && self.luck(i) < 0.06 + skill * 0.28 {
                return (Plan::Attack(MoveId::FlyingKick), 0.12);
            }
            return (Plan::Walk, 0.16 + self.luck(i) * 0.18);
        }
        if dist > 125.0 {
            let r = self.luck(i);
            if r < 0.35 {
                return (Plan::Walk, 0.12);
            }
            if r < 0.55 + skill * 0.25 {
                let id = if self.luck(i) < 0.45 {
                    MoveId::FlyingKick
                } else {
                    MoveId::FaceKick
                };
                return (Plan::Attack(id), 0.12);
            }
            return (Plan::Walk, 0.12);
        }
        let menu: [MoveId; 7] = if i == 2 {
            [
                MoveId::FaceKick,
                MoveId::Sweep,
                MoveId::ShinKick,
                MoveId::FrontPunch,
                MoveId::StomachKick,
                MoveId::Headbutt,
                MoveId::CrouchPunch,
            ]
        } else {
            [
                MoveId::FrontPunch,
                MoveId::Headbutt,
                MoveId::Sweep,
                MoveId::ShinKick,
                MoveId::StomachKick,
                MoveId::FaceKick,
                MoveId::CrouchPunch,
            ]
        };
        let mut pick = menu[(self.luck(i) * menu.len() as f32) as usize % menu.len()];
        if dist < 72.0 && self.luck(i) < 0.55 {
            pick = if self.luck(i) < 0.5 {
                MoveId::Headbutt
            } else {
                MoveId::FrontPunch
            };
        }
        (Plan::Attack(pick), 0.12 + self.luck(i) * (0.26 - skill * 0.14))
    }

    fn threatens(&self, attacker: usize, defender: usize) -> bool {
        let Some(id) = attack_of(self.fighters[attacker].action) else {
            return false;
        };
        let spec = spec(id);
        let t = self.fighters[attacker].t;
        if t < spec.hit0 - 0.06 || t > spec.hit1 {
            return false;
        }
        let dx = self.fighters[defender].x - self.fighters[attacker].x;
        let dist = dx.abs();
        if dist > spec.max_d + 24.0 {
            return false;
        }
        spec.both || dx * self.fighters[attacker].facing > 0.0
    }

    fn luck(&mut self, i: usize) -> f32 {
        let r = &mut self.fighters[i].ai_rng;
        *r = r.wrapping_mul(1664525).wrapping_add(1013904223);
        ((*r >> 8) & 0x00FF_FFFF) as f32 / 16_777_216.0
    }

    fn begin_attack(&mut self, i: usize, id: MoveId) {
        let spec = spec(id);
        let f = &mut self.fighters[i];
        if spec.turns {
            f.facing = -f.facing;
        }
        f.action = Action::Attack(id);
        f.t = 0.0;
        f.hit_mask = 0;
        if spec.hop > 0.0 {
            f.vy = -spec.hop;
        }
        self.sound(spec.sfx);
    }

    fn begin_jump(&mut self, i: usize) {
        let f = &mut self.fighters[i];
        f.action = Action::Jump;
        f.t = 0.0;
        f.vy = -760.0;
        self.sound(Sfx::Whoosh);
    }

    fn begin_flip(&mut self, i: usize) {
        let f = &mut self.fighters[i];
        f.action = Action::Backflip;
        f.t = 0.0;
        f.vy = -500.0;
        f.i_frames = 0.58;
        self.sound(Sfx::Whoosh);
    }

    fn integrate(&mut self, i: usize, dt: f32) {
        let mut dust_at = None;
        {
            let f = &mut self.fighters[i];
            f.t += dt;
            if f.y < 0.0 || f.vy != 0.0 {
                f.vy += GRAVITY * dt;
                f.y += f.vy * dt;
                if f.y >= 0.0 {
                    f.y = 0.0;
                    f.vy = 0.0;
                    if matches!(f.action, Action::Fall | Action::Backflip | Action::Jump) {
                        dust_at = Some(f.x);
                    }
                }
            }
            match f.action {
                Action::Attack(id) => {
                    let speed = spec(id).speed;
                    f.x += f.facing * speed * dt;
                }
                Action::Backflip => {
                    f.x -= f.facing * 330.0 * dt;
                }
                Action::Fall => {
                    f.x += f.knock * dt;
                    f.knock *= (1.0 - dt * 2.4).max(0.0);
                }
                _ => {}
            }
        }
        if let Some(x) = dust_at {
            self.puff(x, GROUND - 2.0, 0xE7D3A4, 6, 70.0);
        }
    }

    fn expire_states(&mut self) {
        for i in 0..3 {
            let f = &self.fighters[i];
            let next = match f.action {
                Action::Attack(id) if f.t >= spec(id).dur && f.y >= 0.0 => Some(Action::Idle),
                Action::Jump if f.y >= 0.0 && f.t > 0.18 => Some(Action::Idle),
                Action::Backflip if f.t >= 0.56 && f.y >= 0.0 => Some(Action::Idle),
                Action::Fall if f.y >= 0.0 && f.t > 0.26 => Some(Action::Down),
                Action::Down if f.t > 0.78 => Some(Action::Rise),
                Action::Rise if f.t > 0.30 => Some(Action::Idle),
                _ => None,
            };
            if let Some(action) = next {
                let x = self.fighters[i].x;
                self.fighters[i].action = action;
                self.fighters[i].t = 0.0;
                if action == Action::Down {
                    self.puff(x, GROUND - 6.0, 0xD9C08A, 8, 90.0);
                }
                if action == Action::Rise {
                    self.fighters[i].i_frames = 0.42;
                }
            }
        }
    }

    fn resolve_hits(&mut self) {
        for i in 0..3 {
            let Some(id) = attack_of(self.fighters[i].action) else {
                continue;
            };
            let spec = spec(id);
            let t = self.fighters[i].t;
            if t < spec.hit0 || t > spec.hit1 {
                continue;
            }
            for j in 0..3 {
                if i == j {
                    continue;
                }
                self.try_hit(i, j, id, spec);
            }
        }
    }

    fn try_hit(&mut self, i: usize, j: usize, id: MoveId, spec: Spec) {
        if self.fighters[i].hit_mask & (1 << j) != 0 {
            return;
        }
        if self.fighters[j].i_frames > 0.0 {
            return;
        }
        if matches!(
            self.fighters[j].action,
            Action::Fall | Action::Down | Action::Rise | Action::Backflip
        ) {
            return;
        }
        let ax = self.fighters[i].x;
        let af = self.fighters[i].facing;
        let dx = self.fighters[j].x - ax;
        let dist = dx.abs();
        let in_front = dx * af > 0.0;
        if spec.both {
            if dist < spec.min_d || dist > spec.max_d {
                return;
            }
        } else if !in_front || dist < spec.min_d || dist > spec.max_d {
            return;
        }
        if !height_ok(spec.height, self.fighters[j].y) {
            return;
        }
        let sees = (ax - self.fighters[j].x) * self.fighters[j].facing > 0.0;
        let blocked = self.fighters[j].action == Action::Block && spec.blockable && sees && spec.height != 0;
        self.fighters[i].hit_mask |= 1 << j;
        if blocked {
            self.fighters[j].x += af * 14.0;
            self.fighters[j].flash = 0.05;
            let hx = (ax + self.fighters[j].x) * 0.5;
            let hy = GROUND - 130.0 + self.fighters[j].y;
            self.popup(hx, hy, "BLOCK".to_string(), 0xD7E7FF, 2);
            self.burst(hx, hy, 0xEAF4FF, 5, 80.0);
            self.sound(Sfx::Block);
            return;
        }
        let from_front = sees || id == MoveId::DoubleKick;
        let mut pips = if from_front { 2 } else { 1 };
        let mut pts = if from_front { spec.score } else { spec.score / 2 };
        if id == MoveId::DoubleKick {
            pips = 2;
            pts = spec.score;
        }
        let room = (6 - self.fighters[i].pips).max(0);
        pips = pips.min(room);
        self.fighters[i].pips += pips;
        self.fighters[i].round_score += pts;
        self.fighters[i].total_score += pts;
        self.fighters[i].last_hit_time = self.time;
        self.fighters[j].action = Action::Fall;
        self.fighters[j].t = 0.0;
        self.fighters[j].vy = -320.0;
        self.fighters[j].knock = af * 260.0;
        self.fighters[j].flash = 0.14;
        self.fighters[j].hit_mask = 0;
        let hx = (self.fighters[i].x + self.fighters[j].x) * 0.5;
        let hy = GROUND - 128.0 + self.fighters[j].y * 0.5;
        self.popup(hx, hy - 18.0, format!("+{pts}"), 0xFFE08A, 3);
        if self.fighters[i].human {
            self.popup(hx, hy + 16.0, id.name().to_string(), 0xF7F4EF, 2);
        }
        self.burst(hx, hy, 0xFFF6D8, 10, 160.0);
        self.burst(hx, hy, 0xE07040, 4, 90.0);
        self.shake = self.shake.max((spec.score as f32 / 180.0).min(8.0));
        self.hitstop = self.hitstop.max(if spec.score >= 800 { 0.048 } else { 0.03 });
        self.sound(Sfx::Hit);
        self.sound(Sfx::Fall);
    }

    fn enter_resolve(&mut self) {
        self.mode = Mode::Resolve;
        self.resolve_t = 0.0;
        self.ranks = self.compute_ranks();
        self.time_bonus = 0;
        let first = self.ranks[0];
        if self.fighters[first].pips >= 6 {
            let bonus = (self.timer.ceil() as i32) * 100;
            self.time_bonus = bonus;
            self.fighters[first].total_score += bonus;
            self.fighters[first].round_score += bonus;
        }
        self.note = format!("{} TAKES THE ROUND", fighter_name(first));
        self.sound(Sfx::Cheer);
    }

    fn compute_ranks(&self) -> [usize; 3] {
        let mut order = [0usize, 1, 2];
        order.sort_by(|&a, &b| {
            let fa = &self.fighters[a];
            let fb = &self.fighters[b];
            fb.pips
                .cmp(&fa.pips)
                .then(fb.round_score.cmp(&fa.round_score))
                .then_with(|| fb.last_hit_time.partial_cmp(&fa.last_hit_time).unwrap_or(Ordering::Equal))
                .then(a.cmp(&b))
        });
        order
    }

    fn update_resolve(&mut self, pads: [Pad; 2], dt: f32) {
        self.resolve_t += dt;
        for f in &mut self.fighters {
            if f.flash > 0.0 {
                f.flash -= dt;
            }
            if matches!(f.action, Action::Fall | Action::Down | Action::Rise | Action::Jump | Action::Backflip) {
                // Let a knockdown finish under the results card.
            } else {
                f.action = Action::Idle;
                f.phase += dt;
            }
        }
        // Finish falls so the results pose is calm.
        for i in 0..3 {
            if matches!(self.fighters[i].action, Action::Fall | Action::Jump | Action::Backflip | Action::Attack(_)) {
                self.integrate(i, dt);
            }
        }
        self.expire_states();
        if self.resolve_t > 3.3 || pads.iter().any(|p| p.start) {
            self.leave_resolve();
        }
    }

    fn leave_resolve(&mut self) {
        let last = self.ranks[2];
        let humans = self.fighters.iter().filter(|f| f.human).count();
        if self.fighters[last].human && humans <= 1 {
            self.mode = Mode::GameOver;
            self.note = format!("ELIMINATED  ·  ROUND {}", self.round);
            self.sound(Sfx::Ui);
            return;
        }
        let out_name = fighter_name(last);
        if self.fighters[last].human {
            self.fighters[last].human = false;
            self.note = format!("{out_name} IS OUT");
        } else {
            self.note = format!("{out_name} IS REPLACED");
        }
        self.fresh_challenger(last);
        for f in &mut self.fighters {
            f.pips = 0;
            f.round_score = 0;
        }
        if self.round % 2 == 0 {
            self.start_bonus();
        } else {
            self.next_round();
        }
    }

    fn fresh_challenger(&mut self, i: usize) {
        let mut f = Fighter::spawn(i);
        f.human = false;
        f.ai_rng ^= self.seed.wrapping_add(self.round as u32 * 17);
        self.fighters[i] = f;
    }

    fn next_round(&mut self) {
        self.round += 1;
        self.timer = ROUND_TIME;
        self.intro_t = 0.0;
        self.mode = Mode::Intro;
        self.hitstop = 0.0;
        self.belt = belt_name(self.round);
        self.time_bonus = 0;
        self.park();
        for f in &mut self.fighters {
            f.i_frames = 0.55;
        }
    }

    fn start_bonus(&mut self) {
        let player = if self.fighters[0].human { 0 } else { 1 };
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        self.bonus = Bonus::start(self.round, player, self.seed);
        self.mode = Mode::Bonus;
        self.hitstop = 0.0;
        self.park();
        for f in &mut self.fighters {
            f.action = Action::Idle;
            f.y = 0.0;
        }
        self.fighters[player].x = 360.0;
        self.fighters[player].facing = 1.0;
        self.note = "DEFLECT EVERY BALL".to_string();
        self.sound(Sfx::Ui);
    }

    fn update_bonus(&mut self, pads: [Pad; 2], dt: f32) {
        let player = self.bonus.player;
        let pad = if player == 0 { pads[0] } else { pads[1] };
        if self.fighters[player].human {
            self.fighters[player].x += pad.x as f32 * 460.0 * dt;
            self.fighters[player].x = self.fighters[player].x.clamp(LEFT, RIGHT);
            self.fighters[player].facing = 1.0;
            self.fighters[player].action = if pad.x != 0 { Action::Walk } else { Action::Idle };
            if pad.x != 0 {
                self.fighters[player].phase += dt * 9.0;
            }
        }
        if !self.bonus.clear {
            self.bonus.spawn_t -= dt;
            let alive = self.bonus.balls.iter().filter(|b| b.alive).count();
            if self.bonus.remaining > 0 && alive < 3 && self.bonus.spawn_t <= 0.0 {
                self.spawn_ball();
                self.bonus.spawn_t = 0.62;
            }
            let shield_x = self.fighters[player].x;
            let shield_y = GROUND - 150.0;
            let mut pops: Vec<(f32, f32, bool)> = Vec::new();
            for ball in &mut self.bonus.balls {
                if !ball.alive {
                    continue;
                }
                ball.x += ball.vx * dt;
                ball.y += ball.vy * dt;
                if ball.x < 48.0 || ball.x > SCREEN_W as f32 - 48.0 {
                    ball.vx = -ball.vx;
                    ball.x = ball.x.clamp(48.0, SCREEN_W as f32 - 48.0);
                }
                if ball.y < 56.0 {
                    ball.vy = ball.vy.abs();
                    ball.y = 56.0;
                }
                let hit_shield = ball.vy > 0.0
                    && (ball.y - shield_y).abs() < 30.0
                    && (ball.x - shield_x).abs() < 86.0;
                if hit_shield {
                    ball.vy = -ball.vy.abs() * 0.96;
                    ball.vx += (ball.x - shield_x) * 1.6;
                    ball.vx = ball.vx.clamp(-280.0, 280.0);
                    ball.y = shield_y - 28.0;
                    self.bonus.deflected += 1;
                    self.fighters[player].total_score += 250;
                    pops.push((ball.x, ball.y, true));
                } else if ball.y > GROUND - 16.0 {
                    ball.alive = false;
                    self.bonus.missed += 1;
                    pops.push((ball.x, ball.y, false));
                }
            }
            for (x, y, caught) in pops {
                if caught {
                    self.popup(x, y - 20.0, "+250".to_string(), 0xFFE08A, 2);
                    self.burst(x, y, 0xFFF4D2, 5, 90.0);
                    self.sound(Sfx::Deflect);
                } else {
                    self.puff(x, GROUND - 8.0, 0xC9B48A, 6, 70.0);
                    self.sound(Sfx::Miss);
                }
            }
            let none = self.bonus.balls.iter().all(|b| !b.alive);
            if self.bonus.remaining <= 0 && none {
                self.finish_bonus();
            }
        } else {
            self.bonus.clear_t += dt;
            if self.bonus.clear_t > 1.7 {
                self.next_round();
            }
        }
    }

    fn spawn_ball(&mut self) {
        self.bonus.remaining -= 1;
        self.bonus.rng = self.bonus.rng.wrapping_mul(1664525).wrapping_add(1013904223);
        let u = ((self.bonus.rng >> 8) & 0xFFFF) as f32 / 65535.0;
        self.bonus.rng = self.bonus.rng.wrapping_mul(1664525).wrapping_add(1013904223);
        let v = ((self.bonus.rng >> 8) & 0xFFFF) as f32 / 65535.0;
        let x = 180.0 + u * 920.0;
        let vx = (v - 0.5) * 280.0;
        self.bonus.balls.push(Ball {
            x,
            y: 40.0,
            vx,
            vy: 210.0 + u * 90.0,
            alive: true,
        });
    }

    fn finish_bonus(&mut self) {
        if !self.bonus.awarded {
            self.bonus.awarded = true;
            if self.bonus.missed == 0 {
                let p = self.bonus.player;
                self.fighters[p].total_score += 5000;
                self.note = "SURVIVAL BONUS  5000".to_string();
                self.sound(Sfx::Cheer);
            } else {
                self.note = format!("{} MISSED", self.bonus.missed);
            }
        }
        self.bonus.clear = true;
        self.bonus.clear_t = 0.0;
    }

    fn update_over(&mut self, pads: [Pad; 2]) {
        for f in &mut self.fighters {
            f.phase += 0.016;
            if !matches!(f.action, Action::Down | Action::Fall | Action::Rise) {
                f.action = Action::Idle;
            }
        }
        if pads.iter().any(|p| p.start) {
            let players = self.players;
            self.start_match(players);
        }
    }

    fn place_title(&mut self) {
        let xs = [230.0, 640.0, 1050.0];
        let fac = [1.0, -1.0, -1.0];
        for i in 0..3 {
            self.fighters[i].x = xs[i];
            self.fighters[i].y = 0.0;
            self.fighters[i].vy = 0.0;
            self.fighters[i].facing = fac[i];
        }
    }

    fn park(&mut self) {
        let xs = [250.0, 640.0, 1030.0];
        let fac = [1.0, -1.0, -1.0];
        for i in 0..3 {
            let f = &mut self.fighters[i];
            f.x = xs[i];
            f.y = 0.0;
            f.vy = 0.0;
            f.knock = 0.0;
            f.facing = fac[i];
            f.action = Action::Idle;
            f.t = 0.0;
            f.hit_mask = 0;
            f.flash = 0.0;
            f.plan = Plan::None;
            f.plan_t = 0.15 + i as f32 * 0.12;
        }
    }

    fn tick_fx(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.life -= dt;
            p.vy += 900.0 * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
        self.particles.retain(|p| p.life > 0.0);
        for f in &mut self.floaters {
            f.life -= dt;
            f.y -= 28.0 * dt;
        }
        self.floaters.retain(|f| f.life > 0.0);
    }

    fn burst(&mut self, x: f32, y: f32, color: u32, n: i32, speed: f32) {
        for k in 0..n {
            if self.particles.len() > 90 {
                break;
            }
            self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let a = (k as f32 / n as f32) * std::f32::consts::TAU + (self.seed & 15) as f32 * 0.02;
            let sp = speed * (0.45 + ((self.seed >> 4) & 15) as f32 / 20.0);
            self.particles.push(Particle {
                x,
                y,
                vx: a.cos() * sp,
                vy: a.sin() * sp - 40.0,
                life: 0.28 + (k % 3) as f32 * 0.06,
                max: 0.36,
                color,
                r: 2.2 + (k % 3) as f32,
            });
        }
    }

    fn puff(&mut self, x: f32, y: f32, color: u32, n: i32, speed: f32) {
        self.burst(x, y, color, n, speed);
    }

    fn popup(&mut self, x: f32, y: f32, text: String, color: u32, scale: i32) {
        if self.floaters.len() > 12 {
            self.floaters.remove(0);
        }
        self.floaters.push(Floater {
            x,
            y,
            life: 0.85,
            text,
            color,
            scale,
        });
    }

    fn sound(&mut self, sfx: Sfx) {
        if self.sounds.len() < 8 {
            self.sounds.push(sfx);
        }
    }

    /// Pose a scene for stills. Does not run the simulation.
    pub fn stage(&mut self, scene: &str) {
        *self = Game::new();
        self.ai_enabled = false;
        self.fighters[0].human = true;
        match scene {
            "title" => {
                self.mode = Mode::Title;
                self.menu = 0;
                self.place_title();
                self.time = 1.2;
            }
            "fight" => {
                self.mode = Mode::Fight;
                self.round = 1;
                self.belt = belt_name(1);
                self.timer = 21.0;
                self.park();
                self.fighters[0].pips = 2;
                self.fighters[0].total_score = 1600;
                self.fighters[0].round_score = 1600;
                self.fighters[1].pips = 1;
                self.fighters[1].total_score = 400;
                self.fighters[1].round_score = 400;
                self.fighters[0].phase = 0.4;
                self.fighters[1].phase = 1.7;
                self.fighters[2].phase = 3.1;
                self.time = 2.0;
            }
            "clash" => {
                self.mode = Mode::Fight;
                self.round = 3;
                self.belt = belt_name(3);
                self.timer = 14.0;
                self.park();
                self.fighters[0].x = 340.0;
                self.fighters[0].facing = 1.0;
                self.fighters[0].action = Action::Attack(MoveId::FrontPunch);
                self.fighters[0].t = 0.12;
                self.fighters[0].pips = 3;
                self.fighters[0].total_score = 2400;
                self.fighters[1].x = 560.0;
                self.fighters[1].facing = -1.0;
                self.fighters[1].action = Action::Attack(MoveId::FaceKick);
                self.fighters[1].t = 0.18;
                self.fighters[1].pips = 2;
                self.fighters[1].total_score = 1600;
                self.fighters[2].x = 860.0;
                self.fighters[2].facing = -1.0;
                self.fighters[2].action = Action::Block;
                self.fighters[2].pips = 1;
                self.fighters[2].total_score = 800;
                self.time = 4.0;
            }
            "specials" => {
                self.mode = Mode::Fight;
                self.round = 4;
                self.belt = belt_name(4);
                self.timer = 18.0;
                self.park();
                self.fighters[0].x = 640.0;
                self.fighters[0].y = -46.0;
                self.fighters[0].facing = 1.0;
                self.fighters[0].action = Action::Attack(MoveId::DoubleKick);
                self.fighters[0].t = 0.22;
                self.fighters[1].x = 250.0;
                self.fighters[1].y = -58.0;
                self.fighters[1].facing = 1.0;
                self.fighters[1].action = Action::Backflip;
                self.fighters[1].t = 0.26;
                self.fighters[2].x = 1040.0;
                self.fighters[2].y = -36.0;
                self.fighters[2].facing = -1.0;
                self.fighters[2].action = Action::Attack(MoveId::FlyingKick);
                self.fighters[2].t = 0.24;
                self.time = 3.0;
            }
            "sunset" => {
                self.mode = Mode::Fight;
                self.round = 8;
                self.belt = belt_name(8);
                self.timer = 6.0;
                self.park();
                self.fighters[0].pips = 5;
                self.fighters[0].total_score = 18400;
                self.fighters[0].x = 420.0;
                self.fighters[0].action = Action::Idle;
                self.fighters[1].x = 620.0;
                self.fighters[1].action = Action::Down;
                self.fighters[1].t = 0.35;
                self.fighters[1].pips = 2;
                self.fighters[1].total_score = 9600;
                self.fighters[2].pips = 3;
                self.fighters[2].total_score = 12100;
                self.fighters[2].x = 980.0;
                self.time = 8.0;
                self.popup(620.0, GROUND - 180.0, "+1000".to_string(), 0xFFE08A, 3);
            }
            "bonus" => {
                self.mode = Mode::Bonus;
                self.round = 2;
                self.belt = belt_name(2);
                self.timer = 30.0;
                self.park();
                self.fighters[0].human = true;
                self.fighters[0].x = 480.0;
                self.fighters[0].facing = 1.0;
                self.bonus = Bonus::start(2, 0, 3);
                self.bonus.remaining = 6;
                self.bonus.deflected = 3;
                self.bonus.balls = vec![
                    Ball { x: 480.0, y: 250.0, vx: 40.0, vy: 220.0, alive: true },
                    Ball { x: 760.0, y: 160.0, vx: -80.0, vy: 180.0, alive: true },
                    Ball { x: 300.0, y: 390.0, vx: 60.0, vy: 200.0, alive: true },
                ];
                self.note = "DEFLECT EVERY BALL".to_string();
                self.time = 2.4;
            }
            "result" => {
                self.mode = Mode::Resolve;
                self.round = 5;
                self.belt = belt_name(5);
                self.timer = 8.0;
                self.park();
                self.fighters[0].pips = 6;
                self.fighters[0].round_score = 5200;
                self.fighters[0].total_score = 14800;
                self.fighters[1].pips = 2;
                self.fighters[1].round_score = 800;
                self.fighters[1].total_score = 6400;
                self.fighters[2].pips = 3;
                self.fighters[2].round_score = 1600;
                self.fighters[2].total_score = 9100;
                self.ranks = [0, 2, 1];
                self.time_bonus = 800;
                self.note = "WHITE TAKES THE ROUND".to_string();
                self.resolve_t = 0.4;
                self.time = 5.0;
            }
            _ => {}
        }
    }
}

fn interruptible(action: Action) -> bool {
    matches!(action, Action::Idle | Action::Walk | Action::Block)
}

fn attack_of(action: Action) -> Option<MoveId> {
    match action {
        Action::Attack(id) => Some(id),
        _ => None,
    }
}

fn height_ok(height: u8, y: f32) -> bool {
    match height {
        0 => y > -36.0,
        1 => y > -78.0,
        _ => y > -120.0,
    }
}

fn intent_from_pad(pad: Pad, facing: f32) -> Intent {
    if pad.x == 0 && pad.y == 0 {
        return Intent::None;
    }
    let forward = if facing >= 0.0 { pad.x > 0 } else { pad.x < 0 };
    let back = if facing >= 0.0 { pad.x < 0 } else { pad.x > 0 };
    let up = pad.y < 0;
    let down = pad.y > 0;
    if pad.fire {
        if up && forward {
            return Intent::Attack(MoveId::Headbutt);
        }
        if up && back {
            return Intent::Attack(MoveId::DoubleKick);
        }
        if up {
            return Intent::Attack(MoveId::FlyingKick);
        }
        if down && forward {
            return Intent::Attack(MoveId::FaceKick);
        }
        if down && back {
            return Intent::Attack(MoveId::BackStepKick);
        }
        if down {
            return Intent::Attack(MoveId::ReverseSweep);
        }
        if forward {
            return Intent::Attack(MoveId::StomachKick);
        }
        if back {
            return Intent::Flip;
        }
    } else {
        if up && forward {
            return Intent::Attack(MoveId::FrontPunch);
        }
        if up && back {
            return Intent::Attack(MoveId::ReversePunch);
        }
        if up {
            return Intent::Jump;
        }
        if down && forward {
            return Intent::Attack(MoveId::ShinKick);
        }
        if down && back {
            return Intent::Attack(MoveId::CrouchPunch);
        }
        if down {
            return Intent::Attack(MoveId::Sweep);
        }
        if forward {
            return Intent::Walk;
        }
        if back {
            return Intent::Block;
        }
    }
    Intent::None
}

pub fn fighter_name(i: usize) -> &'static str {
    ["WHITE", "RED", "BLUE"][i]
}

pub fn belt_name(round: i32) -> String {
    const NAMES: [&str; 8] = [
        "WHITE BELT",
        "YELLOW BELT",
        "ORANGE BELT",
        "GREEN BELT",
        "BLUE BELT",
        "PURPLE BELT",
        "BROWN BELT",
        "BLACK BELT",
    ];
    let n = round.max(1);
    if n <= 8 {
        NAMES[(n - 1) as usize].to_string()
    } else {
        format!("BLACK BELT  DAN {}", n - 7)
    }
}

pub fn jacket(i: usize) -> (u32, u32, u32, u32) {
    // gi, gi shadow, skin, hair
    match i {
        0 => (0xF7F4EF, 0xD5D0C6, 0xE0A56A, 0x241C16),
        1 => (0xC42B34, 0x8A1C24, 0xC68642, 0x1A120E),
        _ => (0x2A4C86, 0x183258, 0x8D5524, 0x140E0C),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(g: &mut Game, n: i32) {
        for _ in 0..n {
            g.update([Pad::default(); 2], 1.0 / 60.0);
        }
    }

    #[test]
    fn front_punch_scores_two_pips_and_drops_them() {
        let mut g = Game::fresh_fight();
        g.fighters[0].x = 400.0;
        g.fighters[0].facing = 1.0;
        g.fighters[1].x = 480.0;
        g.fighters[1].facing = -1.0;
        g.fighters[2].x = 1100.0;
        g.begin_attack(0, MoveId::FrontPunch);
        step(&mut g, 40);
        assert_eq!(g.fighters[0].pips, 2, "pips");
        assert_eq!(g.fighters[0].round_score, 800, "score");
        assert!(
            matches!(g.fighters[1].action, Action::Fall | Action::Down | Action::Rise),
            "defender should be down, was {:?}",
            g.fighters[1].action
        );
    }

    #[test]
    fn a_hit_from_behind_is_worth_half() {
        let mut g = Game::fresh_fight();
        g.fighters[0].x = 400.0;
        g.fighters[0].facing = 1.0;
        g.fighters[1].x = 480.0;
        g.fighters[1].facing = 1.0;
        g.fighters[2].x = 1100.0;
        g.begin_attack(0, MoveId::FrontPunch);
        step(&mut g, 40);
        assert_eq!(g.fighters[0].pips, 1);
        assert_eq!(g.fighters[0].round_score, 400);
    }

    #[test]
    fn block_stops_a_high_punch() {
        let mut g = Game::fresh_fight();
        g.fighters[0].x = 400.0;
        g.fighters[0].facing = 1.0;
        g.fighters[1].x = 480.0;
        g.fighters[1].facing = -1.0;
        g.fighters[1].action = Action::Block;
        g.fighters[2].x = 1100.0;
        g.begin_attack(0, MoveId::FrontPunch);
        step(&mut g, 30);
        assert_eq!(g.fighters[0].pips, 0);
        assert_eq!(g.fighters[1].action, Action::Block);
    }

    #[test]
    fn sweep_misses_someone_in_the_air() {
        let mut g = Game::fresh_fight();
        g.fighters[0].x = 400.0;
        g.fighters[0].facing = 1.0;
        g.fighters[1].x = 490.0;
        g.fighters[1].facing = -1.0;
        g.fighters[1].y = -100.0;
        g.fighters[2].x = 1100.0;
        g.begin_attack(0, MoveId::Sweep);
        step(&mut g, 20);
        assert_eq!(g.fighters[0].pips, 0);
        assert!(!matches!(g.fighters[1].action, Action::Fall | Action::Down));
    }

    #[test]
    fn double_kick_scores_on_both_sides() {
        let mut g = Game::fresh_fight();
        g.fighters[0].x = 640.0;
        g.fighters[0].facing = 1.0;
        g.fighters[1].x = 550.0;
        g.fighters[1].facing = 1.0;
        g.fighters[2].x = 730.0;
        g.fighters[2].facing = -1.0;
        g.begin_attack(0, MoveId::DoubleKick);
        step(&mut g, 30);
        assert_eq!(g.fighters[0].pips, 4, "both opponents are two pips each");
        assert_eq!(g.fighters[0].round_score, 2000);
        assert!(matches!(g.fighters[1].action, Action::Fall | Action::Down | Action::Rise));
        assert!(matches!(g.fighters[2].action, Action::Fall | Action::Down | Action::Rise));
    }

    #[test]
    fn backflip_is_safe() {
        let mut g = Game::fresh_fight();
        g.fighters[0].x = 400.0;
        g.fighters[0].facing = 1.0;
        g.fighters[1].x = 490.0;
        g.fighters[1].facing = -1.0;
        g.fighters[1].action = Action::Backflip;
        g.fighters[1].i_frames = 0.5;
        g.fighters[2].x = 1100.0;
        g.begin_attack(0, MoveId::FrontPunch);
        step(&mut g, 20);
        assert_eq!(g.fighters[0].pips, 0);
    }

    #[test]
    fn last_human_ends_the_match_and_a_cpu_does_not() {
        let mut g = Game::fresh_fight();
        g.fighters[0].human = true;
        g.players = 1;
        g.fighters[0].pips = 0;
        g.fighters[1].pips = 4;
        g.fighters[2].pips = 2;
        g.timer = 0.01;
        g.update([Pad::default(); 2], 1.0 / 60.0);
        assert_eq!(g.mode, Mode::Resolve);
        assert_eq!(g.ranks[2], 0, "white is last");
        g.leave_resolve();
        assert_eq!(g.mode, Mode::GameOver);

        let mut g = Game::fresh_fight();
        g.fighters[0].human = true;
        g.players = 1;
        g.fighters[0].pips = 4;
        g.fighters[0].total_score = 2000;
        g.fighters[1].pips = 0;
        g.fighters[1].total_score = 900;
        g.fighters[2].pips = 2;
        g.timer = 0.01;
        g.update([Pad::default(); 2], 1.0 / 60.0);
        assert_eq!(g.ranks[0], 0);
        g.leave_resolve();
        assert_eq!(g.mode, Mode::Intro);
        assert_eq!(g.round, 2);
        assert_eq!(g.fighters[0].total_score, 2000);
        assert_eq!(g.fighters[1].total_score, 0, "new challenger");
        assert!(!g.fighters[1].human);
    }

    #[test]
    fn reaching_six_pays_the_time_bonus_and_stops_the_clock() {
        let mut g = Game::fresh_fight();
        g.fighters[0].human = true;
        g.timer = 12.2;
        g.fighters[0].x = 400.0;
        g.fighters[0].facing = 1.0;
        g.fighters[0].pips = 4;
        g.fighters[1].x = 480.0;
        g.fighters[1].facing = -1.0;
        g.fighters[2].x = 1100.0;
        g.begin_attack(0, MoveId::FrontPunch);
        step(&mut g, 40);
        assert_eq!(g.fighters[0].pips, 6);
        assert_eq!(g.mode, Mode::Resolve);
        assert_eq!(g.time_bonus, 1300);
        assert!(g.fighters[0].total_score >= 800 + 1300);
    }

    #[test]
    fn even_rounds_open_the_bonus_stage() {
        let mut g = Game::fresh_fight();
        g.round = 2;
        g.fighters[0].human = true;
        g.players = 1;
        g.fighters[0].pips = 3;
        g.fighters[1].pips = 1;
        g.fighters[2].pips = 2;
        g.ranks = g.compute_ranks();
        g.leave_resolve();
        assert_eq!(g.mode, Mode::Bonus);
        assert_eq!(g.bonus.player, 0);
    }

    #[test]
    fn ai_bout_stays_in_the_arena_and_someone_scores() {
        let mut g = Game::fresh_fight();
        g.ai_enabled = true;
        for f in &mut g.fighters {
            f.human = false;
        }
        for _ in 0..(20 * 60) {
            g.update([Pad::default(); 2], 1.0 / 60.0);
            for f in &g.fighters {
                assert!(f.x.is_finite() && f.y.is_finite(), "non-finite body");
                assert!((LEFT..=RIGHT).contains(&f.x), "x {}", f.x);
                assert!(f.y <= 1.0 && f.y > -420.0, "y {}", f.y);
                assert!((0..=6).contains(&f.pips));
            }
        }
        let scored: i32 = g.fighters.iter().map(|f| f.total_score).sum();
        assert!(scored > 0, "the computers should actually fight, mode {:?}", g.mode);
    }

    #[test]
    fn held_forward_walks_and_back_blocks_relative_to_facing() {
        let mut g = Game::fresh_fight();
        g.fighters[0].human = true;
        g.fighters[0].x = 400.0;
        g.fighters[0].facing = 1.0;
        let pad = Pad { x: 1, y: 0, fire: false, start: false, y_edge: 0 };
        g.update([pad, Pad::default()], 0.1);
        assert!(g.fighters[0].x > 400.0);
        assert_eq!(g.fighters[0].action, Action::Walk);
        g.fighters[0].action = Action::Idle;
        let pad = Pad { x: -1, y: 0, fire: false, start: false, y_edge: 0 };
        let x = g.fighters[0].x;
        g.update([pad, Pad::default()], 0.1);
        assert!(g.fighters[0].x < x);
        assert_eq!(g.fighters[0].action, Action::Block);
        assert_eq!(g.fighters[0].facing, 1.0, "blocking does not turn you");
    }
}
