use crate::errors::{Error, FankorErrorCode, FankorResult};
use crate::models::{Account, FankorContext, FankorContextExitAction, Program, System, Zc};
use crate::prelude::CopyType;
use crate::traits::{
    AccountInfoVerification, AccountType, Instruction, PdaChecker, SingleInstructionAccount,
};
use crate::utils::close::close_account;
use crate::utils::realloc::realloc_account_to_size;
use crate::utils::rent::make_rent_exempt;
use crate::utils::writers::ArrayWriter;
use solana_program::account_info::AccountInfo;
use solana_program::clock::Epoch;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::system_program;
use solana_program::sysvar::Sysvar;
use std::any::type_name;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::io::Write;
use std::marker::PhantomData;

/// An initialized account deserialized in Zero Copy mode.
pub struct ZcAccount<'info, T: AccountType + CopyType<'info>> {
    context: &'info FankorContext<'info>,
    info: &'info AccountInfo<'info>,
    _data: PhantomData<T>,
    dropped: bool,
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

        // Check discriminant.
        {
            let data = info.data.borrow();
            let actual = data[0];

            if !T::check_discriminant(actual) {
                return Err(FankorErrorCode::AccountDiscriminantMismatch {
                    account: format!("ZcAccount<{}>", type_name::<T>()),
                }
                .into());
            }
        }

        Ok(ZcAccount {
            context,
            info,
            _data: PhantomData,
            dropped: false,
        })
    }

    pub fn new_unchecked(
        context: &'info FankorContext<'info>,
        info: &'info AccountInfo<'info>,
    ) -> ZcAccount<'info, T> {
        ZcAccount {
            context,
            info,
            _data: PhantomData,
            dropped: false,
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
            offset: 1, // Skip the discriminant.
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

    /// Whether the account is owned by the current program.
    pub fn is_owned_by_program(&self) -> bool {
        self.info.owner == self.context.program_id()
    }

    // METHODS ----------------------------------------------------------------

    /// Closes the account and sends the lamports to the `destination_account`.
    pub fn close(mut self, destination_account: &AccountInfo<'info>) -> FankorResult<()> {
        close_account(self.info, self.context(), destination_account)?;

        // Prevent account to execute the drop actions.
        self.dropped = true;
        Ok(())
    }

    /// Reallocates the account to the given `size`. If a `payer` is provided,
    /// fankor will add funds to the account to make it rent-exempt.
    pub fn realloc(
        &self,
        size: usize,
        zero_bytes: bool,
        payer: Option<&'info AccountInfo<'info>>,
        system_program: &Program<System>,
    ) -> FankorResult<()> {
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

        realloc_account_to_size(system_program, size, zero_bytes, self.info, payer)
    }

    /// Makes the account rent-exempt by adding or removing funds from/to `payer`
    /// if necessary.
    pub fn make_rent_exempt(
        &self,
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
        make_rent_exempt(new_size, payer, self.info, system_program)
    }

    /// Transmutes the current account into another type.
    /// This method automatically saves the new value into the account.
    pub fn transmute<D: AccountType>(
        mut self,
        new_value: D,
        zero_bytes: bool,
        payer: &'info AccountInfo<'info>,
        system_program: &Program<'info, System>,
    ) -> FankorResult<Account<'info, D>> {
        if !self.is_owned_by_program() {
            return Err(FankorErrorCode::AccountNotOwnedByProgram {
                address: *self.address(),
                action: "transmute",
            }
            .into());
        }

        if !self.is_writable() {
            return Err(FankorErrorCode::ReadonlyAccountModification {
                address: *self.address(),
                action: "transmute",
            }
            .into());
        }

        if self.context.is_account_uninitialized(self.info) {
            return Err(FankorErrorCode::AlreadyClosedAccount {
                address: *self.address(),
                action: "transmute",
            }
            .into());
        }

        let new_account = Account::new_unchecked(self.context, self.info, new_value);

        // Serialize the new value.
        let mut data_bytes = Vec::with_capacity(new_account.info().data_len());
        new_account.data().try_serialize(&mut data_bytes)?;

        // Realloc account.
        new_account.realloc_unchecked(data_bytes.len(), zero_bytes, Some(payer), system_program)?;

        // Save data.
        let mut data = self.info.try_borrow_mut_data()?;
        let dst: &mut [u8] = &mut data;
        let mut writer = ArrayWriter::new(dst);
        writer.write_all(&data_bytes)?;

        // Prevent old account to execute the drop actions.
        self.dropped = true;

        Ok(new_account)
    }

    /// Deserializes the zero-copy value and creates a new account.
    pub fn into_account(mut self) -> FankorResult<Account<'info, T>> {
        let data = self.data().try_value()?;
        let new_account = Account::new_unchecked(self.context, self.info, data);

        // Prevent old account to execute the drop actions.
        self.dropped = true;

        Ok(new_account)
    }

    /// Invalidates the exit action for this account.
    pub fn remove_exit_action(&self) {
        self.context().remove_exit_action(self.info);
    }

    /// Makes the account be rent-exempt at exit.
    ///
    /// This replaces other exit actions associated with this account.
    pub fn make_rent_exempt_at_exit(
        &self,
        payer: &'info AccountInfo<'info>,
        system_program: &Program<'info, System>,
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

        self.context().set_exit_action(
            self.info,
            FankorContextExitAction::Realloc {
                payer: Some(payer),
                zero_bytes: false,
                system_program: system_program.info(),
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

impl<'info, T: AccountType + CopyType<'info>> Instruction<'info> for ZcAccount<'info, T> {
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

        // Check discriminant.
        {
            let data = info.data.borrow();
            let actual = data[0];

            if !T::check_discriminant(actual) {
                return Err(FankorErrorCode::AccountDiscriminantMismatch {
                    account: format!("ZcAccount<{}>", type_name::<T>()),
                }
                .into());
            }
        }

        let result = ZcAccount::new_unchecked(context, info);

        *accounts = &accounts[1..];
        Ok(result)
    }
}

impl<'info, T: AccountType + CopyType<'info>> SingleInstructionAccount<'info>
    for ZcAccount<'info, T>
{
}

impl<'info, T: AccountType + CopyType<'info>> PdaChecker<'info> for ZcAccount<'info, T> {
    fn pda_info(&self) -> Option<&'info AccountInfo<'info>> {
        Some(self.info)
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
        // Ignore if not owned by program.
        if !self.is_owned_by_program() {
            return;
        }

        // Ignore already dropped accounts.
        if self.dropped {
            return;
        }

        if let Err(e) = drop_aux(self) {
            crate::macros::panic_error!(e);
        }
    }
}

fn drop_aux<'info, T: AccountType + CopyType<'info>>(
    account: &mut ZcAccount<'info, T>,
) -> FankorResult<()> {
    match account.context.get_exit_action(account.info) {
        None => {}
        Some(FankorContextExitAction::ProcessedByZeroCopy) => {}
        Some(FankorContextExitAction::Processed) => {
            return Err(FankorErrorCode::DuplicatedWritableAccounts {
                address: *account.address(),
            }
            .into());
        }
        Some(FankorContextExitAction::Realloc {
            payer,
            system_program,
            ..
        }) => {
            let payer = match payer {
                Some(payer) => payer,
                None => {
                    // Ignore if payer is not provided because all ZcAccounts are
                    // already reallocated.
                    return Ok(());
                }
            };

            account.make_rent_exempt(payer, &Program::new(account.context(), system_program)?)?;

            // Prevent executing this action twice.
            account
                .context
                .set_exit_action(account.info, FankorContextExitAction::ProcessedByZeroCopy);
        }
        Some(FankorContextExitAction::Close {
            destination_account,
        }) => {
            close_account(account.info(), account.context(), destination_account)?;

            // Prevent executing this action twice.
            account
                .context
                .set_exit_action(account.info, FankorContextExitAction::ProcessedByZeroCopy);
        }
    }

    Ok(())
}
