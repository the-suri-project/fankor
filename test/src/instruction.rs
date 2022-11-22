use crate::accounts::{
    EnumAccountData, StructAccountData, ZeroCopyEnumAccountData, ZeroCopyStructAccountData,
};
use fankor::prelude::*;

#[derive(InstructionAccounts)]
pub struct InstructionStructAccounts<'info> {
    #[account(owner = &crate::ID)]
    #[account(writable)]
    #[account(executable)]
    #[account(rent_exempt)]
    #[account(signer)]
    pub account1: Account<'info, StructAccountData>,

    #[account(writable = false)]
    #[account(executable = false)]
    #[account(rent_exempt = false)]
    #[account(signer = false)]
    pub account2: Box<Account<'info, StructAccountData>>,

    pub account3: Option<Account<'info, StructAccountData>>,

    pub optional_account: OptionalAccount<'info, StructAccountData>,

    pub unchecked_account: UncheckedAccount<'info>,

    pub zero_copy_account: ZcAccount<'info, ZeroCopyStructAccountData>,

    pub zero_copy_enum_account: ZcAccount<'info, ZeroCopyEnumAccountData>,

    #[account(address = &crate::ID)]
    pub program: Program<'info, System>,

    pub list: Vec<Account<'info, StructAccountData>>,

    #[account(min = 2)]
    pub list2: Vec<Account<'info, StructAccountData>>,

    #[account(min = 2)]
    #[account(max = 5)]
    pub list3: Vec<Account<'info, StructAccountData>>,

    #[account(size = 15)]
    pub list4: Vec<Account<'info, StructAccountData>>,

    #[account(max = 5)]
    pub list5: Vec<Account<'info, StructAccountData>>,

    pub either: Either<Account<'info, StructAccountData>, Account<'info, EnumAccountData>>,

    pub uninitialized: UninitializedAccount<'info, StructAccountData>,

    pub maybe_uninitialized: MaybeUninitializedAccount<'info, StructAccountData>,

    #[account(writable)]
    pub other_struct: Box<InstructionStructAccounts<'info>>,

    #[account(writable)]
    pub other_enum: Box<InstructionEnumAccounts<'info>>,

    // Must be placed in the last position.
    #[account(min = 2)]
    #[account(max = 5)]
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
    Account1(Account<'info, StructAccountData>),

    #[account(writable = false)]
    #[account(executable = false)]
    #[account(rent_exempt = false)]
    #[account(signer = false)]
    Account2(Box<Account<'info, StructAccountData>>),

    // Do not use `Optional` in enums, it invalidates the next variants.
    // Account3(Option<Account<'info, StructAccountData>>),
    OptionalAccount(OptionalAccount<'info, StructAccountData>),

    UncheckedAccount(UncheckedAccount<'info>),

    #[account(address = &crate::ID)]
    Program(Program<'info, System>),

    List(Vec<Account<'info, StructAccountData>>),

    #[account(min = 2)]
    List2(Vec<Account<'info, StructAccountData>>),

    #[account(min = 2)]
    #[account(max = 5)]
    List3(Vec<Account<'info, StructAccountData>>),

    #[account(size = 15)]
    List4(Vec<Account<'info, StructAccountData>>),

    #[account(max = 15)]
    #[account(min_accounts = 10)]
    List5(Vec<Account<'info, StructAccountData>>),

    Either(Either<Account<'info, StructAccountData>, Account<'info, EnumAccountData>>),

    Uninitialized(UninitializedAccount<'info, StructAccountData>),

    MaybeUninitialized(MaybeUninitializedAccount<'info, StructAccountData>),

    #[account(writable)]
    OtherStruct(Box<InstructionStructAccounts<'info>>),

    #[account(writable)]
    #[account(min_accounts = 10)]
    OtherEnum(Box<InstructionEnumAccounts<'info>>),

    // Must be placed in the last position.
    #[account(min = 2)]
    #[account(max = 5)]
    #[account(writable)]
    Rest(Rest<'info>),
}
