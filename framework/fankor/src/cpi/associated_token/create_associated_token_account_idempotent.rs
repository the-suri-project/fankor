use solana_program::account_info::AccountInfo;

use crate::errors::Error;
use crate::models::{AssociatedToken, Program};
use crate::prelude::FankorResult;

pub struct CpiCreateAssociatedTokenAccountIdempotent<'info> {
    pub funding_address: AccountInfo<'info>,
    pub wallet_address: AccountInfo<'info>,
    pub token_mint_address: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

pub fn create_associated_token_account_idempotent(
    _program: &Program<AssociatedToken>,
    accounts: CpiCreateAssociatedTokenAccountIdempotent,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = spl_associated_token_account::instruction::create_associated_token_account_idempotent(
        accounts.funding_address.key,
        accounts.wallet_address.key,
        accounts.token_mint_address.key,
        accounts.token_program.key,
    );

    solana_program::program::invoke_signed(
        &ix,
        &[
            accounts.funding_address,
            accounts.wallet_address,
            accounts.token_mint_address,
            accounts.token_program,
        ],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
