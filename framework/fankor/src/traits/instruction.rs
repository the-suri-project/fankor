use crate::errors::FankorResult;
use crate::models::FankorContext;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use std::io::Write;

/// Trait for instruction definitions.
pub trait Instruction<'info>: Sized {
    type CPI: CpiInstruction<'info>;
    type LPI: LpiInstruction;

    /// Verifies the account info with specific data.
    #[allow(unused_variables)]
    fn verify_account_infos<'a>(
        &self,
        config: &mut AccountInfoVerification<'a, 'info>,
    ) -> FankorResult<()> {
        unreachable!("Custom enums and structs cannot have check attributes");
    }

    fn try_from(
        context: &'info FankorContext<'info>,
        data: &mut &[u8],
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self>;
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[derive(Default)]
#[allow(clippy::type_complexity)]
pub struct AccountInfoVerification<'a, 'info> {
    pub account_info: Option<&'a mut dyn Fn(&AccountInfo<'info>) -> FankorResult<()>>,
    pub constraints: Option<&'a mut dyn Fn(&AccountInfo<'info>) -> FankorResult<()>>,
}

impl<'a, 'info> AccountInfoVerification<'a, 'info> {
    // METHODS ----------------------------------------------------------------

    pub fn verify(&mut self, account: &AccountInfo<'info>) -> FankorResult<()> {
        if let Some(account_info) = &mut self.account_info {
            (account_info)(account)?;
        }

        if let Some(constraints) = &mut self.constraints {
            (constraints)(account)?;
        }

        Ok(())
    }

    pub fn verify_only_constraints(&mut self, account: &AccountInfo<'info>) -> FankorResult<()> {
        if let Some(constraints) = &mut self.constraints {
            (constraints)(account)?;
        }

        Ok(())
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub trait CpiInstruction<'info> {
    fn serialize_into_instruction_parts<W: Write>(
        &self,
        writer: &mut W,
        metas: &mut Vec<AccountMeta>,
        infos: &mut Vec<AccountInfo<'info>>,
    ) -> FankorResult<()>;
}

impl<'info> CpiInstruction<'info> for AccountInfo<'info> {
    fn serialize_into_instruction_parts<W: Write>(
        &self,
        _writer: &mut W,
        metas: &mut Vec<AccountMeta>,
        infos: &mut Vec<AccountInfo<'info>>,
    ) -> FankorResult<()> {
        metas.push(AccountMeta {
            pubkey: *self.key,
            is_writable: false,
            is_signer: false,
        });
        infos.push(self.clone());
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub trait LpiInstruction {
    fn serialize_into_instruction_parts<W: Write>(
        &self,
        writer: &mut W,
        metas: &mut Vec<AccountMeta>,
    ) -> FankorResult<()>;
}

impl LpiInstruction for Pubkey {
    fn serialize_into_instruction_parts<W: Write>(
        &self,
        _writer: &mut W,
        metas: &mut Vec<AccountMeta>,
    ) -> FankorResult<()> {
        metas.push(AccountMeta {
            pubkey: *self,
            is_writable: false,
            is_signer: false,
        });
        Ok(())
    }
}
