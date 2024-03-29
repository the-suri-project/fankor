use std::any::type_name;

use solana_program::account_info::AccountInfo;

use crate::errors::{FankorErrorCode, FankorResult};
use crate::traits::{CopyType, ZeroCopyType};

impl<'info> ZeroCopyType<'info> for () {
    fn new(
        _info: &'info AccountInfo<'info>,
        _offset: usize,
    ) -> FankorResult<(Self, Option<usize>)> {
        Ok(((), Some(0)))
    }

    fn read_byte_size(_bytes: &[u8]) -> FankorResult<usize> {
        Ok(0)
    }
}

impl<'info> CopyType<'info> for () {
    type ZeroCopyType = ();

    fn min_byte_size() -> usize {
        0
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

macro_rules! impl_tuple {
    ($($types: ident),* $(,)?) => {
        #[allow(non_snake_case)]
        impl<'info, $($types: ZeroCopyType<'info>),*> ZeroCopyType<'info> for ($($types),*) {
            fn new(
                info: &'info AccountInfo<'info>,
                mut offset: usize,
            ) -> FankorResult<(Self, Option<usize>)> {
                let original_offset = offset;

                $(
                    let ($types, size) = $types::new(info, offset)?;

                    if let Some(size) = size {
                        offset += size
                    } else {
                        let bytes =
                            info.try_borrow_data()
                                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                                    type_name: type_name::<Self>(),
                                })?;
                        let bytes = &bytes[offset..];
                        offset += $types::read_byte_size(bytes)?
                    }
                )*

                Ok((($($types),*), Some(offset - original_offset)))
            }

            fn read_byte_size(bytes: &[u8]) -> FankorResult<usize> {
                let mut size = 0;

                $(size += $types::read_byte_size(&bytes[size..])?;)*

                Ok(size)
            }
        }

        #[allow(non_snake_case)]
        impl<'info, $($types: CopyType<'info>),*> CopyType<'info> for ($($types),*) {
            type ZeroCopyType = ($($types::ZeroCopyType),*);

            fn byte_size(&self) -> usize {
                let mut size = 0;

                let ($($types),*) = self;

                $(size += $types.byte_size();)*

                size
            }

            fn min_byte_size() -> usize {
                let mut size = 0;

                $(size += <$types>::min_byte_size();)*

                size
            }
        }
    };
}

impl_tuple!(T0, T1);
impl_tuple!(T0, T1, T2);
impl_tuple!(T0, T1, T2, T3);
impl_tuple!(T0, T1, T2, T3, T4);
impl_tuple!(T0, T1, T2, T3, T4, T5);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
