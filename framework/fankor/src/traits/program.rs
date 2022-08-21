use solana_program::pubkey::Pubkey;

pub trait Program {
    /// The name that identifies the program.
    fn name() -> &'static str;

    /// The address that identifies the program.
    fn address() -> &'static Pubkey;
}
