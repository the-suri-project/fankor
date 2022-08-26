use crate::errors::Error;
use crate::models::{Metadata, Program};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiCreateMasterEditionV3<'info> {
    pub edition: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub update_authority: AccountInfo<'info>,
    pub mint_authority: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub rent_sysvar: AccountInfo<'info>,
}

pub fn create_master_edition_v3(
    program: &Program<Metadata>,
    accounts: CpiCreateMasterEditionV3,
    max_supply: Option<u64>,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = mpl_token_metadata::instruction::create_master_edition_v3(
        *program.address(),
        *accounts.edition.key,
        *accounts.mint.key,
        *accounts.update_authority.key,
        *accounts.mint_authority.key,
        *accounts.metadata.key,
        *accounts.payer.key,
        max_supply,
    );

    solana_program::program::invoke_signed(
        &ix,
        &[
            accounts.edition,
            accounts.mint,
            accounts.update_authority,
            accounts.mint_authority,
            accounts.metadata,
            accounts.payer,
            accounts.token_program,
            accounts.system_program,
            accounts.rent_sysvar,
        ],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
