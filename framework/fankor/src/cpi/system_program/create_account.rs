use crate::errors::Error::ProgramError;
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

pub struct CpiCreateAccount<'info> {
    pub from: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
}

pub fn create_account(
    accounts: CpiCreateAccount,
    lamports: u64,
    space: u64,
    owner: &Pubkey,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = solana_program::system_instruction::create_account(
        accounts.from.key,
        accounts.to.key,
        lamports,
        space,
        owner,
    );

    solana_program::program::invoke_signed(&ix, &[accounts.from, accounts.to], signer_seeds)
        .map_or_else(|e| Err(ProgramError(e)), |_| Ok(()))
}
