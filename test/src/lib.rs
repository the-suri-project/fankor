#![allow(deprecated)]

use fankor::prelude::*;

mod accounts;
mod arguments;
mod errors;
mod instruction;
mod program;

#[cfg(all(test, feature = "test"))]
mod tests;

setup!("7JKciYMdWKBo1yPhjVe5eDDjoxYfB8YhkAL7DRpJj3xE");
