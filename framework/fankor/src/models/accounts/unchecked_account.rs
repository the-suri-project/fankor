use crate::errors::{FankorErrorCode, FankorResult};
use crate::models;
use crate::models::{FankorContext, System};
use crate::traits::{InstructionAccount, Program};
use crate::utils::close::close_account;
use crate::utils::realloc::realloc_account_to_size;
use solana_program::account_info::AccountInfo;
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

    /// Whether the account is closed or not, i.e. it matches all these constraints:
    /// - it does not have lamports
    /// - It is writable
    /// - the discriminator is zeroed
    ///
    /// Otherwise it is considered open.
    ///
    /// Note: if the account contains less data than CLOSED_ACCOUNT_DISCRIMINATOR, all must
    /// be zeroed out to be considered closed.
    pub fn is_closed(&self) -> bool {
        let info = self.info;
        if info.lamports() > 0 {
            return false;
        }

        if !info.is_writable {
            return false;
        }

        let data = info.try_borrow_data().unwrap_or_else(|_| {
            panic!(
                "There's probably a deadlock reading account data of: {}",
                info.key
            )
        });

        for i in data
            .iter()
            .take(self.context().discriminator_length() as usize)
        {
            if *i != 0 {
                return false;
            }
        }

        true
    }

    // METHODS ----------------------------------------------------------------

    /// Closes the account and sends the lamports to the `sol_destination`.
    pub fn close(self, sol_destination: &AccountInfo<'info>) -> FankorResult<()> {
        close_account(self.info, self.context(), sol_destination)
    }

    /// Reallocates the account to the given `size`. If a `payer` is provided,
    /// fankor will add funds to the account to make it rent-exempt.
    pub fn realloc(
        &self,
        size: usize,
        zero_bytes: bool,
        payer: Option<&'info AccountInfo<'info>>,
    ) -> FankorResult<()> {
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

        if self.context.is_account_closed(self.info) {
            return Err(FankorErrorCode::AlreadyClosedAccount {
                address: *self.address(),
                action: "reallocate",
            }
            .into());
        }

        realloc_account_to_size(
            &models::Program::new(self.context, program)?,
            self.info,
            size,
            zero_bytes,
            payer,
        )
    }
}

impl<'info> InstructionAccount<'info> for UncheckedAccount<'info> {
    type CPI = AccountInfo<'info>;
    type LPI = Pubkey;

    #[inline(always)]
    fn min_accounts() -> usize {
        1
    }

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
            return Err(FankorErrorCode::NotEnoughAccountKeys.into());
        }

        let info = &accounts[0];
        *accounts = &accounts[1..];
        Ok(UncheckedAccount::new(context, info))
    }
}

impl<'info> Debug for UncheckedAccount<'info> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("UncheckedAccount")
            .field("info", &self.info)
            .finish()
    }
}
