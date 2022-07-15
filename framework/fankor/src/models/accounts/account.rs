use crate::errors::{Error, ErrorCode, FankorResult};
use crate::models::{FankorContext, FankorContextExitAction};
use crate::traits::{AccountSize, InstructionAccount};
use crate::utils::bpf_writer::BpfWriter;
use crate::utils::close::close_account;
use crate::utils::realloc::realloc_account_to_size;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::system_program;
use solana_program::sysvar::Sysvar;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::io::Write;
use std::mem::ManuallyDrop;

pub struct Account<'info, T: Debug + crate::traits::Account> {
    context: &'info FankorContext<'info>,
    info: &'info AccountInfo<'info>,
    data: ManuallyDrop<T>,
}

impl<'info, T: Debug + crate::traits::Account> Account<'info, T> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new account with the given data.
    pub fn new(
        context: &'info FankorContext<'info>,
        info: &'info AccountInfo<'info>,
        data: T,
    ) -> FankorResult<Account<'info, T>> {
        if info.owner == &system_program::ID && info.lamports() == 0 {
            return Err(ErrorCode::AccountNotInitialized { address: *info.key }.into());
        }

        if info.owner != T::owner() {
            return Err(ErrorCode::AccountOwnedByWrongProgram {
                address: *info.key,
                expected: *T::owner(),
                actual: *info.owner,
            }
            .into());
        }

        // Check it is not closed.
        if context.is_account_closed(info) {
            return Err(ErrorCode::NewFromClosedAccount { address: *info.key }.into());
        }

        Ok(Account {
            context,
            info,
            data: ManuallyDrop::new(data),
        })
    }

    pub(crate) fn new_without_checks(
        context: &'info FankorContext<'info>,
        info: &'info AccountInfo<'info>,
        data: T,
    ) -> Account<'info, T> {
        Account {
            context,
            info,
            data: ManuallyDrop::new(data),
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
    pub fn data(&self) -> &T {
        &self.data
    }

    #[inline(always)]
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
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

    /// This gets the inner data of the account removing any exit action.
    pub fn unwrap_inner(mut self) -> T {
        self.context
            .set_exit_action(self.info, FankorContextExitAction::Ignore);

        unsafe {
            // Safety: the Drop impl won't use this field because of the exit action.
            ManuallyDrop::take(&mut self.data)
        }
    }

    /// Reloads the account from storage. This is useful, for example, when
    /// observing side effects after CPI.
    pub fn reload(&mut self) -> FankorResult<()> {
        let result = {
            let info = self.info();
            let mut data: &[u8] = &info.try_borrow_data()?;
            T::try_deserialize(&mut data)?
        };
        *self.data = result;

        Ok(())
    }

    /// Saves the account changes into the storage. This is useful, for example,
    /// to expose new content before a CPI.
    pub fn save(&self) -> FankorResult<()> {
        if !self.is_owned_by_program() {
            return Err(ErrorCode::AccountNotOwnedByProgram {
                address: *self.address(),
                action: "write",
            }
            .into());
        }

        if !self.is_writable() {
            return Err(ErrorCode::ReadonlyAccountModification {
                address: *self.address(),
                action: "write",
            }
            .into());
        }

        if self.context.is_account_closed(self.info) {
            return Err(ErrorCode::AlreadyClosedAccount {
                address: *self.address(),
                action: "write",
            }
            .into());
        }

        let mut data = self.info.try_borrow_mut_data()?;
        let dst: &mut [u8] = &mut data;
        let mut writer = BpfWriter::new(dst);
        self.data.try_serialize(&mut writer)?;

        Ok(())
    }

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
        if !self.is_owned_by_program() {
            return Err(ErrorCode::AccountNotOwnedByProgram {
                address: *self.address(),
                action: "reallocate",
            }
            .into());
        }

        if !self.is_writable() {
            return Err(ErrorCode::ReadonlyAccountModification {
                address: *self.address(),
                action: "reallocate",
            }
            .into());
        }

        if self.context.is_account_closed(self.info) {
            return Err(ErrorCode::AlreadyClosedAccount {
                address: *self.address(),
                action: "reallocate",
            }
            .into());
        }

        realloc_account_to_size(self.info, size, zero_bytes, payer)
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

    /// Reallocates the account at the end of the instruction if the encoded data
    /// exceeds the maximum the account can contain. If a `payer` is provided, the
    /// fankor will add funds to the account to make it rent-exempt.
    ///
    /// This replaces other exit actions associated with this account.
    pub fn realloc_at_exit(
        &self,
        payer: Option<&'info AccountInfo<'info>>,
        zero_bytes: bool,
    ) -> FankorResult<()> {
        if !self.is_owned_by_program() {
            return Err(ErrorCode::AccountNotOwnedByProgram {
                address: *self.address(),
                action: "reallocate",
            }
            .into());
        }

        if !self.is_writable() {
            return Err(ErrorCode::ReadonlyAccountModification {
                address: *self.address(),
                action: "reallocate",
            }
            .into());
        }

        if self.context.is_account_closed(self.info) {
            return Err(ErrorCode::AlreadyClosedAccount {
                address: *self.address(),
                action: "reallocate",
            }
            .into());
        }

        self.context().set_exit_action(
            self.info,
            FankorContextExitAction::Realloc { payer, zero_bytes },
        );

        Ok(())
    }

    /// Closes the account at the end of the instruction sending the lamports to
    /// the `sol_destination` account.
    ///
    /// This replaces other exit actions associated with this account.
    pub fn close_account_at_exit(
        &self,
        sol_destination: &'info AccountInfo<'info>,
    ) -> FankorResult<()> {
        if !self.is_owned_by_program() {
            return Err(ErrorCode::AccountNotOwnedByProgram {
                address: *self.address(),
                action: "close",
            }
            .into());
        }

        if !self.is_writable() {
            return Err(ErrorCode::ReadonlyAccountModification {
                address: *self.address(),
                action: "close",
            }
            .into());
        }

        let context = self.context();
        context.set_exit_action(
            self.info,
            FankorContextExitAction::Close { sol_destination },
        );

        Ok(())
    }
}

impl<'info, T: Debug + crate::traits::Account + AccountSize> Account<'info, T> {
    // METHODS ----------------------------------------------------------------

    /// Reallocates the account to the actual account `data` size plus the discriminator
    /// length. If a `payer` is provided, fankor will add funds to the account to make it
    /// rent-exempt.
    pub fn realloc_to_contain_data(
        &self,
        zero_bytes: bool,
        payer: Option<&'info AccountInfo<'info>>,
    ) -> FankorResult<()> {
        self.realloc(
            self.data.actual_account_size() + self.context().discriminator_length() as usize,
            zero_bytes,
            payer,
        )
    }
}

impl<'info, T: Debug + crate::traits::Account> InstructionAccount<'info> for Account<'info, T> {
    #[inline(never)]
    fn try_from(
        context: &'info FankorContext<'info>,
        accounts: &mut &'info [AccountInfo<'info>],
    ) -> FankorResult<Self> {
        if accounts.is_empty() {
            return Err(ErrorCode::NotEnoughAccountKeys.into());
        }

        let info = &accounts[0];
        if info.owner == &system_program::ID && info.lamports() == 0 {
            return Err(ErrorCode::AccountNotInitialized { address: *info.key }.into());
        }

        if info.owner != T::owner() {
            return Err(ErrorCode::AccountOwnedByWrongProgram {
                address: *info.key,
                expected: *T::owner(),
                actual: *info.owner,
            }
            .into());
        }

        let mut data: &[u8] = &info.try_borrow_data()?;
        Ok(Account::new_without_checks(
            context,
            info,
            T::try_deserialize(&mut data)?,
        ))
    }
}

impl<'info, T: Debug + crate::traits::Account> Debug for Account<'info, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Account")
            .field("data", &self.data)
            .field("info", &self.info)
            .finish()
    }
}

/// Execute the last actions over the account.
impl<'info, T: Debug + crate::traits::Account> Drop for Account<'info, T> {
    fn drop(&mut self) {
        if let Err(e) = drop_aux(self) {
            crate::macros::panic_error!(e);
        }
    }
}

fn drop_aux<T: Debug + crate::traits::Account>(account: &mut Account<T>) -> FankorResult<()> {
    // Ignore if not owned by program.
    if !account.is_owned_by_program() {
        return Ok(());
    }

    match account.context.get_exit_action(account.info) {
        None => {
            // Ignore if not writable.
            if account.is_writable() {
                account.save()?;
            }
        }
        Some(FankorContextExitAction::Ignore) => {}
        Some(FankorContextExitAction::Realloc { zero_bytes, payer }) => {
            if !account.is_writable() {
                return Err(ErrorCode::ReadonlyAccountModification {
                    address: *account.address(),
                    action: "reallocate",
                }
                .into());
            }

            if account.context.is_account_closed(account.info) {
                return Err(ErrorCode::AlreadyClosedAccount {
                    address: *account.address(),
                    action: "reallocate",
                }
                .into());
            }

            // Serialize.
            let mut serialized = Vec::with_capacity(account.info.data_len());
            account.data.try_serialize(&mut serialized)?;

            // Reallocate.
            account.realloc(serialized.len(), zero_bytes, payer)?;

            // Write data.
            let mut data = account.info.try_borrow_mut_data()?;
            let dst: &mut [u8] = &mut data;
            let mut writer = BpfWriter::new(dst);
            writer.write_all(&serialized)?;
        }
        Some(FankorContextExitAction::Close { sol_destination }) => {
            close_account(account.info(), account.context(), sol_destination)?;
        }
    }

    Ok(())
}
