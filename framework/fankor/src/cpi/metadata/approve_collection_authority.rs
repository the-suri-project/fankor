use solana_program::account_info::AccountInfo;

use crate::errors::Error;
use crate::models::{Metadata, Program};
use crate::prelude::FankorResult;

pub struct CpiApproveCollectionAuthority<'info> {
    pub collection_authority_record: AccountInfo<'info>,
    pub new_collection_authority: AccountInfo<'info>,
    pub update_authority: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub rent_sysvar: AccountInfo<'info>,
}

pub fn approve_collection_authority(
    program: &Program<Metadata>,
    accounts: CpiApproveCollectionAuthority,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = mpl_token_metadata::instruction::approve_collection_authority(
        *program.address(),
        *accounts.collection_authority_record.key,
        *accounts.new_collection_authority.key,
        *accounts.update_authority.key,
        *accounts.payer.key,
        *accounts.metadata.key,
        *accounts.mint.key,
    );

    solana_program::program::invoke_signed(
        &ix,
        &[
            accounts.collection_authority_record,
            accounts.new_collection_authority,
            accounts.update_authority,
            accounts.payer,
            accounts.metadata,
            accounts.mint,
            accounts.system_program,
            accounts.rent_sysvar,
        ],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
