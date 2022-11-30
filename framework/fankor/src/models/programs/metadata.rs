use crate::errors::FankorResult;
use crate::models::programs::macros::impl_account;
use crate::traits::{AccountDeserialize, AccountSerialize, ProgramType};
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata::state::TokenMetadataAccount;
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
