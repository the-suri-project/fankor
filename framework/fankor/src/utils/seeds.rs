use solana_program::pubkey::MAX_SEED_LEN;

/// Converts a list of seeds into a list of slices of seeds with at most
/// `MAX_SEED_LEN` bytes per each.
pub fn byte_seeds_to_slices(seeds: &[u8]) -> Vec<&[u8]> {
    let mut final_seeds = Vec::with_capacity((seeds.len() + MAX_SEED_LEN - 1) / MAX_SEED_LEN);
    final_seeds.extend(seeds.chunks(MAX_SEED_LEN));

    final_seeds
}
