use crate::errors::FankorResult;
use crate::models::FankorContext;
use crate::traits::{AccountInfoVerification, InstructionAccount};
use solana_program::account_info::AccountInfo;

impl<'info, T: InstructionAccount<'info>> InstructionAccount<'info> for Vec<T> {
    type CPI = Vec<T::CPI>;
    type LPI = Vec<T::LPI>;

    #[inline(always)]
    fn min_accounts() -> usize {
        0 // Because can be any size.
    }

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
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        let mut result = Vec::new();
        let mut new_accounts = *accounts;

        loop {
            let mut step_accounts = new_accounts;
            if let Ok(account) = T::try_from(context, &mut step_accounts) {
                new_accounts = step_accounts;
                result.push(account);
            } else {
                break;
            }
        }

        *accounts = new_accounts;

        Ok(result)
    }
}
