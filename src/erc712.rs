//
// Stylus workshop NFT
//

use alloc::{string::String, vec, vec::Vec};
use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolError};
use core::{borrow::BorrowMut, marker::PhantomData};
use stylus_sdk::{abi::Bytes, evm, msg, prelude::*};

pub trait Erc712Params {
    const NAME: &'static str;
    const SYMBOL: &'static str;

    fn token_uri(token_id: U256) -> String;
}

sol_storage! {
    pub struct Erc712<T: Erc712Params> {
        mapping(uint256 => address) owners;
        mapping(uint256 => address) approved;
        mapping(address => uint256) balance;
        mapping(address => mapping(address => bool)) approved_for_all;
        PhantomData<T> _phantom;
    }
}

sol! {
    event Transfer(address indexed from, address indexed to, uint256 indexed token_id);
    event Approval(address indexed owner, address indexed approved, uint256 indexed token_id);
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);

    error InvalidTokenId(uint256 token_id);
    error NotOwner(address from, uint256 token_id, address real_owner);
    error NotApproved(uint256 token_id, address owner, address spender);
    error TransferToZero(uint256 token_id);
    error ReceiverRefused(address receiver, uint256 token_id, bytes4 returned);
}

pub enum NftError {
    InvalidTokenId(InvalidTokenId),
    NotOwner(NotOwner),
    NotApproved(NotApproved),
    TransferToZero(TransferToZero),
    ReceiverRefused(ReceiverRefused),
    ExternalCall(stylus_sdk::call::Error),
}

impl From<stylus_sdk::call::Error> for NftError {
    fn from(err: stylus_sdk::call::Error) -> Self {
        Self::ExternalCall(err)
    }
}

impl From<NftError> for Vec<u8> {
    fn from(val: NftError) -> Self {
        match val {
            NftError::InvalidTokenId(err) => err.encode(),
            NftError::NotOwner(err) => err.encode(),
            NftError::NotApproved(err) => err.encode(),
            NftError::TransferToZero(err) => err.encode(),
            NftError::ReceiverRefused(err) => err.encode(),
            NftError::ExternalCall(err) => err.into(),
        }
    }
}

type Result<T, E = NftError> = core::result::Result<T, E>;

impl<T: Erc712Params> Erc712<T> {
    /// Requires that msg::sender() is authorized to spend a given token
    fn require_authorized_to_spend(&self, from: Address, token_id: U256) -> Result<()> {
        let owner = self.owner_of(token_id)?;
        if from != owner {
            return Err(NftError::NotOwner(NotOwner {
                from,
                token_id,
                real_owner: owner,
            }));
        }

        if msg::sender() == owner {
            return Ok(());
        }
        if self.approved_for_all.getter(owner).get(msg::sender()) {
            return Ok(());
        }
        if msg::sender() == self.approved.get(token_id) {
            return Ok(());
        }
        Err(NftError::NotApproved(NotApproved {
            owner,
            spender: msg::sender(),
            token_id,
        }))
    }

    /// Transfers `token_id` from `from` to `to`.
    /// This function does check that `from` is the owner of the token, but it does not check
    /// that `to` is not the zero address, as this function is usable for burning.
    fn transfer_impl(&mut self, token_id: U256, from: Address, to: Address) -> Result<()> {
        let mut owner = self.owners.setter(token_id);
        let previous_owner = owner.get(); // should be in cache so this safety check is cheap
        if previous_owner != from {
            return Err(NftError::NotOwner(NotOwner {
                from,
                token_id,
                real_owner: previous_owner,
            }));
        }
        owner.set(to);

        self.approved.delete(token_id);
        evm::log(Transfer { from, to, token_id });
        Ok(())
    }
}

sol_interface! {
    interface IERC721TokenReceiver {
        function onERC721Received(address operator, address from, uint256 token_id, bytes data) external returns(bytes4);
    }
}

#[external]
impl<T: Erc712Params> Erc712<T> {
    pub fn name() -> Result<String> {
        Ok(T::NAME.into())
    }

    pub fn symbol() -> Result<String> {
        Ok(T::SYMBOL.into())
    }

    pub fn token_uri(&self, token_id: U256) -> Result<String> {
        self.owner_of(token_id)?; // require NFT exist
        Ok(T::token_uri(token_id))
    }

    pub fn supports_interface(interface: [u8; 4]) -> Result<bool> {
        if interface == [0xff; 4] {
            // special cased in the ERC165 standard
            return Ok(false);
        }

        // TODO: IERC721Enumerable, which has selector 0x780e9d63
        Ok(matches!(
            u32::from_be_bytes(interface),
            0x01ffc9a7 | // IERC165
            0x80ac58cd // IERC721
        ))
    }

    pub fn balance_of(&self, owner: Address) -> Result<U256> {
        Ok(U256::from(self.balance.get(owner)))
    }

    /// Gets the owner of the NFT, if it exists
    pub fn owner_of(&self, token_id: U256) -> Result<Address> {
        let owner = self.owners.get(token_id);
        if owner.is_zero() {
            return Err(NftError::InvalidTokenId(InvalidTokenId { token_id }));
        }
        Ok(owner)
    }

    #[selector(name = "safeTransferFrom")]
    pub fn safe_transfer_from_with_data<S: TopLevelStorage + BorrowMut<Self>>(
        storage: &mut S,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<()> {
        if to.is_zero() {
            return Err(NftError::TransferToZero(TransferToZero { token_id }));
        }
        storage
            .borrow_mut()
            .require_authorized_to_spend(from, token_id)?;

        if to.has_code() {
            let receiver = IERC721TokenReceiver::new(to);
            let received = receiver
                .on_erc_721_received(&mut *storage, msg::sender(), from, token_id, data.0)?
                .0;

            if u32::from_be_bytes(received) != 0x150b7a02 {
                return Err(NftError::ReceiverRefused(ReceiverRefused {
                    receiver: receiver.address,
                    token_id,
                    returned: received,
                }));
            }
        }
        storage.borrow_mut().transfer_impl(token_id, from, to)?;
        Ok(())
    }

    pub fn safe_transfer_from<S: TopLevelStorage + BorrowMut<Self>>(
        storage: &mut S,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<()> {
        Self::safe_transfer_from_with_data(storage, from, to, token_id, Bytes(vec![]))
    }

    pub fn transfer_from(&mut self, from: Address, to: Address, token_id: U256) -> Result<()> {
        if to.is_zero() {
            return Err(NftError::TransferToZero(TransferToZero { token_id }));
        }
        self.require_authorized_to_spend(from, token_id)?;
        self.transfer_impl(token_id, from, to)?;
        Ok(())
    }

    pub fn approve(&mut self, approved: Address, token_id: U256) -> Result<()> {
        let owner = self.owners.get(token_id);
        if owner.is_zero() {
            return Err(NftError::InvalidTokenId(InvalidTokenId { token_id }));
        }

        // require authorization
        if msg::sender() != owner && !self.approved_for_all.getter(owner).get(msg::sender()) {
            return Err(NftError::NotApproved(NotApproved {
                owner,
                spender: msg::sender(),
                token_id,
            }));
        }
        self.approved.insert(token_id, approved);

        evm::log(Approval {
            approved,
            owner,
            token_id,
        });
        Ok(())
    }

    pub fn set_approval_for_all(&mut self, operator: Address, approved: bool) -> Result<()> {
        let owner = msg::sender();
        self.approved_for_all
            .setter(owner)
            .insert(operator, approved);

        evm::log(ApprovalForAll {
            owner,
            operator,
            approved,
        });
        Ok(())
    }

    pub fn get_approved(&mut self, token_id: U256) -> Result<Address> {
        Ok(self.approved.get(token_id))
    }

    pub fn is_approved_for_all(&mut self, owner: Address, operator: Address) -> Result<bool> {
        Ok(self.approved_for_all.getter(owner).get(operator))
    }
}
