use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::FankorContext;
use crate::traits::{InstructionAccount, PdaChecker, ProgramType};
use solana_program::account_info::AccountInfo;
use solana_program::clock::Epoch;
use solana_program::pubkey::Pubkey;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

/// An account that represents a program.
#[derive(Clone)]
pub struct Program<'info, T: ProgramType> {
    context: &'info FankorContext<'info>,
    info: &'info AccountInfo<'info>,
    _data: PhantomData<T>,
}

impl<'info, T: ProgramType> Program<'info, T> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new account with the given data.
    pub fn new(
        context: &'info FankorContext<'info>,
        info: &'info AccountInfo<'info>,
    ) -> FankorResult<Program<'info, T>> {
        if info.key != T::address() {
            return Err(FankorErrorCode::InvalidProgram {
                expected: *T::address(),
                actual: *info.key,
            }
            .into());
        }

        if !info.executable {
            return Err(FankorErrorCode::ProgramIsNotExecutable { program: *info.key }.into());
        }

        Ok(Program {
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

impl<'info, T: ProgramType> InstructionAccount<'info> for Program<'info, T> {
    type CPI = AccountInfo<'info>;
    type LPI = Pubkey;

    #[inline(always)]
    fn min_accounts() -> usize {
        1
    }

    fn verify_account_infos<F>(&self, f: &mut F) -> FankorResult<()>
    where
        F: FnMut(&AccountInfo<'info>) -> FankorResult<()>,
    {
        f(self.info)
    }

    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        if accounts.is_empty() {
            return Err(FankorErrorCode::NotEnoughAccountKeys.into());
        }

        let info = &accounts[0];
        let result = Program::new(context, info)?;

        *accounts = &accounts[1..];
        Ok(result)
    }
}

impl<'info, T: ProgramType> PdaChecker<'info> for Program<'info, T> {
    fn pda_info(&self) -> Option<&'info AccountInfo<'info>> {
        Some(self.info)
    }
}

impl<'info, T: ProgramType> Debug for Program<'info, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Program").field("info", &self.info).finish()
    }
}

impl<'info, T: ProgramType> AsRef<AccountInfo<'info>> for Program<'info, T> {
    fn as_ref(&self) -> &AccountInfo<'info> {
        self.info
    }
}
