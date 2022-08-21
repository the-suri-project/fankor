use crate::prelude::FnkUInt;
use borsh::{BorshDeserialize, BorshSerialize};
use std::io::{ErrorKind, Write};
use std::mem::{forget, size_of};
use std::ops::{Deref, DerefMut};

/// Wrapper over `Vec` that serializes the length into a `FnkUInt`.
#[derive(Debug, Clone)]
pub struct FnkVec<T>(pub Vec<T>);

impl<T> FnkVec<T> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(inner: Vec<T>) -> Self {
        Self(inner)
    }

    // METHODS ----------------------------------------------------------------

    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<T> Default for FnkVec<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> AsRef<Vec<T>> for FnkVec<T> {
    fn as_ref(&self) -> &Vec<T> {
        &self.0
    }
}

impl<T> Deref for FnkVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for FnkVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<Vec<T>> for FnkVec<T> {
    fn from(v: Vec<T>) -> Self {
        Self(v)
    }
}

impl<T> From<FnkVec<T>> for Vec<T> {
    fn from(v: FnkVec<T>) -> Self {
        v.0
    }
}

impl<T: BorshSerialize> BorshSerialize for FnkVec<T> {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let length = FnkUInt::from(self.0.len() as u64);

        length.serialize(writer)?;

        if let Some(u8_slice) = T::u8_slice(&self.0) {
            writer.write_all(u8_slice)?;
        } else {
            for item in &self.0 {
                item.serialize(writer)?;
            }
        }

        Ok(())
    }
}

impl<T: BorshDeserialize> BorshDeserialize for FnkVec<T> {
    #[inline]
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let len = FnkUInt::deserialize(buf)?;
        let len = match len.get_u32() {
            Some(v) => v,
            None => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "Unexpected length of input",
                ));
            }
        };

        if len == 0 {
            Ok(FnkVec::default())
        } else if let Some(vec_bytes) = T::vec_from_bytes(len, buf)? {
            Ok(FnkVec::new(vec_bytes))
        } else if size_of::<T>() == 0 {
            let mut result = vec![T::deserialize(buf)?];

            let p = result.as_mut_ptr();
            unsafe {
                forget(result);
                let len = len.try_into().map_err(|_| ErrorKind::InvalidInput)?;
                let result = Vec::from_raw_parts(p, len, len);
                Ok(FnkVec::new(result))
            }
        } else {
            let mut result = Vec::with_capacity({
                let el_size = size_of::<T>() as u32;
                core::cmp::max(core::cmp::min(len, 4096 / el_size), 1) as usize
            });
            for _ in 0..len {
                result.push(T::deserialize(buf)?);
            }
            Ok(FnkVec::new(result))
        }
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
        let data: Vec<String> = vec![];
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkVec::from(data.clone());
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize");

        assert_eq!(buffer[0], data.len() as u8);
        assert_eq!(buffer.len(), 1);

        let mut de_buf = buffer.as_slice();
        let deserialized =
            FnkVec::<String>::deserialize(&mut de_buf).expect("Failed to deserialize");

        assert_eq!(deserialized.0, data, "Incorrect result");
        assert!(de_buf.is_empty(), "Buffer not empty");
    }

    #[test]
    fn test_serialize_deserialize_bytes() {
        let data: Vec<u8> = vec![0, 1, 2, 3];
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkVec::from(data.clone());
        fnk_number
            .serialize(&mut cursor)
            .expect("Failed to serialize");

        assert_eq!(buffer[0], data.len() as u8);
        assert_eq!(buffer[1], data[0]);
        assert_eq!(buffer[2], data[1]);
        assert_eq!(buffer[3], data[2]);
        assert_eq!(buffer[4], data[3]);
        assert_eq!(buffer.len(), data.len() + 1);

        let mut de_buf = buffer.as_slice();
        let deserialized = FnkVec::<u8>::deserialize(&mut de_buf).expect("Failed to deserialize");

        assert_eq!(deserialized.0, data, "Incorrect result");
        assert!(de_buf.is_empty(), "Buffer not empty");
    }

    #[test]
    fn test_serialize_deserialize_data() {
        let data: Vec<&str> = vec!["a", "b"];
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let fnk_number = FnkVec::from(data.clone());
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
            FnkVec::<String>::deserialize(&mut de_buf).expect("Failed to deserialize");

        assert_eq!(deserialized.0, data, "Incorrect result");
        assert!(de_buf.is_empty(), "Buffer not empty");
    }
}
