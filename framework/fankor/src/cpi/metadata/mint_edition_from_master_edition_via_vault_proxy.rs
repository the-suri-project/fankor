use crate::errors::Error;
use crate::models::{Metadata, Program};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiMintNewEditionFromMasterEditionViaVaultProxy<'info> {
    pub new_metadata: AccountInfo<'info>,
    pub new_edition: AccountInfo<'info>,
    pub master_edition: AccountInfo<'info>,
    pub new_mint: AccountInfo<'info>,
    pub edition_mark_pda: AccountInfo<'info>,
    pub new_mint_authority: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub vault_authority: AccountInfo<'info>,
    pub safety_deposit_store: AccountInfo<'info>,
    pub safety_deposit_box: AccountInfo<'info>,
    pub vault: AccountInfo<'info>,
    pub new_metadata_update_authority: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub token_vault_program_info: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub rent_sysvar: AccountInfo<'info>,
}

pub fn mint_edition_from_master_edition_via_vault_proxy(
    program: &Program<Metadata>,
    accounts: CpiMintNewEditionFromMasterEditionViaVaultProxy,
    edition: u64,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = mpl_token_metadata::instruction::mint_edition_from_master_edition_via_vault_proxy(
        *program.address(),
        *accounts.new_metadata.key,
        *accounts.new_edition.key,
        *accounts.master_edition.key,
        *accounts.new_mint.key,
        *accounts.edition_mark_pda.key,
        *accounts.new_mint_authority.key,
        *accounts.payer.key,
        *accounts.vault_authority.key,
        *accounts.safety_deposit_store.key,
        *accounts.safety_deposit_box.key,
        *accounts.vault.key,
        *accounts.new_metadata_update_authority.key,
        *accounts.metadata.key,
        *accounts.token_program.key,
        *accounts.token_vault_program_info.key,
        edition,
    );

    solana_program::program::invoke_signed(
        &ix,
        &[
            accounts.new_metadata,
            accounts.new_edition,
            accounts.master_edition,
            accounts.new_mint,
            accounts.edition_mark_pda,
            accounts.new_mint_authority,
            accounts.payer,
            accounts.vault_authority,
            accounts.safety_deposit_store,
            accounts.safety_deposit_box,
            accounts.vault,
            accounts.new_metadata_update_authority,
            accounts.metadata,
            accounts.token_program,
            accounts.token_vault_program_info,
            accounts.system_program,
            accounts.rent_sysvar,
        ],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
