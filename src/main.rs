mod audio;
mod render;
mod sim;

use audio::Audio;
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use render::{Frame, Renderer};
use sim::{Game, Pad, SCREEN_H, SCREEN_W};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--shots") {
        render_shots();
        return;
    }
    let mut window = Window::new(
        "IK+",
        SCREEN_W as usize,
        SCREEN_H as usize,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    )
    .expect("could not open a window");
    window.set_target_fps(60);
    let mut game = Game::new();
    let mut renderer = Renderer::new();
    let mut frame = Frame::new();
    let mut audio = Audio::open();
    if audio.is_none() {
        eprintln!("audio output is unavailable; the game will run silent");
    }
    let mut prev = std::time::Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();
        let dt = (now - prev).as_secs_f32().clamp(1.0 / 240.0, 1.0 / 20.0);
        prev = now;
        if window.is_key_pressed(Key::F1, KeyRepeat::No) {
            game.help = !game.help;
        }
        if window.is_key_pressed(Key::P, KeyRepeat::No) {
            game.toggle_pause();
        }
        if window.is_key_pressed(Key::M, KeyRepeat::No) {
            if let Some(out) = audio.as_mut() {
                out.toggle_mute();
            }
        }
        if window.is_key_pressed(Key::Minus, KeyRepeat::No) {
            game.nudge_speed(-1);
        }
        if window.is_key_pressed(Key::Equal, KeyRepeat::No) {
            game.nudge_speed(1);
        }
        if game.mode == sim::Mode::Title {
            if window.is_key_pressed(Key::Key1, KeyRepeat::No) {
                game.start_match(1);
            }
            if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
                game.start_match(2);
            }
        }
        let pads = [pad_player1(&window), pad_player2(&window)];
        game.update(pads, dt);
        let cues = game.take_sounds();
        if let Some(out) = audio.as_mut() {
            out.sync(game.mode, game.paused_mode, cues);
        }
        renderer.draw(&game, &mut frame);
        if window
            .update_with_buffer(&frame.px, SCREEN_W as usize, SCREEN_H as usize)
            .is_err()
        {
            break;
        }
    }
}

fn pad_player1(window: &Window) -> Pad {
    read_pad(
        window,
        Key::Left,
        Key::Right,
        Key::Up,
        Key::Down,
        &[Key::Space, Key::Z, Key::RightShift],
    )
}

fn pad_player2(window: &Window) -> Pad {
    read_pad(
        window,
        Key::A,
        Key::D,
        Key::W,
        Key::S,
        &[Key::F, Key::LeftShift, Key::Q],
    )
}

fn read_pad(window: &Window, left: Key, right: Key, up: Key, down: Key, fire: &[Key]) -> Pad {
    let x = axis(window, left, right);
    let y = axis(window, up, down);
    let y_edge = if window.is_key_pressed(up, KeyRepeat::No) && !window.is_key_pressed(down, KeyRepeat::No) {
        -1
    } else if window.is_key_pressed(down, KeyRepeat::No) && !window.is_key_pressed(up, KeyRepeat::No) {
        1
    } else {
        0
    };
    Pad {
        x,
        y,
        fire: fire.iter().any(|k| window.is_key_down(*k)),
        start: window.is_key_pressed(Key::Enter, KeyRepeat::No),
        y_edge,
    }
}

fn axis(window: &Window, neg: Key, pos: Key) -> i8 {
    let n = window.is_key_down(neg);
    let p = window.is_key_down(pos);
    if n == p {
        0
    } else if p {
        1
    } else {
        -1
    }
}

fn render_shots() {
    std::fs::create_dir_all("shots").expect("shots dir");
    let mut renderer = Renderer::new();
    let mut frame = Frame::new();
    let scenes = [
        ("title", "01_title.png"),
        ("fight", "02_fight.png"),
        ("clash", "03_clash.png"),
        ("specials", "04_specials.png"),
        ("sunset", "05_sunset.png"),
        ("bonus", "06_bonus.png"),
        ("result", "07_round.png"),
    ];
    for (scene, name) in scenes {
        let mut game = Game::new();
        game.stage(scene);
        renderer.draw(&game, &mut frame);
        save_png(&format!("shots/{name}"), &frame);
        println!("wrote shots/{name}");
    }
}

fn save_png(path: &str, frame: &Frame) {
    let mut img = image::RgbaImage::new(frame.w as u32, frame.h as u32);
    for y in 0..frame.h {
        for x in 0..frame.w {
            let p = frame.px[(y * frame.w + x) as usize];
            img.put_pixel(x as u32, y as u32, image::Rgba([(p >> 16) as u8, (p >> 8) as u8, p as u8, 255]));
        }
    }
    img.save(path).expect("save png");
}
