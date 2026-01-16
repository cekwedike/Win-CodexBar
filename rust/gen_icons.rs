/// Standalone icon generator - doesn't depend on main crate

use std::fs::File;
use std::io::Write;

fn main() {
    std::fs::create_dir_all("icons").expect("Failed to create icons dir");

    // Create ICO file
    create_ico("icons/icon.ico");
    println!("Generated icons/icon.ico");

    // Create PNG file
    create_png("icons/icon.png", 256);
    println!("Generated icons/icon.png");
}

fn create_ico(path: &str) {
    let mut file = File::create(path).expect("Failed to create ICO file");

    let size = 32u32;
    let rgba = generate_icon_rgba(size);

    // ICO header
    file.write_all(&0u16.to_le_bytes()).unwrap(); // reserved
    file.write_all(&1u16.to_le_bytes()).unwrap(); // type = ICO
    file.write_all(&1u16.to_le_bytes()).unwrap(); // count = 1

    // Directory entry
    let bmp_header_size = 40u32;
    let pixel_data_size = size * size * 4;
    let mask_row_size = ((size + 31) / 32) * 4;
    let mask_size = mask_row_size * size;
    let bmp_size = bmp_header_size + pixel_data_size + mask_size;

    file.write_all(&[size as u8]).unwrap(); // width
    file.write_all(&[size as u8]).unwrap(); // height
    file.write_all(&[0u8]).unwrap();        // palette
    file.write_all(&[0u8]).unwrap();        // reserved
    file.write_all(&1u16.to_le_bytes()).unwrap(); // color planes
    file.write_all(&32u16.to_le_bytes()).unwrap(); // bits per pixel
    file.write_all(&bmp_size.to_le_bytes()).unwrap(); // size
    file.write_all(&22u32.to_le_bytes()).unwrap(); // offset (6 + 16)

    // BMP info header
    file.write_all(&bmp_header_size.to_le_bytes()).unwrap();
    file.write_all(&(size as i32).to_le_bytes()).unwrap();
    file.write_all(&((size * 2) as i32).to_le_bytes()).unwrap(); // doubled height
    file.write_all(&1u16.to_le_bytes()).unwrap(); // planes
    file.write_all(&32u16.to_le_bytes()).unwrap(); // bits
    file.write_all(&0u32.to_le_bytes()).unwrap(); // compression
    file.write_all(&(pixel_data_size + mask_size).to_le_bytes()).unwrap();
    file.write_all(&0i32.to_le_bytes()).unwrap(); // x ppm
    file.write_all(&0i32.to_le_bytes()).unwrap(); // y ppm
    file.write_all(&0u32.to_le_bytes()).unwrap(); // colors used
    file.write_all(&0u32.to_le_bytes()).unwrap(); // important colors

    // Pixel data (BGRA, bottom-up)
    for y in (0..size).rev() {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            file.write_all(&[rgba[idx+2], rgba[idx+1], rgba[idx], rgba[idx+3]]).unwrap();
        }
    }

    // AND mask
    for _ in 0..(mask_row_size * size) {
        file.write_all(&[0u8]).unwrap();
    }
}

fn create_png(path: &str, size: u32) {
    let mut file = File::create(path).expect("Failed to create PNG file");
    let rgba = generate_icon_rgba(size);

    file.write_all(&[137, 80, 78, 71, 13, 10, 26, 10]).unwrap();

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&size.to_be_bytes());
    ihdr.extend_from_slice(&size.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);

    write_chunk(&mut file, b"IHDR", &ihdr);

    let mut raw = Vec::new();
    for y in 0..size {
        raw.push(0);
        let start = (y * size * 4) as usize;
        raw.extend_from_slice(&rgba[start..start + (size * 4) as usize]);
    }

    write_chunk(&mut file, b"IDAT", &compress(&raw));
    write_chunk(&mut file, b"IEND", &[]);
}

fn generate_icon_rgba(size: u32) -> Vec<u8> {
    let mut rgba = Vec::new();
    let (r, g, b) = (76u8, 175u8, 80u8);
    let scale = size as f32 / 32.0;
    let margin = (2.0 * scale) as u32;
    let border = (1.0 * scale).max(1.0) as u32;
    let cx = size / 2;
    let cy = size / 2;
    let outer_r = (10.0 * scale) as u32;
    let inner_r = (5.0 * scale) as u32;

    for y in 0..size {
        for x in 0..size {
            let in_bounds = x >= margin && x < size - margin && y >= margin && y < size - margin;
            let in_border = in_bounds && (x < margin + border || x >= size - margin - border ||
                                          y < margin + border || y >= size - margin - border);

            let dx = (x as i32 - cx as i32).abs() as u32;
            let dy = (y as i32 - cy as i32).abs() as u32;
            let dist_sq = dx * dx + dy * dy;
            let in_ring = dist_sq >= inner_r * inner_r && dist_sq <= outer_r * outer_r;
            let gap_t = (2.0 * scale) as u32;
            let is_gap = x > cx && dy < dx / 2 + gap_t;
            let is_c = in_ring && !is_gap;

            let (pr, pg, pb, pa) = if !in_bounds {
                (0, 0, 0, 0)
            } else if in_border {
                (r / 2, g / 2, b / 2, 255)
            } else if is_c {
                (255, 255, 255, 255)
            } else {
                (r, g, b, 255)
            };
            rgba.extend_from_slice(&[pr, pg, pb, pa]);
        }
    }
    rgba
}

fn write_chunk(file: &mut File, chunk_type: &[u8; 4], data: &[u8]) {
    file.write_all(&(data.len() as u32).to_be_bytes()).unwrap();
    file.write_all(chunk_type).unwrap();
    file.write_all(data).unwrap();

    let mut crc_data = chunk_type.to_vec();
    crc_data.extend_from_slice(data);
    file.write_all(&crc32(&crc_data).to_be_bytes()).unwrap();
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffffffffu32;
    for b in data {
        crc ^= *b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xedb88320 } else { crc >> 1 };
        }
    }
    !crc
}

fn compress(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let mut i = 0;
    while i < data.len() {
        let chunk = (data.len() - i).min(65535);
        let is_last = i + chunk >= data.len();
        out.push(if is_last { 1 } else { 0 });
        out.extend_from_slice(&(chunk as u16).to_le_bytes());
        out.extend_from_slice(&(!(chunk as u16)).to_le_bytes());
        out.extend_from_slice(&data[i..i + chunk]);
        i += chunk;
    }
    let (mut a, mut b) = (1u32, 0u32);
    for byte in data { a = (a + *byte as u32) % 65521; b = (b + a) % 65521; }
    out.extend_from_slice(&((b << 16) | a).to_be_bytes());
    out
}
