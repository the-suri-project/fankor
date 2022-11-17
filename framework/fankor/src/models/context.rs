use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct FankorContext<'info> {
    /// The current program id.
    program_id: &'info Pubkey,

    /// The list of all accounts passed to the instruction.
    accounts: &'info [AccountInfo<'info>],

    /// The reference to the mutable part of the context.
    inner: Rc<RefCell<FankorContextInnerMut<'info>>>,
}

struct FankorContextInnerMut<'info> {
    // End actions to perform at the end of the instruction.
    // The key is u8 because the maximum number of accounts per transaction is 256.
    exit_actions: BTreeMap<u8, FankorContextExitAction<'info>>,
}

#[derive(Clone)]
/// The action to perform at the end of the instruction for a specific account.
pub enum FankorContextExitAction<'info> {
    /// Ignores the account and does nothing. This is useful to avoid writing
    /// twice an account.
    Ignore,

    /// Reallocates the account to contain all the data and optionally makes
    /// the account rent-exempt.
    Realloc {
        zero_bytes: bool,
        payer: Option<&'info AccountInfo<'info>>,
    },

    /// Closes the account.
    Close {
        destination_account: &'info AccountInfo<'info>,
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

    /// Gets the corresponding account info for the given account key.
    pub fn get_account_from_address(&self, address: &Pubkey) -> Option<&AccountInfo<'info>> {
        self.accounts.iter().find(|account| account.key == address)
    }

    pub(crate) fn get_index_for_account(&self, account: &AccountInfo<'info>) -> u8 {
        self.accounts
            .iter()
            .position(|a| a.key == account.key)
            .expect("Undefined account") as u8
    }

    /// Whether the account is uninitialized or not, i.e. it matches all these constraints:
    /// - it does not have lamports
    /// - its data is empty
    /// - its owner is the system program
    pub fn is_account_uninitialized(&self, account: &AccountInfo<'info>) -> bool {
        account.lamports() == 0
            && account.data_is_empty()
            && account.owner == &solana_program::system_program::ID
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
