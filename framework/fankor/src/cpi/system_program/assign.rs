use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

use crate::errors::Error;
use crate::models::{Program, System};
use crate::prelude::FankorResult;

pub struct CpiAssign<'info> {
    pub account_to_assign: AccountInfo<'info>,
}

pub fn assign(
    _program: &Program<System>,
    accounts: CpiAssign,
    owner: &Pubkey,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = solana_program::system_instruction::assign(accounts.account_to_assign.key, owner);

    solana_program::program::invoke_signed(&ix, &[accounts.account_to_assign], signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
