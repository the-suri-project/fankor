use solana_program::account_info::AccountInfo;

use crate::errors::Error;
use crate::models::{Program, Token};
use crate::prelude::FankorResult;

pub struct CpiCloseAccount<'info> {
    pub account: AccountInfo<'info>,
    pub destination: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
}

pub fn close_account(
    program: &Program<Token>,
    accounts: CpiCloseAccount,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = spl_token::instruction::close_account(
        program.address(),
        accounts.account.key,
        accounts.destination.key,
        accounts.authority.key,
        &[],
    )?;

    solana_program::program::invoke_signed(
        &ix,
        &[accounts.account, accounts.destination, accounts.authority],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct CpiCloseAccountMultisig<'info> {
    pub account: AccountInfo<'info>,
    pub destination: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub signers: Vec<AccountInfo<'info>>,
}

pub fn close_account_multisig(
    program: &Program<Token>,
    accounts: CpiCloseAccountMultisig,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let signer_pubkeys = accounts.signers.iter().map(|v| v.key).collect::<Vec<_>>();
    let ix = spl_token::instruction::close_account(
        program.address(),
        accounts.account.key,
        accounts.destination.key,
        accounts.authority.key,
        &signer_pubkeys,
    )?;

    let mut infos = Vec::with_capacity(3 + accounts.signers.len());
    infos.push(accounts.account);
    infos.push(accounts.destination);
    infos.push(accounts.authority);
    infos.extend(accounts.signers.into_iter());

    solana_program::program::invoke_signed(&ix, &infos, signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
