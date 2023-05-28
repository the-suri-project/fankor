use std::mem::size_of;

use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

use crate::errors::{FankorErrorCode, FankorResult};
use crate::traits::{CopyType, ZeroCopyType};

impl<'info> ZeroCopyType<'info> for Pubkey {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        let bytes =
            info.try_borrow_data()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: "Pubkey",
                })?;
        let bytes = &bytes[offset..];
        let size = size_of::<Pubkey>();

        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                type_name: "Pubkey",
            }
                .into());
        }

        let bytes: [u8; 32] = bytes[..size].try_into().unwrap();
        Ok((Pubkey::from(bytes), Some(size)))
    }

    fn read_byte_size(bytes: &[u8]) -> FankorResult<usize> {
        let size = size_of::<Pubkey>();

        if bytes.len() < size {
            return Err(FankorErrorCode::ZeroCopyNotEnoughLength {
                type_name: "Pubkey",
            }
                .into());
        }

        Ok(size)
    }
}

impl<'info> CopyType<'info> for Pubkey {
    type ZeroCopyType = Pubkey;

    fn min_byte_size() -> usize {
        size_of::<[u8; 32]>()
    }
}
