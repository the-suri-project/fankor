use std::mem::size_of;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;

use crate::errors::FankorResult;
use crate::models::binary_map::ZcFnkBMap;
use crate::prelude::binary_map::ZcFnkBMapIter;
use crate::prelude::{CopyType, FnkBSet, Node};
use crate::traits::ZeroCopyType;

pub struct ZcFnkBSet<'info, V: CopyType<'info>>(ZcFnkBMap<'info, V, ()>);

impl<'info, V: CopyType<'info>> ZcFnkBSet<'info, V> {
    // GETTERS ----------------------------------------------------------------

    /// Returns the number of elements in the map.
    pub fn len(&self) -> FankorResult<u16> {
        self.0.len()
    }

    /// Returns true if the map contains no elements.
    pub fn is_empty(&self) -> FankorResult<bool> {
        self.0.is_empty()
    }

    // METHODS ----------------------------------------------------------------

    /// Clears the map, deallocating all memory.
    pub fn clear(&self) -> FankorResult<()> {
        self.0.clear()
    }
}

#[cfg(test)]
impl<'info, V: Ord + Copy + BorshSerialize + BorshDeserialize + CopyType<'info>>
    ZcFnkBSet<'info, V>
{
    // GETTERS ----------------------------------------------------------------

    /// Returns the height of the map.
    pub fn height(&self) -> u8 {
        self.0.height()
    }

    // METHODS ----------------------------------------------------------------

    /// Validates the tree is ok.
    pub fn validate(&self) -> i16 {
        self.0.validate()
    }
}

impl<'info, V: Ord + Copy + BorshSerialize + BorshDeserialize + CopyType<'info>>
    ZcFnkBSet<'info, V>
{
    // GETTERS ----------------------------------------------------------------

    /// Returns a reference to the root value.
    pub fn root_entry(&self) -> FankorResult<Option<V>> {
        self.0.root_entry().map(|v| v.map(|(v, _)| v))
    }

    // METHODS ----------------------------------------------------------------

    /// Returns whether `value` is contained in the set or not.
    pub fn contains(&self, value: &V) -> FankorResult<bool> {
        Ok(self.0.get(value)?.is_some())
    }

    /// Inserts a new element into the vector. It will panic if the maximum
    /// number of nodes is exceeded. If the key already exists, it will
    /// overwrite the value and return the old one.
    pub fn insert(&self, value: V) -> FankorResult<()> {
        let _ = self.0.insert(value, ())?;

        Ok(())
    }

    /// Removes the entry from the map and returns whether it contained the value or not.
    pub fn remove(&self, value: &V) -> FankorResult<bool> {
        self.0.remove(value).map(|v| v.is_some())
    }

    /// Returns an iterator over the map.
    pub fn iter(&self) -> FankorResult<ZcFnkBSetIter<'info, V>> {
        Ok(ZcFnkBSetIter(self.0.iter()?))
    }
}

impl<'info, V: CopyType<'info>> ZeroCopyType<'info> for ZcFnkBSet<'info, V> {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        let (map, offset) = ZcFnkBMap::new(info, offset)?;
        Ok((ZcFnkBSet(map), offset))
    }

    fn read_byte_size(bytes: &[u8]) -> FankorResult<usize> {
        <ZcFnkBMap<V, ()>>::read_byte_size(bytes)
    }
}

impl<'info, V: CopyType<'info>> CopyType<'info> for FnkBSet<V> {
    type ZeroCopyType = ZcFnkBSet<'info, V>;

    fn byte_size(&self) -> usize {
        // Size field + root position + nodes
        size_of::<u16>() + size_of::<u16>() + self.len() * Node::<V, ()>::byte_size()
    }

    fn min_byte_size() -> usize {
        size_of::<u16>() * 2
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct ZcFnkBSetIter<'info, V>(ZcFnkBMapIter<'info, V, ()>);

impl<'info, V: Ord + Copy + BorshSerialize + BorshDeserialize + CopyType<'info>> Iterator
    for ZcFnkBSetIter<'info, V>
{
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(v, _)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'info, V: Ord + Copy + BorshSerialize + BorshDeserialize + CopyType<'info>> ExactSizeIterator
    for ZcFnkBSetIter<'info, V>
{
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use rand::Rng;

    use crate::tests::create_account_info_for_tests;

    use super::*;

    /// This test can take some time to complete.
    #[test]
    fn test_iterator() {
        let mut lamports = 0;

        for _ in 0..100 {
            let mut vector = vec![0u8; 10_000];
            let info = create_account_info_for_tests(&mut lamports, &mut vector);
            let mut rng = rand::thread_rng();
            let (map, _) = ZcFnkBSet::new(&info, 0).unwrap();

            let mut keys = HashSet::with_capacity(100);

            for _ in 0..100 {
                let next = rng.gen_range(0..u32::MAX);
                map.insert(next).expect("Cannot insert into ZcFnkBSet");
                keys.insert(next);
            }

            let mut sorted_keys = keys.iter().copied().collect::<Vec<_>>();
            sorted_keys.sort();

            let map_keys = map
                .iter()
                .expect("Cannot iter over ZcFnkBSet")
                .collect::<Vec<_>>();

            assert_eq!(sorted_keys, map_keys);
        }
    }

    /// This test can take some time to complete.
    #[test]
    fn test_insert_get_and_remove_random() {
        let mut lamports = 0;
        let bits = 10;
        let combinations = 2u32.pow(bits);

        for _ in 0..10 {
            let mut vector = vec![0u8; combinations as usize * 45];
            let info = create_account_info_for_tests(&mut lamports, &mut vector);
            let mut rng = rand::thread_rng();
            let (map, _) = ZcFnkBSet::new(&info, 0).unwrap();

            assert_eq!(map.validate(), 0, "(0) Invalid height");

            let mut values = HashSet::with_capacity(combinations as usize);
            let max_height = 1.44 * (combinations as f64).log2();

            for i in 0..combinations {
                let next = rng.gen_range(0..u32::MAX);
                map.insert(next).expect("Cannot insert into ZcFnkBSet");
                values.insert(next);

                let height = map.validate();

                if height < 0 {
                    panic!("(I{}) Incorrect tree state when adding {}", i + 1, next);
                }

                assert!(
                    height as f64 <= max_height,
                    "(I{}) Invalid height when adding {}. Actual: {}, Max: {}",
                    i + 1,
                    next,
                    height,
                    max_height,
                );

                assert!(
                    map.contains(&next).expect("Cannot get from ZcFnkBSet"),
                    "(I{}) Map does not contain {}",
                    i + 1,
                    next,
                );
            }

            assert_eq!(
                map.len().expect("Cannot get length from ZcFnkBSet") as usize,
                values.len(),
                "Invalid map length"
            );

            for (i, next) in values.iter().enumerate() {
                assert!(
                    map.remove(next).expect("Cannot remove from ZcFnkBSet"),
                    "(R{}) Map does not contain {}",
                    i + 1,
                    next,
                );

                let height = map.validate();

                if height < 0 {
                    panic!("(R{}) Incorrect tree state when removing {}", i + 1, next);
                }

                assert!(
                    height as f64 <= max_height,
                    "(R{}) Invalid height when removing {}. Actual: {}, Max: {}",
                    i + 1,
                    next,
                    height,
                    max_height,
                );

                assert!(
                    !map.contains(next).expect("Cannot get from ZcFnkBSet"),
                    "(R{}) Map still contains {}",
                    i + 1,
                    next,
                );
            }

            assert_eq!(
                map.len().expect("Cannot get len from ZcFnkBSet"),
                0,
                "Map is not empty"
            );
        }
    }
}
