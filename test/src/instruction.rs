use crate::accounts::*;
use crate::arguments::*;
use fankor::prelude::*;
use std::cmp::Ordering;

#[derive(InstructionAccounts)]
#[instruction(args = "InstructionArgs")]
pub struct InstructionStructAccounts<'info> {
    #[account(owner = &crate::ID)]
    #[account(writable)]
    #[account(executable)]
    #[account(rent_exempt)]
    #[account(signer)]
    #[account(pda = [crate::ID.as_ref(), &self.account2.data().value1.to_le_bytes(), &args.arg2.to_le_bytes()])]
    pub account1: Account<'info, StructAccountData>,

    #[account(writable = false)]
    #[account(executable = false)]
    #[account(rent_exempt = false)]
    #[account(signer = false)]
    #[account(pda = [crate::ID.as_ref(), &self.account2.data().value1.to_le_bytes(), &args.arg2.to_le_bytes()])]
    #[account(pda_program_id = &crate::ID)]
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
pub struct InstructionStructAccountsWithoutAssociatedType<'info> {
    #[account(constraint = (1 + 1).cmp(&2) == Ordering::Equal)]
    pub account: Account<'info, StructAccountData>,
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[derive(InstructionAccounts)]
#[instruction(args = "InstructionArgs")]
pub enum InstructionEnumAccounts<'info> {
    Struct1(InstructionStructAccounts<'info>),

    Struct2(Box<InstructionStructAccounts<'info>>),

    OptionalAccount(Option<InstructionStructAccounts<'info>>),
}
