#![no_std]
#![no_main]

mod display;
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
use topics::Topics;

esp_bootloader_esp_idf::esp_app_desc!();

const MESSAGE: &[u8] = b"hello everynwan";
const GLYPH: i32 = 8;
const SCALE: i32 = 3;
const TEXT_X: i32 = (WIDTH - MESSAGE.len() as i32 * GLYPH * SCALE) / 2;
const TEXT_Y: i32 = (HEIGHT - GLYPH * SCALE) / 2;

const SAMPLE_PERIOD: Duration = Duration::from_millis(8);
const BUTTON_DEBOUNCE: Duration = Duration::from_millis(24);
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

    let button = Input::new(
        peripherals.GPIO0,
        InputConfig::default().with_pull(Pull::Up),
    );
    let mut button = Button::new(button);
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

    let mut show_topics = false;
    let mut topics = Topics::new();
    let mut last_touch = Instant::now();
    let mut reported_drag = false;
    loop {
        if button.pressed() {
            show_topics = !show_topics;
            topics.drag(None);
            if show_topics {
                display = display.draw(|y, row| topics.draw_row(y, row));
                report(&mut usb, b"Topic map open\r\n");
            } else {
                display = display.draw(draw_greeting);
                report(&mut usb, b"Greeting open\r\n");
            }
        }

        if show_topics {
            match touch_position(&mut i2c) {
                Ok(position) => {
                    last_touch = Instant::now();
                    if topics.drag(position) {
                        display = display.draw(|y, row| topics.draw_row(y, row));
                        if !reported_drag {
                            report(&mut usb, b"Topic map moved\r\n");
                            reported_drag = true;
                        }
                    }
                }
                Err(_) if last_touch.elapsed() >= TOUCH_TIMEOUT => {
                    topics.drag(None);
                }
                Err(_) => {}
            }
        }
        wait_for_interrupt();
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
