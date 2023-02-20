use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::FankorContext;
use crate::traits::{
    AccountInfoVerification, CpiInstruction, Instruction, LpiInstruction, PdaChecker,
};
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use std::any::type_name;
use std::io::Write;

impl<'info, T: Instruction<'info>> Instruction<'info> for Option<T> {
    type CPI = Option<T::CPI>;
    type LPI = Option<T::LPI>;

    fn verify_account_infos<'a>(
        &self,
        config: &mut AccountInfoVerification<'a, 'info>,
    ) -> FankorResult<()> {
        match self {
            Some(account) => account.verify_account_infos(config),
            None => Ok(()),
        }
    }

    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        buf: &mut &[u8],
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        if buf.is_empty() {
            return Err(FankorErrorCode::NotEnoughDataToDeserializeInstruction.into());
        }

        let condition = buf[0];
        *buf = &buf[1..];

        let result = match condition {
            0 => None,
            1 => Some(T::try_from(context, buf, accounts)?),
            _ => {
                return Err(FankorErrorCode::InstructionDidNotDeserialize {
                    account: type_name::<Self>().to_string(),
                }
                .into())
            }
        };

        Ok(result)
    }
}

impl<'info, T: PdaChecker<'info>> PdaChecker<'info> for Option<T> {
    fn pda_info(&self) -> Option<&'info AccountInfo<'info>> {
        match self {
            Some(v) => v.pda_info(),
            None => None,
        }
    }
}

impl<'info, T: CpiInstruction<'info>> CpiInstruction<'info> for Option<T> {
    fn serialize_into_instruction_parts<W: Write>(
        &self,
        writer: &mut W,
        metas: &mut Vec<AccountMeta>,
        infos: &mut Vec<AccountInfo<'info>>,
    ) -> FankorResult<()> {
        if let Some(v) = self {
            writer.write_all(&[1])?;
            v.serialize_into_instruction_parts(writer, metas, infos)?;
        } else {
            writer.write_all(&[0])?;
        }

        Ok(())
    }
}

impl<T: LpiInstruction> LpiInstruction for Option<T> {
    fn serialize_into_instruction_parts<W: Write>(
        &self,
        writer: &mut W,
        metas: &mut Vec<AccountMeta>,
    ) -> FankorResult<()> {
        if let Some(v) = self {
            writer.write_all(&[1])?;
            v.serialize_into_instruction_parts(writer, metas)?;
        } else {
            writer.write_all(&[0])?;
        }

        Ok(())
    }
}
