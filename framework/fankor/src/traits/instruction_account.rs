use crate::errors::FankorResult;
use crate::models::FankorContext;
use solana_program::account_info::AccountInfo;

/// Trait for account wrappers.
pub trait InstructionAccount<'info>: Sized {
    fn verify_account_infos<F>(&self, f: &mut F) -> FankorResult<()>
    where
        F: FnMut(&FankorContext<'info>, &AccountInfo<'info>) -> FankorResult<()>;

    fn try_from(
        context: &'info FankorContext<'info>,
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self>;
}

// TODO
// check the accounts are unique, otherwise it can create errors.
//
// // Functional
// init
// seeds: // checks the pda
