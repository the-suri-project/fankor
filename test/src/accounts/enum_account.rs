use fankor::prelude::*;

#[account]
#[derive(AccountSize, AccountOffsets)]
pub enum EnumAccountData {
    A,
    B(u32, u64),
    C { value1: u32, value2: String },
}
