use crate::accounts::{ProgramAccount, ProgramAccountDiscriminant};
use fankor::prelude::*;

#[account(base = ProgramAccount)]
pub struct StructAccountData {
    pub value1: u32,
    pub value2: String,
}

#[account(base = ProgramAccount)]
#[derive(FankorZeroCopy)]
pub struct StructAccountData2 {
    pub value: String,
}

#[account(base = ProgramAccount)]
#[derive(FankorZeroCopy, FieldOffsets)]
pub struct ZeroCopyStructAccountData {
    pub value1: u32,
    pub value2: String,
    pub value3: Vec<u8>,
    pub value4: (),
    pub value5: FnkExtension,
}
