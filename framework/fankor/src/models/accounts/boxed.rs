use crate::errors::FankorResult;
use crate::models::FankorContext;
use crate::traits::{
    AccountInfoVerification, CpiInstruction, Instruction, LpiInstruction, PdaChecker,
    SingleInstructionAccount,
};
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use std::io::Write;

impl<'info, T: Instruction<'info>> Instruction<'info> for Box<T> {
    type CPI = Box<T::CPI>;
    type LPI = Box<T::LPI>;

    fn verify_account_infos<'a>(
        &self,
        config: &mut AccountInfoVerification<'a, 'info>,
    ) -> FankorResult<()> {
        T::verify_account_infos(self, config)
    }

    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        buf: &mut &[u8],
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        Ok(Box::new(T::try_from(context, buf, accounts)?))
    }
}

impl<'info, T: SingleInstructionAccount<'info>> SingleInstructionAccount<'info> for Box<T> {}

impl<'info, T: PdaChecker<'info>> PdaChecker<'info> for Box<T> {
    fn pda_info(&self) -> Option<&'info AccountInfo<'info>> {
        let aux: &T = self;
        aux.pda_info()
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl<'info, T: CpiInstruction<'info>> CpiInstruction<'info> for Box<T> {
    fn serialize_into_instruction_parts<W: Write>(
        &self,
        writer: &mut W,
        metas: &mut Vec<AccountMeta>,
        infos: &mut Vec<AccountInfo<'info>>,
    ) -> FankorResult<()> {
        T::serialize_into_instruction_parts(self, writer, metas, infos)
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl<T: LpiInstruction> LpiInstruction for Box<T> {
    fn serialize_into_instruction_parts<W: Write>(
        &self,
        writer: &mut W,
        metas: &mut Vec<AccountMeta>,
    ) -> FankorResult<()> {
        T::serialize_into_instruction_parts(self, writer, metas)
    }
}
