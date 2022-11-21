use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::programs::macros::{impl_account, impl_zero_copy_account};
use crate::models::ZeroCopyType;
use crate::prelude::ZC;
use crate::traits::{AccountDeserialize, AccountSerialize, Program};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_option::COption;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use std::any::type_name;
use std::io::{ErrorKind, Write};
use std::marker::PhantomData;
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
    [ZC: spl_token::state::Mint::LEN],
    [Default]
);

impl_account!(
    TokenAccount,
    spl_token::state::Account,
    &spl_token::ID,
    unpack,
    unpack,
    [ZC: spl_token::state::Account::LEN],
    [Default]
);

impl_account!(
    TokenMultisig,
    spl_token::state::Multisig,
    &spl_token::ID,
    unpack,
    unpack,
    [ZC: spl_token::state::Multisig::LEN],
    [Default]
);

// ----------------------------------------------------------------------------
// Zero Copy ------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl_zero_copy_account!(
    Mint,
    mint_authority: Option<Pubkey>,
    supply: u64,
    decimals: u8,
    is_initialized: bool,
    freeze_authority: Option<Pubkey>
);

impl_zero_copy_account!(
    TokenAccount,
    mint: Pubkey,
    owner: Pubkey,
    amount: u64,
    delegate: COption<Pubkey>,
    state: spl_token::state::AccountState,
    is_native: COption<u64>,
    delegated_amount: u64,
    close_authority: COption<Pubkey>
);

impl_zero_copy_account!(
    TokenMultisig,
    m: u8,
    n: u8,
    is_initialized: bool,
    signers: [Pubkey; spl_token::instruction::MAX_SIGNERS],
);

impl ZeroCopyType for spl_token::state::AccountState {
    fn byte_size_from_instance(&self) -> usize {
        1
    }

    fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
        if bytes.is_empty() {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: type_name::<Self>(),
            }
            .into());
        }

        Ok(1)
    }
}
