use crate::errors::Error;
use crate::models::{Program, Token};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiTransferChecked<'info> {
    pub from: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
}

pub fn transfer_checked(
    program: &Program<Token>,
    accounts: CpiTransferChecked,
    amount: u64,
    decimals: u8,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let ix = spl_token::instruction::transfer_checked(
        program.address(),
        accounts.from.key,
        accounts.mint.key,
        accounts.to.key,
        accounts.authority.key,
        &[],
        amount,
        decimals,
    )?;

    solana_program::program::invoke_signed(
        &ix,
        &[
            accounts.from,
            accounts.mint,
            accounts.to,
            accounts.authority,
        ],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct CpiTransferCheckedMultisig<'info> {
    pub from: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub signers: Vec<AccountInfo<'info>>,
}

pub fn transfer_checked_multisig(
    program: &Program<Token>,
    accounts: CpiTransferCheckedMultisig,
    amount: u64,
    decimals: u8,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let signer_pubkeys = accounts.signers.iter().map(|v| v.key).collect::<Vec<_>>();
    let ix = spl_token::instruction::transfer_checked(
        program.address(),
        accounts.from.key,
        accounts.mint.key,
        accounts.to.key,
        accounts.authority.key,
        &signer_pubkeys,
        amount,
        decimals,
    )?;

    let mut infos = Vec::with_capacity(4 + accounts.signers.len());
    infos.push(accounts.from);
    infos.push(accounts.mint);
    infos.push(accounts.to);
    infos.push(accounts.authority);
    infos.extend(accounts.signers.into_iter());

    solana_program::program::invoke_signed(&ix, &infos, signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
