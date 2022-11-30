use crate::errors::FankorResult;
use solana_program::pubkey::Pubkey;
use std::io::Write;

pub trait AccountType:
    borsh::BorshSerialize + borsh::BorshDeserialize + AccountSerialize + AccountDeserialize
{
    /// The discriminant of the account.
    fn discriminant() -> u8;

    /// Defines an address expected to own an account.
    fn owner() -> &'static Pubkey;
}

pub trait AccountSerialize {
    /// Serializes the account data into `writer`.
    fn try_serialize<W: Write>(&self, _writer: &mut W) -> FankorResult<()>;
}

pub trait AccountDeserialize: Sized {
    /// Deserializes previously initialized account data. Should fail for all
    /// uninitialized accounts, where the bytes are zeroed.
    fn try_deserialize(buf: &mut &[u8]) -> FankorResult<Self> {
        unsafe { Self::try_deserialize_unchecked(buf) }
    }

    /// Deserializes account data without checking the account discriminant.
    /// This should only be used on account initialization, when the bytes of
    /// the account are zeroed.
    ///
    /// # Safety
    /// This is unsafe because it does not check the account discriminant. It is
    /// the caller's responsibility to ensure that the account is of the correct
    /// type.
    unsafe fn try_deserialize_unchecked(buf: &mut &[u8]) -> FankorResult<Self>;
}
