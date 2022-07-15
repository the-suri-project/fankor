use solana_program::pubkey::Pubkey;

pub trait Program {
    /// The address that identify the program.
    fn address() -> &'static Pubkey;
}
