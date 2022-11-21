use crate::errors::FankorResult;
use crate::models::ZeroCopyType;
use crate::traits::AccountSize;
use solana_program::pubkey::Pubkey;
use std::io::ErrorKind;
use std::mem::size_of;

impl ZeroCopyType for Pubkey {
    fn byte_size_from_instance(&self) -> usize {
        self.actual_account_size()
    }

    fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
        let size = size_of::<Pubkey>();

        if bytes.len() < size {
            return Err(
                std::io::Error::new(ErrorKind::InvalidInput, "Unexpected length of input").into(),
            );
        }

        Ok(size)
    }
}
