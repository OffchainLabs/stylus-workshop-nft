//
// Stylus workshop NFT
//

//! Warning: this code is a template only and has not been audited.

// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
#![cfg_attr(not(test), no_std)]
extern crate alloc;

mod art;
pub mod erc712;
#[cfg(test)]
mod print_art;
pub mod utils;

use crate::erc712::Erc712;
use alloc::{string::String, vec::Vec};
use alloy_primitives::{uint, U256};
use alloy_sol_types::{sol, SolError};
use base64::Engine;
use erc712::{Erc712Error, Erc712Params};
use stylus_sdk::{
    abi::Bytes,
    call::{self, Call},
    msg,
    prelude::*,
};

/// Initializes a custom, global allocator for Rust programs compiled to WASM.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Configures the NFT data.
struct StylusWorkshopParams;

impl Erc712Params for StylusWorkshopParams {
    const NAME: &'static str = "Stylus Workshop NFT";
    const SYMBOL: &'static str = "SNFT";

    fn token_uri(token_id: U256) -> String {
        let image = art::generate_nft(stylus_sdk::contract::address(), token_id);
        let png = image.make_png();
        let mut out = String::from("data:image/png;base64,");
        base64::engine::general_purpose::STANDARD.encode_string(&png, &mut out);
        out
    }
}

// Here is where one declares storage.
sol_storage! {
    #[entrypoint]
    struct StylusWorkshopNft {
        #[borrow]
        Erc712<StylusWorkshopParams> erc712;
    }
}

/// The price of a mint, measured in wei: 0.0001 eth
const MINT_PRICE: U256 = uint!(100_000_000_000_000_U256);

// Declare Solidity error types
sol! {
    error IncorrectMintValue(uint256 paid, uint256 expected);
}

/// Represents the ways methods may fail.
pub enum StylusWorkshopNftError {
    IncorrectMintValue(IncorrectMintValue),
    Erc712Error(Erc712Error),
    ExternalCallError(call::Error),
}

impl Into<Vec<u8>> for StylusWorkshopNftError {
    fn into(self) -> Vec<u8> {
        match self {
            Self::IncorrectMintValue(err) => err.encode(),
            Self::Erc712Error(err) => err.into(),
            Self::ExternalCallError(err) => err.into(),
        }
    }
}

impl From<Erc712Error> for StylusWorkshopNftError {
    fn from(err: Erc712Error) -> Self {
        StylusWorkshopNftError::Erc712Error(err)
    }
}

impl From<call::Error> for StylusWorkshopNftError {
    fn from(err: call::Error) -> Self {
        StylusWorkshopNftError::ExternalCallError(err)
    }
}

type Result<T, E = StylusWorkshopNftError> = core::result::Result<T, E>;

// These methods aren't external, but are helpers used by external methods.
impl StylusWorkshopNft {
    fn check_mint_price(&self) -> Result<()> {
        if msg::value() != MINT_PRICE {
            return Err(StylusWorkshopNftError::IncorrectMintValue(
                IncorrectMintValue {
                    paid: msg::value(),
                    expected: MINT_PRICE,
                },
            ));
        }
        Ok(())
    }
}

// these methods are external to other contracts
#[external]
#[inherit(Erc712<StylusWorkshopParams>)]
impl StylusWorkshopNft {
    /// Mints an NFT, but does not call onErc712Received
    /// Requires the caller supply MINT_VALUE
    #[payable]
    pub fn mint(&mut self) -> Result<()> {
        self.check_mint_price()?;
        self.erc712.mint(msg::sender())?;
        Ok(())
    }

    /// Mints an NFT and calls onErc712Received with empty data
    /// Requires the caller supply MINT_VALUE
    #[payable]
    pub fn safe_mint(&mut self) -> Result<()> {
        self.check_mint_price()?;
        Erc712::safe_mint(self, msg::sender(), Vec::new())?;
        Ok(())
    }

    /// Mints an NFT and calls onErc712Received with the specified data
    /// Requires the caller supply MINT_VALUE
    #[payable]
    #[selector(name = "safeMint")]
    pub fn safe_mint_with_data(&mut self, data: Bytes) -> Result<()> {
        self.check_mint_price()?;
        Erc712::safe_mint(self, msg::sender(), data.0)?;
        Ok(())
    }

    /// Burns an NFT and returns the MINT_VALUE to the caller
    /// Requires the caller be able to receiver eth with no calldata
    pub fn burn(&mut self, token_id: U256) -> Result<()> {
        // This function checks that msg::sender() owns the specified token_id
        self.erc712.burn(msg::sender(), token_id)?;
        stylus_sdk::call::call(Call::new_in(self).value(MINT_PRICE), msg::sender(), &[])?;
        Ok(())
    }
}
