use fankor::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("Hello, world!")]
    A,

    #[msg("D: {}", a)]
    #[discriminant = 50]
    D { a: u64, b: u64 },

    #[discriminant = 77]
    C,

    #[msg("A: {}", v0)]
    #[deprecated]
    #[discriminant = 78]
    B(u64),
}
