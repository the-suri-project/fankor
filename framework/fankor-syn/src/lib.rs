use syn::Error;

pub mod expressions;

pub type Result<T> = std::result::Result<T, Error>;
