use core::ops::Range;

use crate::{
    display::{Color, Row, WIDTH, rgb},
    gestures::Gesture,
};

pub const REDRAW_X: Range<i32> = 48..418;
pub const REDRAW_Y: Range<i32> = 44..418;

const CENTER: i32 = WIDTH / 2;
const GROUND: i32 = 355;
const FRAME_MS: u64 = 48;
const SPRITE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/pet-sprite.bin"));
const SIDE: usize = 96;
const PALETTE_BYTES: usize = 32;
#[derive(Clone, Copy)]
#[repr(usize)]
enum Pose {
    Stand,
    Breeze,
    Bashful,
    Cheer,
    Stride,
    Step,
    Airborne,
    Curled,
}
const SPRITE_SCALE: i32 = 2;

const BACKGROUND: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/moonlake.bin"));
const SHADOW: Color = rgb(23, 23, 36);
const STAR: Color = rgb(249, 218, 153);
const WHITE: Color = rgb(255, 242, 214);
const PINK: Color = rgb(246, 114, 146);

const HEART: &[u8] = &[
    0b01100110, 0b11111111, 0b11111111, 0b01111110, 0b00111100, 0b00011000,
];
const SPARK: &[u8] = &[
    0b00010000, 0b00010000, 0b00111000, 0b11111110, 0b00111000, 0b00010000, 0b00010000,
];

#[derive(Clone, Copy, PartialEq)]
enum Action {
    Idle,
    Happy,
    Petted,
    Walk,
    Jump,
    Sleep,
}

impl Action {
    fn frames(self) -> i32 {
        match self {
            Self::Idle | Self::Sleep => 0,
            Self::Happy => 15,
            Self::Petted => 25,
            Self::Walk => 14,
            Self::Jump => 16,
        }
    }

    fn caption(self) -> (&'static str, Color) {
        match self {
            Self::Idle => ("Ждёт тебя", WHITE),
            Self::Happy => ("Рад тебя видеть", PINK),
            Self::Petted => ("Мурчит", PINK),
            Self::Walk => ("Гуляет", WHITE),
            Self::Jump => ("Прыгает", STAR),
            Self::Sleep => ("Спит", WHITE),
        }
    }
}

pub struct Pet {
    x: i32,
    faces_left: bool,
    action: Action,
    started: u64,
    frame: i32,
}

impl Pet {
    pub const fn new() -> Self {
        Self {
            x: CENTER,
            faces_left: false,
            action: Action::Idle,
            started: 0,
            frame: 0,
        }
    }

    pub fn react(&mut self, gesture: Gesture, now_ms: u64) {
        self.x = self.position();
        self.started = now_ms;
        self.frame = 0;
        self.action = match gesture {
            Gesture::Tap => Action::Happy,
            Gesture::Hold => Action::Petted,
            Gesture::Up => Action::Jump,
            Gesture::Down => Action::Sleep,
            Gesture::Left | Gesture::Right => {
                self.faces_left = gesture == Gesture::Left;
                Action::Walk
            }
        };
    }

    pub fn tick(&mut self, now_ms: u64) -> bool {
        let duration = self.action.frames();
        if duration == 0 {
            let phase = match self.action {
                Action::Sleep => ((now_ms - self.started) / 800 % 2) as i32,
                _ => i32::from((now_ms - self.started) / 600 % 4 == 1),
            };
            let changed = phase != self.frame;
            self.frame = phase;
            return changed;
        }
        let frame = ((now_ms - self.started) / FRAME_MS).min(duration as u64) as i32;
        if frame == self.frame {
            return false;
        }
        self.frame = frame;
        if self.frame == duration {
            self.x = self.position();
            self.action = Action::Idle;
            self.started = now_ms;
            self.frame = 0;
        }
        true
    }

    fn position(&self) -> i32 {
        if self.action != Action::Walk {
            return self.x;
        }
        let dx = self.frame * 64 / self.action.frames();
        let dx = if self.faces_left { -dx } else { dx };
        (self.x + dx).clamp(CENTER - 64, CENTER + 64)
    }

    fn lift(&self, height: i32) -> i32 {
        let duration = self.action.frames();
        4 * self.frame * (duration - self.frame) * height / (duration * duration)
    }

    pub fn draw_row(&self, y: i32, row: &mut Row<'_>) {
        background_row(y, row);
        let x = self.position();
        let shadow_y = (y - GROUND).abs();
        if shadow_y < 7 {
            let radius = 68 - shadow_y * 5;
            row.span(x - radius, x + radius, SHADOW);
        }

        let lift = match self.action {
            Action::Happy => self.lift(23),
            Action::Petted => 2 * (self.frame % 4).min(4 - self.frame % 4),
            Action::Walk => 3 * (self.frame % 6).min(6 - self.frame % 6),
            Action::Jump => self.lift(51),
            _ => 0,
        };
        let pose = match self.action {
            Action::Idle if self.frame == 1 => Pose::Breeze,
            Action::Idle => Pose::Stand,
            Action::Happy if self.frame < 3 => Pose::Bashful,
            Action::Happy => Pose::Cheer,
            Action::Petted => Pose::Bashful,
            Action::Walk => match (self.frame / 2) % 4 {
                0 => Pose::Stride,
                2 => Pose::Step,
                _ => Pose::Stand,
            },
            Action::Jump if (3..14).contains(&self.frame) => Pose::Airborne,
            Action::Jump => Pose::Cheer,
            Action::Sleep => Pose::Curled,
        };
        let breathing = if self.action == Action::Sleep {
            self.frame * 2
        } else {
            0
        };
        let height = SIDE as i32 * SPRITE_SCALE - breathing;
        let local_y = y - (GROUND - height - lift);
        if (0..height).contains(&local_y) {
            sprite_row(
                row,
                (local_y * SIDE as i32 / height) as usize,
                x,
                pose,
                self.faces_left,
            );
        }

        match self.action {
            Action::Happy => icon(row, y, x + 57, 137 - self.frame * 3, HEART, 3, PINK),
            Action::Petted => {
                let rise = self.frame * 2;
                icon(row, y, x - 65, 157 - rise, HEART, 2, PINK);
                icon(row, y, x + 68, 177 - rise, HEART, 3, PINK);
            }
            Action::Jump => {
                icon(row, y, x - 57, 201 - lift, SPARK, 3, STAR);
                icon(row, y, x + 62, 151 - lift, SPARK, 2, STAR);
            }
            Action::Sleep => {
                row.label(y - 118, x + 63, "Z", WHITE);
                row.label(y - 100, x + 81, "Z", WHITE);
            }
            _ => {}
        }

        if (22..54).contains(&y) || (385..417).contains(&y) {
            let top = if y < 54 { 22 } else { 385 };
            let radius = if y < 54 { 53 } else { 90 };
            let inset = (9 - (y - top).min(top + 31 - y)).max(0);
            row.span(
                CENTER - radius + inset,
                CENTER + radius - inset,
                rgb(202, 160, 118),
            );
            if (top + 2..top + 30).contains(&y) {
                row.span(
                    CENTER - radius + inset + 2,
                    CENTER + radius - inset - 2,
                    rgb(49, 26, 45),
                );
            }
        }
        row.label(y - 29, CENTER, "Спайк", WHITE);
        let (text, color) = self.action.caption();
        row.label(y - 392, CENTER, text, color);
    }
}

fn sprite_row(row: &mut Row<'_>, y: usize, center: i32, pose: Pose, faces_left: bool) {
    let start = PALETTE_BYTES + (pose as usize * SIDE + y) * SIDE / 2;
    let line = &SPRITE[start..start + SIDE / 2];
    let index_at = |x: usize| {
        let pair = line[x / 2];
        if x.is_multiple_of(2) {
            pair >> 4
        } else {
            pair & 15
        }
    };
    let left = center - SIDE as i32 * SPRITE_SCALE / 2;
    let mut x = 0;
    while x < SIDE {
        let first = x;
        let index = index_at(x);
        x += 1;
        while x < SIDE && index_at(x) == index {
            x += 1;
        }
        if index != 0 {
            let offset = if faces_left { SIDE - x } else { first };
            let begin = left + offset as i32 * SPRITE_SCALE;
            let palette = usize::from(index) * 2;
            row.span(
                begin,
                begin + (x - first) as i32 * SPRITE_SCALE,
                [SPRITE[palette], SPRITE[palette + 1]],
            );
        }
    }
}

fn background_row(y: i32, row: &mut Row<'_>) {
    let index = 512 + y as usize / 2 * 2;
    let offset = |i: usize| usize::from(u16::from_le_bytes([BACKGROUND[i], BACKGROUND[i + 1]]));
    let mut x = 0;
    for run in BACKGROUND[offset(index)..offset(index + 2)].chunks_exact(2) {
        let next = x + 2 * i32::from(run[0]);
        let i = usize::from(run[1]) * 2;
        row.span(x, next, [BACKGROUND[i], BACKGROUND[i + 1]]);
        x = next;
    }
}

fn icon(row: &mut Row<'_>, y: i32, x: i32, top: i32, bits: &[u8], scale: i32, color: Color) {
    if let Some(&line) = (y >= top)
        .then(|| (y - top) / scale)
        .and_then(|i| bits.get(i as usize))
    {
        for bit in 0..8 {
            if line & (128 >> bit) != 0 {
                let left = x + (bit - 4) * scale;
                row.span(left, left + scale, color);
            }
        }
    }
}
