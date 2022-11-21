use crate::errors::FankorResult;
use crate::models::ZeroCopyType;
use std::io::ErrorKind;

impl ZeroCopyType for bool {
    fn byte_size_from_instance(&self) -> usize {
        1
    }

    fn byte_size(bytes: &[u8]) -> FankorResult<usize> {
        if bytes.is_empty() {
            return Err(
                std::io::Error::new(ErrorKind::InvalidInput, "Unexpected length of input").into(),
            );
        }

        Ok(1)
    }
}
