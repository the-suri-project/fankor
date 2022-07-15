use crate::errors::FankorResult;

pub trait Instruction: InstructionBase {
    fn processor(self) -> FankorResult<()>;
}

pub trait InstructionBase {
    fn discriminator() -> &'static [u8];

    fn base_processor(self) -> FankorResult<()>;
}
