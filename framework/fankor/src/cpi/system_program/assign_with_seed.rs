use crate::errors::Error;
use crate::models::{Program, System};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

pub struct CpiAssignWithSeed<'info> {
    pub account_to_assign: AccountInfo<'info>,
    pub base: AccountInfo<'info>,
}

pub fn assign_with_seed(
    _program: &Program<System>,
    accounts: CpiAssignWithSeed,
    seed: &str,
    owner: &Pubkey,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = solana_program::system_instruction::assign_with_seed(
        accounts.account_to_assign.key,
        accounts.base.key,
        seed,
        owner,
    );

    solana_program::program::invoke_signed(
        &ix,
        &[accounts.account_to_assign, accounts.base],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
