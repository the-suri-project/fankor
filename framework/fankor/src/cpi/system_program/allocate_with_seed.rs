use crate::errors::Error;
use crate::models::{Program, System};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

pub struct CpiAllocateWithSeed<'info> {
    pub account_to_allocate: AccountInfo<'info>,
    pub base: AccountInfo<'info>,
}

pub fn allocate_with_seed(
    _program: &Program<System>,
    accounts: CpiAllocateWithSeed,
    seed: &str,
    space: u64,
    owner: &Pubkey,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = solana_program::system_instruction::allocate_with_seed(
        accounts.account_to_allocate.key,
        accounts.base.key,
        seed,
        space,
        owner,
    );

    solana_program::program::invoke_signed(
        &ix,
        &[accounts.account_to_allocate, accounts.base],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
