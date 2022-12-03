use solana_program::account_info::AccountInfo;

/// Trait to implement in order to use the `#[account(pda = ...)]` attribute.
pub trait PdaChecker<'info> {
    fn pda_info(&self) -> Option<&'info AccountInfo<'info>>;
}
