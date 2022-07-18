use crate::errors::Error;
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

pub struct CpiCreateAccountWithSeed<'info> {
    pub from: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
    pub base: AccountInfo<'info>,
}

pub fn create_account_with_seed(
    accounts: CpiCreateAccountWithSeed,
    seed: &str,
    lamports: u64,
    space: u64,
    owner: &Pubkey,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = solana_program::system_instruction::create_account_with_seed(
        accounts.from.key,
        accounts.to.key,
        accounts.base.key,
        seed,
        lamports,
        space,
        owner,
    );

    solana_program::program::invoke_signed(
        &ix,
        &[accounts.from, accounts.to, accounts.base],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
