//! Drawing functions.

use crate::utils::{Color, Pixels};
use alloc::boxed::Box;
use stylus_sdk::alloy_primitives::U256;

/// Represents an image.
pub struct Image<const R: usize, const C: usize> {
    pub pixels: Pixels<R, C>,
}

/// Represents a cell on the grid.
type Cell = (usize, usize);

#[allow(dead_code)]
impl<const R: usize, const C: usize> Image<R, C> {
    /// Creates a new image with a default background color.
    pub fn new(bg_color: Color) -> Image<R, C> {
        Image {
            pixels: Box::new([[bg_color; C]; R]),
        }
    }

    /// Draws a line from
    fn draw_line(start: Cell, end: Cell, color: Color) {
        todo!("implement a line drawing algorithm")
    }
}

/// Generates the image for a given NFT token ID
/// Returns an array of rows of pixels
/// Every row must be the same length
pub fn generate_nft(token_id: U256) -> Image<256, 256> {
    let bg_color = Color {
        red: 0xe3,
        green: 0x06,
        blue: 0x6e,
    };
    let mut image = Image::new(bg_color);

    //image.pixels
    image
}
