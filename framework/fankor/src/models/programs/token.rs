use crate::errors::FankorResult;
use crate::models::programs::macros::impl_account;
use crate::traits::{AccountDeserialize, AccountSerialize, Program};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use std::io::{ErrorKind, Write};
use std::ops::Deref;

#[derive(Debug, Copy, Clone)]
pub struct Token;

impl Program for Token {
    fn name() -> &'static str {
        "Token"
    }

    fn address() -> &'static Pubkey {
        &spl_token::ID
    }
}

// ----------------------------------------------------------------------------
// ACCOUNTS -------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl_account!(
    Mint,
    spl_token::state::Mint,
    &spl_token::ID,
    unpack,
    unpack,
    [Default]
);

impl_account!(
    TokenAccount,
    spl_token::state::Account,
    &spl_token::ID,
    unpack,
    unpack,
    [Default]
);

impl_account!(
    TokenMultisig,
    spl_token::state::Multisig,
    &spl_token::ID,
    unpack,
    unpack,
    [Default]
);
