#[cfg(feature = "token-program")]
pub use associated_token::*;
#[cfg(feature = "metadata-program")]
pub use metadata::*;
pub use system_program::*;
#[cfg(feature = "token-program")]
pub use token::*;
#[cfg(feature = "token-program-2022")]
pub use token_2022::*;

mod system_program;
#[cfg(feature = "token-program")]
mod token;

mod macros;
#[cfg(feature = "metadata-program")]
mod metadata;

#[cfg(feature = "token-program")]
mod associated_token;

#[cfg(feature = "token-program-2022")]
mod token_2022;
