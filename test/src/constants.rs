use fankor::prelude::*;

#[constant]
const BOOLEAN: bool = true;

#[constant]
const NUMBER: u8 = 1;

#[constant]
const BIG_NUMBER: u64 = 500;

#[constant]
const STRING: &str = "string";

#[constant]
const ARRAY: [u8; 4] = [3, 4, 5, 9];

#[constant]
const TUPLE: (u8, bool) = (1, true);

#[constant]
const OPTION: Option<&str> = Some("hello");

#[constant]
const OPTION2: Option<&str> = None;
