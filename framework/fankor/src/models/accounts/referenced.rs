use crate::errors::{Error, FankorErrorCode, FankorResult};
use crate::models::{FankorContext, FankorContextExitAction, Program, System};
use crate::traits::{
    AccountInfoVerification, AccountSize, AccountType, InstructionAccount, PdaChecker,
};
use crate::utils::bpf_writer::BpfWriter;
use crate::utils::close::close_account;
use crate::utils::realloc::realloc_account_to_size;
use crate::utils::rent::make_rent_exempt;
use solana_program::account_info::AccountInfo;
use solana_program::clock::Epoch;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::system_program;
use solana_program::sysvar::Sysvar;
use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::io::Write;
use std::rc::Rc;

/// An initialized account whose value is inside a reference to avoid duplicates.
pub struct RefAccount<'info, T: AccountType + 'static> {
    context: &'info FankorContext<'info>,
    info: &'info AccountInfo<'info>,
    data: Rc<RefCell<T>>,
    dropped: bool,
}

impl<'info, T: AccountType + 'static> RefAccount<'info, T> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new account with the given data.
    pub fn new<F>(
        context: &'info FankorContext<'info>,
        info: &'info AccountInfo<'info>,
        default: F,
    ) -> FankorResult<RefAccount<'info, T>>
    where
        F: FnOnce() -> T,
    {
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

        let data = match context.get_deserialized_data_for_account(info) {
            Some(data) => {
                {
                    let data = &*data.borrow();

                    if !data.is::<T>() {
                        return Err(FankorErrorCode::DuplicatedAccountWithDifferentType {
                            address: *info.key,
                        }
                        .into());
                    }
                }

                unsafe { Rc::from_raw(Rc::into_raw(data) as *const RefCell<T>) }
            }
            None => {
                let value = default();
                let value = Rc::new(RefCell::new(value));

                context.set_deserialized_data_for_account(info, value.clone());

                value
            }
        };

        Ok(RefAccount {
            context,
            info,
            data,
            dropped: false,
        })
    }

    pub(crate) fn new_without_checks(
        context: &'info FankorContext<'info>,
        info: &'info AccountInfo<'info>,
        data: Rc<RefCell<T>>,
    ) -> RefAccount<'info, T> {
        RefAccount {
            context,
            info,
            data,
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
    pub fn rent_epoch(&self) -> Epoch {
        self.info.rent_epoch
    }

    #[inline(always)]
    pub fn info(&self) -> &'info AccountInfo<'info> {
        self.info
    }

    #[inline(always)]
    pub fn data(&self) -> Ref<T> {
        self.data.borrow()
    }

    #[inline(always)]
    pub fn data_mut(&self) -> RefMut<T> {
        RefCell::borrow_mut(&self.data)
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

    /// Reloads the account from storage. This is useful, for example, when
    /// observing side effects after CPI.
    pub fn reload(&mut self) -> FankorResult<()> {
        let result = {
            let info = self.info();
            let mut data: &[u8] = &info.try_borrow_data()?;
            T::try_deserialize(&mut data)?
        };

        let mut data = RefCell::borrow_mut(&self.data);
        *data = result;

        Ok(())
    }

    /// Saves the account changes into the storage. This is useful, for example,
    /// to expose new content before a CPI.
    pub fn save(&self) -> FankorResult<()> {
        if !self.is_owned_by_program() {
            return Err(FankorErrorCode::AccountNotOwnedByProgram {
                address: *self.address(),
                action: "write",
            }
            .into());
        }

        if !self.is_writable() {
            return Err(FankorErrorCode::ReadonlyAccountModification {
                address: *self.address(),
                action: "write",
            }
            .into());
        }

        if self.context.is_account_uninitialized(self.info) {
            return Err(FankorErrorCode::AlreadyClosedAccount {
                address: *self.address(),
                action: "write",
            }
            .into());
        }

        let mut data = self.info.try_borrow_mut_data()?;
        let dst: &mut [u8] = &mut data;
        let mut writer = BpfWriter::new(dst);
        self.data.borrow().try_serialize(&mut writer)?;

        Ok(())
    }

    /// Closes the account and sends the lamports to the `destination_account`.
    pub fn close(mut self, destination_account: &AccountInfo<'info>) -> FankorResult<()> {
        close_account(self.info, self.context(), destination_account)?;

        // Prevent account to execute the drop actions.
        self.dropped = true;
        Ok(())
    }

    /// Reallocates the account to the given `size`. If a `payer` is provided,
    /// fankor will add funds to the account to make it rent-exempt.
    ///
    /// # Safety
    ///
    /// This method is unsafe because the provided `size` can be less than what
    /// the actual data needs to be writen into the account.
    pub unsafe fn realloc(
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

    /// Invalidates the exit action for this account.
    pub fn remove_exit_action(&self) {
        self.context().remove_exit_action(self.info);
    }

    /// Reallocates the account at the end of the instruction if the encoded data
    /// exceeds the maximum the account can contain. If a `payer` is provided,
    /// fankor will add funds to the account to make it rent-exempt.
    ///
    /// This replaces other exit actions associated with this account.
    pub fn realloc_at_exit(
        &self,
        zero_bytes: bool,
        payer: Option<&'info AccountInfo<'info>>,
        system_program: &Program<'info, System>,
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

        self.context().set_exit_action(
            self.info,
            FankorContextExitAction::Realloc {
                payer,
                zero_bytes,
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

impl<'info, T: AccountType + AccountSize + 'static> RefAccount<'info, T> {
    // GETTERS ----------------------------------------------------------------

    /// Whether the account has enough lamports to be rent-exempt or not.
    ///
    /// This is the same as [`is_rent_exempt`] but gets the byte size from the value instead
    /// of the written bytes in the account. Useful to check if the new value fits in the existing
    /// space.
    pub fn is_value_rent_exempt(&self) -> bool {
        let info = self.info();
        let lamports = info.lamports();
        let data_len = self.data.borrow().actual_account_size();

        let rent = Rent::get().expect("Cannot access Rent Sysvar");

        rent.is_exempt(lamports, data_len)
    }

    // METHODS ----------------------------------------------------------------

    /// Reallocates the account to the actual account `data` size plus the discriminant
    /// length. If a `payer` is provided, fankor will add funds to the account to make it
    /// rent-exempt.
    pub fn realloc_to_contain_data(
        &self,
        zero_bytes: bool,
        payer: Option<&'info AccountInfo<'info>>,
        system_program: &Program<System>,
    ) -> FankorResult<()> {
        unsafe {
            self.realloc(
                self.data.borrow().actual_account_size() + 1,
                zero_bytes,
                payer,
                system_program,
            )
        }
    }

    /// Makes the account rent-exempt by adding or removing funds from/to `payer`
    /// if necessary. The size to calculate the rent is the actual account `data` size
    /// plus the discriminant.
    pub fn make_rent_exempt_to_contain_data(
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

        let new_size = self.data.borrow().actual_account_size() + 1;
        make_rent_exempt(new_size, payer, self.info, system_program)
    }
}

impl<'info, T: AccountType + 'static> InstructionAccount<'info> for RefAccount<'info, T> {
    type CPI = AccountInfo<'info>;
    type LPI = Pubkey;

    #[inline(always)]
    fn min_accounts() -> usize {
        1
    }

    fn verify_account_infos<'a>(
        &self,
        config: &mut AccountInfoVerification<'a, 'info>,
    ) -> FankorResult<()> {
        config.verify(self.info)
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

        let data = match context.get_deserialized_data_for_account(info) {
            Some(data) => {
                {
                    let data = &*data.borrow();

                    if !data.is::<T>() {
                        return Err(FankorErrorCode::DuplicatedAccountWithDifferentType {
                            address: *info.key,
                        }
                        .into());
                    }
                }

                unsafe { Rc::from_raw(Rc::into_raw(data) as *const RefCell<T>) }
            }
            None => {
                let mut data: &[u8] = &info.try_borrow_data()?;
                let value = T::try_deserialize(&mut data)?;
                let value = Rc::new(RefCell::new(value));

                context.set_deserialized_data_for_account(info, value.clone());

                value
            }
        };

        let result = RefAccount::new_without_checks(context, info, data);

        *accounts = &accounts[1..];
        Ok(result)
    }
}

impl<'info, T: AccountType> PdaChecker<'info> for RefAccount<'info, T> {
    fn pda_info(&self) -> Option<&'info AccountInfo<'info>> {
        Some(self.info)
    }
}

impl<'info, T: AccountType> Debug for RefAccount<'info, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Account").field("info", &self.info).finish()
    }
}

/// Execute the last actions over the account.
impl<'info, T: AccountType + 'static> Drop for RefAccount<'info, T> {
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

fn drop_aux<T: AccountType + 'static>(account: &mut RefAccount<T>) -> FankorResult<()> {
    let rc_count = Rc::strong_count(&account.data);

    // Ignore if not the last time.
    // 2 because of the reference at the context and at the current account.
    if rc_count > 2 {
        return Ok(());
    }

    match account.context.get_exit_action(account.info) {
        None => {
            // Ignore if not writable or non from current program.
            if account.is_writable() && account.is_owned_by_program() {
                // Write the data.
                account.save()?;

                // Prevent executing this action twice.
                account
                    .context
                    .set_exit_action(account.info, FankorContextExitAction::Processed);
            }
        }
        Some(FankorContextExitAction::Processed) => {
            return Err(FankorErrorCode::DuplicatedWritableAccounts {
                address: *account.address(),
            }
            .into());
        }
        Some(FankorContextExitAction::Realloc {
            zero_bytes,
            payer,
            system_program,
        }) => {
            // Serialize.
            let mut serialized = Vec::with_capacity(account.info.data_len());
            account.data.borrow().try_serialize(&mut serialized)?;

            // Reallocate.
            unsafe {
                account.realloc(
                    serialized.len(),
                    zero_bytes,
                    payer,
                    &Program::new(account.context(), system_program)?,
                )?;
            }

            // Write data.
            let mut data = account.info.try_borrow_mut_data()?;
            let dst: &mut [u8] = &mut data;
            let mut writer = BpfWriter::new(dst);
            writer.write_all(&serialized)?;

            // Prevent executing this action twice.
            account
                .context
                .set_exit_action(account.info, FankorContextExitAction::Processed);
        }
        Some(FankorContextExitAction::Close {
            destination_account,
        }) => {
            close_account(account.info(), account.context(), destination_account)?;

            // Prevent executing this action twice.
            account
                .context
                .set_exit_action(account.info, FankorContextExitAction::Processed);
        }
    }

    Ok(())
}
