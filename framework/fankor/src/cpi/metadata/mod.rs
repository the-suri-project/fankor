pub use approve_collection_authority::*;
pub use approve_use_authority::*;
pub use burn_nft::*;
pub use create_master_edition_v3::*;
pub use create_metadata_accounts_v3::*;
pub use mint_edition_from_master_edition_via_vault_proxy::*;
pub use revoke_use_authority::*;
pub use set_and_verify_sized_collection_item::*;
pub use set_collection_size::*;
pub use set_token_standard::*;
pub use unverify_collection::*;
pub use unverify_sized_collection_item::*;
pub use utilize::*;
pub use verify_collection::*;
pub use verify_sized_collection_item::*;

use crate::cpi::macros::impl_cpi_method;
use crate::errors::Error;
use crate::models::{Metadata, Program};
use crate::prelude::AccountInfo;
use crate::prelude::FankorResult;
use mpl_token_metadata::state::DataV2;
use solana_program::pubkey::Pubkey;

mod approve_collection_authority;
mod approve_use_authority;
mod burn_nft;
mod create_master_edition_v3;
mod create_metadata_accounts_v3;
mod mint_edition_from_master_edition_via_vault_proxy;
mod revoke_use_authority;
mod set_and_verify_sized_collection_item;
mod set_collection_size;
mod set_token_standard;
mod unverify_collection;
mod unverify_sized_collection_item;
mod utilize;
mod verify_collection;
mod verify_sized_collection_item;

macro_rules! impl_metadata_cpi_method {
    ($cpi_name: ident, $name: ident, accounts: [$($accounts:ident),* $(,)?], args: [$($arg_keys:ident : $arg_types: ty),* $(,)?] $(,)?) => {
        impl_cpi_method!(Metadata, $cpi_name, $name, mpl_token_metadata::instruction::$name, accounts: [$($accounts),*], args: [$($arg_keys: $arg_types),*], account_access_token: *);
    };
}

// ----------------------------------------------------------------------------
// ACCOUNTS -------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl_metadata_cpi_method!(
    CpiUpdateMetadataAccountV2,
    update_metadata_accounts_v2,
    accounts: [
        metadata,
        update_authority,
    ],
    args: [
        new_update_authority: Option<Pubkey>,
        data: Option<DataV2>,
        primary_sale_happened: Option<bool>,
        is_mutable: Option<bool>,
    ]
);

impl_metadata_cpi_method!(
    CpiPuffMetadataAccount,
    puff_metadata_account,
    accounts: [
        metadata,
    ],
    args: []
);

impl_metadata_cpi_method!(
    CpiUpdatePrimarySaleHappenedViaToken,
    update_primary_sale_happened_via_token,
    accounts: [
        metadata,
        owner,
        token,
    ],
    args: []
);

// mint_new_edition_from_master_edition_via_token cannot be mapped to a CPI call because uses Pubkey::find_program_address,
// therefore it cannot be checked that the derived pubkey is present in the current instruction.

impl_metadata_cpi_method!(
    CpiSignMetadata,
    sign_metadata,
    accounts: [
        metadata,
        creator,
    ],
    args: []
);

impl_metadata_cpi_method!(
    CpiRemoveCreatorVerification,
    remove_creator_verification,
    accounts: [
        metadata,
        creator,
    ],
    args: []
);

impl_metadata_cpi_method!(
    CpiConvertMasterEditionV1ToV2,
    convert_master_edition_v1_to_v2,
    accounts: [
        master_edition,
        one_time_auth,
        printing_mint,
    ],
    args: []
);

impl_metadata_cpi_method!(
    CpiRevokeCollectionAuthority,
    revoke_collection_authority,
    accounts: [
        collection_authority_record,
        delegate_authority,
        revoke_authority,
        metadata,
        mint,
    ],
    args: []
);

impl_metadata_cpi_method!(
    CpiFreezeDelegatedAccount,
    freeze_delegated_account,
    accounts: [
        delegate,
        token_account,
        edition,
        mint,
    ],
    args: []
);

impl_metadata_cpi_method!(
    CpiThawDelegatedAccount,
    thaw_delegated_account,
    accounts: [
        delegate,
        token_account,
        edition,
        mint,
    ],
    args: []
);
