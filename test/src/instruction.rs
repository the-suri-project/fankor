use crate::accounts::{AccountData, EnumAccountData};
use fankor::prelude::*;

#[derive(InstructionAccounts)]
pub struct InstructionStructAccounts<'info> {
    #[account(owner = &crate::ID)]
    #[account(writable)]
    #[account(executable)]
    #[account(rent_exempt)]
    #[account(signer)]
    pub account1: Account<'info, AccountData>,

    #[account(writable = false)]
    #[account(executable = false)]
    #[account(rent_exempt = false)]
    #[account(signer = false)]
    pub account2: Box<Account<'info, AccountData>>,

    pub account3: Option<Account<'info, AccountData>>,

    pub unchecked_account: UncheckedAccount<'info>,

    #[account(address = &crate::ID)]
    pub program: Program<'info, System>,

    pub list: Vec<Account<'info, AccountData>>,

    #[account(min = 2)]
    pub list2: Vec<Account<'info, AccountData>>,

    #[account(min = 2)]
    #[account(max = 5)]
    pub list3: Vec<Account<'info, AccountData>>,

    #[account(size = 15)]
    pub list4: Vec<Account<'info, AccountData>>,

    pub either: Either<Account<'info, AccountData>, Account<'info, EnumAccountData>>,

    pub uninitialized: UninitializedAccount<'info, AccountData>,

    pub maybe_uninitialized: MaybeUninitializedAccount<'info, AccountData>,

    // Must be placed in the last position.
    #[account(writable)]
    pub rest: Rest<'info>,
}
