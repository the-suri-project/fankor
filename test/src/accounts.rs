use fankor::prelude::*;

#[account]
pub struct Account {
    pub value1: u32,
    pub value2: String,
}

#[account]
pub enum EnumAccount {
    A,
    B(u32, u64),
    C { value1: u32, value2: String },
}
