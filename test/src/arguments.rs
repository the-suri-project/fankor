use fankor::prelude::*;

#[derive(Clone, FankorSerialize, FankorDeserialize)]
pub struct InstructionArgs {
    pub arg1: bool,
    pub arg2: u32,
    pub arg3: u64,
}
