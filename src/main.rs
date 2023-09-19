//
// Stylus workshop NFT
//

//! Warning: this code is a template only and has not been audited.

// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
#![cfg_attr(not(test), no_std)]
extern crate alloc;

mod art;
mod erc712;
mod utils;

use crate::{erc712::Erc712, utils::make_png};
use alloc::string::String;
use base64::Engine;
use erc712::Erc712Params;
use stylus_sdk::{alloy_primitives::U256, prelude::*};

/// Initializes a custom, global allocator for Rust programs compiled to WASM.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct StylusWorkshopParams;

impl Erc712Params for StylusWorkshopParams {
    const NAME: &'static str = "Stylus Workshop NFT";
    const SYMBOL: &'static str = "SNFT";

    fn token_uri(token_id: U256) -> String {
        let pixels = art::generate_nft(token_id);
        let png = make_png(pixels);
        let mut out = String::from("data:image/png;base64,");
        base64::engine::general_purpose::STANDARD.encode_string(&png, &mut out);
        out
    }
}

sol_storage! {
    #[entrypoint]
    struct StylusWorkshopNft {
        #[borrow]
        Erc712<StylusWorkshopParams> erc712;
    }
}

#[external]
#[inherit(Erc712<StylusWorkshopParams>)]
impl StylusWorkshopNft {}
