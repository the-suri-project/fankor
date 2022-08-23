#[macro_export]
macro_rules! panic_error {
    ($error:expr) => {
        let err: Error = $error.into();
        err.log();

        panic!("Panic due to previous error");
    };
}

/// Ensures a condition is true, otherwise returns with the given error.
/// Use this with or without a custom error type.
#[macro_export]
macro_rules! require {
    ($invariant:expr, $error:expr $(,)?) => {
        if !($invariant) {
            return Err($error.into());
        }
    };
}

/// Same as `require!` but the condition must be negative.
#[macro_export]
macro_rules! require_not {
    ($invariant:expr, $error:expr $(,)?) => {
        if ($invariant) {
            return Err($error.into());
        }
    };
}

pub use panic_error;
pub use require;
pub use require_not;
