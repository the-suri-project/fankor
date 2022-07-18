use crate::errors::FankorResult;
use crate::models::FankorContext;
use crate::traits::InstructionAccount;
use solana_program::account_info::AccountInfo;

impl<'info, T: InstructionAccount<'info>> InstructionAccount<'info> for Option<T> {
    type CPI = Option<T::CPI>;

    #[cfg(feature = "library")]
    type LPI = Option<T::LPI>;

    #[inline(always)]
    fn min_accounts() -> usize {
        0 // Because None does not require any accounts.
    }

    fn verify_account_infos<F>(&self, f: &mut F) -> FankorResult<()>
    where
        F: FnMut(&FankorContext<'info>, &AccountInfo<'info>) -> FankorResult<()>,
    {
        match self {
            Some(v) => v.verify_account_infos(f),
            None => Ok(()),
        }
    }

    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        let mut new_accounts = *accounts;
        match T::try_from(context, &mut new_accounts) {
            Ok(v) => {
                *accounts = new_accounts;
                Ok(Some(v))
            }
            Err(_) => Ok(None),
        }
    }
}
