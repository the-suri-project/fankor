use crate::errors::FankorResult;
use crate::models::ZeroCopyType;
use crate::prelude::{FnkString, FnkUInt};
use crate::traits::AccountSize;
use borsh::BorshDeserialize;

impl<'a> ZeroCopyType for FnkString<'a> {
    fn byte_size_from_instance(&self) -> usize {
        self.actual_account_size()
    }

    fn byte_size(mut bytes: &[u8]) -> FankorResult<usize> {
        let bytes = &mut bytes;
        let initial_len = bytes.len();
        let length = FnkUInt::deserialize(bytes)?;

        Ok(length.0 as usize + initial_len - bytes.len())
    }
}
