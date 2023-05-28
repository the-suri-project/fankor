use fankor::prelude::*;

use crate::instruction::*;

#[program(testable, fallback)]
enum TestProgram {
    #[discriminant = 3]
    StructAccounts,

    #[deprecated]
    StructAccountsWithoutAssociatedType,

    #[discriminant = 5]
    #[return_type = u8]
    EnumAccounts,

    #[return_type = u8]
    #[boxed]
    EnumAccountsWithoutArgs,
}

#[allow(dead_code)]
fn fallback<'info>(
    _program_id: &'info Pubkey,
    accounts: &'info [AccountInfo],
    _instruction_data: &'info [u8],
) -> FankorResult<()> {
    msg!("fallback instruction");

    require!(accounts.len() == 1, crate::errors::Errors::A);
    require_not!(accounts.len() == 1, crate::errors::Errors::A);

    Ok(())
}
