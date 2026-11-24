#![no_std]
#![no_main]

mod display;
mod gestures;
mod pet;
mod topics;

use core::{cell::RefCell, panic::PanicInfo};

use critical_section::Mutex;
use embedded_graphics::geometry::Point;
use esp_hal::{
    Blocking,
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    handler,
    i2c::master::{Config as I2cConfig, Error as I2cError, I2c},
    interrupt::wait_for_interrupt,
    spi::master::{Config as SpiConfig, Spi},
    time::{Duration, Instant, Rate},
    timer::{PeriodicTimer, timg::TimerGroup},
    usb::usb_serial_jtag::UsbSerialJtag,
};
use font8x8::legacy::BASIC_LEGACY;

use display::{Display, HEIGHT, Row, WIDTH};
use gestures::{Gesture, Gestures};
use pet::Pet;
use topics::Topics;

esp_bootloader_esp_idf::esp_app_desc!();

const MESSAGE: &[u8] = b"hello everynwan";
const GLYPH: i32 = 8;
const SCALE: i32 = 3;
const TEXT_X: i32 = (WIDTH - MESSAGE.len() as i32 * GLYPH * SCALE) / 2;
const TEXT_Y: i32 = (HEIGHT - GLYPH * SCALE) / 2;

const SAMPLE_PERIOD: Duration = Duration::from_millis(8);
const BUTTON_DEBOUNCE: Duration = Duration::from_millis(24);
const BOOT_HOLD: Duration = Duration::from_millis(900);
const SLEEP_OPTIONS: [(u64, &str); 3] = [(30, "30 с"), (60, "60 с"), (120, "120 с")];
const TOUCH_TIMEOUT: Duration = Duration::from_millis(90);
const TOUCH_ADDRESS: u8 = 0x15;
const TOUCH_POINTS_REGISTER: u8 = 0x02;

static INPUT_TIMER: Mutex<RefCell<Option<PeriodicTimer<'static, Blocking>>>> =
    Mutex::new(RefCell::new(None));

#[esp_hal::main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let delay = Delay::new();

    // GPIO18 latches the board's power on when running from a battery.
    let _power_hold = Output::new(peripherals.GPIO18, Level::High, OutputConfig::default());
    let mut reset = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());
    let mut touch_reset = Output::new(peripherals.GPIO7, Level::Low, OutputConfig::default());
    delay.delay_millis(10);
    reset.set_high();
    touch_reset.set_high();
    delay.delay_millis(150);

    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default().with_frequency(Rate::from_mhz(40)),
    )
    .unwrap()
    .with_cs(peripherals.GPIO10)
    .with_sck(peripherals.GPIO11)
    .with_sio0(peripherals.GPIO12)
    .with_sio1(peripherals.GPIO13)
    .with_sio2(peripherals.GPIO14)
    .with_sio3(peripherals.GPIO15)
    .with_dma(peripherals.DMA_CH0);
    let mut display = Display::new(spi, &delay).draw(draw_greeting).show();
    let mut usb = UsbSerialJtag::new(peripherals.USB_DEVICE);
    report(&mut usb, b"AMOLED frame sent\r\n");

    let boot_pin = Input::new(
        peripherals.GPIO0,
        InputConfig::default().with_pull(Pull::Up),
    );
    let pwr_pin = Input::new(
        peripherals.GPIO17,
        InputConfig::default().with_pull(Pull::Up),
    );
    let (mut boot, mut pwr) = (Button::new(boot_pin), Button::new(pwr_pin));
    let mut i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
        .unwrap()
        .with_sda(peripherals.GPIO47)
        .with_scl(peripherals.GPIO48);

    let mut timer = PeriodicTimer::new(TimerGroup::new(peripherals.TIMG0).timer0);
    timer.set_interrupt_handler(sample_tick);
    critical_section::with(|cs| {
        timer.start(SAMPLE_PERIOD).unwrap();
        timer.listen();
        INPUT_TIMER.borrow_ref_mut(cs).replace(timer);
    });

    let mut screen = Screen::Greeting;
    let mut topics = Topics::new();
    let mut pet = Pet::new();
    let mut gestures = Gestures::default();
    let mut last_touch = Instant::now();
    let mut last_input = Instant::now();
    let mut boot_since = None;
    let mut sleep_choice = 1;
    let mut settings_contact = false;
    loop {
        let (boot_pressed, pwr_pressed) = (boot.pressed(), pwr.pressed());
        if boot_pressed {
            boot_since = Some(boot.changed);
        }
        let next = if pwr_pressed {
            boot_since = None;
            Some(Screen::Pet)
        } else if let Some(start) = boot_since {
            if !boot.stable || (boot.raw && start.elapsed() >= BOOT_HOLD) {
                boot_since = None;
                Some(if boot.stable || boot.changed - start >= BOOT_HOLD {
                    Screen::Topics
                } else {
                    Screen::Settings
                })
            } else {
                None
            }
        } else {
            None
        };
        if let Some(next) = next {
            screen = if next == screen {
                Screen::Greeting
            } else {
                next
            };
            settings_contact = true;
            topics.close_note();
            topics.cancel_contact();
            gestures.clear();
            let _ = pet.tick(Instant::now().duration_since_epoch().as_millis());
            display = screen.draw(display, &topics, &pet, sleep_choice);
            report(&mut usb, screen.message());
        }

        let now = Instant::now();
        let now_ms = now.duration_since_epoch().as_millis();
        let sample = match touch_position(&mut i2c) {
            Ok(position) => {
                last_touch = now;
                Some(position)
            }
            Err(_) if last_touch.elapsed() >= TOUCH_TIMEOUT => Some(None),
            Err(_) => None,
        };

        if boot.stable || pwr.stable || matches!(sample, Some(Some(_))) {
            last_input = now;
        } else if last_input.elapsed() >= Duration::from_secs(SLEEP_OPTIONS[sleep_choice].0) {
            display = display.sleep();
            report(&mut usb, b"Display asleep\r\n");
            settings_contact = true;
            topics.cancel_contact();
            gestures.clear();
            // Sleep-in needs time to settle before sending sleep-out.
            let asleep_since = Instant::now();
            loop {
                wait_for_interrupt();
                let buttons = (boot.pressed(), pwr.pressed());
                if buttons.0 || buttons.1 {
                    break;
                }
            }
            while asleep_since.elapsed() < Duration::from_millis(120) {
                wait_for_interrupt();
            }
            display = display.wake(&delay);
            pet.tick(Instant::now().duration_since_epoch().as_millis());
            display = screen.draw(display, &topics, &pet, sleep_choice).show();
            last_input = Instant::now();
            last_touch = last_input;
            report(&mut usb, b"Display awake\r\n");
            continue;
        }

        match screen {
            Screen::Topics => {
                if sample.is_some_and(|position| {
                    let tap = gestures.sample(position, now_ms) == Some(Gesture::Tap);
                    topics.touch(position, tap)
                }) {
                    display = display.draw(|y, row| topics.draw_row(y, row));
                }
            }
            Screen::Pet => {
                let gesture = sample.and_then(|position| gestures.sample(position, now_ms));
                if let Some(gesture) = gesture {
                    pet.react(gesture, now_ms);
                    report(&mut usb, gesture_message(gesture));
                }
                if gesture.is_some() || pet.tick(now_ms) {
                    display = display
                        .draw_region(pet::REDRAW_X, pet::REDRAW_Y, |y, row| pet.draw_row(y, row));
                }
            }
            Screen::Settings => {
                if let Some(position) = sample {
                    if !settings_contact
                        && let Some(p) = position
                        && (127..337).contains(&p.x)
                        && (206..258).contains(&p.y)
                    {
                        sleep_choice = ((p.x - 127) / 70) as usize;
                        display = screen.draw(display, &topics, &pet, sleep_choice);
                    }
                    settings_contact = position.is_some();
                }
            }
            Screen::Greeting => {}
        }
        wait_for_interrupt();
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Greeting,
    Settings,
    Topics,
    Pet,
}

impl Screen {
    fn draw(self, display: Display, topics: &Topics, pet: &Pet, sleep_choice: usize) -> Display {
        display.draw(|y, row| match self {
            Screen::Settings => draw_settings(y, row, sleep_choice),
            Screen::Greeting => draw_greeting(y, row),
            Screen::Topics => topics.draw_row(y, row),
            Screen::Pet => pet.draw_row(y, row),
        })
    }

    fn message(self) -> &'static [u8] {
        match self {
            Self::Settings => b"Settings open\r\n",
            Self::Greeting => b"Greeting open\r\n",
            Self::Topics => b"Topic map open\r\n",
            Self::Pet => b"Pet open\r\n",
        }
    }
}

fn gesture_message(gesture: Gesture) -> &'static [u8] {
    match gesture {
        Gesture::Tap => b"Pet: tap\r\n",
        Gesture::Hold => b"Pet: hold\r\n",
        Gesture::Left => b"Pet: left\r\n",
        Gesture::Right => b"Pet: right\r\n",
        Gesture::Up => b"Pet: up\r\n",
        Gesture::Down => b"Pet: down\r\n",
    }
}

// The timer wakes the CPU for input samples, leaving it idle between updates.
#[handler]
fn sample_tick() {
    critical_section::with(|cs| {
        if let Some(timer) = INPUT_TIMER.borrow_ref_mut(cs).as_mut() {
            timer.clear_interrupt();
        }
    });
}

struct Button {
    pin: Input<'static>,
    raw: bool,
    stable: bool,
    changed: Instant,
}

impl Button {
    fn new(pin: Input<'static>) -> Self {
        let raw = pin.is_low();
        Self {
            pin,
            raw,
            stable: raw,
            changed: Instant::now(),
        }
    }

    fn pressed(&mut self) -> bool {
        let down = self.pin.is_low();
        if down != self.raw {
            self.raw = down;
            self.changed = Instant::now();
        }
        if self.raw != self.stable && self.changed.elapsed() >= BUTTON_DEBOUNCE {
            self.stable = self.raw;
            return self.stable;
        }
        false
    }
}

fn touch_position(i2c: &mut I2c<'_, Blocking>) -> Result<Option<Point>, I2cError> {
    let mut data = [0; 5];
    i2c.write_read(TOUCH_ADDRESS, &[TOUCH_POINTS_REGISTER], &mut data)?;
    if data[0] == 0 || data[1] >> 6 == 1 {
        return Ok(None);
    }

    let x = i32::from(u16::from_be_bytes([data[1] & 0x0F, data[2]]));
    let y = i32::from(u16::from_be_bytes([data[3] & 0x0F, data[4]]));
    if x >= WIDTH || y >= HEIGHT {
        return Ok(None);
    }
    Ok(Some(Point::new(WIDTH - 1 - x, HEIGHT - 1 - y)))
}

fn draw_settings(y: i32, row: &mut Row<'_>, choice: usize) {
    use display::rgb;
    let text = rgb(232, 241, 255);
    row.fill(rgb(12, 19, 30));
    row.label(y - 94, WIDTH / 2, "Настройки", text);
    row.label(y - 168, WIDTH / 2, "Гасить экран через", text);
    for (index, &(_, label)) in SLEEP_OPTIONS.iter().enumerate() {
        let x = 127 + 70 * index as i32;
        if (206..258).contains(&y) {
            let fill = if index == choice {
                rgb(30, 106, 98)
            } else {
                rgb(31, 48, 65)
            };
            row.span(x, x + 68, fill);
        }
        row.label(y - 223, x + 34, label, text);
    }
    row.label(y - 304, WIDTH / 2, "До перезагрузки", text);
}
fn draw_greeting(y: i32, row: &mut Row<'_>) {
    row.fill([0; 2]);
    if !(TEXT_Y..TEXT_Y + GLYPH * SCALE).contains(&y) {
        return;
    }
    let glyph_y = ((y - TEXT_Y) / SCALE) as usize;
    for (column, &ch) in MESSAGE.iter().enumerate() {
        let glyph = BASIC_LEGACY[usize::from(ch & 0x7F)][glyph_y];
        let start = TEXT_X + column as i32 * GLYPH * SCALE;
        for bit in 0..GLYPH {
            if glyph & (1 << bit) != 0 {
                let x = start + bit * SCALE;
                row.span(x, x + SCALE, [0xFF; 2]);
            }
        }
    }
}

fn report(usb: &mut UsbSerialJtag<'_, Blocking>, message: &[u8]) {
    for &byte in message {
        if usb.write_byte_nb(byte).is_err() {
            break;
        }
    }
    let _ = usb.flush_tx_nb();
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        wait_for_interrupt();
    }
}
