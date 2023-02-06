use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::FankorContext;
use crate::traits::{AccountInfoVerification, Instruction};
use solana_program::account_info::AccountInfo;

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
        let mut result = Vec::with_capacity(size);
        let mut new_buf = &buf[1..];
        let mut new_accounts = *accounts;

        for _ in 0..size {
            result.push(T::try_from(context, &mut new_buf, &mut new_accounts)?);
        }

        *accounts = new_accounts;
        *buf = new_buf;

        Ok(result)
    }
}
