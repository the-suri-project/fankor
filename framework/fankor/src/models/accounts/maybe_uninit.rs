use crate::errors::FankorResult;
use crate::models::{FankorContext, UninitializedAccount};
use crate::prelude::PdaChecker;
use crate::traits::{AccountInfoVerification, Instruction, SingleInstructionAccount};
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use std::fmt;
use std::fmt::{Debug, Formatter};

/// Tries to deserialize an actual account or its uninitialized counterpart.
pub enum MaybeUninitialized<'info, T> {
    Init(T),
    Uninit(UninitializedAccount<'info>),
}

impl<'info, T: SingleInstructionAccount<'info>> MaybeUninitialized<'info, T> {
    // GETTERS -----------------------------------------------------------------

    pub fn is_init(&self) -> bool {
        matches!(self, Self::Init(_))
    }

    pub fn is_uninit(&self) -> bool {
        matches!(self, Self::Uninit(_))
    }

    pub fn init(&self) -> Option<&T> {
        match self {
            Self::Init(v) => Some(v),
            Self::Uninit(_) => None,
        }
    }

    pub fn init_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Init(v) => Some(v),
            Self::Uninit(_) => None,
        }
    }

    pub fn uninit(&self) -> Option<&UninitializedAccount<'info>> {
        match self {
            Self::Init(_) => None,
            Self::Uninit(v) => Some(v),
        }
    }

    pub fn uninit_mut(&mut self) -> Option<&mut UninitializedAccount<'info>> {
        match self {
            Self::Init(_) => None,
            Self::Uninit(v) => Some(v),
        }
    }

    // METHODS ----------------------------------------------------------------

    pub fn unwrap_init(self) -> Option<T> {
        match self {
            Self::Init(v) => Some(v),
            Self::Uninit(_) => None,
        }
    }

    pub fn unwrap_uninit(self) -> Option<UninitializedAccount<'info>> {
        match self {
            Self::Init(_) => None,
            Self::Uninit(v) => Some(v),
        }
    }
}

impl<'info, T: Instruction<'info>> Instruction<'info> for MaybeUninitialized<'info, T> {
    type CPI = AccountInfo<'info>;
    type LPI = Pubkey;

    fn verify_account_infos<'a>(
        &self,
        config: &mut AccountInfoVerification<'a, 'info>,
    ) -> FankorResult<()> {
        match self {
            Self::Init(v) => v.verify_account_infos(config),
            Self::Uninit(v) => v.verify_account_infos(config),
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
        let result = match <T as Instruction>::try_from(context, &mut new_buf, &mut new_accounts) {
            Ok(v) => {
                *buf = new_buf;
                *accounts = new_accounts;

                Self::Init(v)
            }
            Err(_) => Self::Uninit(<UninitializedAccount as Instruction>::try_from(
                context, buf, accounts,
            )?),
        };

        Ok(result)
    }
}

impl<'info, T: SingleInstructionAccount<'info>> SingleInstructionAccount<'info>
    for MaybeUninitialized<'info, T>
{
    fn info(&self) -> &'info AccountInfo<'info> {
        match self {
            MaybeUninitialized::Init(v) => v.info(),
            MaybeUninitialized::Uninit(v) => v.info(),
        }
    }

    fn context(&self) -> &'info FankorContext<'info> {
        match self {
            MaybeUninitialized::Init(v) => v.context(),
            MaybeUninitialized::Uninit(v) => v.context(),
        }
    }
}

impl<'info, T: PdaChecker<'info>> PdaChecker<'info> for MaybeUninitialized<'info, T> {
    fn pda_info(&self) -> Option<&'info AccountInfo<'info>> {
        match self {
            Self::Init(v) => v.pda_info(),
            Self::Uninit(v) => v.pda_info(),
        }
    }
}

impl<'info, T: Debug> Debug for MaybeUninitialized<'info, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Init(v) => f
                .debug_struct("MaybeUninitializedAccount")
                .field("Init", &v)
                .field("Uninit", &Option::<UninitializedAccount<'info>>::None)
                .finish(),
            Self::Uninit(v) => f
                .debug_struct("MaybeUninitializedAccount")
                .field("Init", &Option::<T>::None)
                .field("Uninit", &v)
                .finish(),
        }
    }
}
