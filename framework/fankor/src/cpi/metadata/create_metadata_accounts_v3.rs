use mpl_token_metadata::state::{Collection, CollectionDetails, Creator, Uses};
use solana_program::account_info::AccountInfo;

use crate::errors::Error;
use crate::models::{Metadata, Program};
use crate::prelude::FankorResult;

pub struct CpiCreateMetadataAccountV3<'info> {
    pub metadata: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub mint_authority: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub update_authority: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub rent_sysvar: AccountInfo<'info>,
}

#[allow(clippy::too_many_arguments)]
pub fn create_metadata_accounts_v3(
    program: &Program<Metadata>,
    accounts: CpiCreateMetadataAccountV3,
    name: String,
    symbol: String,
    uri: String,
    creators: Option<Vec<Creator>>,
    seller_fee_basis_points: u16,
    update_authority_is_signer: bool,
    is_mutable: bool,
    collection: Option<Collection>,
    uses: Option<Uses>,
    collection_details: Option<CollectionDetails>,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = mpl_token_metadata::instruction::create_metadata_accounts_v3(
        *program.address(),
        *accounts.metadata.key,
        *accounts.mint.key,
        *accounts.mint_authority.key,
        *accounts.payer.key,
        *accounts.update_authority.key,
        name,
        symbol,
        uri,
        creators,
        seller_fee_basis_points,
        update_authority_is_signer,
        is_mutable,
        collection,
        uses,
        collection_details,
    );

    solana_program::program::invoke_signed(
        &ix,
        &[
            accounts.metadata,
            accounts.mint,
            accounts.mint_authority,
            accounts.payer,
            accounts.update_authority,
            accounts.system_program,
            accounts.rent_sysvar,
        ],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
