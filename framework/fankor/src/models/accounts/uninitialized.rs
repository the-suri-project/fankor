use crate::cpi::system_program::CpiCreateAccount;
use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::{Account, Either, FankorContext, System};
use crate::traits::{AccountSize, AccountType, InstructionAccount, ProgramType};
use crate::{cpi, models};
use solana_program::account_info::AccountInfo;
use solana_program::clock::Epoch;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::system_program;
use solana_program::sysvar::Sysvar;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

/// Alias for the common case of having either an actual account or its uninitialized counterpart.
pub type MaybeUninitializedAccount<'info, T> =
    Either<Account<'info, T>, UninitializedAccount<'info, T>>;

/// Wrapper for `AccountInfo` to explicitly define an uninitialized account.
pub struct UninitializedAccount<'info, T: AccountType> {
    context: &'info FankorContext<'info>,
    info: &'info AccountInfo<'info>,
    _data: PhantomData<T>,
}

impl<'info, T: AccountType> UninitializedAccount<'info, T> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new account with the given data.
    pub fn new(
        context: &'info FankorContext<'info>,
        info: &'info AccountInfo<'info>,
    ) -> FankorResult<UninitializedAccount<'info, T>> {
        if info.owner != &system_program::ID || info.lamports() > 0 {
            return Err(FankorErrorCode::AccountAlreadyInitialized { address: *info.key }.into());
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
        self.info.key
    }

    #[inline(always)]
    pub fn owner(&self) -> &'info Pubkey {
        self.info.owner
    }

    #[inline(always)]
    pub fn is_writable(&self) -> bool {
        self.info.is_writable
    }

    #[inline(always)]
    pub fn is_executable(&self) -> bool {
        self.info.executable
    }

    #[inline(always)]
    pub fn balance(&self) -> u64 {
        self.info.lamports()
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

impl<'info, T: Default + AccountType> UninitializedAccount<'info, T> {
    // METHODS ----------------------------------------------------------------

    /// Initializes the account transferring the necessary lamports to cover the rent
    /// for the given `space` using `payer` as the funding account.
    pub fn init(self, payer: &AccountInfo<'info>, space: usize) -> FankorResult<Account<'info, T>> {
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(space as usize);
        let program = match self.context.get_account_from_address(System::address()) {
            Some(v) => v,
            None => {
                return Err(FankorErrorCode::MissingProgram {
                    address: *System::address(),
                    name: System::name(),
                }
                .into());
            }
        };

        cpi::system_program::create_account(
            &models::Program::new(self.context, program)?,
            CpiCreateAccount {
                from: payer.clone(),
                to: self.info.clone(),
            },
            lamports,
            space as u64,
            self.context.program_id(),
            &[],
        )?;

        Ok(Account::new_without_checks(
            self.context,
            self.info,
            T::default(),
        ))
    }

    /// Initializes the PDA account transferring the necessary lamports to cover the rent
    /// for the given `space` using `payer` as the funding account.
    pub fn init_pda(
        self,
        payer: &AccountInfo<'info>,
        space: usize,
        seeds: &[&[u8]],
    ) -> FankorResult<Account<'info, T>> {
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(space as usize);
        let program = match self.context.get_account_from_address(System::address()) {
            Some(v) => v,
            None => {
                return Err(FankorErrorCode::MissingProgram {
                    address: *System::address(),
                    name: System::name(),
                }
                .into());
            }
        };

        cpi::system_program::create_account(
            &models::Program::new(self.context, program)?,
            CpiCreateAccount {
                from: payer.clone(),
                to: self.info.clone(),
            },
            lamports,
            space as u64,
            self.context.program_id(),
            &[seeds],
        )?;

        Ok(Account::new_without_checks(
            self.context,
            self.info,
            T::default(),
        ))
    }
}

impl<'info, T: Default + AccountType + AccountSize> UninitializedAccount<'info, T> {
    // METHODS ----------------------------------------------------------------

    /// Initializes the account transferring the necessary lamports to cover the rent
    /// for the minimum space to contain the smallest value of `T`
    /// using `payer` as the funding account.
    pub fn init_with_min_space(
        self,
        payer: &AccountInfo<'info>,
    ) -> FankorResult<Account<'info, T>> {
        self.init(payer, T::min_account_size())
    }

    /// Initializes the PDA account transferring the necessary lamports to cover the rent
    /// for the minimum space to contain the smallest value of `T`
    /// using `payer` as the funding account.
    pub fn init_pda_with_min_space(
        self,
        payer: &AccountInfo<'info>,
        seeds: &[&[u8]],
    ) -> FankorResult<Account<'info, T>> {
        self.init_pda(payer, T::min_account_size(), seeds)
    }
}

impl<'info, T: AccountType + AccountSize> UninitializedAccount<'info, T> {
    // METHODS ----------------------------------------------------------------

    /// Initializes the account transferring the necessary lamports to cover the rent
    /// for the required space to contain `value` using `payer` as the funding account.
    pub fn init_with_value(
        self,
        payer: &AccountInfo<'info>,
        value: T,
    ) -> FankorResult<Account<'info, T>> {
        let rent = Rent::get()?;
        let space = value.actual_account_size();
        let lamports = rent.minimum_balance(space as usize);
        let program = match self.context.get_account_from_address(System::address()) {
            Some(v) => v,
            None => {
                return Err(FankorErrorCode::MissingProgram {
                    address: *System::address(),
                    name: System::name(),
                }
                .into());
            }
        };

        cpi::system_program::create_account(
            &models::Program::new(self.context, program)?,
            CpiCreateAccount {
                from: payer.clone(),
                to: self.info.clone(),
            },
            lamports,
            space as u64,
            self.context.program_id(),
            &[],
        )?;

        Ok(Account::new_without_checks(self.context, self.info, value))
    }

    /// Initializes the account transferring the necessary lamports to cover the rent
    /// for the required space to contain `value` using `payer` as the funding account.
    pub fn init_pda_with_value(
        self,
        payer: &AccountInfo<'info>,
        value: T,
        seeds: &[&[u8]],
    ) -> FankorResult<Account<'info, T>> {
        let rent = Rent::get()?;
        let space = value.actual_account_size();
        let lamports = rent.minimum_balance(space as usize);
        let program = match self.context.get_account_from_address(System::address()) {
            Some(v) => v,
            None => {
                return Err(FankorErrorCode::MissingProgram {
                    address: *System::address(),
                    name: System::name(),
                }
                .into());
            }
        };

        cpi::system_program::create_account(
            &models::Program::new(self.context, program)?,
            CpiCreateAccount {
                from: payer.clone(),
                to: self.info.clone(),
            },
            lamports,
            space as u64,
            self.context.program_id(),
            &[seeds],
        )?;

        Ok(Account::new_without_checks(self.context, self.info, value))
    }
}

impl<'info, T: AccountType> InstructionAccount<'info> for UninitializedAccount<'info, T> {
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
        *accounts = &accounts[1..];
        UninitializedAccount::new(context, info)
    }
}

impl<'info, T: AccountType> Debug for UninitializedAccount<'info, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("UninitializedAccount")
            .field("info", &self.info)
            .finish()
    }
}
