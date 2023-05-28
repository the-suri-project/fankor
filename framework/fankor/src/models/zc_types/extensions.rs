use solana_program::account_info::AccountInfo;

use crate::errors::{FankorErrorCode, FankorResult};
use crate::prelude::FnkExtension;
use crate::traits::{CopyType, ZeroCopyType};

impl<'info> ZeroCopyType<'info> for FnkExtension {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        let bytes =
            info.try_borrow_data()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: "FnkExtension",
                })?;
        let bytes = &bytes[offset..];

        if bytes.is_empty() {
            return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                type_name: "FnkExtension",
            }
                .into());
        }

        if bytes[0] != 0 {
            return Err(FankorErrorCode::ZeroCopyCannotDeserialize {
                type_name: "FnkExtension",
            }
                .into());
        }

        Ok((FnkExtension, Some(1)))
    }

    fn read_byte_size(bytes: &[u8]) -> FankorResult<usize> {
        if bytes.is_empty() {
            return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                type_name: "FnkExtension",
            }
                .into());
        }

        Ok(1)
    }
}

impl<'info> CopyType<'info> for FnkExtension {
    type ZeroCopyType = FnkExtension;

    fn min_byte_size() -> usize {
        1
    }
}
