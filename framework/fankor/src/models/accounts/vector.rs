use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::FankorContext;
use crate::traits::{AccountInfoVerification, CpiInstruction, Instruction, LpiInstruction};
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use std::io::Write;

impl<'info, T: Instruction<'info>> Instruction<'info> for Vec<T> {
    type CPI = Vec<T::CPI>;
    type LPI = Vec<T::LPI>;

    fn verify_account_infos<'a>(
        &self,
        config: &mut AccountInfoVerification<'a, 'info>,
    ) -> FankorResult<()> {
        for account in self {
            account.verify_account_infos(config)?;
        }

        Ok(())
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

        let size = buf[0] as usize;
        *buf = &buf[1..];

        let mut result = Vec::with_capacity(size);

        for _ in 0..size {
            result.push(T::try_from(context, buf, accounts)?);
        }

        Ok(result)
    }
}

impl<'info, T: CpiInstruction<'info>> CpiInstruction<'info> for Vec<T> {
    fn serialize_into_instruction_parts<W: Write>(
        &self,
        writer: &mut W,
        metas: &mut Vec<AccountMeta>,
        infos: &mut Vec<AccountInfo<'info>>,
    ) -> FankorResult<()> {
        let length = self.len();
        if length > u8::MAX as usize {
            return Err(FankorErrorCode::TooManyAccounts { size: length }.into());
        }

        writer.write_all(&[length as u8])?;

        for v in self {
            v.serialize_into_instruction_parts(writer, metas, infos)?;
        }

        Ok(())
    }
}

impl<T: LpiInstruction> LpiInstruction for Vec<T> {
    fn serialize_into_instruction_parts<W: Write>(
        &self,
        writer: &mut W,
        metas: &mut Vec<AccountMeta>,
    ) -> FankorResult<()> {
        let length = self.len();
        if length > u8::MAX as usize {
            return Err(FankorErrorCode::TooManyAccounts { size: length }.into());
        }

        writer.write_all(&[length as u8])?;

        for v in self {
            v.serialize_into_instruction_parts(writer, metas)?;
        }

        Ok(())
    }
}
