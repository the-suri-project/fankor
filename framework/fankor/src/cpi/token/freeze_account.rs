use crate::errors::Error;
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiFreezeAccount<'info> {
    pub account: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
}

pub fn freeze_account(accounts: CpiFreezeAccount, signer_seeds: &[&[&[u8]]]) -> FankorResult<()> {
    let ix = spl_token::instruction::freeze_account(
        &spl_token::ID,
        accounts.account.key,
        accounts.mint.key,
        accounts.authority.key,
        &[],
    )?;

    solana_program::program::invoke_signed(
        &ix,
        &[accounts.account, accounts.mint, accounts.authority],
        signer_seeds,
    )
    .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct CpiFreezeAccountMultisig<'info> {
    pub account: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub signers: Vec<AccountInfo<'info>>,
}

pub fn freeze_account_multisig(
    accounts: CpiFreezeAccountMultisig,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let signer_pubkeys = accounts.signers.iter().map(|v| v.key).collect::<Vec<_>>();
    let ix = spl_token::instruction::freeze_account(
        &spl_token::ID,
        accounts.account.key,
        accounts.mint.key,
        accounts.authority.key,
        &signer_pubkeys,
    )?;

    let mut infos = Vec::with_capacity(3 + accounts.signers.len());
    infos.push(accounts.account);
    infos.push(accounts.mint);
    infos.push(accounts.authority);
    infos.extend(accounts.signers.into_iter());

    solana_program::program::invoke_signed(&ix, &infos, signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
