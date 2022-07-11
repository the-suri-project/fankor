use fankor::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("Hello, world!")]
    A,
    #[msg("D: {}", a)]
    #[continue_from(50)]
    D {
        a: u64,
        b: u64,
    },
    #[msg("A: {}", v0)]
    B(u64, u64, u64) = 3,
    C = 4,
}
