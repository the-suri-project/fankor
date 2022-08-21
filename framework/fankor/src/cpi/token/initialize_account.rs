use crate::errors::Error;
use crate::models::{Program, Token};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiInitializeAccount<'info> {
    pub account: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub owner: AccountInfo<'info>,
}

pub fn initialize_account(
    program: &Program<Token>,
    accounts: CpiInitializeAccount,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = spl_token::instruction::initialize_account(
        program.address(),
        accounts.account.key,
        accounts.mint.key,
        accounts.owner.key,
    )?;

    solana_program::program::invoke_signed(
        &ix,
        &[accounts.account, accounts.mint, accounts.owner],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
