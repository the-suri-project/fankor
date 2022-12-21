use crate::errors::Error;
use crate::models::{Program, Token2022};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiSyncNative<'info> {
    pub account: AccountInfo<'info>,
}

pub fn sync_native(
    program: &Program<Token2022>,
    accounts: CpiSyncNative,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = spl_token_2022::instruction::sync_native(program.address(), accounts.account.key)?;

    solana_program::program::invoke_signed(&ix, &[accounts.account], signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
