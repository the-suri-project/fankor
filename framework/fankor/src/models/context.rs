use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct FankorContext<'info> {
    /// The accounts that need to be writen in the end.
    program_id: &'info Pubkey,

    /// The list of all accounts passed to the instruction.
    accounts: &'info [AccountInfo<'info>],

    /// The reference to the mutable part of the context.
    inner: Rc<RefCell<FankorContextInnerMut<'info>>>,
}

struct FankorContextInnerMut<'info> {
    // A `u64` is used as bit flag to know which account is already closed.
    closed_accounts: u64,

    // End actions to perform at the end of the instruction.
    // The key is u8 because the maximum transaction size forbids to send
    // to the instruction more than `u8::MAX` accounts.
    exit_actions: BTreeMap<u8, FankorContextExitAction<'info>>,
}

#[derive(Clone)]
/// The action to perform at the end of the instruction for a specific account.
pub enum FankorContextExitAction<'info> {
    /// Ignores the account and does nothing. This is useful to avoid writing
    /// twice an account.
    Ignore,

    /// Reallocates the account to contain all the data.
    Realloc {
        zero_bytes: bool,
        payer: Option<&'info AccountInfo<'info>>,
    },

    /// Closes the account.
    Close {
        sol_destination: &'info AccountInfo<'info>,
    },
}

impl<'info> FankorContext<'info> {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new context with the given data.
    ///
    /// # Safety
    /// The params are not not checked. If you use this method manually, it can cause
    /// undefined behaviours.
    pub unsafe fn new(
        program_id: &'info Pubkey,
        accounts: &'info [AccountInfo<'info>],
    ) -> FankorContext<'info> {
        Self {
            program_id,
            accounts,
            inner: Rc::new(RefCell::new(FankorContextInnerMut {
                closed_accounts: 0,
                exit_actions: Default::default(),
            })),
        }
    }

    // GETTERS ----------------------------------------------------------------

    pub fn program_id(&self) -> &'info Pubkey {
        self.program_id
    }

    pub fn all_accounts(&self) -> &'info [AccountInfo<'info>] {
        self.accounts
    }

    // METHODS ----------------------------------------------------------------

    pub fn get_account_from_address(&self, address: &Pubkey) -> Option<&AccountInfo<'info>> {
        self.accounts.iter().find(|account| account.key == address)
    }

    pub(crate) fn get_index_for_account(&self, account: &AccountInfo<'info>) -> u8 {
        for (i, acc) in self.accounts.iter().enumerate() {
            if acc.key == account.key {
                return i as u8;
            }
        }

        panic!("Undefined account")
    }

    pub(crate) fn is_account_closed(&self, account: &AccountInfo<'info>) -> bool {
        let index = self.get_index_for_account(account);
        let inner = (*self.inner).borrow_mut();

        (inner.closed_accounts & (1u64 << index as u64)) != 0
    }

    pub(crate) fn set_closed_account(&self, account: &AccountInfo<'info>, closed: bool) {
        let index = self.get_index_for_account(account);
        let mut inner = (*self.inner).borrow_mut();

        if closed {
            inner.closed_accounts |= 1u64 << index as u64;
        } else {
            inner.closed_accounts &= !(1u64 << index as u64);
        }
    }

    pub(crate) fn get_exit_action(
        &'info self,
        account: &AccountInfo<'info>,
    ) -> Option<FankorContextExitAction<'info>> {
        let index = self.get_index_for_account(account);
        (*self.inner).borrow().exit_actions.get(&index).cloned()
    }

    pub(crate) fn set_exit_action(
        &self,
        account: &AccountInfo<'info>,
        exit_action: FankorContextExitAction<'info>,
    ) {
        let index = self.get_index_for_account(account);
        (*self.inner)
            .borrow_mut()
            .exit_actions
            .insert(index, exit_action);
    }

    pub(crate) fn remove_exit_action(&self, account: &AccountInfo<'info>) {
        let index = self.get_index_for_account(account);
        (*self.inner).borrow_mut().exit_actions.remove(&index);
    }
}
