use crate::errors::{ErrorCode, FankorResult};
use borsh::BorshDeserialize;
use solana_program::pubkey::Pubkey;
use std::marker::PhantomData;

/// Model to get the return value of a CPI instruction.
#[derive(Copy, Clone)]
pub struct CpiReturn<T> {
    phantom: PhantomData<T>,
}

impl<T: BorshDeserialize> CpiReturn<T> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new() -> CpiReturn<T> {
        Self {
            phantom: PhantomData,
        }
    }

    // METHODS ----------------------------------------------------------------

    pub fn get(&self, program_id: &Pubkey) -> FankorResult<T> {
        let (key, data) =
            solana_program::program::get_return_data().ok_or(ErrorCode::EmptyIntermediateBuffer)?;

        if key != *program_id {
            return Err(ErrorCode::IntermediateBufferIncorrectProgramId {
                actual: key,
                expected: *program_id,
            }
            .into());
        }

        Ok(T::try_from_slice(&data)?)
    }

    pub fn get_ignoring_program(&self) -> FankorResult<T> {
        let (_key, data) =
            solana_program::program::get_return_data().ok_or(ErrorCode::EmptyIntermediateBuffer)?;
        Ok(T::try_from_slice(&data)?)
    }
}

impl<T: BorshDeserialize> Default for CpiReturn<T> {
    fn default() -> Self {
        Self::new()
    }
}
