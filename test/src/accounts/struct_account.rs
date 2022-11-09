use fankor::prelude::*;

#[account]
#[derive(AccountSize, AccountOffsets)]
pub struct StructAccountData {
    pub value1: u32,
    pub value2: String,
}
