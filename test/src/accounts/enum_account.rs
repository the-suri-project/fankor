use crate::accounts::{ProgramAccount, ProgramAccountDiscriminant};
use fankor::prelude::*;

#[account(base = "ProgramAccount")]
#[derive(AccountSize, AccountOffsets)]
pub enum EnumAccountData {
    #[discriminant = 3]
    A ,
    B(u32, u64),
    
    #[discriminant = 5]
    C {
        value1: u32,
        value2: String,
    } ,
}

#[account(base = "ProgramAccount")]
#[derive(AccountSize, AccountOffsets, FankorZeroCopy)]
pub enum ZeroCopyEnumAccountData {
    A,
    B(u32, u64),
    C { value1: u32, value2: String },
}

#[derive(EnumDiscriminants, FankorZeroCopy)]
pub enum ZeroCopyEnumWithoutValues {
    A,
    B,
    C,
}
