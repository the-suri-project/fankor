use crate::models::types::read_length;
use borsh::{BorshDeserialize, BorshSerialize};
use std::collections::BTreeMap;
use std::io::{ErrorKind, Write};
use std::ops::{Deref, DerefMut};

pub type FnkMapU8<K, T> = FnkMap<K, T, 1>;
pub type FnkMapU16<K, T> = FnkMap<K, T, 2>;
pub type FnkMapU24<K, T> = FnkMap<K, T, 3>;

/// Wrapper over `BTreeMap` that serializes the length into a different size.
#[derive(Debug)]
pub struct FnkMap<K, T, const C: usize>(pub BTreeMap<K, T>);

impl<K, T, const C: usize> FnkMap<K, T, C> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(inner: BTreeMap<K, T>) -> Self {
        Self(inner)
    }

    // METHODS ----------------------------------------------------------------

    pub fn into_inner(self) -> BTreeMap<K, T> {
        self.0
    }
}

impl<K, T, const C: usize> Default for FnkMap<K, T, C> {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

impl<K, T, const C: usize> AsRef<BTreeMap<K, T>> for FnkMap<K, T, C> {
    fn as_ref(&self) -> &BTreeMap<K, T> {
        &self.0
    }
}

impl<K, T, const C: usize> Deref for FnkMap<K, T, C> {
    type Target = BTreeMap<K, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, T, const C: usize> DerefMut for FnkMap<K, T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K: BorshSerialize, T: BorshSerialize, const C: usize> BorshSerialize for FnkMap<K, T, C> {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let len_bytes = <[u8; C]>::try_from(self.len().to_le_bytes().as_slice())
            .map_err(|_| ErrorKind::InvalidInput)?;
        writer.write_all(len_bytes.as_slice())?;

        for (key, value) in &self.0 {
            key.serialize(writer)?;
            value.serialize(writer)?;
        }

        Ok(())
    }
}

impl<K: BorshDeserialize + Ord + core::hash::Hash, T: BorshDeserialize, const C: usize>
    BorshDeserialize for FnkMap<K, T, C>
{
    #[inline]
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let len = read_length(buf, C)?;
        let mut map = BTreeMap::new();

        for _ in 0..len {
            let key = K::deserialize(buf)?;
            let value = T::deserialize(buf)?;
            map.insert(key, value);
        }

        Ok(FnkMap::new(map))
    }
}
