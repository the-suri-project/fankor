use fankor::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("Hello, world!")]
    A,

    #[msg("D: {}", a)]
    D {
        a: u64,
        b: u64,
    } = 50,

    C = 78,

    #[msg("A: {}", v0)]
    #[deprecated]
    B(u64, u64, u64) = 77,
}
