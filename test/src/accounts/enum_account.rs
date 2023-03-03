use fankor::prelude::*;

#[fankor_base]
#[derive(FieldOffsets)]
pub enum EnumAccountData {
    A,
    B(u32),
    C { value1: u32, value2_snake: String },
    D { value4: (), value5: FnkExtension },
}

#[derive(EnumDiscriminants, FankorZeroCopy)]
pub enum ZeroCopyEnumWithoutValues {
    A,
    B,
    C,
}
