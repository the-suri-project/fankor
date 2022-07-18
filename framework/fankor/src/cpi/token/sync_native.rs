use crate::errors::Error;
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiSyncNative<'info> {
    pub account: AccountInfo<'info>,
}

pub fn sync_native(accounts: CpiSyncNative, signer_seeds: &[&[&[u8]]]) -> FankorResult<()> {
    let ix = spl_token::instruction::sync_native(&spl_token::ID, accounts.account.key)?;

    solana_program::program::invoke_signed(&ix, &[accounts.account], signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
