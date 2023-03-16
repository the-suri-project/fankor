use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::{FankorContext, Program, System};
use crate::traits::{AccountInfoVerification, Instruction, PdaChecker, SingleInstructionAccount};
use crate::utils::close::close_account;
use crate::utils::realloc::realloc_account_to_size;
use crate::utils::rent::make_rent_exempt;
use solana_program::account_info::AccountInfo;
use solana_program::clock::Epoch;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use std::fmt;
use std::fmt::{Debug, Formatter};

/// Wrapper for `AccountInfo` to explicitly define an unchecked account.
pub struct UncheckedAccount<'info> {
    context: &'info FankorContext<'info>,
    info: &'info AccountInfo<'info>,
}

impl<'info> UncheckedAccount<'info> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new account with the given data.
    pub fn new(
        context: &'info FankorContext<'info>,
        info: &'info AccountInfo<'info>,
    ) -> UncheckedAccount<'info> {
        UncheckedAccount { context, info }
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

    /// Whether the account has enough lamports to be rent-exempt or not.
    pub fn is_rent_exempt(&self) -> bool {
        let info = self.info();
        let lamports = info.lamports();
        let data_len = info.data_len();

        let rent = Rent::get().expect("Cannot access Rent Sysvar");

        rent.is_exempt(lamports, data_len)
    }

    /// Whether the account is owned by the current program.
    pub fn is_owned_by_program(&self) -> bool {
        self.info.owner == self.context.program_id()
    }

    /// Whether the account is uninitialized or not.
    pub fn is_uninitialized(&self) -> bool {
        self.context.is_account_uninitialized(self.info)
    }

    // METHODS ----------------------------------------------------------------

    /// Closes the account and sends the lamports to the `destination_account`.
    pub fn close(self, destination_account: &AccountInfo<'info>) -> FankorResult<()> {
        close_account(self.info, self.context(), destination_account)
    }

    /// Reallocates the account to the given `size`. If a `payer` is provided,
    /// fankor will add funds to the account to make it rent-exempt.
    ///
    /// # Safety
    /// This method does not check the new account data is valid. It is the caller's
    /// responsibility to ensure the new data is valid.
    pub fn realloc_unchecked(
        &self,
        size: usize,
        zero_bytes: bool,
        payer: Option<&'info AccountInfo<'info>>,
        system_program: &Program<System>,
    ) -> FankorResult<()> {
        if !self.is_owned_by_program() {
            return Err(FankorErrorCode::AccountNotOwnedByProgram {
                address: *self.address(),
                action: "reallocate",
            }
            .into());
        }

        if !self.is_writable() {
            return Err(FankorErrorCode::ReadonlyAccountModification {
                address: *self.address(),
                action: "reallocate",
            }
            .into());
        }

        if self.context.is_account_uninitialized(self.info) {
            return Err(FankorErrorCode::AlreadyClosedAccount {
                address: *self.address(),
                action: "reallocate",
            }
            .into());
        }

        realloc_account_to_size(system_program, size, zero_bytes, self.info, payer)
    }

    /// Makes the account rent-exempt by adding funds from `payer` if necessary.
    pub fn make_rent_exempt(
        &self,
        payer: &'info AccountInfo<'info>,
        system_program: &Program<System>,
    ) -> FankorResult<()> {
        self._make_rent_exempt(false, payer, system_program)
    }

    /// Makes the account rent-exempt by adding or removing funds from/to `payer`
    /// if necessary.
    pub fn make_exact_rent_exempt(
        &self,
        payer: &'info AccountInfo<'info>,
        system_program: &Program<System>,
    ) -> FankorResult<()> {
        self._make_rent_exempt(true, payer, system_program)
    }

    fn _make_rent_exempt(
        &self,
        exact: bool,
        payer: &'info AccountInfo<'info>,
        system_program: &Program<System>,
    ) -> FankorResult<()> {
        if !self.is_owned_by_program() {
            return Err(FankorErrorCode::AccountNotOwnedByProgram {
                address: *self.address(),
                action: "make rent-exempt",
            }
            .into());
        }

        if !self.is_writable() {
            return Err(FankorErrorCode::ReadonlyAccountModification {
                address: *self.address(),
                action: "make rent-exempt",
            }
            .into());
        }

        if self.context.is_account_uninitialized(self.info) {
            return Err(FankorErrorCode::AlreadyClosedAccount {
                address: *self.address(),
                action: "make rent-exempt",
            }
            .into());
        }

        let new_size = self.info.data_len();
        make_rent_exempt(new_size, exact, payer, self.info, system_program)
    }
}

impl<'info> Instruction<'info> for UncheckedAccount<'info> {
    type CPI = AccountInfo<'info>;
    type LPI = Pubkey;

    fn verify_account_infos<'a>(
        &self,
        config: &mut AccountInfoVerification<'a, 'info>,
    ) -> FankorResult<()> {
        config.verify(self.info)
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
        *accounts = &accounts[1..];
        Ok(UncheckedAccount::new(context, info))
    }
}

impl<'info> SingleInstructionAccount<'info> for UncheckedAccount<'info> {}

impl<'info> PdaChecker<'info> for UncheckedAccount<'info> {
    fn pda_info(&self) -> Option<&'info AccountInfo<'info>> {
        Some(self.info)
    }
}

impl<'info> Debug for UncheckedAccount<'info> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("UncheckedAccount")
            .field("info", &self.info)
            .finish()
    }
}
