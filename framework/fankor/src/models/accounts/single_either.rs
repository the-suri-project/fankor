use crate::errors::FankorResult;
use crate::models::FankorContext;
use crate::prelude::PdaChecker;
use crate::traits::{AccountInfoVerification, Instruction, SingleInstructionAccount};
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use std::fmt;
use std::fmt::{Debug, Formatter};

/// Tries to deserialize `L` first and then `R` if `L` fails.
/// Note that `L` and `R` must be disjoint types, otherwise the deserialization will
/// always return `L`.
pub enum SingleEither<L, R> {
    Left(L),
    Right(R),
}

impl<'info, L: SingleInstructionAccount<'info>, R: SingleInstructionAccount<'info>>
    SingleEither<L, R>
{
    // GETTERS -----------------------------------------------------------------

    pub fn is_left(&self) -> bool {
        matches!(self, SingleEither::Left(_))
    }

    pub fn is_right(&self) -> bool {
        matches!(self, SingleEither::Right(_))
    }

    pub fn left(&self) -> Option<&L> {
        match self {
            SingleEither::Left(v) => Some(v),
            SingleEither::Right(_) => None,
        }
    }

    pub fn left_mut(&mut self) -> Option<&mut L> {
        match self {
            SingleEither::Left(v) => Some(v),
            SingleEither::Right(_) => None,
        }
    }

    pub fn right(&self) -> Option<&R> {
        match self {
            SingleEither::Left(_) => None,
            SingleEither::Right(v) => Some(v),
        }
    }

    pub fn right_mut(&mut self) -> Option<&mut R> {
        match self {
            SingleEither::Left(_) => None,
            SingleEither::Right(v) => Some(v),
        }
    }

    // METHODS ----------------------------------------------------------------

    pub fn unwrap_left(self) -> Option<L> {
        match self {
            SingleEither::Left(v) => Some(v),
            SingleEither::Right(_) => None,
        }
    }

    pub fn unwrap_right(self) -> Option<R> {
        match self {
            SingleEither::Left(_) => None,
            SingleEither::Right(v) => Some(v),
        }
    }
}

impl<'info, L: Instruction<'info>, R: Instruction<'info>> Instruction<'info>
    for SingleEither<L, R>
{
    type CPI = AccountInfo<'info>;
    type LPI = Pubkey;

    fn verify_account_infos<'a>(
        &self,
        config: &mut AccountInfoVerification<'a, 'info>,
    ) -> FankorResult<()> {
        match self {
            SingleEither::Left(v) => v.verify_account_infos(config),
            SingleEither::Right(v) => v.verify_account_infos(config),
        }
    }

    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        buf: &mut &[u8],
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        let mut new_buf = *buf;
        let mut new_accounts = *accounts;
        let result = match <L as Instruction>::try_from(context, &mut new_buf, &mut new_accounts) {
            Ok(v) => {
                *buf = new_buf;
                *accounts = new_accounts;

                Self::Left(v)
            }
            Err(_) => Self::Right(<R as Instruction>::try_from(context, buf, accounts)?),
        };

        Ok(result)
    }
}

impl<'info, L: SingleInstructionAccount<'info>, R: SingleInstructionAccount<'info>>
    SingleInstructionAccount<'info> for SingleEither<L, R>
{
    fn info(&self) -> &'info AccountInfo<'info> {
        match self {
            SingleEither::Left(v) => v.info(),
            SingleEither::Right(v) => v.info(),
        }
    }

    fn context(&self) -> &'info FankorContext<'info> {
        match self {
            SingleEither::Left(v) => v.context(),
            SingleEither::Right(v) => v.context(),
        }
    }
}

impl<'info, L: PdaChecker<'info>, R: PdaChecker<'info>> PdaChecker<'info> for SingleEither<L, R> {
    fn pda_info(&self) -> Option<&'info AccountInfo<'info>> {
        match self {
            SingleEither::Left(v) => v.pda_info(),
            SingleEither::Right(v) => v.pda_info(),
        }
    }
}

impl<'info, L: Debug + Instruction<'info>, R: Debug + Instruction<'info>> Debug
    for SingleEither<L, R>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SingleEither::Left(v) => f
                .debug_struct("SingleEither")
                .field("Left", &v)
                .field("Right", &Option::<R>::None)
                .finish(),
            SingleEither::Right(v) => f
                .debug_struct("SingleEither")
                .field("Left", &Option::<L>::None)
                .field("Right", &v)
                .finish(),
        }
    }
}
