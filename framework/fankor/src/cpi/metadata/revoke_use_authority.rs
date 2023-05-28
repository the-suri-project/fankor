use solana_program::account_info::AccountInfo;

use crate::errors::Error;
use crate::models::{Metadata, Program};
use crate::prelude::FankorResult;

pub struct CpiRevokeUseAuthority<'info> {
    pub use_authority_record: AccountInfo<'info>,
    pub user: AccountInfo<'info>,
    pub owner: AccountInfo<'info>,
    pub owner_token_account: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub rent_sysvar: AccountInfo<'info>,
}

pub fn revoke_use_authority(
    program: &Program<Metadata>,
    accounts: CpiRevokeUseAuthority,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = mpl_token_metadata::instruction::revoke_use_authority(
        *program.address(),
        *accounts.use_authority_record.key,
        *accounts.user.key,
        *accounts.owner.key,
        *accounts.owner_token_account.key,
        *accounts.metadata.key,
        *accounts.mint.key,
    );

    solana_program::program::invoke_signed(
        &ix,
        &[
            accounts.use_authority_record,
            accounts.user,
            accounts.owner,
            accounts.owner_token_account,
            accounts.metadata,
            accounts.mint,
            accounts.token_program,
            accounts.system_program,
            accounts.rent_sysvar,
        ],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
