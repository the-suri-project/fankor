use crate::accounts::{ProgramAccount, ProgramAccountDiscriminant};
use fankor::prelude::*;

#[account(base = "ProgramAccount")]
#[derive(AccountSize, AccountOffsets)]
pub enum EnumAccountData {
    A = 5,
    #[deprecated]
    B(u32, u64),
    C {
        value1: u32,
        value2: String,
    } = 3,
}

#[account(base = "ProgramAccount")]
#[derive(AccountSize, AccountOffsets, FankorZeroCopy)]
pub enum ZeroCopyEnumAccountData {
    A,
    B(u32, u64),
    C { value1: u32, value2: String },
}

#[derive(FankorZeroCopy)]
pub enum ZeroCopyEnumWithoutValues {
    A,
    B,
    C,
}
