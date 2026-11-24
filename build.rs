use std::{env, fs, io::BufReader, path::PathBuf};

const ATLAS: &str = "assets/spike-poses.png";
const SIDE: usize = 96;
// Index zero is transparent. Share a single palette across all poses.
const PALETTE: [[u8; 3]; 16] = [
    [0, 0, 0],
    [24, 16, 28],
    [40, 29, 43],
    [58, 42, 60],
    [79, 58, 83],
    [104, 78, 103],
    [52, 19, 36],
    [88, 30, 53],
    [130, 47, 69],
    [169, 74, 91],
    [107, 76, 73],
    [167, 119, 113],
    [214, 157, 145],
    [248, 203, 181],
    [255, 229, 212],
    [219, 210, 211],
];

fn main() {
    encode_background();
    println!("cargo:rustc-link-arg=-Tlinkall.x");
    println!("cargo:rerun-if-changed={ATLAS}");
    let mut decoder = png::Decoder::new(BufReader::new(fs::File::open(ATLAS).unwrap()));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let frame = reader.next_frame(&mut pixels).unwrap();
    assert_eq!(frame.color_type, png::ColorType::Rgba);
    let (w, h) = (frame.width as usize, frame.height as usize);
    let mut data = Vec::new();
    for [r, g, b] in PALETTE {
        let rgb = ((u16::from(r) & 248) << 8) | ((u16::from(g) & 252) << 3) | u16::from(b >> 3);
        data.extend(rgb.to_be_bytes());
    }
    for pose in 0..8 {
        let sample = |x, y| {
            let sx = (pose % 4 * 2 * SIDE + 2 * x + 1) * w / (8 * SIDE);
            let sy = (pose / 4 * 2 * SIDE + 2 * y + 1) * h / (4 * SIDE);
            let i = (sy * w + sx) * 4;
            if pixels[i + 3] < 128 {
                return 0;
            }
            (1..16)
                .min_by_key(|&index| {
                    (0..3)
                        .map(|c| (i32::from(pixels[i + c]) - i32::from(PALETTE[index][c])).pow(2))
                        .sum::<i32>()
                })
                .unwrap() as u8
        };
        for y in 0..SIDE {
            for x in (0..SIDE).step_by(2) {
                data.push(sample(x, y) << 4 | sample(x + 1, y));
            }
        }
    }
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("pet-sprite.bin"),
        data,
    )
    .unwrap();
}

fn encode_background() {
    const PATH: &str = "assets/moonlake.png";
    println!("cargo:rerun-if-changed={PATH}");
    let mut reader = png::Decoder::new(BufReader::new(fs::File::open(PATH).unwrap()))
        .read_info()
        .unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let frame = reader.next_frame(&mut pixels).unwrap();
    assert_eq!((frame.width, frame.height), (233, 233));
    assert_eq!(
        (frame.color_type, frame.bit_depth),
        (png::ColorType::Indexed, png::BitDepth::Eight)
    );
    let palette = reader.info().palette.as_ref().unwrap();
    // RGB565 palette, absolute row offsets, then (length, palette index) runs.
    let mut encoded = vec![0; 512 + (233 + 1) * 2];
    for (i, entry) in palette.chunks_exact(3).enumerate() {
        let [r, g, b] = entry.try_into().unwrap();
        let color = ((u16::from(r) & 248) << 8) | ((u16::from(g) & 252) << 3) | u16::from(b >> 3);
        encoded[2 * i..2 * i + 2].copy_from_slice(&color.to_be_bytes());
    }
    for (y, row) in pixels.chunks_exact(233).enumerate() {
        let offset = u16::try_from(encoded.len()).unwrap();
        encoded[512 + 2 * y..514 + 2 * y].copy_from_slice(&offset.to_le_bytes());
        let mut x = 0;
        while x < row.len() {
            let start = x;
            let index = row[x];
            x += 1;
            while x < row.len() && row[x] == index {
                x += 1;
            }
            encoded.extend([(x - start) as u8, index]);
        }
    }
    let end = u16::try_from(encoded.len()).unwrap();
    encoded[978..980].copy_from_slice(&end.to_le_bytes());
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("moonlake.bin"),
        encoded,
    )
    .unwrap();
}
