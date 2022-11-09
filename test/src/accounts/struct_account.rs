use crate::accounts::{ProgramAccount, ProgramAccountDiscriminant};
use fankor::prelude::*;

#[account(ProgramAccount)]
#[derive(AccountSize, AccountOffsets)]
pub struct StructAccountData {
    pub value1: u32,
    pub value2: String,
}
