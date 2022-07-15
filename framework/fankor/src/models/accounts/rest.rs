use crate::errors::FankorResult;
use crate::models::FankorContext;
use crate::traits::InstructionAccount;
use solana_program::account_info::AccountInfo;
use std::fmt;
use std::fmt::{Debug, Formatter};

/// A wrapper around a `Vec<AccountInfo>` that keeps the rest infos.
pub struct Rest<'info> {
    context: &'info FankorContext<'info>,
    accounts: &'info [AccountInfo<'info>],
}

impl<'info> Rest<'info> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new account with the given data.
    pub fn new(
        context: &'info FankorContext<'info>,
        accounts: &'info [AccountInfo<'info>],
    ) -> FankorResult<Rest<'info>> {
        Ok(Rest { context, accounts })
    }

    // GETTERS ----------------------------------------------------------------

    #[inline(always)]
    pub fn accounts(&self) -> &'info [AccountInfo<'info>] {
        self.accounts
    }

    #[inline(always)]
    pub fn context(&self) -> &'info FankorContext<'info> {
        self.context
    }
}

impl<'info> InstructionAccount<'info> for Rest<'info> {
    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        let result = Rest::new(context, *accounts)?;

        *accounts = &[];
        Ok(result)
    }
}

impl<'info> Debug for Rest<'info> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rest")
            .field("len", &self.accounts.len())
            .finish()
    }
}
