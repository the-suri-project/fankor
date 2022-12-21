use crate::errors::FankorResult;
use crate::models::programs::macros::impl_account;
use crate::traits::{AccountDeserialize, AccountSerialize, ProgramType};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use std::io::{ErrorKind, Write};
use std::ops::Deref;

#[derive(Debug, Copy, Clone)]
pub struct Token2022;

impl ProgramType for Token2022 {
    fn name() -> &'static str {
        "Token2022"
    }

    fn address() -> &'static Pubkey {
        &spl_token_2022::ID
    }
}

// ----------------------------------------------------------------------------
// ACCOUNTS -------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl_account!(
    Mint2022,
    spl_token_2022::state::Mint,
    &spl_token_2022::ID,
    unpack,
    unpack,
    [Default]
);

impl_account!(
    TokenAccount2022,
    spl_token_2022::state::Account,
    &spl_token_2022::ID,
    unpack,
    unpack,
    [Default]
);

impl_account!(
    TokenMultisig2022,
    spl_token_2022::state::Multisig,
    &spl_token_2022::ID,
    unpack,
    unpack,
    [Default]
);
