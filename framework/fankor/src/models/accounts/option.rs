use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::FankorContext;
use crate::traits::{AccountInfoVerification, Instruction, PdaChecker};
use solana_program::account_info::AccountInfo;
use std::any::type_name;

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

        let result = match buf[0] {
            0 => {
                let mut new_buf = &buf[1..];
                let mut new_accounts = *accounts;
                let result = Some(T::try_from(context, &mut new_buf, &mut new_accounts)?);

                *accounts = new_accounts;
                *buf = new_buf;

                result
            }
            1 => None,
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
