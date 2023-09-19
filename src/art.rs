//
// Stylus workshop NFT
//

use crate::utils::Color;
use alloc::{vec, vec::Vec};
use alloy_primitives::U256;

struct Image {
    pixels: Vec<Vec<Color>>,
}

#[allow(dead_code)]
impl Image {
    fn new(width: usize, height: usize, bg_color: Color) -> Image {
        Image {
            pixels: vec![vec![bg_color; width]; height],
        }
    }

    fn draw_line(start: (usize, usize), end: (usize, usize), color: Color) {
        todo!("implement a line drawing algorithm")
    }
}

/// Generates the image for a given NFT token ID
/// Returns an array of rows of pixels
/// Every row must be the same length
pub fn generate_nft(_token_id: U256) -> Vec<Vec<Color>> {
    let bg_color = Color {
        red: 0xe3,
        green: 0x06,
        blue: 0x6e,
    };
    let mut image = Image::new(256, 256, bg_color);
    image.pixels
}
