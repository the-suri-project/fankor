use fankor::prelude::*;

pub use enum_account::*;
pub use struct_account::*;

mod enum_account;
mod struct_account;

#[accounts]
pub enum ProgramAccount {
    StructAccountData,
    ZeroCopyStructAccountData,
    EnumAccountData,
    ZeroCopyEnumAccountData = 10,
}

#[accounts(base = "ProgramAccount")]
pub enum ProgramAccountSubSet {
    StructAccountData,
    EnumAccountData,
}

#[accounts(base = "ProgramAccount")]
pub enum ProgramAccountZeroSubSet {
    ZeroCopyStructAccountData,
    ZeroCopyEnumAccountData,
}
