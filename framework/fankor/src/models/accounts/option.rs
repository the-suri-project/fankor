use crate::errors::{ErrorCode, FankorResult};
use crate::models::FankorContext;
use crate::traits::InstructionAccount;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

impl<'info, T: InstructionAccount<'info>> InstructionAccount<'info> for Option<T> {
    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        if accounts.is_empty() {
            return Err(ErrorCode::NotEnoughAccountKeys.into());
        }

        let info = &accounts[0];
        if info.key == &Pubkey::default() {
            *accounts = &accounts[1..];
            return Ok(None);
        }

        Ok(Some(T::try_from(context, accounts)?))
    }
}
