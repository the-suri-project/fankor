use crate::errors::FankorResult;
use crate::models::FankorContext;
use crate::traits::InstructionAccount;
use solana_program::account_info::AccountInfo;

impl<'info, T: InstructionAccount<'info>> InstructionAccount<'info> for Vec<T> {
    type CPI = Vec<T::CPI>;
    type LPI = Vec<T::LPI>;

    #[inline(always)]
    fn min_accounts() -> usize {
        0 // Because can be any size.
    }

    fn verify_account_infos<F>(&self, f: &mut F) -> FankorResult<()>
    where
        F: FnMut(&FankorContext<'info>, &AccountInfo<'info>) -> FankorResult<()>,
    {
        for v in self {
            v.verify_account_infos(f)?;
        }

        Ok(())
    }

    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        crate::utils::deserialize::try_from_vec_accounts_with_bounds(
            context,
            accounts,
            0,
            usize::MAX,
        )
    }
}
