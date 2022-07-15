use crate::errors::FankorResult;
use solana_program::pubkey::Pubkey;
use std::io::Write;

/// This discriminator is just a zero array to avoid allocating a vector each time
/// it is writen.
pub const CLOSED_ACCOUNT_DISCRIMINATOR: [u8; 32] = [0; 32];

pub trait Account:
    borsh::BorshSerialize + borsh::BorshDeserialize + AccountSerialize + AccountDeserialize
{
    /// The identifier of the account.
    fn discriminator() -> &'static [u8];

    /// Defines an address expected to own an account.
    fn owner() -> &'static Pubkey;
}

pub trait ProgramAccount: Account {
    /// The address that identify the program.
    fn address() -> &'static Pubkey;
}

pub trait AccountSerialize {
    /// Serializes the account data into `writer`.
    fn try_serialize<W: Write>(&self, _writer: &mut W) -> FankorResult<()> {
        Ok(())
    }
}

pub trait AccountDeserialize: Sized {
    /// Deserializes previously initialized account data. Should fail for all
    /// uninitialized accounts, where the bytes are zeroed.
    fn try_deserialize(buf: &mut &[u8]) -> FankorResult<Self> {
        unsafe { Self::try_deserialize_unchecked(buf) }
    }

    /// Deserializes account data without checking the account discriminator.
    /// This should only be used on account initialization, when the bytes of
    /// the account are zeroed.
    ///
    /// # Safety
    /// This is unsafe because it does not check the account discriminator. It is
    /// the caller's responsibility to ensure that the account is of the correct
    /// type.
    unsafe fn try_deserialize_unchecked(buf: &mut &[u8]) -> FankorResult<Self>;
}
