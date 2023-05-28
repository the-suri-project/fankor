use solana_program::account_info::AccountInfo;

use crate::errors::Error;
use crate::models::{Program, Token};
use crate::prelude::FankorResult;

pub struct CpiApproveChecked<'info> {
    pub source: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub delegate: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
}

pub fn approve_checked(
    program: &Program<Token>,
    accounts: CpiApproveChecked,
    amount: u64,
    decimals: u8,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = spl_token::instruction::approve_checked(
        program.address(),
        accounts.source.key,
        accounts.mint.key,
        accounts.delegate.key,
        accounts.authority.key,
        &[],
        amount,
        decimals,
    )?;

    solana_program::program::invoke_signed(
        &ix,
        &[
            accounts.source,
            accounts.mint,
            accounts.delegate,
            accounts.authority,
        ],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct CpiApproveCheckedMultisig<'info> {
    pub source: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub delegate: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub signers: Vec<AccountInfo<'info>>,
}

pub fn approve_checked_multisig(
    program: &Program<Token>,
    accounts: CpiApproveCheckedMultisig,
    amount: u64,
    decimals: u8,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let signer_pubkeys = accounts.signers.iter().map(|v| v.key).collect::<Vec<_>>();
    let ix = spl_token::instruction::approve_checked(
        program.address(),
        accounts.source.key,
        accounts.mint.key,
        accounts.delegate.key,
        accounts.authority.key,
        &signer_pubkeys,
        amount,
        decimals,
    )?;

    let mut infos = Vec::with_capacity(4 + accounts.signers.len());
    infos.push(accounts.source);
    infos.push(accounts.mint);
    infos.push(accounts.delegate);
    infos.push(accounts.authority);
    infos.extend(accounts.signers.into_iter());

    solana_program::program::invoke_signed(&ix, &infos, signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
