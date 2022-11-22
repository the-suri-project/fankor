use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::{CopyType, ZeroCopyType};
use crate::traits::AccountSize;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use std::mem::size_of;

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

    fn read_byte_size_from_bytes(bytes: &[u8]) -> FankorResult<usize> {
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

    fn byte_size_from_instance(&self) -> usize {
        self.actual_account_size()
    }
}
