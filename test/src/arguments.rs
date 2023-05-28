use fankor::prelude::*;

#[derive(Clone, FankorSerialize, FankorDeserialize, TsGen)]
pub struct InstructionArgs {
    pub arg1: bool,
    pub arg2: u32,
    pub arg3: u64,
}

#[fankor_base]
#[derive(Clone)]
pub struct InstructionArgs2 {
    pub arg1: bool,
    pub arg2: u32,
    pub arg3: u64,
    pub arg4: Pubkey,
}

#[fankor_base]
#[derive(Clone)]
pub enum InstructionArgs3 {
    Arg1(bool),
    Arg2(u32),
    Arg3(u64),
}

#[fankor_base]
#[derive(Clone)]
pub struct RecursiveArgs {
    pub arg1: Option<Box<RecursiveArgs>>,
}
