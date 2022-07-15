use crate::errors::{ErrorCode, FankorResult};
use crate::models::FankorContext;
use crate::traits::InstructionAccount;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

pub struct Program<'info, T: Debug + crate::traits::ProgramAccount> {
    context: &'info FankorContext<'info>,
    info: &'info AccountInfo<'info>,
    _data: PhantomData<T>,
}

impl<'info, T: Debug + crate::traits::ProgramAccount> Program<'info, T> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new account with the given data.
    pub fn new(
        context: &'info FankorContext<'info>,
        info: &'info AccountInfo<'info>,
    ) -> FankorResult<Program<'info, T>> {
        if info.key != T::address() {
            return Err(ErrorCode::InvalidProgram {
                expected: *T::owner(),
                actual: *info.key,
            }
            .into());
        }

        if !info.executable {
            return Err(ErrorCode::ProgramIsNotExecutable { program: *info.key }.into());
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
    pub fn info(&self) -> &'info AccountInfo<'info> {
        self.info
    }

    #[inline(always)]
    pub fn context(&self) -> &'info FankorContext<'info> {
        self.context
    }
}

impl<'info, T: Debug + crate::traits::ProgramAccount> InstructionAccount<'info>
    for Program<'info, T>
{
    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        if accounts.is_empty() {
            return Err(ErrorCode::NotEnoughAccountKeys.into());
        }

        let info = &accounts[0];
        let result = Program::new(context, info)?;

        *accounts = &accounts[1..];
        Ok(result)
    }
}

impl<'info, T: Debug + crate::traits::ProgramAccount> Debug for Program<'info, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Program").field("info", &self.info).finish()
    }
}

impl<'info, T: Debug + crate::traits::ProgramAccount> AsRef<AccountInfo<'info>>
    for Program<'info, T>
{
    fn as_ref(&self) -> &AccountInfo<'info> {
        self.info
    }
}
