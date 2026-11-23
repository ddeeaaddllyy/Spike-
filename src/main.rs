#![no_std]
#![no_main]

use core::panic::PanicInfo;

use esp_hal::{
    Blocking,
    delay::Delay,
    dma::DmaTxBuf,
    dma_tx_buffer,
    gpio::{Level, Output, OutputConfig},
    interrupt::wait_for_interrupt,
    spi::master::{Address, Command, Config, DataMode, Spi, SpiDma},
    time::Rate,
    usb::usb_serial_jtag::UsbSerialJtag,
};
use font8x8::legacy::BASIC_LEGACY;

esp_bootloader_esp_idf::esp_app_desc!();

const MESSAGE: &[u8] = b"hello everynwan";
const WIDTH: usize = 466;
const HEIGHT: usize = 466;
// The SH8601's visible columns start at GRAM column 6 on this panel.
const COLUMN_OFFSET: u16 = 6;
// SH8601 write windows use even starts and even exclusive ends.
const STRIP_ROWS: usize = 2;
const ROW_BYTES: usize = WIDTH * 2;
const STRIP_BYTES: usize = ROW_BYTES * STRIP_ROWS;
const GLYPH_SIZE: usize = 8;
const SCALE: usize = 3;
const TEXT_X: usize = (WIDTH - MESSAGE.len() * GLYPH_SIZE * SCALE) / 2;
const TEXT_Y: usize = (HEIGHT - GLYPH_SIZE * SCALE) / 2;

#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    // GPIO18 latches the board's power on when running from a battery.
    let _power_hold = Output::new(peripherals.GPIO18, Level::High, OutputConfig::default());
    let mut reset = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());
    delay.delay_millis(10);
    reset.set_high();
    delay.delay_millis(150);

    let spi = Spi::new(
        peripherals.SPI2,
        Config::default().with_frequency(Rate::from_mhz(40)),
    )
    .unwrap()
    .with_cs(peripherals.GPIO10)
    .with_sck(peripherals.GPIO11)
    .with_sio0(peripherals.GPIO12)
    .with_sio1(peripherals.GPIO13)
    .with_sio2(peripherals.GPIO14)
    .with_sio3(peripherals.GPIO15)
    .with_dma(peripherals.DMA_CH0);
    let mut display = Display {
        spi,
        tx: dma_tx_buffer!(STRIP_BYTES).unwrap(),
    }
    .init(&delay);

    display = display.command(
        0x2A,
        &address_range(COLUMN_OFFSET, COLUMN_OFFSET + WIDTH as u16),
    );
    for y in (0..HEIGHT).step_by(STRIP_ROWS) {
        display = display.command(0x2B, &address_range(y as u16, (y + STRIP_ROWS) as u16));
        render_strip(y, display.tx.as_mut_slice());
        display = display.transfer(0x32, 0x2C, DataMode::Quad, STRIP_BYTES);
    }
    let _display = display.command(0x29, &[]); // Display on after the first frame.

    let mut usb = UsbSerialJtag::new(peripherals.USB_DEVICE);
    for &byte in b"AMOLED frame sent\r\n" {
        if usb.write_byte_nb(byte).is_err() {
            break;
        }
    }
    let _ = usb.flush_tx_nb();
    idle()
}

struct Display {
    spi: SpiDma<'static, Blocking>,
    tx: DmaTxBuf,
}

impl Display {
    fn init(mut self, delay: &Delay) -> Self {
        // Panel parameters for the Waveshare ESP32-S3-Touch-AMOLED-1.32.
        for (register, data) in [
            (0xFE, 0x00), // Command page 0.
            (0xC4, 0x80),
            (0x36, 0x00), // RGB, native orientation.
            (0x3A, 0x55), // RGB565.
            (0x35, 0x00),
            (0x53, 0x20),
            (0x51, 0x7F), // Brightness.
            (0x63, 0xFF),
        ] {
            self = self.command(register, &[data]);
        }
        self = self.command(0x11, &[]); // Sleep out.
        delay.delay_millis(120);
        self
    }

    fn command(mut self, register: u8, data: &[u8]) -> Self {
        self.tx.as_mut_slice()[..data.len()].copy_from_slice(data);
        self.transfer(0x02, register, DataMode::Single, data.len())
    }

    fn transfer(self, opcode: u16, register: u8, mode: DataMode, len: usize) -> Self {
        // The 32-bit header travels on one wire. Only pixel data uses four wires.
        let (spi, tx) = self
            .spi
            .half_duplex_write_buffer(
                mode,
                Command::_8Bit(opcode, DataMode::Single),
                Address::_24Bit(u32::from(register) << 8, DataMode::Single),
                0,
                len,
                self.tx,
            )
            .map_err(|(error, _, _)| error)
            .unwrap()
            .wait();
        Self { spi, tx }
    }
}

fn address_range(start: u16, end: u16) -> [u8; 4] {
    let [start_hi, start_lo] = start.to_be_bytes();
    let [end_hi, end_lo] = (end - 1).to_be_bytes();
    [start_hi, start_lo, end_hi, end_lo]
}

fn render_strip(y: usize, pixels: &mut [u8]) {
    pixels.fill(0);
    for (i, row) in pixels.chunks_exact_mut(ROW_BYTES).enumerate() {
        // The panel's native scan orientation is upside down on this board.
        let screen_y = HEIGHT - 1 - (y + i);
        if !(TEXT_Y..TEXT_Y + GLYPH_SIZE * SCALE).contains(&screen_y) {
            continue;
        }
        let glyph_y = (screen_y - TEXT_Y) / SCALE;
        for (column, &ch) in MESSAGE.iter().enumerate() {
            let glyph = BASIC_LEGACY[usize::from(ch & 0x7F)][glyph_y];
            let start = (TEXT_X + column * GLYPH_SIZE * SCALE) * 2;
            for bit in 0..GLYPH_SIZE {
                if glyph & (1 << bit) != 0 {
                    let pixel = ROW_BYTES - start - (bit + 1) * SCALE * 2;
                    row[pixel..pixel + SCALE * 2].fill(0xFF);
                }
            }
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    idle()
}

fn idle() -> ! {
    loop {
        wait_for_interrupt();
    }
}
