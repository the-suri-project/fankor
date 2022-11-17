use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::ZeroCopyType;
use crate::prelude::{FnkInt, FnkUInt};
use crate::traits::AccountSize;
use std::io::ErrorKind;
use std::mem::size_of;

macro_rules! impl_type {
    ($ty: ty) => {
        impl ZeroCopyType for $ty {
            fn byte_size_from_instance(&self) -> usize {
                size_of::<$ty>()
            }

            fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
                let size = size_of::<$ty>();
                if bytes.len() < size {
                    return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                        type_name: stringify!($ty),
                    }
                    .into());
                }

                Ok(size)
            }
        }
    };
}

impl_type!(u8);
impl_type!(u16);
impl_type!(u32);
impl_type!(u64);
impl_type!(u128);
impl_type!(i8);
impl_type!(i16);
impl_type!(i32);
impl_type!(i64);
impl_type!(i128);

impl ZeroCopyType for FnkInt {
    fn byte_size_from_instance(&self) -> usize {
        self.actual_account_size()
    }

    fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
        if bytes.is_empty() {
            return Err(
                std::io::Error::new(ErrorKind::InvalidInput, "Unexpected length of input").into(),
            );
        }

        let first_byte = bytes[0];
        let result = if first_byte & 0x80 == 0 {
            // Flag encoding.
            let mut size = 1;

            if first_byte & 0x40 != 0 {
                loop {
                    if bytes.is_empty() {
                        return Err(std::io::Error::new(
                            ErrorKind::InvalidInput,
                            "Unexpected length of input",
                        )
                        .into());
                    }

                    let byte = bytes[size];
                    size += 1;

                    if (byte & 0x80) == 0 {
                        break;
                    }
                }
            }

            size
        } else {
            // Length encoding.
            let byte_length = first_byte & 0x3F;

            byte_length as usize
        };

        Ok(result)
    }
}

impl ZeroCopyType for FnkUInt {
    fn byte_size_from_instance(&self) -> usize {
        self.actual_account_size()
    }

    fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
        if bytes.is_empty() {
            return Err(
                std::io::Error::new(ErrorKind::InvalidInput, "Unexpected length of input").into(),
            );
        }

        let first_byte = bytes[0];
        let result = if first_byte & 0x80 == 0 {
            // Flag encoding.
            let mut size = 1;

            if first_byte & 0x40 != 0 {
                loop {
                    if bytes.is_empty() {
                        return Err(std::io::Error::new(
                            ErrorKind::InvalidInput,
                            "Unexpected length of input",
                        )
                        .into());
                    }

                    let byte = bytes[size];
                    size += 1;

                    if (byte & 0x80) == 0 {
                        break;
                    }
                }
            }

            size
        } else {
            // Length encoding.
            let byte_length = first_byte & 0x7F;

            byte_length as usize
        };

        Ok(result)
    }
}
