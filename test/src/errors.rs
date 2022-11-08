use fankor::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("Hello, world!")]
    A,

    #[msg("D: {}", a)]
    #[code(50)]
    D { a: u64, b: u64 },

    #[code(78)]
    C,

    #[code(77)]
    #[msg("A: {}", v0)]
    #[deprecated]
    B(u64, u64, u64),
}
