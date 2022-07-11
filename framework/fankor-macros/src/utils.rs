use sha2::{Digest, Sha256};

/// Computes a 256-bit hash from a text.
pub fn generate_discriminator(account_name: &str, remove_highest_bit: bool) -> (String, [u8; 32]) {
    // Compute the hash from the key.
    let mut hasher = Sha256::default();
    hasher.update(account_name);

    let bytes = hasher.finalize();
    let mut final_bytes = [0u8; 32];

    // Remove the highest bit of each byte.
    if remove_highest_bit {
        for (i, byte) in bytes.iter().enumerate() {
            final_bytes[i] = byte & !0x80;
        }
    } else {
        for (i, byte) in bytes.iter().enumerate() {
            final_bytes[i] = *byte;
        }
    }

    (bs58::encode(&final_bytes).into_string(), final_bytes)
}
