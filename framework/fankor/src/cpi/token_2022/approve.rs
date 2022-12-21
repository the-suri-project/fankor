use crate::errors::Error;
use crate::models::{Program, Token2022};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiApprove<'info> {
    pub source: AccountInfo<'info>,
    pub delegate: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
}

pub fn approve(
    program: &Program<Token2022>,
    accounts: CpiApprove,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = spl_token_2022::instruction::approve(
        program.address(),
        accounts.source.key,
        accounts.delegate.key,
        accounts.authority.key,
        &[],
        amount,
    )?;

    solana_program::program::invoke_signed(
        &ix,
        &[accounts.source, accounts.delegate, accounts.authority],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct CpiApproveMultisig<'info> {
    pub source: AccountInfo<'info>,
    pub delegate: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub signers: Vec<AccountInfo<'info>>,
}

pub fn approve_multisig(
    program: &Program<Token2022>,
    accounts: CpiApproveMultisig,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let signer_pubkeys = accounts.signers.iter().map(|v| v.key).collect::<Vec<_>>();
    let ix = spl_token_2022::instruction::approve(
        program.address(),
        accounts.source.key,
        accounts.delegate.key,
        accounts.authority.key,
        &signer_pubkeys,
        amount,
    )?;

    let mut infos = Vec::with_capacity(3 + accounts.signers.len());
    infos.push(accounts.source);
    infos.push(accounts.delegate);
    infos.push(accounts.authority);
    infos.extend(accounts.signers.into_iter());

    solana_program::program::invoke_signed(&ix, &infos, signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
