macro_rules! impl_cpi_method {
    ($program: ident, $cpi_name: ident, $name: ident, $func: expr, accounts: [$($accounts:ident),* $(,)?], args: [$($arg_keys:ident : $arg_types: ty),* $(,)?] $(, instruction_error_handle: $instruction_error_handle: tt)? $(,)?) => {
        pub struct $cpi_name<'info> {
            $(pub $accounts: AccountInfo<'info>,)*
        }

        pub fn $name(
            program: &Program<$program>,
            accounts: $cpi_name,
            $($arg_keys: $arg_types,)*
            signer_seeds: &[&[&[u8]]],
        ) -> FankorResult<()> {
            let ix = $func(
                program.address(),
                $(accounts.$accounts.key,)*
                $($arg_keys,)*
            ) $($instruction_error_handle)?;

            solana_program::program::invoke_signed(
                &ix,
                &[$(accounts.$accounts),*],
                signer_seeds,
            )
            .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
        }
    };
    ($program: ident, $cpi_name: ident, $name: ident, $func: expr, accounts: [$($accounts:ident),* $(,)?], args: [$($arg_keys:ident : $arg_types: ty),* $(,)?], account_access_token: * $(, instruction_error_handle: $instruction_error_handle: tt)? $(,)?) => {
        pub struct $cpi_name<'info> {
            $(pub $accounts: AccountInfo<'info>,)*
        }

        pub fn $name(
            program: &Program<$program>,
            accounts: $cpi_name,
            $($arg_keys: $arg_types,)*
            signer_seeds: &[&[&[u8]]],
        ) -> FankorResult<()> {
            let ix = $func(
                *program.address(),
                $(*accounts.$accounts.key,)*
                $($arg_keys,)*
            ) $($instruction_error_handle)?;

            solana_program::program::invoke_signed(
                &ix,
                &[$(accounts.$accounts),*],
                signer_seeds,
            )
            .map_or_else(|e| Err(Error::ProgramError(e)), |_| Ok(()))
        }
    };
}

pub(crate) use impl_cpi_method;
