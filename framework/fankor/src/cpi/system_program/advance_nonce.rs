use crate::errors::Error;
use crate::models::{Program, System};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiAdvanceNonceAccount<'info> {
    pub nonce: AccountInfo<'info>,
    pub authorized: AccountInfo<'info>,
    pub recent_blockhashes: AccountInfo<'info>,
}

pub fn advance_nonce_account(
    _program: &Program<System>,
    accounts: CpiAdvanceNonceAccount,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = solana_program::system_instruction::advance_nonce_account(
        accounts.nonce.key,
        accounts.authorized.key,
    );

    solana_program::program::invoke_signed(
        &ix,
        &[
            accounts.nonce,
            accounts.recent_blockhashes,
            accounts.authorized,
        ],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
