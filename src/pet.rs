use core::ops::Range;

use crate::{
    display::{Color, Row, WIDTH, rgb},
    gestures::Gesture,
};

pub const REDRAW_X: Range<i32> = 48..418;
pub const REDRAW_Y: Range<i32> = 44..416;

const CENTER: i32 = WIDTH / 2;
const GROUND: i32 = 355;
const FRAME_MS: u64 = 48;
const SPRITE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/pet-sprite.bin"));
const SPRITE_WIDTH: i32 = SPRITE[0] as i32;
const SPRITE_HEIGHT: i32 = SPRITE[1] as i32;
const RUNS_START: usize = 2 + (SPRITE_HEIGHT as usize + 1) * 2;
const SPRITE_SCALE: i32 = 2;

const SKY: Color = rgb(15, 17, 29);
const HALO: Color = rgb(63, 57, 78);
const FLOOR: Color = rgb(30, 31, 48);
const SHADOW: Color = rgb(23, 23, 36);
const STAR: Color = rgb(249, 218, 153);
const WHITE: Color = rgb(241, 233, 250);
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
            return false;
        }
        let frame = ((now_ms - self.started) / FRAME_MS).min(duration as u64) as i32;
        if frame == self.frame {
            return false;
        }
        self.frame = frame;
        if self.frame == duration {
            self.x = self.position();
            self.action = Action::Idle;
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
        row.fill(SKY);
        disk(row, y, CENTER, 223, 145, HALO);
        disk(row, y, 338, 93, 21, STAR);
        disk(row, y, 347, 86, 17, SKY);
        for (x, star_y) in [
            (120, 113),
            (159, 79),
            (103, 184),
            (314, 146),
            (366, 166),
            (134, 255),
        ] {
            if (y - star_y).abs() <= 1 {
                row.span(x - 1, x + 2, STAR);
            }
        }

        if (340..375).contains(&y) {
            row.span(70, WIDTH - 70, FLOOR);
        }
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
        let height =
            SPRITE_HEIGHT * SPRITE_SCALE - if self.action == Action::Sleep { 38 } else { 0 };
        let top = GROUND - height - lift;
        sprite_row(row, y - top, x, height, self.faces_left);

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

        row.label(y - 29, CENTER, "Спайк", WHITE);
        let (text, color) = self.action.caption();
        row.label(y - 392, CENTER, text, color);
    }
}

fn sprite_row(row: &mut Row<'_>, y: i32, center: i32, height: i32, faces_left: bool) {
    if !(0..height).contains(&y) {
        return;
    }
    let index = (y * SPRITE_HEIGHT / height) as usize;
    let offset = |index| {
        let index = 2 + index * 2;
        RUNS_START + usize::from(u16::from_le_bytes([SPRITE[index], SPRITE[index + 1]]))
    };
    let left = center - SPRITE_WIDTH * SPRITE_SCALE / 2;
    let mut x = 0;
    for run in SPRITE[offset(index)..offset(index + 1)].chunks_exact(3) {
        let length = i32::from(run[0]);
        let color = [run[1], run[2]];
        if color != [0, 0] {
            let offset = if faces_left {
                SPRITE_WIDTH - x - length
            } else {
                x
            };
            let start = left + offset * SPRITE_SCALE;
            row.span(start, start + length * SPRITE_SCALE, color);
        }
        x += length;
    }
}

fn disk(row: &mut Row<'_>, y: i32, center: i32, cy: i32, radius: i32, color: Color) {
    let distance = y - cy;
    if distance.abs() < radius {
        let half = ((radius * radius - distance * distance) as u32).isqrt() as i32;
        row.span(center - half, center + half, color);
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
