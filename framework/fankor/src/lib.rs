pub mod cpi;
pub mod errors;
pub mod macros;
pub mod models;
pub mod prelude;
pub mod traits;
mod utils;

pub use utils::deserialize::try_from_vec_accounts_with_bounds;
