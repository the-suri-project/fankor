use crate::accounts::{ProgramAccount, ProgramAccountDiscriminant};
use fankor::prelude::*;

#[account(base = ProgramAccount)]
#[derive(AccountSize, AccountOffsets)]
pub struct StructAccountData {
    pub value1: u32,
    pub value2: String,
}

#[account(base = ProgramAccount)]
#[derive(AccountSize, AccountOffsets)]
pub struct StructAccountData2 {
    pub value: String,
}

#[account(base = ProgramAccount)]
#[derive(AccountSize, AccountOffsets, FankorZeroCopy)]
pub struct ZeroCopyStructAccountData {
    pub value1: u32,
    pub value2: String,
    pub value3: Vec<u8>,
}
