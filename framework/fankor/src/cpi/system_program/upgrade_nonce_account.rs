use crate::errors::Error::ProgramError;
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiUpgradeNonceAccount<'info> {
    pub nonce: AccountInfo<'info>,
}

pub fn upgrade_nonce_account(
    accounts: CpiUpgradeNonceAccount,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = solana_program::system_instruction::upgrade_nonce_account(*accounts.nonce.key);

    solana_program::program::invoke_signed(&ix, &[accounts.nonce], signer_seeds)
        .map_or_else(|e| Err(ProgramError(e)), |_| Ok(()))
}
