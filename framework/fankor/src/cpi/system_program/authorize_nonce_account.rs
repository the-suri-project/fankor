use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

use crate::errors::Error;
use crate::models::{Program, System};
use crate::prelude::FankorResult;

pub struct CpiAuthorizeNonceAccount<'info> {
    pub nonce: AccountInfo<'info>,
    pub authorized: AccountInfo<'info>,
}

pub fn authorize_nonce_account(
    _program: &Program<System>,
    accounts: CpiAuthorizeNonceAccount,
    new_authority: &Pubkey,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = solana_program::system_instruction::authorize_nonce_account(
        accounts.nonce.key,
        accounts.authorized.key,
        new_authority,
    );

    solana_program::program::invoke_signed(
        &ix,
        &[accounts.nonce, accounts.authorized],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
