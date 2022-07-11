use crate::errors::FankorResult;
use solana_program::pubkey::Pubkey;
use std::io::Write;

pub trait Account:
    borsh::BorshSerialize + borsh::BorshDeserialize + AccountSerialize + AccountDeserialize
{
    /// The identifier of the account.
    fn discriminator() -> &'static [u8];

    /// Defines an address expected to own an account.
    fn owner() -> Pubkey;
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
        Self::try_deserialize_unchecked(buf)
    }

    /// Deserializes account data without checking the account discriminator.
    /// This should only be used on account initialization, when the bytes of
    /// the account are zeroed.
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> FankorResult<Self>;
}
