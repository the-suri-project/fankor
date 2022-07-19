use crate::errors::{ErrorCode, FankorResult};
use crate::models::FankorContext;
use crate::traits::{CpiInstructionAccount, InstructionAccount};
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;

/// A struct that receives no accounts.
pub struct NoAccounts<'info> {
    context: &'info FankorContext<'info>,
}

impl<'info> NoAccounts<'info> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new account with the given data.
    pub fn new(context: &'info FankorContext<'info>) -> NoAccounts<'info> {
        NoAccounts { context }
    }

    // GETTERS ----------------------------------------------------------------

    #[inline(always)]
    pub fn context(&self) -> &'info FankorContext<'info> {
        self.context
    }
}

impl<'info> InstructionAccount<'info> for NoAccounts<'info> {
    type CPI = CpiNoAccounts;

    type LPI = LpiNoAccounts;

    #[inline(always)]
    fn min_accounts() -> usize {
        0
    }

    fn verify_account_infos<F>(&self, _f: &mut F) -> FankorResult<()>
    where
        F: FnMut(&FankorContext<'info>, &AccountInfo<'info>) -> FankorResult<()>,
    {
        Ok(())
    }

    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        if !accounts.is_empty() {
            return Err(ErrorCode::NotAccountsExpected.into());
        }

        Ok(NoAccounts::new(context))
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct CpiNoAccounts;

impl<'info> CpiInstructionAccount<'info> for CpiNoAccounts {
    fn to_account_metas_and_infos(
        &self,
        _metas: &mut Vec<AccountMeta>,
        _infos: &mut Vec<AccountInfo<'info>>,
    ) -> FankorResult<()> {
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct LpiNoAccounts;

impl crate::traits::LpiInstructionAccount for LpiNoAccounts {
    fn to_account_metas(&self, _metas: &mut Vec<AccountMeta>) -> FankorResult<()> {
        Ok(())
    }
}
