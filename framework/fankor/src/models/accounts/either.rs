use crate::errors::FankorResult;
use crate::models::FankorContext;
use crate::traits::{CpiInstructionAccount, InstructionAccount};
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use std::fmt;
use std::fmt::{Debug, Formatter};

/// Tries to deserialize `L` first and then `R` if `L` fails.
///
/// This is useful to have a fallback for some type, for example, it can be used for maybe
/// uninitialized accounts: `Either<Account<'info, T>, UninitializedAccount<'info, T>>`.
/// For this case you can use the `MaybeUninitializedAccount` type alias.
///
/// Note that `L` and `R` must be disjoint types, otherwise the deserialization will
/// always return `L`.
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<'info, L: InstructionAccount<'info>, R: InstructionAccount<'info>> Either<L, R> {
    // GETTERS -----------------------------------------------------------------

    pub fn is_left(&self) -> bool {
        matches!(self, Either::Left(_))
    }

    pub fn is_right(&self) -> bool {
        matches!(self, Either::Right(_))
    }

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
    type LPI = LpiEither<L::LPI, R::LPI>;

    #[inline]
    fn min_accounts() -> usize {
        L::min_accounts().min(R::min_accounts())
    }

    fn verify_account_infos<F>(&self, f: &mut F) -> FankorResult<()>
    where
        F: FnMut(&AccountInfo<'info>) -> FankorResult<()>,
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
        let mut new_accounts = *accounts;
        match L::try_from(context, &mut new_accounts) {
            Ok(l) => {
                *accounts = new_accounts;
                Ok(Either::Left(l))
            }
            Err(_) => Ok(Either::Right(R::try_from(context, accounts)?)),
        }
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
    fn to_account_metas_and_infos(
        &self,
        metas: &mut Vec<AccountMeta>,
        infos: &mut Vec<AccountInfo<'info>>,
    ) -> FankorResult<()> {
        match self {
            CpiEither::Left(v) => v.to_account_metas_and_infos(metas, infos),
            CpiEither::Right(v) => v.to_account_metas_and_infos(metas, infos),
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub enum LpiEither<L, R> {
    Left(L),
    Right(R),
}

impl<L: crate::traits::LpiInstructionAccount, R: crate::traits::LpiInstructionAccount>
    crate::traits::LpiInstructionAccount for LpiEither<L, R>
{
    fn to_account_metas(&self, metas: &mut Vec<AccountMeta>) -> FankorResult<()> {
        match self {
            LpiEither::Left(v) => v.to_account_metas(metas),
            LpiEither::Right(v) => v.to_account_metas(metas),
        }
    }
}
