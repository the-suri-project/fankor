use crate::prelude::FnkUInt;
use crate::traits::AccountSize;
use borsh::{BorshDeserialize, BorshSerialize};
use std::collections::BTreeMap;
use std::io::{ErrorKind, Write};
use std::ops::{Deref, DerefMut};

/// Wrapper over `BTreeMap` that serializes the length into a `FnkUInt`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnkMap<K, T>(pub BTreeMap<K, T>);

impl<K, T> FnkMap<K, T> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(inner: BTreeMap<K, T>) -> Self {
        Self(inner)
    }

    // METHODS ----------------------------------------------------------------

    pub fn into_inner(self) -> BTreeMap<K, T> {
        self.0
    }
}

impl<K, T> Default for FnkMap<K, T> {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

impl<K, T> AsRef<BTreeMap<K, T>> for FnkMap<K, T> {
    fn as_ref(&self) -> &BTreeMap<K, T> {
        &self.0
    }
}

impl<K, T> Deref for FnkMap<K, T> {
    type Target = BTreeMap<K, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, T> DerefMut for FnkMap<K, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K, T> From<BTreeMap<K, T>> for FnkMap<K, T> {
    fn from(v: BTreeMap<K, T>) -> Self {
        Self(v)
    }
}

impl<K, T> From<FnkMap<K, T>> for BTreeMap<K, T> {
    fn from(v: FnkMap<K, T>) -> Self {
        v.0
    }
}

impl<K: BorshSerialize, T: BorshSerialize> BorshSerialize for FnkMap<K, T> {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let length = FnkUInt::from(self.0.len() as u64);

        length.serialize(writer)?;

        for (key, value) in &self.0 {
            key.serialize(writer)?;
            value.serialize(writer)?;
        }

        Ok(())
    }
}

impl<K: BorshDeserialize + Ord + core::hash::Hash, T: BorshDeserialize> BorshDeserialize
    for FnkMap<K, T>
{
    #[inline]
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let len = FnkUInt::deserialize(buf)?;
        let len = match len.get_usize() {
            Some(v) => v,
            None => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "Unexpected length of input",
                ));
            }
        };
        let mut map = BTreeMap::new();

        for _ in 0..len {
            let key = K::deserialize(buf)?;
            let value = T::deserialize(buf)?;
            map.insert(key, value);
        }

        Ok(FnkMap::new(map))
    }
}

impl<K: AccountSize, T: AccountSize> AccountSize for FnkMap<K, T> {
    fn min_account_size() -> usize {
        FnkUInt::min_account_size()
    }

    fn actual_account_size(&self) -> usize {
        let length = FnkUInt::from(self.0.len() as u64);
        let mut size = length.actual_account_size();

        for (k, v) in &self.0 {
            size += k.actual_account_size();
            size += v.actual_account_size();
        }

        size
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_serialize_deserialize_empty() {
        let data: BTreeMap<String, u8> = BTreeMap::new();
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkMap::from(data.clone());
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize");

        assert_eq!(buffer[0], data.len() as u8);
        assert_eq!(buffer.len(), 1);

        let mut de_buf = buffer.as_slice();
        let deserialized =
            FnkMap::<String, String>::deserialize(&mut de_buf).expect("Failed to deserialize");

        assert!(deserialized.0.is_empty(), "Result is not empty");
        assert!(de_buf.is_empty(), "Buffer not empty");
    }

    #[test]
    fn test_serialize_deserialize_data() {
        let mut data: BTreeMap<&str, &str> = BTreeMap::new();
        data.insert("a", "b");

        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkMap::from(data.clone());
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize");

        assert_eq!(buffer[0], data.len() as u8);
        assert_eq!(buffer[1], 1);
        assert_eq!(buffer[2], 0);
        assert_eq!(buffer[3], 0);
        assert_eq!(buffer[4], 0);
        assert_eq!(buffer[5], b'a');
        assert_eq!(buffer[6], 1);
        assert_eq!(buffer[7], 0);
        assert_eq!(buffer[8], 0);
        assert_eq!(buffer[9], 0);
        assert_eq!(buffer[10], b'b');
        assert_eq!(
            buffer.len(),
            data.iter()
                .map(|(k, v)| 8 + k.len() + v.len())
                .sum::<usize>()
                + 1
        );

        let mut de_buf = buffer.as_slice();
        let deserialized =
            FnkMap::<String, String>::deserialize(&mut de_buf).expect("Failed to deserialize");

        assert!(de_buf.is_empty(), "Buffer not empty");
        assert_eq!(deserialized.len(), data.len());

        for (key, value) in data {
            assert!(
                deserialized.0.contains_key(key),
                "Result not present: {:?}",
                key
            );

            assert_eq!(
                deserialized.0.get(key).map(|v| v.as_str()),
                Some(value),
                "Incorrect result: {:?}",
                key
            );
        }
    }
}
