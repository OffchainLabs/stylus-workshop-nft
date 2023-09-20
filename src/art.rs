//! Drawing functions.

use crate::utils::{Color, FnvHasher, Pixels};
use alloc::boxed::Box;
use alloy_primitives::Address;
use fastrand::Rng;
use stylus_sdk::alloy_primitives::U256;

/// Represents an image.
pub struct Image<const R: usize, const C: usize> {
    pub pixels: Pixels<R, C>,
}

/// Represents a cell on the grid.
struct Cell {
    x: usize,
    y: usize,
}

impl Cell {
    fn new(x: usize, y: usize) -> Cell {
        Cell { x, y }
    }
}

/// If true, never leaves a line connected by just a diagonal
const THICK_LINES: bool = false;

// Drawing algorithms are from http://members.chello.at/~easyfilter/Bresenham.pdf
impl<const R: usize, const C: usize> Image<R, C> {
    /// Creates a new image with a default background color.
    pub fn new(bg_color: Color) -> Image<R, C> {
        Image {
            pixels: Box::new([[bg_color; C]; R]),
        }
    }

    /// Draws a line from `start` to `end` with the given `color`
    fn draw_line(&mut self, start: Cell, end: Cell, color: Color) {
        let dx = end.x.abs_diff(start.x) as isize;
        let dy = -(end.y.abs_diff(start.y) as isize);
        let sx = if end.x > start.x { 1 } else { -1 };
        let sy = if end.y > start.y { 1 } else { -1 };
        let mut error = dx + dy;
        let mut x = start.x;
        let mut y = start.y;
        self.pixels[y][x] = color;
        while x != end.x || y != end.y {
            let error2 = error * 2;
            if error2 >= dy {
                debug_assert!(x != end.x);
                error += dy;
                x = x.saturating_add_signed(sx);
                if THICK_LINES {
                    self.pixels[y][x] = color;
                }
            }
            if error2 <= dx {
                debug_assert!(y != end.y);
                error += dx;
                y = y.saturating_add_signed(sy);
                if THICK_LINES {
                    self.pixels[y][x] = color;
                }
            }
            if !THICK_LINES {
                self.pixels[y][x] = color;
            }
        }
    }

    /// Draws an ellipse centered at `center` with width `a` and height `b`.
    /// Only draws the quadrants set to `true` in `draw_quadrants`.
    /// `draw_quadrants` is an array of quadrant I through quadrant IV; i.e.
    /// it starts in the top right and goes counter-clockwise.
    fn draw_ellipse(
        &mut self,
        center: Cell,
        a: usize,
        b: usize,
        draw_quadrants: [bool; 4],
        color: Color,
    ) {
        let mut x = a; // IV. quadrant
        let mut y = 0;
        let mut dx = (1 - 2 * x as isize) * (b * b) as isize;
        let mut dy = (x * x) as isize;
        let mut error = dx + dy;
        // Draws coordinates if in-bound
        let mut draw = |x: Option<usize>, y: Option<usize>| {
            if let (Some(x), Some(y)) = (x, y) {
                if x < C && y < R {
                    self.pixels[y][x] = color;
                }
            }
        };
        loop {
            if draw_quadrants[0] {
                // I. Quadrant
                draw(center.x.checked_add(x), center.y.checked_sub(y));
            }
            if draw_quadrants[1] {
                // II. Quadrant
                draw(center.x.checked_sub(x), center.y.checked_sub(y));
            }
            if draw_quadrants[2] {
                // III. Quadrant
                draw(center.x.checked_sub(x), center.y.checked_add(y));
            }
            if draw_quadrants[3] {
                // IV. Quadrant
                draw(center.x.checked_add(x), center.y.checked_add(y));
            }
            let error2 = error * 2;
            if error2 >= dx {
                if x == 0 {
                    break;
                }
                x -= 1;
                dx += (2 * b * b) as isize;
                error += dx;
            }
            if error2 <= dy {
                y += 1;
                dy += (2 * a * a) as isize;
                error += dy;
            }
        }
        // Handle very flat ellipses (a=1)
        while y < b {
            y += 1;
            if draw_quadrants[0] || draw_quadrants[1] {
                draw(Some(center.x), center.y.checked_sub(y));
            }
            if draw_quadrants[2] || draw_quadrants[3] {
                draw(Some(center.x), center.y.checked_add(y));
            }
        }
    }
}

/// Generates the image for a given NFT token ID
/// Returns an array of rows of pixels
/// Every row must be the same length
pub fn generate_nft(this_address: Address, token_id: U256) -> Image<16, 16> {
    let mut hasher = FnvHasher::new();
    hasher.update(token_id.as_le_slice());
    hasher.update(this_address.as_slice());
    let mut rng = Rng::with_seed(hasher.output());

    let bg_color = Color {
        red: 0xe3,
        green: 0x06,
        blue: 0x6e,
    };
    let fg_color = Color {
        red: rng.u8(..80), // don't generate something too red or it'll blend in
        green: rng.u8(..),
        blue: rng.u8(..),
    };

    let mut image = Image::new(bg_color);
    image.draw_line(Cell::new(4, 4), Cell::new(4, 6), fg_color);
    image.draw_line(Cell::new(10, 4), Cell::new(10, 6), fg_color);
    image.draw_ellipse(Cell::new(7, 9), 3, 3, [false, false, true, true], fg_color);
    image
}
