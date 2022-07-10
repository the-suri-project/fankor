use fankor::prelude::*;

mod arguments;
mod constants;
#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

setup!();
