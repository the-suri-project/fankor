use fankor::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("Hello, world!")]
    A,

    #[msg("D: {}", a)]
    #[continue_from(50)]
    D { a: u64, b: u64 },

    #[continue_from(77)]
    #[msg("A: {}", v0)]
    #[deprecated]
    B(u64, u64, u64),

    #[continue_from(78)]
    C,
}
