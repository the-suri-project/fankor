use crate::accounts::{ProgramAccount, ProgramAccountDiscriminant};
use fankor::prelude::*;

#[fankor_base]
#[derive(AccountSize, AccountOffsets)]
pub enum EnumAccountData {
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
