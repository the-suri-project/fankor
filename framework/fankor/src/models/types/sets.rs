use crate::prelude::FnkUInt;
use borsh::{BorshDeserialize, BorshSerialize};
use std::collections::BTreeSet;
use std::io::{ErrorKind, Write};
use std::ops::{Deref, DerefMut};

/// Wrapper over `BTreeSet` that serializes the length into a `FnkUInt`.
#[derive(Debug)]
pub struct FnkSet<T>(pub BTreeSet<T>);

impl<T> FnkSet<T> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(inner: BTreeSet<T>) -> Self {
        Self(inner)
    }

    // METHODS ----------------------------------------------------------------

    pub fn into_inner(self) -> BTreeSet<T> {
        self.0
    }
}

impl<T> Default for FnkSet<T> {
    fn default() -> Self {
        Self(BTreeSet::new())
    }
}

impl<T> AsRef<BTreeSet<T>> for FnkSet<T> {
    fn as_ref(&self) -> &BTreeSet<T> {
        &self.0
    }
}

impl<T> Deref for FnkSet<T> {
    type Target = BTreeSet<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for FnkSet<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<BTreeSet<T>> for FnkSet<T> {
    fn from(v: BTreeSet<T>) -> Self {
        Self(v)
    }
}

impl<T> From<FnkSet<T>> for BTreeSet<T> {
    fn from(v: FnkSet<T>) -> Self {
        v.0
    }
}

impl<T: BorshSerialize> BorshSerialize for FnkSet<T> {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let length = FnkUInt::from(self.0.len() as u64);

        length.serialize(writer)?;

        for item in &self.0 {
            item.serialize(writer)?;
        }

        Ok(())
    }
}

impl<T: BorshDeserialize + Ord> BorshDeserialize for FnkSet<T> {
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
        let mut set = BTreeSet::new();

        for _ in 0..len {
            let item = T::deserialize(buf)?;
            set.insert(item);
        }

        Ok(FnkSet::new(set))
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
        let data: BTreeSet<String> = BTreeSet::new();
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkSet::from(data.clone());
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize");

        assert_eq!(buffer[0], data.len() as u8);
        assert_eq!(buffer.len(), 1);

        let mut de_buf = buffer.as_slice();
        let deserialized =
            FnkSet::<String>::deserialize(&mut de_buf).expect("Failed to deserialize");

        assert_eq!(deserialized.0, data, "Incorrect result");
        assert!(de_buf.is_empty(), "Buffer not empty");
    }

    #[test]
    fn test_serialize_deserialize_data() {
        let mut data: BTreeSet<&str> = BTreeSet::new();
        data.insert("a");
        data.insert("b");

        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkSet::from(data.clone());
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
            data.iter().map(|v| 4 + v.len()).sum::<usize>() + 1
        );

        let mut de_buf = buffer.as_slice();
        let deserialized =
            FnkSet::<String>::deserialize(&mut de_buf).expect("Failed to deserialize");

        assert!(de_buf.is_empty(), "Buffer not empty");
        assert_eq!(deserialized.len(), data.len());

        for element in data {
            assert!(
                deserialized.0.contains(element),
                "Result not present: {:?}",
                element
            );
        }
    }
}
