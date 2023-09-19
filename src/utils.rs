use alloc::{vec, vec::Vec};
use hex_literal::hex;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

/// Doesn't actually compress, just changes formats
/// Equivalent to zlib compression level 0
pub fn zlib_format(mut data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return hex!("789c030000000001").to_vec();
    }
    let mut out = vec![0x08, 0x1d];
    let checksum = adler::adler32_slice(data);
    // Split the data into max sized raw chunks
    while !data.is_empty() {
        let chunk;
        (chunk, data) = data.split_at(core::cmp::min(data.len(), 65535));
        let last_block = data.is_empty() as u8;

        // Raw block is indicated by the next two bits being "00"
        out.push(last_block); // The other bits will be 0

        // Write the length of the block (LSB first)
        out.extend((chunk.len() as u16).to_le_bytes());

        // Write the one's complement of the length (for raw blocks)
        out.extend((!chunk.len() as u16).to_le_bytes());

        // Write the raw data
        out.extend_from_slice(chunk);
    }
    out.extend(checksum.to_be_bytes());
    out
}

fn uncompressed_pixel_data(pixels: Vec<Vec<Color>>) -> Vec<u8> {
    let height = pixels.len();
    let width = pixels.get(0).map(|row| row.len()).unwrap_or_default();
    let mut out = Vec::with_capacity(height * (1 + width * 3));
    for row in pixels {
        out.push(0); // Filter type: none
        for pixel in row {
            out.push(pixel.red);
            out.push(pixel.green);
            out.push(pixel.blue);
        }
    }
    out
}

pub fn make_png(pixels: Vec<Vec<Color>>) -> Vec<u8> {
    let height = pixels.len();
    let width = pixels.get(0).map(|row| row.len()).unwrap_or_default();
    let idat = zlib_format(&uncompressed_pixel_data(pixels));
    let mut out = Vec::new();
    out.extend(hex!("89504E470D0A1A0A")); // PNG signature
    let mut append_chunk = |name: &[u8; 4], chunk: &[u8]| {
        out.extend((chunk.len() as u32).to_be_bytes());
        let start = out.len();
        out.extend(name);
        out.extend(chunk);
        let crc = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
        out.extend(crc.checksum(&out[start..]).to_be_bytes());
    };
    let mut ihdr = Vec::new();
    ihdr.extend((width as u32).to_be_bytes());
    ihdr.extend((height as u32).to_be_bytes());
    ihdr.push(8); // bit depth
    ihdr.push(2); // colour type: truecolour
    ihdr.push(0); // compression: deflate
    ihdr.push(0); // filter method: adapative
    ihdr.push(0); // interlace: no interlace
    append_chunk(b"IHDR", &ihdr);
    drop(ihdr);
    append_chunk(b"IDAT", &idat);
    append_chunk(b"IEND", &[]);
    out
}

#[cfg(test)]
mod tests {
    use super::{make_png, zlib_format, Color};
    use std::io::Read;

    #[test]
    fn test_deflate() {
        for len in [0, 1, 10, 100_000] {
            println!("Testing input length {len}");
            let input = vec![0x12; len];
            let output = zlib_format(&input);
            let mut reader = flate2::read::ZlibDecoder::new(std::io::Cursor::new(output));
            let mut inflated = Vec::new();
            reader.read_to_end(&mut inflated).unwrap();
            assert_eq!(inflated, input);
        }
    }

    #[test]
    fn test_png() {
        let pixels = vec![
            vec![
                Color {
                    red: 1,
                    green: 0,
                    blue: 2,
                };
                128
            ];
            128
        ];
        let encoded = make_png(pixels);
        let decoder = png::Decoder::new(std::io::Cursor::new(encoded));
        let mut data_reader = decoder.read_info().expect("Failed to read PNG info");
        while data_reader
            .next_row()
            .expect("Failed to read PNG data")
            .is_some()
        {}
    }
}
