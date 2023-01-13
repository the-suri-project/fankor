use fankor::prelude::*;

#[derive(Clone, FankorSerialize, FankorDeserialize)]
pub struct InstructionArgs {
    pub arg1: bool,
    pub arg2: u32,
    pub arg3: u64,
}

#[derive(Clone, FankorBase)]
pub struct InstructionArgs2 {
    pub arg1: bool,
    pub arg2: u32,
    pub arg3: u64,
}

#[derive(Clone, FankorBase)]
pub enum InstructionArgs3 {
    Arg1(bool),
    Arg2(u32),
    Arg3(u64),
}
