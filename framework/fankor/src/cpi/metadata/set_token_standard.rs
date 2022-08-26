use crate::errors::Error;
use crate::models::{Metadata, Program};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiSetTokenStandard<'info> {
    pub metadata: AccountInfo<'info>,
    pub update_authority: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub edition: Option<AccountInfo<'info>>,
}

pub fn set_token_standard(
    program: &Program<Metadata>,
    accounts: CpiSetTokenStandard,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = mpl_token_metadata::instruction::set_token_standard(
        *program.address(),
        *accounts.metadata.key,
        *accounts.update_authority.key,
        *accounts.mint.key,
        accounts.edition.as_ref().map(|e| *e.key),
    );

    let mut infos = Vec::with_capacity(4);
    infos.push(accounts.metadata);
    infos.push(accounts.update_authority);
    infos.push(accounts.mint);

    if let Some(edition) = accounts.edition {
        infos.push(edition);
    }

    solana_program::program::invoke_signed(&ix, &infos, signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
