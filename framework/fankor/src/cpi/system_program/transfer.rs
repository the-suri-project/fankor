use solana_program::account_info::AccountInfo;

use crate::errors::Error;
use crate::models::{Program, System};
use crate::prelude::FankorResult;

pub struct CpiTransfer<'info> {
    pub from: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
}

pub fn transfer(
    _program: &Program<System>,
    accounts: CpiTransfer,
    lamports: u64,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix =
        solana_program::system_instruction::transfer(accounts.from.key, accounts.to.key, lamports);

    solana_program::program::invoke_signed(&ix, &[accounts.from, accounts.to], signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
