use crate::models::types::vectors::FnkVec;
use borsh::{BorshDeserialize, BorshSerialize};
use std::io::{ErrorKind, Write};
use std::ops::{Deref, DerefMut};

pub type FnkStringU8 = FnkString<1>;
pub type FnkStringU16 = FnkString<2>;
pub type FnkStringU24 = FnkString<3>;

/// Wrapper over `String` that serializes the length into a different size.
#[derive(Debug, Default)]
pub struct FnkString<const C: usize>(pub String);

impl<const C: usize> FnkString<C> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(inner: String) -> Self {
        Self(inner)
    }

    // METHODS ----------------------------------------------------------------

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<const C: usize> AsRef<String> for FnkString<C> {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl<const C: usize> Deref for FnkString<C> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const C: usize> DerefMut for FnkString<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const C: usize> BorshSerialize for FnkString<C> {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let bytes = self.0.as_bytes();
        let len_bytes = <[u8; C]>::try_from(bytes.len().to_le_bytes().as_slice())
            .map_err(|_| ErrorKind::InvalidInput)?;
        writer.write_all(len_bytes.as_slice())?;

        writer.write_all(bytes)?;

        Ok(())
    }
}

impl<const C: usize> BorshDeserialize for FnkString<C> {
    #[inline]
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        String::from_utf8(FnkVec::<u8, C>::deserialize(buf)?.0)
            .map(FnkString::new)
            .map_err(|err| {
                let msg = err.to_string();
                std::io::Error::new(ErrorKind::InvalidData, msg)
            })
    }
}
