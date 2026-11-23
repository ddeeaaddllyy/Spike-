use embedded_graphics::geometry::Point;

const SWIPE_DISTANCE: i32 = 34;
const TAP_DISTANCE: i32 = 12;
const HOLD_MS: u64 = 550;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Gesture {
    Tap,
    Hold,
    Left,
    Right,
    Up,
    Down,
}

struct Contact {
    origin: Point,
    started: u64,
    travel: i32,
    fired: bool,
}

#[derive(Default)]
pub struct Gestures {
    contact: Option<Contact>,
}

impl Gestures {
    pub fn clear(&mut self) {
        self.contact = None;
    }

    pub fn sample(&mut self, position: Option<Point>, now_ms: u64) -> Option<Gesture> {
        let Some(position) = position else {
            let contact = self.contact.take()?;
            if contact.fired || contact.travel > TAP_DISTANCE {
                return None;
            }
            return Some(if now_ms - contact.started >= HOLD_MS {
                Gesture::Hold
            } else {
                Gesture::Tap
            });
        };

        let contact = self.contact.get_or_insert(Contact {
            origin: position,
            started: now_ms,
            travel: 0,
            fired: false,
        });
        if contact.fired {
            return None;
        }

        let delta = position - contact.origin;
        let (x, y) = (delta.x.abs(), delta.y.abs());
        contact.travel = contact.travel.max(x.max(y));
        let gesture = if x >= SWIPE_DISTANCE && x >= y {
            if delta.x < 0 {
                Gesture::Left
            } else {
                Gesture::Right
            }
        } else if y >= SWIPE_DISTANCE {
            if delta.y < 0 {
                Gesture::Up
            } else {
                Gesture::Down
            }
        } else if contact.travel <= TAP_DISTANCE && now_ms - contact.started >= HOLD_MS {
            Gesture::Hold
        } else {
            return None;
        };
        contact.fired = true;
        Some(gesture)
    }
}
