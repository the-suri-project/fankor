use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::{CopyType, ZeroCopyType};
use crate::prelude::{FnkInt, FnkUInt};
use crate::traits::AccountSize;
use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;

impl<'info> ZeroCopyType<'info> for FnkInt {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        let bytes =
            info.try_borrow_data()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;
        let mut bytes = &bytes[offset..];
        let initial_size = bytes.len();
        let value = FnkInt::deserialize(&mut bytes)?;

        Ok((value, Some(initial_size - bytes.len())))
    }

    fn read_byte_size_from_bytes(bytes: &[u8]) -> FankorResult<usize> {
        if bytes.is_empty() {
            return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                type_name: "FnkInt",
            }
            .into());
        }

        let first_byte = bytes[0];
        let result = if first_byte & 0x80 == 0 {
            // Flag encoding.
            let mut size = 1;

            if first_byte & 0x40 != 0 {
                loop {
                    if bytes.is_empty() {
                        return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                            type_name: "FnkInt",
                        }
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

impl<'info> CopyType<'info> for FnkInt {
    type ZeroCopyType = FnkInt;

    fn byte_size_from_instance(&self) -> usize {
        self.actual_account_size()
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl<'info> ZeroCopyType<'info> for FnkUInt {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        let bytes =
            info.try_borrow_data()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;
        let mut bytes = &bytes[offset..];
        let initial_size = bytes.len();
        let value = FnkUInt::deserialize(&mut bytes)?;

        Ok((value, Some(initial_size - bytes.len())))
    }

    fn read_byte_size_from_bytes(bytes: &[u8]) -> FankorResult<usize> {
        if bytes.is_empty() {
            return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                type_name: "FnkUInt",
            }
            .into());
        }

        let first_byte = bytes[0];
        let result = if first_byte & 0x80 == 0 {
            // Flag encoding.
            let mut size = 1;

            if first_byte & 0x40 != 0 {
                loop {
                    if bytes.is_empty() {
                        return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                            type_name: "FnkUInt",
                        }
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

impl<'info> CopyType<'info> for FnkUInt {
    type ZeroCopyType = FnkUInt;

    fn byte_size_from_instance(&self) -> usize {
        self.actual_account_size()
    }
}
