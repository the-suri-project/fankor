use crate::errors::{Error, FankorErrorCode, FankorResult};
use crate::models;
use crate::models::{FankorContext, FankorContextExitAction, System, Zc};
use crate::prelude::CopyType;
use crate::traits::{AccountType, InstructionAccount, ProgramType};
use crate::utils::close::close_account;
use crate::utils::realloc::realloc_account_to_size;
use crate::utils::rent::make_rent_exempt;
use solana_program::account_info::AccountInfo;
use solana_program::clock::Epoch;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::system_program;
use solana_program::sysvar::Sysvar;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

/// An initialized account deserialized in Zero Copy mode.
pub struct ZcAccount<'info, T: AccountType + CopyType<'info>> {
    context: &'info FankorContext<'info>,
    info: &'info AccountInfo<'info>,
    _data: PhantomData<T>,
}

impl<'info, T: AccountType + CopyType<'info>> ZcAccount<'info, T> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new account with the given data.
    pub fn new(
        context: &'info FankorContext<'info>,
        info: &'info AccountInfo<'info>,
    ) -> FankorResult<ZcAccount<'info, T>> {
        if info.owner == &system_program::ID && info.lamports() == 0 {
            return Err(FankorErrorCode::AccountNotInitialized { address: *info.key }.into());
        }

        if info.owner != T::owner() {
            return Err(FankorErrorCode::AccountOwnedByWrongProgram {
                address: *info.key,
                expected: *T::owner(),
                actual: *info.owner,
            }
            .into());
        }

        // Check it is not closed.
        if context.is_account_uninitialized(info) {
            return Err(FankorErrorCode::NewFromClosedAccount { address: *info.key }.into());
        }

        Ok(ZcAccount {
            context,
            info,
            _data: PhantomData,
        })
    }

    pub(crate) fn new_without_checks(
        context: &'info FankorContext<'info>,
        info: &'info AccountInfo<'info>,
    ) -> ZcAccount<'info, T> {
        ZcAccount {
            context,
            info,
            _data: PhantomData,
        }
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
    pub fn rent_epoch(&self) -> Epoch {
        self.info.rent_epoch
    }

    #[inline(always)]
    pub fn data(&self) -> Zc<'info, T> {
        Zc {
            info: self.info,
            offset: 0,
            _data: PhantomData,
        }
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

    /// The exit action of this account.
    pub fn exit_action(&self) -> Option<FankorContextExitAction<'info>> {
        self.context().get_exit_action(self.info)
    }

    /// Whether the account is uninitialized or not.
    pub fn is_uninitialized(&self) -> bool {
        self.info.owner == &system_program::ID && self.info.lamports() == 0
    }

    /// Whether the account is owned by the current program.
    pub fn is_owned_by_program(&self) -> bool {
        self.info.owner == self.context.program_id()
    }

    // METHODS ----------------------------------------------------------------

    /// Closes the account and sends the lamports to the `destination_account`.
    pub fn close(self, destination_account: &AccountInfo<'info>) -> FankorResult<()> {
        close_account(self.info, self.context(), destination_account)
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
                action: "realloc",
            }
            .into());
        }

        if !self.is_writable() {
            return Err(FankorErrorCode::ReadonlyAccountModification {
                address: *self.address(),
                action: "realloc",
            }
            .into());
        }

        if self.context.is_account_uninitialized(self.info) {
            return Err(FankorErrorCode::AlreadyClosedAccount {
                address: *self.address(),
                action: "realloc",
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

    /// Makes the account rent-exempt by adding or removing funds from/to `payer`
    /// if necessary.
    pub fn make_rent_exempt(&self, payer: &'info AccountInfo<'info>) -> FankorResult<()> {
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
        make_rent_exempt(
            &models::Program::new(self.context, program)?,
            self.info,
            new_size,
            payer,
        )
    }

    /// Invalidates the exit action for this account.
    pub fn remove_exit_action(&self) {
        self.context().remove_exit_action(self.info);
    }

    /// Ignores the account in the exit of the instruction. This is useful to avoid writing
    /// twice the same account.
    ///
    /// This replaces other exit actions associated with this account.
    pub fn ignore_at_exit(&self) {
        self.context()
            .set_exit_action(self.info, FankorContextExitAction::Ignore);
    }

    /// Makes the account be rent-exempt at exit.
    ///
    /// This replaces other exit actions associated with this account.
    pub fn make_rent_exempt_at_exit(&self, payer: &'info AccountInfo<'info>) -> FankorResult<()> {
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

        self.context().set_exit_action(
            self.info,
            FankorContextExitAction::Realloc {
                payer: Some(payer),
                zero_bytes: false,
            },
        );

        Ok(())
    }

    /// Closes the account at the end of the instruction sending the lamports to
    /// the `destination_account` account.
    ///
    /// This replaces other exit actions associated with this account.
    pub fn close_account_at_exit(
        &self,
        destination_account: &'info AccountInfo<'info>,
    ) -> FankorResult<()> {
        if !self.is_owned_by_program() {
            return Err(FankorErrorCode::AccountNotOwnedByProgram {
                address: *self.address(),
                action: "close",
            }
            .into());
        }

        if !self.is_writable() {
            return Err(FankorErrorCode::ReadonlyAccountModification {
                address: *self.address(),
                action: "close",
            }
            .into());
        }

        let context = self.context();
        context.set_exit_action(
            self.info,
            FankorContextExitAction::Close {
                destination_account,
            },
        );

        Ok(())
    }
}

impl<'info, T: AccountType + CopyType<'info>> InstructionAccount<'info> for ZcAccount<'info, T> {
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
        if info.owner == &system_program::ID && info.lamports() == 0 {
            return Err(FankorErrorCode::AccountNotInitialized { address: *info.key }.into());
        }

        if info.owner != T::owner() {
            return Err(FankorErrorCode::AccountOwnedByWrongProgram {
                address: *info.key,
                expected: *T::owner(),
                actual: *info.owner,
            }
            .into());
        }

        let result = ZcAccount::new_without_checks(context, info);

        *accounts = &accounts[1..];
        Ok(result)
    }
}

impl<'info, T: AccountType + CopyType<'info>> Debug for ZcAccount<'info, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Account").field("info", &self.info).finish()
    }
}

/// Execute the last actions over the account.
impl<'info, T: AccountType + CopyType<'info>> Drop for ZcAccount<'info, T> {
    fn drop(&mut self) {
        if let Err(e) = drop_aux(self) {
            crate::macros::panic_error!(e);
        }
    }
}

fn drop_aux<'info, T: AccountType + CopyType<'info>>(
    account: &mut ZcAccount<'info, T>,
) -> FankorResult<()> {
    // Ignore if not owned by program.
    if !account.is_owned_by_program() {
        return Ok(());
    }

    match account.context.get_exit_action(account.info) {
        None => {}
        Some(FankorContextExitAction::Ignore) => {}
        Some(FankorContextExitAction::Realloc { payer, .. }) => {
            let payer = match payer {
                Some(payer) => payer,
                None => {
                    // Ignore if payer is not provided because all ZCAccounts are
                    // already reallocated.
                    return Ok(());
                }
            };

            account.make_rent_exempt(payer)?;
        }
        Some(FankorContextExitAction::Close {
            destination_account,
        }) => {
            close_account(account.info(), account.context(), destination_account)?;
        }
    }

    Ok(())
}
