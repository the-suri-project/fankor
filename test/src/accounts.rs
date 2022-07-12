use fankor::prelude::*;

#[account]
#[derive(AccountSize, AccountOffsets)]
pub struct Account {
    pub value1: u32,
    pub value2: String,
}

#[account]
#[derive(AccountSize, AccountOffsets)]
pub enum EnumAccount {
    A,
    B(u32, u64),
    C { value1: u32, value2: String },
}
