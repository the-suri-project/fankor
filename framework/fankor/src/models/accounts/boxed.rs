use crate::errors::FankorResult;
use crate::models::FankorContext;
use crate::traits::InstructionAccount;
use solana_program::account_info::AccountInfo;

impl<'info, T: InstructionAccount<'info>> InstructionAccount<'info> for Box<T> {
    type CPI = AccountInfo<'info>;

    #[cfg(feature = "library")]
    type LPI = solana_program::pubkey::Pubkey;

    fn verify_account_infos<F>(&self, f: &mut F) -> FankorResult<()>
    where
        F: FnMut(&FankorContext<'info>, &AccountInfo<'info>) -> FankorResult<()>,
    {
        T::verify_account_infos(self, f)
    }

    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        Ok(Box::new(T::try_from(context, accounts)?))
    }
}
