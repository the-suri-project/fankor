use crate::errors::FankorResult;
use crate::models::FankorContext;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use std::fmt::Debug;

/// Trait for instruction account definitions.
pub trait InstructionAccounts<'info>: Sized {
    fn try_deserialize(
        program_id: Pubkey,
        accounts: &mut &[AccountInfo<'info>],
    ) -> FankorResult<Self>;
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// Trait for account wrappers.
pub trait InstructionAccount<'info>: Sized + Debug {
    fn try_from(
        context: &'info FankorContext<'info>,
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self>;
}
