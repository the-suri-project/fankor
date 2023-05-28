use solana_program::account_info::AccountInfo;

use crate::errors::Error;
use crate::models::{Program, System};
use crate::prelude::FankorResult;

pub struct CpiUpgradeNonceAccount<'info> {
    pub nonce: AccountInfo<'info>,
}

pub fn upgrade_nonce_account(
    _program: &Program<System>,
    accounts: CpiUpgradeNonceAccount,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = solana_program::system_instruction::upgrade_nonce_account(*accounts.nonce.key);

    solana_program::program::invoke_signed(&ix, &[accounts.nonce], signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
