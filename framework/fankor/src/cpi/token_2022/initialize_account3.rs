use crate::errors::Error;
use crate::models::{Program, Token2022};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

pub struct CpiInitializeAccount3<'info> {
    pub account: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
}

pub fn initialize_account3(
    program: &Program<Token2022>,
    accounts: CpiInitializeAccount3,
    owner: &Pubkey,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = spl_token_2022::instruction::initialize_account3(
        program.address(),
        accounts.account.key,
        accounts.mint.key,
        owner,
    )?;

    solana_program::program::invoke_signed(&ix, &[accounts.account, accounts.mint], signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
