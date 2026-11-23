use std::{env, fs, io::BufReader, path::PathBuf};

const SPRITE_PATH: &str = "Untitled.png";
const SPRITE_HEIGHT: usize = 128;

fn main() {
    println!("cargo:rustc-link-arg=-Tlinkall.x");
    println!("cargo:rerun-if-changed={SPRITE_PATH}");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out_dir.join("pet-sprite.bin"), sprite_bytes()).expect("write encoded pet sprite");
}

fn sprite_bytes() -> Vec<u8> {
    let file = fs::File::open(SPRITE_PATH).expect("open pet sprite PNG");
    let mut decoder = png::Decoder::new(BufReader::new(file));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().expect("read pet sprite PNG header");
    let mut pixels = vec![
        0;
        reader
            .output_buffer_size()
            .expect("sprite PNG is too large")
    ];
    let frame = reader
        .next_frame(&mut pixels)
        .expect("decode pet sprite PNG");
    assert_eq!(
        frame.color_type,
        png::ColorType::Rgba,
        "pet sprite needs an alpha channel"
    );

    let (width, height) = (frame.width as usize, frame.height as usize);
    let (mut left, mut top, mut right, mut bottom) = (width, height, 0, 0);
    for (index, pixel) in pixels[..frame.buffer_size()].chunks_exact(4).enumerate() {
        if pixel[3] >= 128 {
            let (x, y) = (index % width, index / width);
            left = left.min(x);
            top = top.min(y);
            right = right.max(x + 1);
            bottom = bottom.max(y + 1);
        }
    }
    assert!(
        left < right && top < bottom,
        "pet sprite is fully transparent"
    );
    let (crop_width, crop_height) = (right - left, bottom - top);
    let sprite_width = (crop_width * SPRITE_HEIGHT).div_ceil(crop_height);
    let mut header = vec![
        u8::try_from(sprite_width).expect("pet sprite is too wide"),
        SPRITE_HEIGHT as u8,
    ];
    let mut runs = Vec::new();

    // Each row has an offset into the run list. Zero is reserved for transparency.
    for y in 0..SPRITE_HEIGHT {
        header.extend(
            u16::try_from(runs.len())
                .expect("pet sprite exceeds 64 KiB")
                .to_le_bytes(),
        );
        let source_y = top + (2 * y + 1) * crop_height / (2 * SPRITE_HEIGHT);
        let colors = (0..sprite_width).map(|x| {
            let source_x = left + (2 * x + 1) * crop_width / (2 * sprite_width);
            let index = (source_y * width + source_x) * 4;
            let [red, green, blue, alpha] = pixels[index..index + 4].try_into().unwrap();
            if alpha < 128 {
                return 0;
            }
            let rgb = ((u16::from(red) & 0xF8) << 8)
                | ((u16::from(green) & 0xFC) << 3)
                | (u16::from(blue) >> 3);
            rgb.max(1)
        });
        let mut colors = colors.peekable();
        while let Some(color) = colors.next() {
            let mut length = 1;
            while colors.next_if_eq(&color).is_some() {
                length += 1;
            }
            runs.push(length);
            runs.extend(color.to_be_bytes());
        }
    }
    header.extend(
        u16::try_from(runs.len())
            .expect("pet sprite exceeds 64 KiB")
            .to_le_bytes(),
    );
    header.extend(runs);
    header
}
