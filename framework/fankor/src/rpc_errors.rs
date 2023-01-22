use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpcFankorError<'a> {
    pub code: u32,
    pub name: Cow<'a, str>,
    pub message: Cow<'a, str>,
}

impl<'a> RpcFankorError<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(code: u32, name: Cow<'a, str>, message: Cow<'a, str>) -> Self {
        Self {
            code,
            name,
            message,
        }
    }

    pub fn from_logs(logs: &'a [String]) -> Option<Self> {
        for log in logs {
            let log = match log.strip_prefix("Program log: FankorError occurred. Error Name: ") {
                Some(v) => v,
                None => continue,
            };
            let position = match log.find('.') {
                Some(v) => v,
                None => continue,
            };

            let (name, log) = log.split_at(position);
            let log = match log.strip_prefix(". Error Code: ") {
                Some(v) => v,
                None => continue,
            };

            let position = match log.find('.') {
                Some(v) => v,
                None => continue,
            };

            let (code, log) = log.split_at(position);
            let code = match u32::from_str(code) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let message = match log.strip_prefix(". Error Message: ") {
                Some(v) => v,
                None => continue,
            };

            return Some(Self::new(code, Cow::Borrowed(name), Cow::Borrowed(message)));
        }

        None
    }
}

impl<'a> Display for RpcFankorError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FankorError {} {}(0x{:X}): {}",
            self.name, self.code, self.code, self.message
        )
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let code = 1006;
        let name = "InvalidPda";
        let message = "The provided PDA (x) does not match expected one (y).";
        let log = format!(
            "Program log: FankorError occurred. Error Name: {}. Error Code: {}. Error Message: {}",
            name, code, message
        );

        let logs = [
            "another log".to_string(),
            "Program log: FankorError occurred.".to_string(),
            "Program log: FankorError occurred. Error Name: ".to_string(),
            "Program log: FankorError occurred. Error Name: {}".to_string(),
            "Program log: FankorError occurred. Error Name: {}. Error Code: ".to_string(),
            "Program log: FankorError occurred. Error Name: {}. Error Code: {}".to_string(),
            "Program log: FankorError occurred. Error Name: {}. Error Code: {}. Error Message:"
                .to_string(),
            log,
        ];

        let error = RpcFankorError::from_logs(&logs).expect("Cannot parse error");

        assert_eq!(error.code, code, "Invalid code");
        assert_eq!(error.name, name, "Invalid name");
        assert_eq!(error.message, message, "Invalid message");
    }
}
