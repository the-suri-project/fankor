use std::cmp::Ordering;

use fankor::prelude::*;

use crate::accounts::*;
use crate::arguments::*;
use crate::errors::Errors;

#[instruction(initial_validation, final_validation)]
#[allow(dead_code)]
pub struct StructAccounts<'info> {
    pub args: Argument<InstructionArgs>,

    pub args_rest: RestArguments,

    #[account(owner = & crate::ID)]
    #[account(writable)]
    #[account(executable)]
    #[account(rent_exempt)]
    #[account(signer)]
    #[account(pda = [crate::ID.as_ref(), & self.account2.data().value1.to_le_bytes(), & self.args.arg2.to_le_bytes()])]
    pub account1: Account<'info, StructAccountData>,

    #[account(writable = false)]
    #[account(executable = false)]
    #[account(rent_exempt = false)]
    #[account(signer = false)]
    #[account(pda = [crate::ID.as_ref(), & self.account2.data().value1.to_le_bytes(), & self.args.arg2.to_le_bytes()])]
    #[account(pda_program_id = & Pubkey::default())]
    pub account2: Account<'info, StructAccountData>,

    #[account(pda_bytes = vec![1, 2, 3])]
    pub account3: Option<Account<'info, StructAccountData>>,

    pub unchecked_account: UncheckedAccount<'info>,

    pub zero_copy_account: ZcAccount<'info, ZeroCopyStructAccountData>,

    pub single_either: SingleEither<
        Account<'info, StructAccountData>,
        ZcAccount<'info, ZeroCopyStructAccountData>,
    >,

    #[account(address = & crate::ID)]
    pub program: Program<'info, System>,

    pub list: Vec<Account<'info, StructAccountData>>,

    pub either: Either<Account<'info, StructAccountData>, Account<'info, StructAccountData2>>,

    pub uninitialized: UninitializedAccount<'info>,

    pub maybe_uninitialized: MaybeUninitialized<'info, Account<'info, StructAccountData>>,

    pub other_struct: Box<StructAccountsWithoutAssociatedType<'info>>,

    pub other_enum: Box<EnumAccounts<'info>>,

    pub custom: StructAccountsWithoutAssociatedType<'info>,

    // Must be placed in the last position.
    #[account(writable)]
    pub rest: Rest<'info>,
}

#[allow(dead_code)]
impl<'info> StructAccounts<'info> {
    // METHODS ----------------------------------------------------------------

    pub fn processor(self, _context: FankorContext<'info>) -> FankorResult<()> {
        Ok(())
    }

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

#[instruction(initial_validation, final_validation)]
#[allow(dead_code)]
pub struct StructAccountsWithoutAssociatedType<'info> {
    pub args: Argument<InstructionArgs>,

    #[account(constraint = (1 + 1).cmp(& 2) == Ordering::Equal)]
    #[account(pda = AssociatedToken::get_pda_seeds(self.account.address(), self.boxed_zc_account.address()))]
    #[account(pda_program_id = AssociatedToken::address())]
    pub account: Account<'info, StructAccountData>,

    #[account(constraint = (1 + 1).cmp(& 2) == Ordering::Equal @ Errors::A)]
    #[account(associated_token_pda = (self.account.address(), self.boxed_zc_account.address()))]
    pub boxed_zc_account: ZcAccount<'info, ZeroCopyStructAccountData>,

    #[account(data::x = self.account.address())]
    #[account(metadata_pda = Metadata::get_metadata_pda_seeds(x) @ Errors::A)]
    pub option_zc_account: Option<ZcAccount<'info, ZeroCopyStructAccountData>>,

    pub either:
        Either<Account<'info, StructAccountData>, ZcAccount<'info, ZeroCopyStructAccountData>>,

    pub maybe_uninitialized: MaybeUninitialized<'info, ZcAccount<'info, ZeroCopyStructAccountData>>,

    pub instructions_sysvar: SysvarAccount<'info, Instructions>,
}

#[allow(dead_code)]
impl<'info> StructAccountsWithoutAssociatedType<'info> {
    // METHODS ----------------------------------------------------------------

    pub fn processor(self, _context: FankorContext<'info>) -> FankorResult<()> {
        Ok(())
    }

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

#[instruction]
pub enum EnumAccounts<'info> {
    Struct1(StructAccounts<'info>),

    Struct2(StructAccounts<'info>),

    OptionalAccount(Option<StructAccounts<'info>>),
}

#[allow(dead_code)]
impl<'info> EnumAccounts<'info> {
    // METHODS ----------------------------------------------------------------

    pub fn processor(self, _context: FankorContext<'info>) -> FankorResult<()> {
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[instruction]
pub enum EnumAccountsWithoutArgs<'info> {
    Struct1(StructAccountsWithoutAssociatedType<'info>),

    Struct2(StructAccountsWithoutAssociatedType<'info>),

    #[discriminant = 5]
    OptionalAccount(Option<StructAccountsWithoutAssociatedType<'info>>),

    EmptyVariant,
}

#[allow(dead_code)]
impl<'info> EnumAccountsWithoutArgs<'info> {
    // METHODS ----------------------------------------------------------------

    pub fn processor(self, _context: FankorContext<'info>) -> FankorResult<()> {
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[instruction]
#[allow(dead_code)]
pub struct AuxiliarInstruction<'info> {
    pub args: Argument<InstructionArgs>,
    pub account: Account<'info, StructAccountData>,
}
