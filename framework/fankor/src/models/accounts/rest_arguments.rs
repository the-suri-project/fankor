use std::fmt;
use std::fmt::{Debug, Formatter};
use std::io::Write;
use std::ops::{Deref, DerefMut};

use borsh::BorshSerialize;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;

use crate::errors::FankorResult;
use crate::models::FankorContext;
use crate::prelude::{AccountInfoVerification, LpiInstruction};
use crate::traits::{CpiInstruction, Instruction, PdaChecker};

/// An instruction argument that contains all the remaining data.
pub struct RestArguments(Vec<u8>);

impl RestArguments {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new argument with the given data.
    pub fn new(data: Vec<u8>) -> RestArguments {
        Self(data)
    }

    // GETTERS ----------------------------------------------------------------

    pub fn data(&self) -> &[u8] {
        &self.0
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    // METHODS ----------------------------------------------------------------

    /// Returns the data.
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<Vec<u8>> for RestArguments {
    fn as_ref(&self) -> &Vec<u8> {
        &self.0
    }
}

impl Deref for RestArguments {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RestArguments {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'info> Instruction<'info> for RestArguments {
    type CPI = RestArguments;
    type LPI = RestArguments;

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
        let result = buf.to_vec();

        *buf = &[];

        Ok(RestArguments::new(result))
    }
}

impl<'info> CpiInstruction<'info> for RestArguments {
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

impl LpiInstruction for RestArguments {
    fn serialize_into_instruction_parts<W: Write>(
        &self,
        writer: &mut W,
        _metas: &mut Vec<AccountMeta>,
    ) -> FankorResult<()> {
        BorshSerialize::serialize(&self.0, writer)?;

        Ok(())
    }
}

impl<'info> PdaChecker<'info> for RestArguments {
    fn pda_info(&self) -> Option<&'info AccountInfo<'info>> {
        None
    }
}

impl Debug for RestArguments {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RestArguments")
            .field("data", &self.0)
            .finish()
    }
}
