use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::FankorContext;
use crate::traits::{AccountInfoVerification, Instruction, PdaChecker, SingleInstructionAccount};
use solana_program::account_info::AccountInfo;
use solana_program::clock::Epoch;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::SysvarId;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

/// A Sysvar account.
#[derive(Clone)]
pub struct SysvarAccount<'info, T: SysvarId> {
    context: &'info FankorContext<'info>,
    info: &'info AccountInfo<'info>,
    _data: PhantomData<T>,
}

impl<'info, T: SysvarId> SysvarAccount<'info, T> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new Sysvar account with the given data.
    pub fn new(
        context: &'info FankorContext<'info>,
        info: &'info AccountInfo<'info>,
    ) -> FankorResult<SysvarAccount<'info, T>> {
        if info.owner == &T::id() {
            return Err(FankorErrorCode::IncorrectSysvarAccount {
                actual: *info.owner,
                expected: T::id(),
            }
            .into());
        }

        Ok(SysvarAccount {
            context,
            info,
            _data: PhantomData,
        })
    }

    // GETTERS ----------------------------------------------------------------

    #[inline(always)]
    pub fn address(&self) -> &'info Pubkey {
        self.info().key
    }

    #[inline(always)]
    pub fn owner(&self) -> &'info Pubkey {
        self.info().owner
    }

    #[inline(always)]
    pub fn is_writable(&self) -> bool {
        self.info().is_writable
    }

    #[inline(always)]
    pub fn is_signer(&self) -> bool {
        self.info().is_signer
    }

    #[inline(always)]
    pub fn is_executable(&self) -> bool {
        self.info().executable
    }

    #[inline(always)]
    pub fn balance(&self) -> u64 {
        self.info().lamports()
    }

    #[inline(always)]
    pub fn rent_epoch(&self) -> Epoch {
        self.info.rent_epoch
    }

    #[inline(always)]
    pub fn info(&self) -> &'info AccountInfo<'info> {
        self.info
    }

    #[inline(always)]
    pub fn context(&self) -> &'info FankorContext<'info> {
        self.context
    }
}

impl<'info, T: SysvarId> Instruction<'info> for SysvarAccount<'info, T> {
    type CPI = AccountInfo<'info>;
    type LPI = Pubkey;

    fn verify_account_infos<'a>(
        &self,
        config: &mut AccountInfoVerification<'a, 'info>,
    ) -> FankorResult<()> {
        config.verify_only_constraints(self.info)
    }

    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        _buf: &mut &[u8],
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        if accounts.is_empty() {
            return Err(FankorErrorCode::NotEnoughAccountKeys.into());
        }

        let info = &accounts[0];
        let result = SysvarAccount::new(context, info)?;

        *accounts = &accounts[1..];
        Ok(result)
    }
}

impl<'info, T: SysvarId> SingleInstructionAccount<'info> for SysvarAccount<'info, T> {
    fn info(&self) -> &'info AccountInfo<'info> {
        self.info
    }

    fn context(&self) -> &'info FankorContext<'info> {
        self.context
    }
}

impl<'info, T: SysvarId> PdaChecker<'info> for SysvarAccount<'info, T> {
    fn pda_info(&self) -> Option<&'info AccountInfo<'info>> {
        Some(self.info)
    }
}

impl<'info, T: SysvarId> Debug for SysvarAccount<'info, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SysvarAccount")
            .field("info", &self.info)
            .finish()
    }
}
