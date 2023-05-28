use std::io::{ErrorKind, Write};

use borsh::{BorshDeserialize, BorshSerialize};

/// An extension placeholder with no meaning. It acts as unit type in Rust
/// but occupies one byte valuing 0 in (de)serialization. Moreover, in
/// TsGen it gets skipped in struct fields and it is auto populated.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct FnkExtension;

impl BorshSerialize for FnkExtension {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[0])?;

        Ok(())
    }
}

impl BorshDeserialize for FnkExtension {
    #[inline]
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        if buf.is_empty() {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Unexpected length of input",
            ));
        }

        let byte = buf[0];
        if byte != 0 {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Incorrect FnkExtension byte",
            ));
        }

        *buf = &buf[1..];

        Ok(FnkExtension)
    }
}
