use crate::errors::{ErrorCode, FankorResult};
use crate::models::FankorContext;
use crate::traits::InstructionAccount;
use solana_program::account_info::AccountInfo;
use std::fmt;
use std::fmt::{Debug, Formatter};

/// Tries to deserialize `L` first and then `R` if `L` fails.
/// This is only useful to have a fallback for some type.
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<'info, L: InstructionAccount<'info>, R: InstructionAccount<'info>> InstructionAccount<'info>
    for Either<L, R>
{
    // CONSTRUCTORS -----------------------------------------------------------

    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        if accounts.is_empty() {
            return Err(ErrorCode::NotEnoughAccountKeys.into());
        }

        let mut new_accounts = *accounts;
        let result = match L::try_from(context, &mut new_accounts) {
            Ok(l) => Either::Left(l),
            Err(_) => Either::Right(R::try_from(context, accounts)?),
        };

        Ok(result)
    }
}

impl<'info, L: InstructionAccount<'info>, R: InstructionAccount<'info>> Debug for Either<L, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Either::Left(v) => f
                .debug_struct("Either")
                .field("Left", &v)
                .field("Right", &Option::<R>::None)
                .finish(),
            Either::Right(v) => f
                .debug_struct("Either")
                .field("Left", &Option::<L>::None)
                .field("Right", &v)
                .finish(),
        }
    }
}
