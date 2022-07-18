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

    pub optional_account: OptionalAccount<'info, AccountData>,

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

    #[account(max = 5)]
    pub list5: Vec<Account<'info, AccountData>>,

    pub either: Either<Account<'info, AccountData>, Account<'info, EnumAccountData>>,

    pub uninitialized: UninitializedAccount<'info, AccountData>,

    pub maybe_uninitialized: MaybeUninitializedAccount<'info, AccountData>,

    #[account(writable)]
    pub other_struct: Box<InstructionStructAccounts<'info>>,

    #[account(writable)]
    pub other_enum: Box<InstructionEnumAccounts<'info>>,

    // Must be placed in the last position.
    #[account(min = 2)]
    #[account(writable)]
    pub rest: Rest<'info>,
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[derive(InstructionAccounts)]
pub enum InstructionEnumAccounts<'info> {
    #[account(owner = &crate::ID)]
    #[account(writable)]
    #[account(executable)]
    #[account(rent_exempt)]
    #[account(signer)]
    Account1(Account<'info, AccountData>),

    #[account(writable = false)]
    #[account(executable = false)]
    #[account(rent_exempt = false)]
    #[account(signer = false)]
    Account2(Box<Account<'info, AccountData>>),

    Account3(Option<Account<'info, AccountData>>),

    OptionalAccount(OptionalAccount<'info, AccountData>),

    UncheckedAccount(UncheckedAccount<'info>),

    #[account(address = &crate::ID)]
    Program(Program<'info, System>),

    List(Vec<Account<'info, AccountData>>),

    #[account(min = 2)]
    List2(Vec<Account<'info, AccountData>>),

    #[account(min = 2)]
    #[account(max = 5)]
    List3(Vec<Account<'info, AccountData>>),

    #[account(size = 15)]
    List4(Vec<Account<'info, AccountData>>),

    #[account(max = 15)]
    List5(Vec<Account<'info, AccountData>>),

    Either(Either<Account<'info, AccountData>, Account<'info, EnumAccountData>>),

    Uninitialized(UninitializedAccount<'info, AccountData>),

    MaybeUninitialized(MaybeUninitializedAccount<'info, AccountData>),

    #[account(writable)]
    OtherStruct(Box<InstructionStructAccounts<'info>>),

    #[account(writable)]
    OtherEnum(Box<InstructionEnumAccounts<'info>>),

    // Must be placed in the last position.
    #[account(min = 2)]
    #[account(writable)]
    Rest(Rest<'info>),
}
