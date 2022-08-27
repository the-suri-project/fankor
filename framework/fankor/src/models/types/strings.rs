use crate::models::types::unsigned::FnkUInt;
use crate::traits::AccountSize;
use borsh::{BorshDeserialize, BorshSerialize};
use std::borrow::Cow;
use std::io::{ErrorKind, Write};
use std::ops::{Deref, DerefMut};

/// Wrapper over `String` that serializes the length into a `FnkUInt`.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FnkString<'a>(pub Cow<'a, str>);

impl<'a> FnkString<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(inner: Cow<'a, str>) -> Self {
        Self(inner)
    }

    // METHODS ----------------------------------------------------------------

    pub fn into_inner(self) -> Cow<'a, str> {
        self.0
    }
}

impl<'a> AsRef<Cow<'a, str>> for FnkString<'a> {
    fn as_ref(&self) -> &Cow<'a, str> {
        &self.0
    }
}

impl<'a> Deref for FnkString<'a> {
    type Target = Cow<'a, str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for FnkString<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> From<&'a str> for FnkString<'a> {
    fn from(v: &'a str) -> Self {
        Self(Cow::Borrowed(v))
    }
}

impl<'a> From<String> for FnkString<'a> {
    fn from(v: String) -> Self {
        Self(Cow::Owned(v))
    }
}

impl<'a> From<Cow<'a, str>> for FnkString<'a> {
    fn from(v: Cow<'a, str>) -> Self {
        Self(v)
    }
}

impl<'a> From<FnkString<'a>> for String {
    fn from(v: FnkString) -> Self {
        v.0.to_string()
    }
}

impl<'a> From<FnkString<'a>> for Cow<'a, str> {
    fn from(v: FnkString<'a>) -> Self {
        v.0
    }
}

impl<'a> BorshSerialize for FnkString<'a> {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let length = FnkUInt::from(self.0.len() as u64);

        length.serialize(writer)?;
        writer.write_all(self.0.as_bytes())?;

        Ok(())
    }
}

impl<'a> BorshDeserialize for FnkString<'a> {
    #[inline]
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let length = FnkUInt::deserialize(buf)?;
        let length = match length.get_usize() {
            Some(v) => v,
            None => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "Unexpected length of input",
                ));
            }
        };

        if buf.len() < length {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Unexpected length of input",
            ));
        }

        let result = String::from_utf8(buf[..length].to_vec())
            .map(FnkString::from)
            .map_err(|err| {
                let msg = err.to_string();
                std::io::Error::new(ErrorKind::InvalidData, msg)
            })?;

        *buf = &buf[length..];
        Ok(result)
    }
}

impl<'a> AccountSize for FnkString<'a> {
    fn min_account_size() -> usize {
        FnkUInt::min_account_size()
    }

    fn actual_account_size(&self) -> usize {
        let length = FnkUInt::from(self.0.len() as u64);
        length.actual_account_size() + self.0.len()
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
    fn test_serialize_deserialize() {
        for text in ["", "Hello world!"] {
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let fnk_number = FnkString::from(text);
            fnk_number
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {:?}", text));

            assert_eq!(buffer[0], text.len() as u8);
            assert_eq!(&buffer[1..], text.as_bytes());
            assert_eq!(buffer.len(), text.len() + 1);

            let mut de_buf = buffer.as_slice();
            let deserialized = FnkString::deserialize(&mut de_buf)
                .unwrap_or_else(|_| panic!("Failed to deserialize for {:?}", text));

            assert_eq!(&deserialized.0, text, "Incorrect result for {:?}", text);
            assert!(de_buf.is_empty(), "Buffer not empty for {:?}", text);
        }
    }
}
