use crate::errors::Error;
use crate::models::{Program, Token};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiTransfer<'info> {
    pub from: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
}

pub fn transfer(
    program: &Program<Token>,
    accounts: CpiTransfer,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = spl_token::instruction::transfer(
        program.address(),
        accounts.from.key,
        accounts.to.key,
        accounts.authority.key,
        &[],
        amount,
    )?;

    solana_program::program::invoke_signed(
        &ix,
        &[accounts.from, accounts.to, accounts.authority],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct CpiTransferMultisig<'info> {
    pub from: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub signers: Vec<AccountInfo<'info>>,
}

pub fn transfer_multisig(
    program: &Program<Token>,
    accounts: CpiTransferMultisig,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let signer_pubkeys = accounts.signers.iter().map(|v| v.key).collect::<Vec<_>>();
    let ix = spl_token::instruction::transfer(
        program.address(),
        accounts.from.key,
        accounts.to.key,
        accounts.authority.key,
        &signer_pubkeys,
        amount,
    )?;

    let mut infos = Vec::with_capacity(3 + accounts.signers.len());
    infos.push(accounts.from);
    infos.push(accounts.to);
    infos.push(accounts.authority);
    infos.extend(accounts.signers.into_iter());

    solana_program::program::invoke_signed(&ix, &infos, signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
