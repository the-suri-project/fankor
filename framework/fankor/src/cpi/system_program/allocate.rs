use crate::errors::Error::ProgramError;
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiAllocate<'info> {
    pub account_to_allocate: AccountInfo<'info>,
}

pub fn allocate(accounts: CpiAllocate, space: u64, signer_seeds: &[&[&[u8]]]) -> FankorResult<()> {
    let ix = solana_program::system_instruction::allocate(accounts.account_to_allocate.key, space);

    solana_program::program::invoke_signed(&ix, &[accounts.account_to_allocate], signer_seeds)
        .map_or_else(|e| Err(ProgramError(e)), |_| Ok(()))
}
