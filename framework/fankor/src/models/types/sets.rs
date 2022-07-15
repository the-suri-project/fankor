use crate::models::types::read_length;
use borsh::{BorshDeserialize, BorshSerialize};
use std::collections::BTreeSet;
use std::io::{ErrorKind, Write};
use std::ops::{Deref, DerefMut};

pub type FnkSetU8<T> = FnkSet<T, 1>;
pub type FnkSetU16<T> = FnkSet<T, 2>;
pub type FnkSetU24<T> = FnkSet<T, 3>;

/// Wrapper over `BTreeSet` that serializes the length into a different size.
#[derive(Debug)]
pub struct FnkSet<T, const C: usize>(pub BTreeSet<T>);

impl<T, const C: usize> FnkSet<T, C> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(inner: BTreeSet<T>) -> Self {
        Self(inner)
    }

    // METHODS ----------------------------------------------------------------

    pub fn into_inner(self) -> BTreeSet<T> {
        self.0
    }
}

impl<T, const C: usize> Default for FnkSet<T, C> {
    fn default() -> Self {
        Self(BTreeSet::new())
    }
}

impl<T, const C: usize> AsRef<BTreeSet<T>> for FnkSet<T, C> {
    fn as_ref(&self) -> &BTreeSet<T> {
        &self.0
    }
}

impl<T, const C: usize> Deref for FnkSet<T, C> {
    type Target = BTreeSet<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const C: usize> DerefMut for FnkSet<T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: BorshSerialize, const C: usize> BorshSerialize for FnkSet<T, C> {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let len_bytes = <[u8; C]>::try_from(self.len().to_le_bytes().as_slice())
            .map_err(|_| ErrorKind::InvalidInput)?;
        writer.write_all(len_bytes.as_slice())?;

        for item in &self.0 {
            item.serialize(writer)?;
        }

        Ok(())
    }
}

impl<T: BorshDeserialize + Ord, const C: usize> BorshDeserialize for FnkSet<T, C> {
    #[inline]
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let len = read_length(buf, C)?;
        let mut set = BTreeSet::new();

        for _ in 0..len {
            let item = T::deserialize(buf)?;
            set.insert(item);
        }

        Ok(FnkSet::new(set))
    }
}
