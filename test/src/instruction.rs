use crate::accounts::*;
use crate::arguments::*;
use crate::errors::Errors;
use fankor::prelude::*;
use std::cmp::Ordering;

#[derive(InstructionAccounts)]
#[account(args = InstructionArgs)]
#[account(initial_validation)]
#[account(final_validation)]
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

    pub optional_account: MaybeDefaultAccount<'info, Account<'info, StructAccountData>>,

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

#[allow(dead_code)]
impl<'info> InstructionStructAccounts<'info> {
    // METHODS ----------------------------------------------------------------

    pub fn initial_validation(
        &self,
        _context: &FankorContext<'info>,
        _args: &InstructionArgs,
    ) -> FankorResult<()> {
        Ok(())
    }

    pub fn final_validation(
        &self,
        _context: &FankorContext<'info>,
        _args: &InstructionArgs,
    ) -> FankorResult<()> {
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[derive(InstructionAccounts)]
#[account(initial_validation)]
#[account(final_validation)]
pub struct InstructionStructAccountsWithoutAssociatedType<'info> {
    #[account(constraint = (1 + 1).cmp(&2) == Ordering::Equal)]
    #[account(pda = AssociatedToken::get_pda_seeds(self.account.address(), self.boxed_zc_account.address()))]
    #[account(pda_program_id = AssociatedToken::address())]
    pub account: Account<'info, StructAccountData>,

    #[account(constraint = (1 + 1).cmp(&2) == Ordering::Equal @ Errors::A)]
    #[account(associated_token_pda = (self.account.address(), self.boxed_zc_account.address()))]
    pub boxed_zc_account: Box<ZcAccount<'info, ZeroCopyStructAccountData>>,

    #[account(data::x = self.account.address())]
    #[account(metadata_pda = Metadata::get_metadata_pda_seeds(x) @ Errors::A)]
    pub optional_zc_account:
        MaybeDefaultAccount<'info, ZcAccount<'info, ZeroCopyStructAccountData>>,

    pub option_zc_account: Option<ZcAccount<'info, ZeroCopyStructAccountData>>,

    pub either:
        Either<Account<'info, StructAccountData>, ZcAccount<'info, ZeroCopyStructAccountData>>,

    pub maybe_uninitialized: MaybeUninitializedZcAccount<'info, ZeroCopyStructAccountData>,

    pub instructions_sysvar: SysvarAccount<'info, Instructions>,
}

#[allow(dead_code)]
impl<'info> InstructionStructAccountsWithoutAssociatedType<'info> {
    // METHODS ----------------------------------------------------------------

    pub fn initial_validation(&self, _context: &FankorContext<'info>) -> FankorResult<()> {
        Ok(())
    }

    pub fn final_validation(&self, _context: &FankorContext<'info>) -> FankorResult<()> {
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[derive(InstructionAccounts)]
#[account(args = InstructionArgs)]
#[non_exhaustive]
pub enum InstructionEnumAccounts<'info> {
    Struct1(InstructionStructAccounts<'info>),

    Struct2(Box<InstructionStructAccounts<'info>>),

    OptionalAccount(Option<InstructionStructAccounts<'info>>),
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[derive(InstructionAccounts)]
#[non_exhaustive]
pub enum InstructionEnumAccountsWithoutArgs<'info> {
    Struct1(InstructionStructAccountsWithoutAssociatedType<'info>),

    Struct2(Box<InstructionStructAccountsWithoutAssociatedType<'info>>),

    OptionalAccount(Option<InstructionStructAccountsWithoutAssociatedType<'info>>),
}
