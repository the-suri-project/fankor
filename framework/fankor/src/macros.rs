#[macro_export]
macro_rules! panic_error {
    ($error:expr) => {
        let err: Error = $error.into();
        err.log();

        panic!("Panic due to previous error");
    };
}

pub use panic_error;
