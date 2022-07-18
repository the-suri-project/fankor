use crate::errors::FankorResult;
use crate::traits::{AccountDeserialize, AccountSerialize, Program};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use std::io::{ErrorKind, Write};
use std::ops::Deref;

#[derive(Debug, Copy, Clone)]
pub struct Token;

impl Program for Token {
    fn address() -> &'static Pubkey {
        &spl_token::ID
    }
}

// ----------------------------------------------------------------------------
// ACCOUNTS -------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Mint(spl_token::state::Mint);

impl crate::traits::Account for Mint {
    fn discriminator() -> &'static [u8] {
        &[]
    }

    fn owner() -> &'static Pubkey {
        &spl_token::ID
    }
}

impl AccountSerialize for Mint {
    fn try_serialize<W: Write>(&self, _writer: &mut W) -> FankorResult<()> {
        unreachable!("Cannot write accounts that does not belong to the current program")
    }
}

impl BorshSerialize for Mint {
    fn serialize<W: Write>(&self, _writer: &mut W) -> std::io::Result<()> {
        unreachable!("Cannot write accounts that does not belong to the current program")
    }
}

impl AccountDeserialize for Mint {
    unsafe fn try_deserialize_unchecked(buf: &mut &[u8]) -> FankorResult<Self> {
        spl_token::state::Mint::unpack(buf)
            .map(Mint)
            .map_err(|e| crate::errors::Error::ProgramError(e))
    }
}

impl BorshDeserialize for Mint {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        spl_token::state::Mint::unpack(buf)
            .map(Mint)
            .map_err(|e| std::io::Error::new(ErrorKind::Other, e))
    }
}

impl Deref for Mint {
    type Target = spl_token::state::Mint;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TokenAccount(spl_token::state::Account);

impl crate::traits::Account for TokenAccount {
    fn discriminator() -> &'static [u8] {
        &[]
    }

    fn owner() -> &'static Pubkey {
        &spl_token::ID
    }
}

impl AccountSerialize for TokenAccount {
    fn try_serialize<W: Write>(&self, _writer: &mut W) -> FankorResult<()> {
        unreachable!("Cannot write accounts that does not belong to the current program")
    }
}

impl BorshSerialize for TokenAccount {
    fn serialize<W: Write>(&self, _writer: &mut W) -> std::io::Result<()> {
        unreachable!("Cannot write accounts that does not belong to the current program")
    }
}

impl AccountDeserialize for TokenAccount {
    unsafe fn try_deserialize_unchecked(buf: &mut &[u8]) -> FankorResult<Self> {
        spl_token::state::Account::unpack(buf)
            .map(TokenAccount)
            .map_err(|e| crate::errors::Error::ProgramError(e))
    }
}

impl BorshDeserialize for TokenAccount {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        spl_token::state::Account::unpack(buf)
            .map(TokenAccount)
            .map_err(|e| std::io::Error::new(ErrorKind::Other, e))
    }
}

impl Deref for TokenAccount {
    type Target = spl_token::state::Account;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
