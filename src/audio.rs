//! Original music and hits. Synthesized when the game starts, not taken from IK+.

use crate::sim::{Mode, Sfx};
use rodio::buffer::SamplesBuffer;
use rodio::source::Source;
use rodio::{nz, DeviceSinkBuilder, MixerDeviceSink, Player};
use std::num::NonZeroU32;

const SR: u32 = 22_050;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Track {
    Title,
    Fight,
    Bonus,
    Over,
}

pub struct Audio {
    sink: MixerDeviceSink,
    music: Player,
    track: Option<Track>,
    paused: bool,
    muted: bool,
    title: Vec<f32>,
    fight: Vec<f32>,
    bonus: Vec<f32>,
    over: Vec<f32>,
    fx: [Vec<f32>; 12],
}

impl Audio {
    pub fn open() -> Option<Self> {
        let mut sink = DeviceSinkBuilder::open_default_sink().ok()?;
        sink.log_on_drop(false);
        let music = Player::connect_new(sink.mixer());
        music.set_volume(0.8);
        Some(Self {
            sink,
            music,
            track: None,
            paused: false,
            muted: false,
            title: tune(76.0, &TITLE, Wave::Tri),
            fight: tune(112.0, &FIGHT, Wave::Pulse),
            bonus: tune(128.0, &BONUS, Wave::Square),
            over: sting(),
            fx: [
                punch(),
                kick(),
                sweep_sfx(),
                whoosh(),
                hit(),
                block(),
                fall(),
                cheer(),
                ui(),
                deflect(),
                miss(),
                kiai(),
            ],
        })
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        self.music.set_volume(if self.muted { 0.0 } else { 0.8 });
    }

    pub fn sync(&mut self, mode: Mode, paused_mode: Mode, cues: Vec<Sfx>) {
        if !self.muted {
            for cue in cues {
                self.play(cue);
            }
        }
        let track = match mode {
            Mode::Title => Track::Title,
            Mode::Bonus => Track::Bonus,
            Mode::GameOver => Track::Over,
            Mode::Pause => match paused_mode {
                Mode::Bonus => Track::Bonus,
                Mode::Title => Track::Title,
                _ => Track::Fight,
            },
            _ => Track::Fight,
        };
        if self.track != Some(track) {
            self.track = Some(track);
            self.music.clear();
            self.music.play();
            match track {
                Track::Title => self.music.append(looped(&self.title)),
                Track::Fight => self.music.append(looped(&self.fight)),
                Track::Bonus => self.music.append(looped(&self.bonus)),
                Track::Over => self.music.append(once(&self.over)),
            }
            self.paused = false;
        }
        let want_pause = mode == Mode::Pause;
        if want_pause && !self.paused {
            self.music.pause();
            self.paused = true;
        } else if !want_pause && self.paused {
            self.music.play();
            self.paused = false;
        }
    }

    fn play(&self, cue: Sfx) {
        let i = match cue {
            Sfx::Punch => 0,
            Sfx::Kick => 1,
            Sfx::Sweep => 2,
            Sfx::Whoosh => 3,
            Sfx::Hit => 4,
            Sfx::Block => 5,
            Sfx::Fall => 6,
            Sfx::Cheer => 7,
            Sfx::Ui => 8,
            Sfx::Deflect => 9,
            Sfx::Miss => 10,
            Sfx::Kiai => 11,
        };
        self.sink.mixer().add(once(&self.fx[i]));
    }
}

fn once(data: &[f32]) -> SamplesBuffer {
    SamplesBuffer::new(nz!(1), NonZeroU32::new(SR).expect("sample rate"), data.to_vec())
}

fn looped(data: &[f32]) -> impl Source<Item = f32> + Send + 'static {
    once(data).repeat_infinite()
}

#[derive(Clone, Copy)]
enum Wave {
    Pulse,
    Tri,
    Square,
}

/// (lead midi, bass midi, hat). Zero midi is a rest. Each row is an eighth note.
const FIGHT: [(u8, u8, bool); 32] = [
    (74, 38, true), (0, 0, false), (69, 0, true), (67, 0, false),
    (65, 38, true), (0, 0, true), (62, 0, false), (67, 45, true),
    (69, 45, true), (72, 0, false), (74, 0, true), (0, 0, false),
    (72, 43, true), (69, 0, false), (67, 43, true), (65, 0, true),
    (62, 38, true), (65, 0, false), (67, 38, true), (69, 0, false),
    (72, 45, true), (0, 0, true), (69, 0, false), (67, 45, false),
    (65, 41, true), (62, 0, false), (65, 41, true), (67, 0, true),
    (69, 38, true), (67, 0, false), (65, 0, false), (62, 38, true),
];

const TITLE: [(u8, u8, bool); 32] = [
    (74, 38, false), (0, 0, false), (0, 0, false), (69, 0, false),
    (67, 33, false), (0, 0, false), (65, 0, false), (0, 0, false),
    (62, 38, false), (65, 0, false), (67, 0, false), (69, 0, false),
    (72, 41, false), (0, 0, false), (69, 0, false), (0, 0, false),
    (67, 36, false), (0, 0, false), (65, 0, false), (62, 33, false),
    (0, 0, false), (0, 0, false), (65, 38, false), (67, 0, false),
    (69, 41, false), (0, 0, false), (72, 0, false), (74, 0, false),
    (72, 38, false), (69, 0, false), (67, 0, false), (0, 0, false),
];

const BONUS: [(u8, u8, bool); 16] = [
    (79, 43, true), (76, 0, false), (74, 43, true), (72, 0, true),
    (74, 38, true), (76, 0, false), (79, 38, true), (0, 0, false),
    (81, 45, true), (79, 0, true), (76, 45, false), (74, 0, false),
    (76, 41, true), (74, 0, false), (72, 41, true), (0, 0, true),
];

fn tune(bpm: f32, steps: &[(u8, u8, bool)], lead: Wave) -> Vec<f32> {
    let step = 60.0 / bpm / 2.0;
    let mut buf = vec![0.0; ((steps.len() as f32 * step + 0.05) * SR as f32) as usize];
    let mut rng = 0xA341_316Cu32;
    for (i, &(lead_n, bass_n, hat)) in steps.iter().enumerate() {
        let start = (i as f32 * step * SR as f32) as usize;
        if lead_n > 0 {
            add_osc(&mut buf, start, (step * 0.9 * SR as f32) as usize, freq(lead_n), 0.15, lead);
        }
        if bass_n > 0 {
            add_osc(&mut buf, start, (step * 1.7 * SR as f32) as usize, freq(bass_n), 0.2, Wave::Tri);
        }
        if hat {
            add_noise(&mut buf, start, (0.035 * SR as f32) as usize, 0.06, &mut rng);
        }
    }
    echo(&mut buf, 0.14, 0.15);
    normalize(&mut buf, 0.6);
    buf
}

fn sting() -> Vec<f32> {
    let mut buf = vec![0.0; (1.3 * SR as f32) as usize];
    for (i, note) in [65u8, 62, 58, 53].into_iter().enumerate() {
        let start = (i as f32 * 0.2 * SR as f32) as usize;
        add_osc(&mut buf, start, (0.28 * SR as f32) as usize, freq(note), 0.22, Wave::Square);
    }
    normalize(&mut buf, 0.65);
    buf
}

fn punch() -> Vec<f32> {
    let mut buf = thump(0.07, 120.0, 0.4);
    let mut rng = 0x51u32;
    add_noise(&mut buf, 0, (0.04 * SR as f32) as usize, 0.3, &mut rng);
    normalize(&mut buf, 0.75);
    buf
}

fn kick() -> Vec<f32> {
    let mut buf = sweep(0.14, 220.0, 70.0, 0.3, Wave::Square);
    let mut rng = 0x99u32;
    add_noise(&mut buf, 0, (0.06 * SR as f32) as usize, 0.22, &mut rng);
    normalize(&mut buf, 0.75);
    buf
}

fn sweep_sfx() -> Vec<f32> {
    let mut buf = sweep(0.16, 160.0, 50.0, 0.26, Wave::Tri);
    let mut rng = 0x21u32;
    add_noise(&mut buf, 0, (0.1 * SR as f32) as usize, 0.2, &mut rng);
    normalize(&mut buf, 0.7);
    buf
}

fn whoosh() -> Vec<f32> {
    let n = (0.22 * SR as f32) as usize;
    let mut buf = vec![0.0; n];
    let mut rng = 0xABCDu32;
    add_noise(&mut buf, 0, n, 0.25, &mut rng);
    for (i, s) in buf.iter_mut().enumerate() {
        let t = i as f32 / n as f32;
        *s *= (t * (1.0 - t) * 4.0).min(1.0);
    }
    normalize(&mut buf, 0.45);
    buf
}

fn hit() -> Vec<f32> {
    let mut buf = thump(0.08, 90.0, 0.45);
    let mut rng = 7u32;
    add_noise(&mut buf, 0, (0.05 * SR as f32) as usize, 0.4, &mut rng);
    normalize(&mut buf, 0.85);
    buf
}

fn block() -> Vec<f32> {
    let mut buf = sweep(0.08, 500.0, 180.0, 0.22, Wave::Square);
    normalize(&mut buf, 0.55);
    buf
}

fn fall() -> Vec<f32> {
    sweep(0.28, 280.0, 60.0, 0.28, Wave::Square)
}

fn cheer() -> Vec<f32> {
    let n = (0.45 * SR as f32) as usize;
    let mut buf = vec![0.0; n];
    let mut rng = 0xBEEFu32;
    add_noise(&mut buf, 0, n, 0.2, &mut rng);
    for (i, s) in buf.iter_mut().enumerate() {
        let t = i as f32 / n as f32;
        *s *= (1.0 - t) * (if (i / 70) % 3 == 0 { 1.0 } else { 0.35 });
    }
    add_osc(&mut buf, 0, (0.2 * SR as f32) as usize, freq(76), 0.12, Wave::Tri);
    normalize(&mut buf, 0.55);
    buf
}

fn ui() -> Vec<f32> {
    let mut buf = vec![0.0; (0.16 * SR as f32) as usize];
    add_osc(&mut buf, 0, (0.08 * SR as f32) as usize, freq(76), 0.2, Wave::Tri);
    add_osc(&mut buf, (0.07 * SR as f32) as usize, (0.09 * SR as f32) as usize, freq(81), 0.2, Wave::Tri);
    normalize(&mut buf, 0.6);
    buf
}

fn deflect() -> Vec<f32> {
    let mut buf = sweep(0.1, 420.0, 880.0, 0.2, Wave::Square);
    normalize(&mut buf, 0.6);
    buf
}

fn miss() -> Vec<f32> {
    sweep(0.18, 200.0, 80.0, 0.22, Wave::Square)
}

fn kiai() -> Vec<f32> {
    let mut buf = sweep(0.18, 340.0, 180.0, 0.22, Wave::Square);
    add_osc(&mut buf, 0, (0.12 * SR as f32) as usize, 520.0, 0.12, Wave::Pulse);
    normalize(&mut buf, 0.7);
    buf
}

fn thump(dur: f32, freq_hz: f32, vol: f32) -> Vec<f32> {
    let mut buf = vec![0.0; (dur * SR as f32) as usize + 8];
    let n = buf.len();
    add_osc(&mut buf, 0, n, freq_hz, vol, Wave::Tri);
    buf
}

fn sweep(dur: f32, from: f32, to: f32, vol: f32, wave: Wave) -> Vec<f32> {
    let n = (dur * SR as f32) as usize;
    let mut buf = vec![0.0; n];
    let mut phase = 0.0;
    for i in 0..n {
        let t = i as f32 / n as f32;
        let f = from + (to - from) * t;
        phase = (phase + f / SR as f32) % 1.0;
        let env = (1.0 - t).powf(1.4) * (t / 0.08).min(1.0);
        buf[i] += osc(phase, wave) * env * vol;
    }
    normalize(&mut buf, 0.75);
    buf
}

fn add_osc(buf: &mut [f32], start: usize, len: usize, freq_hz: f32, vol: f32, wave: Wave) {
    let mut phase = 0.0;
    let inc = freq_hz / SR as f32;
    for i in 0..len {
        let at = start + i;
        if at >= buf.len() {
            break;
        }
        let t = i as f32 / len as f32;
        let attack = (t / 0.02).min(1.0);
        let release = ((1.0 - t) / 0.12).min(1.0);
        phase = (phase + inc) % 1.0;
        buf[at] += osc(phase, wave) * attack * release * vol;
    }
}

fn add_noise(buf: &mut [f32], start: usize, len: usize, vol: f32, rng: &mut u32) {
    for i in 0..len {
        let at = start + i;
        if at >= buf.len() {
            break;
        }
        let t = i as f32 / len as f32;
        buf[at] += noise(rng) * (1.0 - t).powi(2) * vol;
    }
}

fn osc(phase: f32, wave: Wave) -> f32 {
    match wave {
        Wave::Pulse => {
            if phase < 0.22 {
                0.65
            } else {
                -0.65
            }
        }
        Wave::Square => {
            if phase < 0.5 {
                0.55
            } else {
                -0.55
            }
        }
        Wave::Tri => {
            if phase < 0.5 {
                phase * 4.0 - 1.0
            } else {
                1.0 - (phase - 0.5) * 4.0
            }
        }
    }
}

fn noise(state: &mut u32) -> f32 {
    *state ^= state.wrapping_shl(13);
    *state ^= state.wrapping_shr(17);
    *state ^= state.wrapping_shl(5);
    (*state as f32 / u32::MAX as f32) * 2.0 - 1.0
}

fn freq(midi: u8) -> f32 {
    440.0 * 2.0_f32.powf((midi as f32 - 69.0) / 12.0)
}

fn echo(buf: &mut [f32], delay_s: f32, mix: f32) {
    let delay = (delay_s * SR as f32) as usize;
    for i in delay..buf.len() {
        buf[i] += buf[i - delay] * mix;
    }
}

fn normalize(buf: &mut [f32], peak: f32) {
    let max = buf.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if max > 0.001 {
        let g = peak / max;
        for s in buf.iter_mut() {
            *s *= g;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn music_and_hits_stay_in_range() {
        for buf in [
            tune(112.0, &FIGHT, Wave::Pulse),
            tune(76.0, &TITLE, Wave::Tri),
            tune(128.0, &BONUS, Wave::Square),
            sting(),
            punch(),
            kick(),
            kiai(),
            cheer(),
        ] {
            assert!(buf.len() > SR as usize / 30);
            let peak = buf.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
            assert!(peak > 0.15 && peak <= 1.0, "peak {peak}");
        }
    }
}
