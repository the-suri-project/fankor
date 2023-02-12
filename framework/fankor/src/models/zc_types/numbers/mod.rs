pub use fnk::*;

mod fnk;

use crate::errors::{FankorErrorCode, FankorResult};
use crate::traits::{CopyType, ZeroCopyType};
use solana_program::account_info::AccountInfo;
use std::mem::size_of;

macro_rules! impl_type {
    ($ty: ty) => {
        impl<'info> ZeroCopyType<'info> for $ty {
            fn new(
                info: &'info AccountInfo<'info>,
                offset: usize,
            ) -> FankorResult<(Self, Option<usize>)> {
                let bytes = info.try_borrow_data().map_err(|_| {
                    FankorErrorCode::ZeroCopyPossibleDeadlock {
                        type_name: std::any::type_name::<$ty>(),
                    }
                })?;
                let bytes = &bytes[offset..];
                let size = size_of::<$ty>();

                if bytes.len() < size {
                    return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                        type_name: std::any::type_name::<$ty>(),
                    }
                    .into());
                }

                let value = <$ty>::from_le_bytes(bytes[..size].try_into().unwrap());

                Ok((value, Some(size)))
            }

            fn read_byte_size(bytes: &[u8]) -> FankorResult<usize> {
                let size = size_of::<$ty>();

                if bytes.len() < size {
                    return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                        type_name: std::any::type_name::<$ty>(),
                    }
                    .into());
                }

                Ok(size)
            }
        }

        impl<'info> CopyType<'info> for $ty {
            type ZeroCopyType = $ty;

            fn min_byte_size() -> usize {
                size_of::<$ty>()
            }
        }
    };
}

impl_type!(u8);
impl_type!(u16);
impl_type!(u32);
impl_type!(u64);
impl_type!(u128);
impl_type!(usize);
impl_type!(i8);
impl_type!(i16);
impl_type!(i32);
impl_type!(i64);
impl_type!(i128);
// impl_type!(isize);
impl_type!(f32);
impl_type!(f64);
