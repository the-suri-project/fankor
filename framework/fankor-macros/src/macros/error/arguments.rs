use crate::fnk_syn::FnkMetaArgumentList;
use crate::Result;

pub struct ErrorArguments {
    /// The starting offset of the error list.
    pub offset: Option<u32>,

    /// Whether to add the TsGen macro or not.
    pub skip_ts_gen: bool,
}

impl ErrorArguments {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new instance of the ErrorArguments struct from the given attributes.
    pub fn from(mut args: FnkMetaArgumentList) -> Result<ErrorArguments> {
        args.error_on_duplicated()?;

        let result = ErrorArguments {
            offset: args.pop_number("offset", true)?,
            skip_ts_gen: args.pop_plain("skip_ts_gen", true)?,
        };

        args.error_on_unknown()?;

        Ok(result)
    }
}
