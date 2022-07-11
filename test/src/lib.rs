use fankor::prelude::*;

mod accounts;
mod arguments;
#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
mod errors;

setup!();
