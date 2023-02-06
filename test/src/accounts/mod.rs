use fankor::prelude::*;

pub use enum_account::*;
pub use struct_account::*;

mod enum_account;
mod struct_account;

#[accounts]
pub enum ProgramAccount {
    StructAccountData,
    StructAccountData2,
    ZeroCopyStructAccountData,
}

#[accounts(base = ProgramAccount)]
pub enum ProgramAccountSubSet {
    StructAccountData,
}

#[accounts(base = ProgramAccount)]
pub enum ProgramAccountZeroSubSet {
    StructAccountData,
    ZeroCopyStructAccountData,
}
