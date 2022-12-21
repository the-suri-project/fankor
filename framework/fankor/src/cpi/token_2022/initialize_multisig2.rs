use crate::errors::Error;
use crate::models::{Program, Token2022};
use crate::prelude::FankorResult;
use solana_program::account_info::AccountInfo;

pub struct CpiInitializeMultisig2<'info> {
    pub multisignature: AccountInfo<'info>,
    pub signers: Vec<AccountInfo<'info>>,
}

pub fn initialize_multisig2(
    program: &Program<Token2022>,
    accounts: CpiInitializeMultisig2,
    m: u8,
    signer_seeds: &[&[&[u8]]],
) -> FankorResult<()> {
    let signer_pubkeys = accounts.signers.iter().map(|v| v.key).collect::<Vec<_>>();
    let ix = spl_token_2022::instruction::initialize_multisig2(
        program.address(),
        accounts.multisignature.key,
        &signer_pubkeys,
        m,
    )?;

    let mut infos = Vec::with_capacity(1 + accounts.signers.len());
    infos.push(accounts.multisignature);
    infos.extend(accounts.signers.into_iter());

    solana_program::program::invoke_signed(&ix, &infos, signer_seeds)
        .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
}
