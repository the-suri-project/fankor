use solana_program::account_info::AccountInfo;

use crate::errors::Error;
use crate::models::{Metadata, Program};
use crate::prelude::FankorResult;

pub struct CpiUtilize<'info> {
    pub metadata: AccountInfo<'info>,
    pub token_account: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub use_authority_record_pda: Option<AccountInfo<'info>>,
    pub use_authority: AccountInfo<'info>,
    pub owner: AccountInfo<'info>,
    pub burner: Option<AccountInfo<'info>>,
    pub token_program: AccountInfo<'info>,
    pub associated_token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub rent_sysvar: AccountInfo<'info>,
}

pub fn utilize(
    program: &Program<Metadata>,
    accounts: CpiUtilize,
    number_of_uses: u64,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = mpl_token_metadata::instruction::utilize(
        *program.address(),
        *accounts.metadata.key,
        *accounts.token_account.key,
        *accounts.mint.key,
        accounts.use_authority_record_pda.as_ref().map(|a| *a.key),
        *accounts.use_authority.key,
        *accounts.owner.key,
        accounts.burner.as_ref().map(|a| *a.key),
        number_of_uses,
    );

    let mut infos = Vec::with_capacity(11);
    infos.push(accounts.metadata);
    infos.push(accounts.token_account);
    infos.push(accounts.mint);
    infos.push(accounts.use_authority);
    infos.push(accounts.owner);
    infos.push(accounts.token_program);
    infos.push(accounts.associated_token_program);
    infos.push(accounts.system_program);
    infos.push(accounts.rent_sysvar);

    if let Some(use_authority_record_pda) = accounts.use_authority_record_pda {
        infos.push(use_authority_record_pda);
    }

    if let Some(burner) = accounts.burner {
        infos.push(burner);
    }

    solana_program::program::invoke_signed(&ix, &infos, signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
