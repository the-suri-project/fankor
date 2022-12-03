use crate::errors::FankorResult;
use crate::models::programs::macros::impl_account;
use crate::traits::{AccountDeserialize, AccountSerialize, ProgramType};
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata::state::{
    TokenMetadataAccount, BURN, COLLECTION_AUTHORITY, EDITION, PREFIX, USER,
};
use solana_program::pubkey::Pubkey;
use std::io::{ErrorKind, Write};
use std::ops::Deref;

#[derive(Debug, Copy, Clone)]
pub struct Metadata;

impl ProgramType for Metadata {
    fn name() -> &'static str {
        "Metadata"
    }

    fn address() -> &'static Pubkey {
        &mpl_token_metadata::ID
    }
}

impl Metadata {
    // METHODS ----------------------------------------------------------------

    pub fn get_edition_pda_seeds<'a>(mint: &'a Pubkey, edition_number: &'a str) -> [&'a [u8]; 5] {
        [
            PREFIX.as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            mint.as_ref(),
            EDITION.as_bytes(),
            edition_number.as_bytes(),
        ]
    }

    pub fn get_master_edition_pda_seeds(mint: &Pubkey) -> [&[u8]; 4] {
        [
            PREFIX.as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            mint.as_ref(),
            EDITION.as_bytes(),
        ]
    }

    pub fn get_metadata_pda_seeds(mint: &Pubkey) -> [&[u8]; 3] {
        [
            PREFIX.as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            mint.as_ref(),
        ]
    }

    pub fn get_use_authority_pda_seeds<'a>(
        mint: &'a Pubkey,
        authority: &'a Pubkey,
    ) -> [&'a [u8]; 5] {
        [
            PREFIX.as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            mint.as_ref(),
            USER.as_bytes(),
            authority.as_ref(),
        ]
    }

    pub fn get_collection_pda_seeds<'a>(mint: &'a Pubkey, authority: &'a Pubkey) -> [&'a [u8]; 5] {
        [
            PREFIX.as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            mint.as_ref(),
            COLLECTION_AUTHORITY.as_bytes(),
            authority.as_ref(),
        ]
    }

    pub fn get_program_as_burner_pda_seeds<'a>() -> [&'a [u8]; 3] {
        [
            PREFIX.as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            BURN.as_bytes(),
        ]
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
    [Eq]
);

impl_account!(
    MasterEditionV1,
    mpl_token_metadata::state::MasterEditionV1,
    &mpl_token_metadata::ID,
    deserialize,
    safe_deserialize,
    [Eq]
);

impl_account!(
    MasterEditionV2,
    mpl_token_metadata::state::MasterEditionV2,
    &mpl_token_metadata::ID,
    deserialize,
    safe_deserialize,
    [Eq]
);

impl_account!(
    Edition,
    mpl_token_metadata::state::Edition,
    &mpl_token_metadata::ID,
    deserialize,
    safe_deserialize,
    [Eq]
);

impl_account!(
    ReservationListV1,
    mpl_token_metadata::state::ReservationListV1,
    &mpl_token_metadata::ID,
    deserialize,
    safe_deserialize,
    [Eq]
);

impl_account!(
    ReservationListV2,
    mpl_token_metadata::state::ReservationListV2,
    &mpl_token_metadata::ID,
    deserialize,
    safe_deserialize,
    [Eq]
);

impl_account!(
    EditionMarker,
    mpl_token_metadata::state::EditionMarker,
    &mpl_token_metadata::ID,
    deserialize,
    safe_deserialize,
    [Eq]
);

impl_account!(
    UseAuthorityRecord,
    mpl_token_metadata::state::UseAuthorityRecord,
    &mpl_token_metadata::ID,
    deserialize,
    safe_deserialize,
    [Eq]
);

impl_account!(
    CollectionAuthorityRecord,
    mpl_token_metadata::state::CollectionAuthorityRecord,
    &mpl_token_metadata::ID,
    deserialize,
    safe_deserialize,
    [Eq]
);
