use solana_program::account_info::AccountInfo;

use crate::errors::Error;
use crate::models::{Program, System};
use crate::prelude::FankorResult;

pub struct CpiAllocate<'info> {
    pub account_to_allocate: AccountInfo<'info>,
}

pub fn allocate(
    _program: &Program<System>,
    accounts: CpiAllocate,
    space: u64,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = solana_program::system_instruction::allocate(accounts.account_to_allocate.key, space);

    solana_program::program::invoke_signed(&ix, &[accounts.account_to_allocate], signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
