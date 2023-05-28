use solana_program::account_info::AccountInfo;

use crate::errors::Error;
use crate::models::{AssociatedToken, Program};
use crate::prelude::FankorResult;

pub struct CpiRecoverNested<'info> {
    pub wallet_address: AccountInfo<'info>,
    pub owner_token_mint_address: AccountInfo<'info>,
    pub nested_token_mint_address: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

pub fn recover_nested(
    _program: &Program<AssociatedToken>,
    accounts: CpiRecoverNested,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = spl_associated_token_account::instruction::recover_nested(
        accounts.wallet_address.key,
        accounts.owner_token_mint_address.key,
        accounts.nested_token_mint_address.key,
        accounts.token_program.key,
    );

    solana_program::program::invoke_signed(
        &ix,
        &[
            accounts.wallet_address,
            accounts.owner_token_mint_address,
            accounts.nested_token_mint_address,
            accounts.token_program,
        ],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
