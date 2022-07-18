use crate::errors::Error;
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

pub struct CpiInitializeAccount2<'info> {
    pub account: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
}

pub fn initialize_account2(
    accounts: CpiInitializeAccount2,
    owner: &Pubkey,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = spl_token::instruction::initialize_account2(
        &spl_token::ID,
        accounts.account.key,
        accounts.mint.key,
        owner,
    )?;

    solana_program::program::invoke_signed(&ix, &[accounts.account, accounts.mint], signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
