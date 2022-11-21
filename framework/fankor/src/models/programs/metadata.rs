use crate::errors::FankorErrorCode;
use crate::errors::FankorResult;
use crate::models::programs::macros::{impl_account, impl_zero_copy_account};
use crate::prelude::ZeroCopyType;
use crate::prelude::ZC;
use crate::traits::{AccountDeserialize, AccountSerialize, Program};
use borsh::{BorshDeserialize, BorshSerialize};
use core::any::type_name;
use mpl_token_metadata::state::TokenMetadataAccount;
use solana_program::pubkey::Pubkey;
use std::io::{ErrorKind, Write};
use std::marker::PhantomData;
use std::ops::Deref;

#[derive(Debug, Copy, Clone)]
pub struct Metadata;

impl Program for Metadata {
    fn name() -> &'static str {
        "Metadata"
    }

    fn address() -> &'static Pubkey {
        &mpl_token_metadata::ID
    }
}

// ----------------------------------------------------------------------------
// ACCOUNTS -------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl_account!(
    MetadataAccount,
    mpl_token_metadata::state::Metadata,
    &mpl_token_metadata::ID,
    deserialize,
    safe_deserialize,
    [ZC: mpl_token_metadata::state::Metadata::size()],
    [Eq]
);

impl_account!(
    MasterEditionV1,
    mpl_token_metadata::state::MasterEditionV1,
    &mpl_token_metadata::ID,
    deserialize,
    safe_deserialize,
    [ZC: mpl_token_metadata::state::MasterEditionV1::size()],
    [Eq]
);

impl_account!(
    MasterEditionV2,
    mpl_token_metadata::state::MasterEditionV2,
    &mpl_token_metadata::ID,
    deserialize,
    safe_deserialize,
    [ZC: mpl_token_metadata::state::MasterEditionV2::size()],
    [Eq]
);

impl_account!(
    Edition,
    mpl_token_metadata::state::Edition,
    &mpl_token_metadata::ID,
    deserialize,
    safe_deserialize,
    [ZC: mpl_token_metadata::state::Edition::size()],
    [Eq]
);

impl_account!(
    ReservationListV1,
    mpl_token_metadata::state::ReservationListV1,
    &mpl_token_metadata::ID,
    deserialize,
    safe_deserialize,
    [ZC: mpl_token_metadata::state::ReservationListV1::size()],
    [Eq]
);

impl_account!(
    ReservationListV2,
    mpl_token_metadata::state::ReservationListV2,
    &mpl_token_metadata::ID,
    deserialize,
    safe_deserialize,
    [ZC: mpl_token_metadata::state::ReservationListV2::size()],
    [Eq]
);

impl_account!(
    EditionMarker,
    mpl_token_metadata::state::EditionMarker,
    &mpl_token_metadata::ID,
    deserialize,
    safe_deserialize,
    [ZC: mpl_token_metadata::state::EditionMarker::size()],
    [Eq]
);

impl_account!(
    UseAuthorityRecord,
    mpl_token_metadata::state::UseAuthorityRecord,
    &mpl_token_metadata::ID,
    deserialize,
    safe_deserialize,
    [ZC: mpl_token_metadata::state::UseAuthorityRecord::size()],
    [Eq]
);

impl_account!(
    CollectionAuthorityRecord,
    mpl_token_metadata::state::CollectionAuthorityRecord,
    &mpl_token_metadata::ID,
    deserialize,
    safe_deserialize,
    [ZC: mpl_token_metadata::state::CollectionAuthorityRecord::size()],
    [Eq]
);

// ----------------------------------------------------------------------------
// Zero Copy ------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl_zero_copy_account!(
    MetadataAccount,
    key: mpl_token_metadata::state::Key,
    update_authority: Pubkey,
    mint: Pubkey,
    data: mpl_token_metadata::state::Data,
    primary_sale_happened: bool,
    is_mutable: bool,
    edition_nonce: Option<u8>,
    token_standard: Option<mpl_token_metadata::state::TokenStandard>,
    collection: Option<mpl_token_metadata::state::Collection>,
    uses: Option<mpl_token_metadata::state::Uses>,
    collection_details: Option<mpl_token_metadata::state::CollectionDetails>,
);

impl_zero_copy_account!(
    MasterEditionV1,
    key: mpl_token_metadata::state::Key,
    supply: u64,
    max_supply: Option<u64>,
    printing_mint: Pubkey,
    one_time_printing_authorization_mint: Pubkey,
);

impl_zero_copy_account!(
    MasterEditionV2,
    key: mpl_token_metadata::state::Key,
    supply: u64,
    max_supply: Option<u64>,
);

impl_zero_copy_account!(
    Edition,
    key: mpl_token_metadata::state::Key,
    parent: Pubkey,
    edition: u64,
);

impl_zero_copy_account!(
    ReservationListV1,
    key: mpl_token_metadata::state::Key,
    master_edition: Pubkey,
    supply_snapshot: Option<u64>,
    reservations: Vec<mpl_token_metadata::state::ReservationV1>,
);

impl_zero_copy_account!(
    ReservationListV2,
    key: mpl_token_metadata::state::Key,
    master_edition: Pubkey,
    supply_snapshot: Option<u64>,
    reservations: Vec<mpl_token_metadata::state::Reservation>,
    total_reservation_spots: u64,
    current_reservation_spots: u64,
);

impl_zero_copy_account!(
    EditionMarker,
    key: mpl_token_metadata::state::Key,
    edger: [u8; 31],
);

impl_zero_copy_account!(
    UseAuthorityRecord,
    key: mpl_token_metadata::state::Key,
    allowed_uses: u64,
    bump: u8
);

impl_zero_copy_account!(
    CollectionAuthorityRecord,
    key: mpl_token_metadata::state::Key,
    bump: u8
);

impl ZeroCopyType for mpl_token_metadata::state::Key {
    fn byte_size_from_instance(&self) -> usize {
        1
    }

    fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
        if bytes.is_empty() {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        Ok(1)
    }
}

impl ZeroCopyType for mpl_token_metadata::state::Data {
    fn byte_size_from_instance(&self) -> usize {
        self.name.byte_size_from_instance()
            + self.symbol.byte_size_from_instance()
            + self.uri.byte_size_from_instance()
            + self.seller_fee_basis_points.byte_size_from_instance()
            + self.creators.byte_size_from_instance()
    }

    fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
        let mut size = Pubkey::byte_size(bytes)?;
        size += String::byte_size(&bytes[size..])?;
        size += String::byte_size(&bytes[size..])?;
        size += String::byte_size(&bytes[size..])?;
        size += u16::byte_size(&bytes[size..])?;
        size += <Option<Vec<mpl_token_metadata::state::Creator>>>::byte_size(&bytes[size..])?;

        Ok(size)
    }
}

impl ZeroCopyType for mpl_token_metadata::state::Creator {
    fn byte_size_from_instance(&self) -> usize {
        self.address.byte_size_from_instance()
            + self.verified.byte_size_from_instance()
            + self.share.byte_size_from_instance()
    }

    fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
        let mut size = Pubkey::byte_size(bytes)?;
        size += bool::byte_size(&bytes[size..])?;
        size += u8::byte_size(&bytes[size..])?;

        Ok(size)
    }
}

impl ZeroCopyType for mpl_token_metadata::state::TokenStandard {
    fn byte_size_from_instance(&self) -> usize {
        1
    }

    fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
        if bytes.is_empty() {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        Ok(1)
    }
}

impl ZeroCopyType for mpl_token_metadata::state::Collection {
    fn byte_size_from_instance(&self) -> usize {
        self.verified.byte_size_from_instance() + self.key.byte_size_from_instance()
    }

    fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
        let mut size = Pubkey::byte_size(bytes)?;
        size += bool::byte_size(&bytes[size..])?;
        size += Pubkey::byte_size(&bytes[size..])?;

        Ok(size)
    }
}

impl ZeroCopyType for mpl_token_metadata::state::Uses {
    fn byte_size_from_instance(&self) -> usize {
        self.use_method.byte_size_from_instance()
            + self.remaining.byte_size_from_instance()
            + self.total.byte_size_from_instance()
    }

    fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
        let mut size = Pubkey::byte_size(bytes)?;
        size += mpl_token_metadata::state::UseMethod::byte_size(&bytes[size..])?;
        size += u64::byte_size(&bytes[size..])?;
        size += u64::byte_size(&bytes[size..])?;

        Ok(size)
    }
}

impl ZeroCopyType for mpl_token_metadata::state::CollectionDetails {
    fn byte_size_from_instance(&self) -> usize {
        match self {
            Self::V1 { size } => 1 + size.byte_size_from_instance(),
        }
    }

    fn byte_size(mut bytes: &[u8]) -> FankorResult<usize> {
        let flag = u8::deserialize(&mut bytes)?;

        let size = match flag {
            0 => 1 + u64::byte_size(bytes)?,
            _ => {
                return Err(FankorErrorCode::ZeroCopyInvalidEnumDiscriminator {
                    type_name: type_name::<Self>(),
                }
                .into());
            }
        };

        Ok(size)
    }
}

impl ZeroCopyType for mpl_token_metadata::state::UseMethod {
    fn byte_size_from_instance(&self) -> usize {
        1
    }

    fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
        if bytes.is_empty() {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        Ok(1)
    }
}

impl ZeroCopyType for mpl_token_metadata::state::ReservationV1 {
    fn byte_size_from_instance(&self) -> usize {
        self.address.byte_size_from_instance()
            + self.spots_remaining.byte_size_from_instance()
            + self.total_spots.byte_size_from_instance()
    }

    fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
        let mut size = Pubkey::byte_size(bytes)?;
        size += Pubkey::byte_size(&bytes[size..])?;
        size += u8::byte_size(&bytes[size..])?;
        size += u8::byte_size(&bytes[size..])?;

        Ok(size)
    }
}

impl ZeroCopyType for mpl_token_metadata::state::Reservation {
    fn byte_size_from_instance(&self) -> usize {
        self.address.byte_size_from_instance()
            + self.spots_remaining.byte_size_from_instance()
            + self.total_spots.byte_size_from_instance()
    }

    fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
        let mut size = Pubkey::byte_size(bytes)?;
        size += Pubkey::byte_size(&bytes[size..])?;
        size += u64::byte_size(&bytes[size..])?;
        size += u64::byte_size(&bytes[size..])?;

        Ok(size)
    }
}

impl_zero_copy_account!(
    mpl_token_metadata::state::Data,
    name: String,
    symbol: String,
    uri: String,
    seller_fee_basis_points: u16,
    creators: Option<Vec<mpl_token_metadata::state::Creator>>,
);

impl_zero_copy_account!(
    mpl_token_metadata::state::Creator,
    address: Pubkey,
    verified: bool,
    share: u8,
);

impl_zero_copy_account!(
    mpl_token_metadata::state::Collection,
    verified: bool,
    key: Pubkey,
);

impl_zero_copy_account!(
    mpl_token_metadata::state::Collection,
    use_method: mpl_token_metadata::state::UseMethod,
    remaining: u64,
    total: u64,
);

impl_zero_copy_account!(
    mpl_token_metadata::state::ReservationV1,
    address: Pubkey,
    spots_remaining: u8,
    total_spots: u8,
);

impl_zero_copy_account!(
    mpl_token_metadata::state::Reservation,
    address: Pubkey,
    spots_remaining: u64,
    total_spots: u64,
);
