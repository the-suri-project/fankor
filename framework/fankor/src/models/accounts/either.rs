use crate::errors::{ErrorCode, FankorResult};
use crate::models::FankorContext;
use crate::traits::{CpiInstructionAccount, InstructionAccount, LpiInstructionAccount};
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use std::fmt;
use std::fmt::{Debug, Formatter};

/// Tries to deserialize `L` first and then `R` if `L` fails.
/// This is only useful to have a fallback for some type.
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<'info, L: InstructionAccount<'info>, R: InstructionAccount<'info>> Either<L, R> {
    // GETTERS -----------------------------------------------------------------

    pub fn left(&self) -> Option<&L> {
        match self {
            Either::Left(v) => Some(v),
            Either::Right(_) => None,
        }
    }

    pub fn left_mut(&mut self) -> Option<&mut L> {
        match self {
            Either::Left(v) => Some(v),
            Either::Right(_) => None,
        }
    }

    pub fn right(&self) -> Option<&R> {
        match self {
            Either::Left(_) => None,
            Either::Right(v) => Some(v),
        }
    }

    pub fn right_mut(&mut self) -> Option<&mut R> {
        match self {
            Either::Left(_) => None,
            Either::Right(v) => Some(v),
        }
    }

    // METHOD -----------------------------------------------------------------

    pub fn unwrap_left(self) -> Option<L> {
        match self {
            Either::Left(v) => Some(v),
            Either::Right(_) => None,
        }
    }

    pub fn unwrap_right(self) -> Option<R> {
        match self {
            Either::Left(_) => None,
            Either::Right(v) => Some(v),
        }
    }
}

impl<'info, L: InstructionAccount<'info>, R: InstructionAccount<'info>> InstructionAccount<'info>
    for Either<L, R>
{
    type CPI = CpiEither<L::CPI, R::CPI>;

    #[cfg(feature = "library")]
    type LPI = LpiEither<L::LPI, R::LPI>;

    fn verify_account_infos<F>(&self, f: &mut F) -> FankorResult<()>
    where
        F: FnMut(&FankorContext<'info>, &AccountInfo<'info>) -> FankorResult<()>,
    {
        match self {
            Either::Left(v) => v.verify_account_infos(f),
            Either::Right(v) => v.verify_account_infos(f),
        }
    }

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

impl<'info, L: Debug + InstructionAccount<'info>, R: Debug + InstructionAccount<'info>> Debug
    for Either<L, R>
{
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

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub enum CpiEither<L, R> {
    Left(L),
    Right(R),
}

impl<'info, L: CpiInstructionAccount<'info>, R: CpiInstructionAccount<'info>>
    CpiInstructionAccount<'info> for CpiEither<L, R>
{
    fn to_account_infos(&self, infos: &mut Vec<&'info AccountInfo<'info>>) -> FankorResult<()> {
        match self {
            CpiEither::Left(v) => v.to_account_infos(infos),
            CpiEither::Right(v) => v.to_account_infos(infos),
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(feature = "library")]
pub enum LpiEither<L, R> {
    Left(L),
    Right(R),
}

#[cfg(feature = "library")]
impl<'info, L: LpiInstructionAccount<'info>, R: LpiInstructionAccount<'info>>
    LpiInstructionAccount<'info> for LpiEither<L, R>
{
    fn to_pubkeys(&self, pubkeys: &mut Vec<Pubkey>) -> FankorResult<()> {
        match self {
            LpiEither::Left(v) => v.to_pubkeys(pubkeys),
            LpiEither::Right(v) => v.to_pubkeys(pubkeys),
        }
    }
}
