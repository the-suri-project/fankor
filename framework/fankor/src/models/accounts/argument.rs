use crate::errors::FankorResult;
use crate::models::FankorContext;
use crate::prelude::{AccountInfoVerification, LpiInstruction};
use crate::traits::{CpiInstruction, Instruction, PdaChecker};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::io::Write;
use std::ops::{Deref, DerefMut};

/// An instruction argument.
pub struct Argument<T>(T);

impl<T> Argument<T> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new argument with the given data.
    pub fn new(data: T) -> Argument<T> {
        Self(data)
    }

    // GETTERS ----------------------------------------------------------------

    #[inline(always)]
    pub fn data(&self) -> &T {
        &self.0
    }

    #[inline(always)]
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.0
    }

    // METHODS ----------------------------------------------------------------

    /// Returns the data.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> AsRef<T> for Argument<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Argument<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Argument<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'info, T: BorshSerialize + BorshDeserialize> Instruction<'info> for Argument<T> {
    type CPI = Argument<T>;
    type LPI = Argument<T>;

    fn verify_account_infos<'a>(
        &self,
        _config: &mut AccountInfoVerification<'a, 'info>,
    ) -> FankorResult<()> {
        Ok(())
    }

    #[inline(never)]
    fn try_from(
        _context: &'info FankorContext<'info>,
        buf: &mut &[u8],
        _accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        let result = BorshDeserialize::deserialize(buf)?;
        Ok(Argument::new(result))
    }
}

impl<'info, T: BorshSerialize> CpiInstruction<'info> for Argument<T> {
    fn serialize_into_instruction_parts<W: Write>(
        &self,
        writer: &mut W,
        _metas: &mut Vec<AccountMeta>,
        _infos: &mut Vec<AccountInfo<'info>>,
    ) -> FankorResult<()> {
        BorshSerialize::serialize(&self.0, writer)?;

        Ok(())
    }
}

impl<T: BorshSerialize> LpiInstruction for Argument<T> {
    fn serialize_into_instruction_parts<W: Write>(
        &self,
        writer: &mut W,
        _metas: &mut Vec<AccountMeta>,
    ) -> FankorResult<()> {
        BorshSerialize::serialize(&self.0, writer)?;

        Ok(())
    }
}

impl<'info, T> PdaChecker<'info> for Argument<T> {
    fn pda_info(&self) -> Option<&'info AccountInfo<'info>> {
        None
    }
}

impl<T: Debug> Debug for Argument<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Argument").field("data", &self.0).finish()
    }
}
