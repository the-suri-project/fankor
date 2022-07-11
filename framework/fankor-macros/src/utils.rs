use sha2::{Digest, Sha256};

/// Computes a 256-bit hash from a text.
pub fn generate_discriminator(account_name: &str) -> [u8; 32] {
    // Compute the hash from the key.
    let mut hasher = Sha256::default();
    hasher.update(account_name);

    let bytes = hasher.finalize();
    bytes.try_into().unwrap()
}
