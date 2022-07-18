use crate::errors::Error::ProgramError;
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

pub struct CpiAssign<'info> {
    pub account_to_assign: AccountInfo<'info>,
}

pub fn assign(accounts: CpiAssign, owner: &Pubkey, signer_seeds: &[&[&[u8]]]) -> FankorResult<()> {
    let ix = solana_program::system_instruction::assign(accounts.account_to_assign.key, owner);

    solana_program::program::invoke_signed(&ix, &[accounts.account_to_assign], signer_seeds)
        .map_or_else(|e| Err(ProgramError(e)), |_| Ok(()))
}
