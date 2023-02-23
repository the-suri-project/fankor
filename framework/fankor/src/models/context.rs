use crate::errors::{FankorErrorCode, FankorResult};
use crate::prelude::byte_seeds_to_slices;
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
    // Data for each account.
    // The key is u8 because the maximum number of accounts per transaction is 256.
    account_data: BTreeMap<u8, FankorContextAccountData<'info>>,
}

struct FankorContextAccountData<'info> {
    // End action to perform at the end of the instruction.
    exit_action: Option<FankorContextExitAction<'info>>,

    // Seeds used to derived the account.
    seeds: Option<Rc<Vec<u8>>>,
}

/// The action to perform at the end of the instruction for a specific account.
#[derive(Clone)]
pub enum FankorContextExitAction<'info> {
    /// Indicates the account has already processed the exit action.
    /// It is used to detect duplicated actions.
    Processed,

    /// Indicates the account has already processed the exit action by zero copy.
    /// It is used to detect duplicated actions.
    ProcessedByZeroCopy,

    /// Reallocates the account to contain all the data and optionally makes
    /// the account rent-exempt.
    Realloc {
        zero_bytes: bool,
        payer: Option<&'info AccountInfo<'info>>,
        system_program: &'info AccountInfo<'info>,
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
    pub fn new_unchecked(
        program_id: &'info Pubkey,
        accounts: &'info [AccountInfo<'info>],
    ) -> FankorContext<'info> {
        Self {
            program_id,
            accounts,
            inner: Rc::new(RefCell::new(FankorContextInnerMut {
                account_data: Default::default(),
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

    /// Gets the corresponding seeds for an account if it was previously computed.
    pub fn get_seeds_for_account(&self, account: &AccountInfo<'info>) -> Option<Rc<Vec<u8>>> {
        let index = self.get_index_for_account(account);
        self.inner
            .borrow()
            .account_data
            .get(&index)
            .and_then(|v| v.seeds.clone())
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
        (*self.inner)
            .borrow()
            .account_data
            .get(&index)
            .and_then(|v| v.exit_action.clone())
    }

    pub(crate) fn set_exit_action(
        &self,
        account: &AccountInfo<'info>,
        exit_action: FankorContextExitAction<'info>,
    ) {
        let index = self.get_index_for_account(account);
        let mut inner = (*self.inner).borrow_mut();

        match inner.account_data.get_mut(&index) {
            Some(v) => v.exit_action = Some(exit_action),
            None => {
                inner.account_data.insert(
                    index,
                    FankorContextAccountData {
                        exit_action: Some(exit_action),
                        seeds: None,
                    },
                );
            }
        }
    }

    pub(crate) fn remove_exit_action(&self, account: &AccountInfo<'info>) {
        let index = self.get_index_for_account(account);
        let mut inner = (*self.inner).borrow_mut();

        if let Some(v) = inner.account_data.get_mut(&index) {
            v.exit_action = None;
        }
    }

    /// Sets the seeds associated with an account.
    ///
    /// # Safety
    /// This method is intended to be used only by the framework.
    pub fn set_seeds_for_account_unchecked(
        &self,
        account: &AccountInfo<'info>,
        seeds: Rc<Vec<u8>>,
    ) {
        let index = self.get_index_for_account(account);
        let mut inner = (*self.inner).borrow_mut();

        match inner.account_data.get_mut(&index) {
            Some(v) => v.seeds = Some(seeds),
            None => {
                inner.account_data.insert(
                    index,
                    FankorContextAccountData {
                        exit_action: None,
                        seeds: Some(seeds),
                    },
                );
            }
        }
    }

    /// Checks whether the given account is a canonical PDA with the given seeds.
    ///
    /// Note: the first time this method is called, it will save the generated bump seed
    /// in the context. If you call this method again with other seeds,
    /// it will return an error because it won't compute the same bump seed.
    pub fn check_canonical_pda(
        &self,
        account: &AccountInfo<'info>,
        seeds: Vec<u8>,
    ) -> FankorResult<()> {
        self.check_canonical_pda_with_program(account, seeds, self.program_id)
    }

    /// Checks whether the given account is a canonical PDA with the given seeds and program_id.
    pub fn check_canonical_pda_with_program(
        &self,
        account: &AccountInfo<'info>,
        mut seeds: Vec<u8>,
        program_id: &Pubkey,
    ) -> FankorResult<()> {
        let index = self.get_index_for_account(account);
        let saved_seeds = self
            .inner
            .borrow()
            .account_data
            .get(&index)
            .and_then(|v| v.seeds.clone());

        if let Some(saved_seeds) = saved_seeds {
            if *saved_seeds == seeds {
                return Ok(());
            }
        }

        let compute_seeds = byte_seeds_to_slices(&seeds);

        let (expected_address, bump_seed) =
            Pubkey::find_program_address(&compute_seeds, program_id);

        if expected_address != *account.key {
            return Err(FankorErrorCode::InvalidPda {
                expected: expected_address,
                actual: *account.key,
            }
            .into());
        }

        // Add the seeds to the context.
        seeds.push(bump_seed);

        let mut inner = (*self.inner).borrow_mut();
        match inner.account_data.get_mut(&index) {
            Some(v) => v.seeds = Some(Rc::new(seeds)),
            None => {
                inner.account_data.insert(
                    index,
                    FankorContextAccountData {
                        exit_action: None,
                        seeds: Some(Rc::new(seeds)),
                    },
                );
            }
        }

        Ok(())
    }
}
