//! Utilities.

use alloc::{boxed::Box, vec, vec::Vec};
use hex_literal::hex;

use crate::art::Image;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub const fn from_hex(value: usize) -> Self {
        Self {
            red: (value >> 16) as u8,
            green: (value >> 8) as u8,
            blue: value as u8,
        }
    }

    pub const fn to_hex(&self) -> usize {
        (self.red as usize) << 16 | (self.green as usize) << 8 | self.blue as usize
    }
}

/// A grid of pixels `R` rows by `C` columns.
pub type Pixels<const R: usize, const C: usize> = Box<[[Color; C]; R]>;

/// Doesn't actually compress, just changes formats.
///
/// Equivalent to zlib compression level 0.
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

impl<const R: usize, const C: usize> Image<R, C> {
    fn uncompressed_pixel_data(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(R * (1 + C * 3));
        for row in &*self.pixels {
            out.push(0); // Filter type: none
            for pixel in row {
                out.push(pixel.red);
                out.push(pixel.green);
                out.push(pixel.blue);
            }
        }
        out
    }

    pub fn make_png(&self) -> Vec<u8> {
        let idat = zlib_format(&self.uncompressed_pixel_data());
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
        ihdr.extend((C as u32).to_be_bytes());
        ihdr.extend((R as u32).to_be_bytes());
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
}

const FNV_PRIME: u64 = 1099511628211;

/// Implements FNV-1a hashing (not cryptographically secure)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FnvHasher(pub u64);

impl Default for FnvHasher {
    fn default() -> Self {
        FnvHasher(14695981039346656037)
    }
}

impl FnvHasher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, input: &[u8]) {
        for &byte in input {
            self.0 ^= byte as u64;
            self.0 = self.0.wrapping_mul(FNV_PRIME);
        }
    }

    pub fn output(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::art::Image;

    use super::{zlib_format, Color};
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
        let color = Color {
            red: 1,
            green: 0,
            blue: 2,
        };
        let image: Image<128, 128> = Image::new(color);

        let encoded = image.make_png();
        let decoder = png::Decoder::new(std::io::Cursor::new(encoded));
        let mut data_reader = decoder.read_info().expect("Failed to read PNG info");
        while data_reader
            .next_row()
            .expect("Failed to read PNG data")
            .is_some()
        {}
    }
}
