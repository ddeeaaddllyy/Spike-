use esp_hal::{
    Blocking,
    delay::Delay,
    dma::DmaTxBuf,
    dma_tx_buffer,
    spi::master::{Address, Command, DataMode, SpiDma},
};

pub const WIDTH: i32 = 466;
pub const HEIGHT: i32 = 466;

const COLUMN_OFFSET: i32 = 6;
const CONTROLLER_SIZE: i32 = 480;
const STRIP_ROWS: i32 = 4;
const ROW_BYTES: usize = WIDTH as usize * 2;
const STRIP_BYTES: usize = CONTROLLER_SIZE as usize * 2 * STRIP_ROWS as usize;

pub type Color = [u8; 2];

pub const fn rgb(red: u8, green: u8, blue: u8) -> Color {
    (((red as u16 & 0xF8) << 8) | ((green as u16 & 0xFC) << 3) | (blue as u16 >> 3)).to_be_bytes()
}

pub struct Display {
    spi: SpiDma<'static, Blocking>,
    tx: DmaTxBuf,
}

impl Display {
    pub fn new(spi: SpiDma<'static, Blocking>, delay: &Delay) -> Self {
        let mut display = Self {
            spi,
            tx: dma_tx_buffer!(STRIP_BYTES).unwrap(),
        };
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
            display = display.command(register, &[data]);
        }
        display = display.command(0x11, &[]); // Sleep out.
        delay.delay_millis(120);
        // Also clear the GRAM margins around the 466×466 viewport.
        display.write_strips(0, CONTROLLER_SIZE, CONTROLLER_SIZE, |_, pixels| {
            pixels.fill(0);
        })
    }

    pub fn show(self) -> Self {
        self.command(0x29, &[])
    }

    pub fn draw(self, mut draw_row: impl FnMut(i32, &mut Row<'_>)) -> Self {
        self.write_strips(COLUMN_OFFSET, WIDTH, HEIGHT, |y, strip| {
            for (index, pixels) in strip.chunks_exact_mut(ROW_BYTES).enumerate() {
                // Apply the board's 180° rotation in both axes.
                draw_row(HEIGHT - 1 - y - index as i32, &mut Row { pixels });
            }
        })
    }

    fn write_strips(
        mut self,
        left: i32,
        width: i32,
        height: i32,
        mut paint: impl FnMut(i32, &mut [u8]),
    ) -> Self {
        let columns = address_range(left, left + width);
        for y in (0..height).step_by(STRIP_ROWS as usize) {
            // The panel takes even window starts and even exclusive ends.
            let end = (y + STRIP_ROWS).min(height);
            let len = (end - y) as usize * width as usize * 2;
            self = self.command(0x2A, &columns);
            self = self.command(0x2B, &address_range(y, end));
            paint(y, &mut self.tx.as_mut_slice()[..len]);
            self = self.transfer(0x32, 0x2C, DataMode::Quad, len);
        }
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

pub struct Row<'a> {
    pixels: &'a mut [u8],
}

impl Row<'_> {
    pub fn fill(&mut self, color: Color) {
        self.span(0, WIDTH, color);
    }

    pub fn span(&mut self, left: i32, right: i32, color: Color) {
        let start = (WIDTH - right.clamp(0, WIDTH)) as usize * 2;
        let end = (WIDTH - left.clamp(0, WIDTH)) as usize * 2;
        let pixels = &mut self.pixels[start..end];
        if color[0] == color[1] {
            pixels.fill(color[0]);
        } else {
            for pixel in pixels.chunks_exact_mut(2) {
                pixel.copy_from_slice(&color);
            }
        }
    }
}

fn address_range(start: i32, end: i32) -> [u8; 4] {
    let [start_hi, start_lo] = (start as u16).to_be_bytes();
    let [end_hi, end_lo] = ((end - 1) as u16).to_be_bytes();
    [start_hi, start_lo, end_hi, end_lo]
}
