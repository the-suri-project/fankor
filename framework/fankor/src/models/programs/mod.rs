#[cfg(feature = "metadata-program")]
pub use metadata::*;
pub use system_program::*;
#[cfg(feature = "token-program")]
pub use token::*;

mod system_program;
#[cfg(feature = "token-program")]
mod token;

mod macros;
#[cfg(feature = "metadata-program")]
mod metadata;
