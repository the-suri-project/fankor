use crate::errors::FankorResult;
use crate::models::FankorContext;
use crate::traits::{InstructionAccount, PdaChecker};
use solana_program::account_info::AccountInfo;

impl<'info, T: InstructionAccount<'info>> InstructionAccount<'info> for Box<T> {
    type CPI = AccountInfo<'info>;
    type LPI = solana_program::pubkey::Pubkey;

    #[inline]
    fn min_accounts() -> usize {
        T::min_accounts()
    }

    fn verify_account_infos<F>(&self, f: &mut F) -> FankorResult<()>
    where
        F: FnMut(&AccountInfo<'info>) -> FankorResult<()>,
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

impl<'info, T: PdaChecker<'info>> PdaChecker<'info> for Box<T> {
    fn pda_info(&self) -> Option<&'info AccountInfo<'info>> {
        let aux: &T = self;
        aux.pda_info()
    }
}
