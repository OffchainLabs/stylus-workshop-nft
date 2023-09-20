use crate::{art, utils::Color};
use alloy_primitives::{Address, U256};
use rand::{thread_rng, Rng};

// To run and view the art: cargo test print_art -- --nocapture
#[test]
fn print_art() {
    fn set_terminal_color(bg: bool, color: Color) {
        let ty = if bg { 48 } else { 38 };
        print!("\x1b[{ty};2;{};{};{}m", color.red, color.green, color.blue);
    }

    let mut rng = thread_rng();
    let mut address = Address::default();
    rng.fill(&mut address.0 .0);
    let id = rng.gen_range(0_u64..1000);

    println!("Generating NFT 0x{} ID {id}:", hex::encode(address));
    let image = art::generate_nft(address, U256::from(id));
    for row_idx in (0..image.pixels.len()).step_by(2) {
        for (col_idx, &top_color) in image.pixels[row_idx].iter().enumerate() {
            set_terminal_color(false, top_color);
            if let Some(bottom_color) = image.pixels.get(row_idx + 1).map(|r| r[col_idx]) {
                set_terminal_color(true, bottom_color);
            }
            print!("▀");
        }
        print!("\x1b[39;49m"); // reset
        println!();
    }
}
