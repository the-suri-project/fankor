use crate::errors::{ErrorCode, FankorResult};
use crate::models::{Account, Either, FankorContext};
use crate::traits::{AccountSize, InstructionAccount};
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use solana_program::system_program;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

pub type MaybeUninitializedAccount<'info, T> =
    Either<Account<'info, T>, UninitializedAccount<'info, T>>;

/// Wrapper for `AccountInfo` to explicitly define an uninitialized account.
pub struct UninitializedAccount<'info, T: crate::traits::Account> {
    context: &'info FankorContext<'info>,
    info: &'info AccountInfo<'info>,
    _data: PhantomData<T>,
}

impl<'info, T: crate::traits::Account> UninitializedAccount<'info, T> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new account with the given data.
    pub fn new(
        context: &'info FankorContext<'info>,
        info: &'info AccountInfo<'info>,
    ) -> FankorResult<UninitializedAccount<'info, T>> {
        if info.owner != &system_program::ID || info.lamports() > 0 {
            return Err(ErrorCode::AccountAlreadyInitialized { address: *info.key }.into());
        }

        Ok(UninitializedAccount {
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

impl<'info, T: Default + crate::traits::Account> UninitializedAccount<'info, T> {
    // METHODS ----------------------------------------------------------------

    pub fn init(self, _payer: &AccountInfo<'info>, _space: usize) -> Account<'info, T> {
        // TODO call CPI to allocate space
        // TODO call CPI to assign account

        Account::new_without_checks(self.context, self.info, T::default())
    }
}

impl<'info, T: Default + crate::traits::Account + AccountSize> UninitializedAccount<'info, T> {
    // METHODS ----------------------------------------------------------------

    pub fn init_with_min_space(self, payer: &AccountInfo<'info>) -> Account<'info, T> {
        self.init(payer, T::min_account_size())
    }
}

impl<'info, T: crate::traits::Account> InstructionAccount<'info>
    for UninitializedAccount<'info, T>
{
    type CPI = &'info AccountInfo<'info>;

    #[cfg(feature = "library")]
    type LPI = Pubkey;

    fn verify_account_infos<F>(&self, f: &mut F) -> FankorResult<()>
    where
        F: FnMut(&FankorContext<'info>, &AccountInfo<'info>) -> FankorResult<()>,
    {
        f(self.context, self.info)
    }

    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        if accounts.is_empty() {
            return Err(ErrorCode::NotEnoughAccountKeys.into());
        }

        let info = &accounts[0];
        UninitializedAccount::new(context, info)
    }
}

impl<'info, T: crate::traits::Account> Debug for UninitializedAccount<'info, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("UninitializedAccount")
            .field("info", &self.info)
            .finish()
    }
}
