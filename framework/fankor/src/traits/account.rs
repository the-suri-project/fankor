use solana_program::pubkey::Pubkey;

pub trait AccountType: borsh::BorshSerialize + borsh::BorshDeserialize {
    /// The discriminant of the account.
    fn discriminant() -> u8;

    /// Defines an address expected to own an account.
    fn owner() -> &'static Pubkey;

    /// Checks whether the discriminant matches this account type.
    /// This is mainly used when there's more than one discriminant
    /// for this account.
    fn check_discriminant(discriminant: u8) -> bool {
        discriminant == Self::discriminant()
    }
}
