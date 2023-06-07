use solana_program::pubkey::Pubkey;

use crate::errors::FankorResult;
use crate::models::FankorContext;
use crate::prelude::byte_seeds_to_slices;

/// Trait that has methods to generate the PDA information of an account.
pub trait PdaGenerator<'info> {
    /// Returns the seeds used to generate the PDA.
    fn get_pda_seeds(&self) -> FankorResult<Vec<u8>>;

    /// Returns the actual PDA address as well as the bump seed and the seeds used to generate it.
    fn get_pda_seeds_with_bump(
        &self,
        context: FankorContext<'info>,
    ) -> FankorResult<(Pubkey, u8, Vec<u8>)> {
        let mut seeds = self.get_pda_seeds()?;
        let seeds_slices = byte_seeds_to_slices(&seeds);

        let (pubkey, bump) = Pubkey::find_program_address(&seeds_slices, context.program_id());

        seeds.push(bump);

        Ok((pubkey, bump, seeds))
    }
}
