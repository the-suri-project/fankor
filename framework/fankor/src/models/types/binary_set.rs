use std::fmt::Debug;
use std::io::Write;

use borsh::{BorshDeserialize, BorshSerialize};

use crate::prelude::{FnkBMap, FnkBMapIter};
use crate::traits::CopyType;

// Based on https://github.com/oliver-anhuth/avl/blob/d53a6e006a5f4a6df04755703c10dca763028fcf/src/map.rs#L1039

pub(crate) const FNK_BINARY_TREE_MAX_HEIGHT: usize = 23; // 1,44 * log2(MAX_NODES) / MAX_NODES = 2 ^ 16 bits

/// Wrapper over `Vec` whose values are sorted and serialized forming a BTree
/// structure.
///
/// This vector uses the size of K and T to calculate the size of the vector
/// in bytes and to traverse it so the data must not contain any padding
/// nor references.
///
/// The maximum number of nodes are 2^16-1. If it is exceeded, it will
/// panic.
#[derive(Debug, Clone)]
pub struct FnkBSet<V>(FnkBMap<V, ()>);

impl<V> FnkBSet<V> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new() -> Self {
        Self(FnkBMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(FnkBMap::with_capacity(capacity))
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the total capacity of the map.
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Returns a reference to the root value.
    pub fn root_entry(&self) -> Option<&V> {
        self.0.root_entry().map(|(k, _)| k)
    }

    // METHODS ----------------------------------------------------------------

    /// Clears the map, deallocating all memory.
    pub fn clear(&mut self) {
        self.0.clear()
    }
}

#[cfg(test)]
impl<V: Debug> FnkBSet<V> {
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

impl<'info, V: Ord + Copy + CopyType<'info>> FnkBSet<V> {
    // METHODS ----------------------------------------------------------------

    /// Returns whether the set contains the `element`.
    pub fn contains(&self, element: &V) -> bool {
        self.0.get(element).is_some()
    }

    /// Inserts a new element into the vector. It will panic if the maximum
    /// number of nodes is exceeded. If the key already exists, it will
    /// overwrite the value and return the old one.
    pub fn insert(&mut self, element: V) {
        let _ = self.0.insert(element, ());
    }

    /// Removes the entry from the map and returns whether it contained a value or not.
    pub fn remove(&mut self, element: &V) -> bool {
        self.0.remove(element).is_some()
    }
}

impl<V> FnkBSet<V> {
    /// Returns an iterator over the map.
    pub fn iter(&self) -> FnkBSetIter<V> {
        FnkBSetIter(self.0.iter())
    }
}

impl<V> Default for FnkBSet<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: BorshSerialize> BorshSerialize for FnkBSet<V> {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.0.serialize(writer)
    }
}

impl<V: BorshDeserialize> BorshDeserialize for FnkBSet<V> {
    #[inline]
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        Ok(Self(FnkBMap::deserialize(buf)?))
    }
}

impl<V: PartialEq> PartialEq for FnkBSet<V> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<V: Eq> Eq for FnkBSet<V> {}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct FnkBSetIter<'a, V>(FnkBMapIter<'a, V, ()>);

impl<'a, V> Iterator for FnkBSetIter<'a, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, V> ExactSizeIterator for FnkBSetIter<'a, V> {}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use rand::Rng;

    use super::*;

    /// This test can take some time to complete.
    #[test]
    fn test_iterator() {
        for _ in 0..100 {
            let mut rng = rand::thread_rng();
            let mut map = FnkBSet::new();

            let mut keys = HashSet::with_capacity(100);

            for _ in 0..100 {
                let next = rng.gen_range(0..u32::MAX);
                map.insert(next);
                keys.insert(next);
            }

            let mut sorted_keys = keys.iter().copied().collect::<Vec<_>>();
            sorted_keys.sort();

            let map_keys = map.iter().cloned().collect::<Vec<_>>();

            assert_eq!(sorted_keys, map_keys);
        }
    }

    /// This test can take some time to complete.
    #[test]
    fn test_insert_get_and_remove_random() {
        for _ in 0..100 {
            let mut rng = rand::thread_rng();
            let mut map = FnkBSet::new();

            assert_eq!(map.validate(), 0, "(0) Invalid height");

            let bits = 10;
            let combinations = 2u32.pow(bits);
            let mut values = HashSet::with_capacity(combinations as usize);
            let max_height = 1.44 * (combinations as f64).log2();

            for i in 0..combinations {
                let next = rng.gen_range(0..u32::MAX);
                map.insert(next);
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
                    map.contains(&next),
                    "(I{}) Map does not contain {}",
                    i + 1,
                    next,
                );
            }

            assert_eq!(map.len(), values.len(), "Invalid map length");

            for (i, next) in values.iter().enumerate() {
                assert!(
                    map.remove(next),
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
                    !map.contains(next),
                    "(R{}) Map still contains {}",
                    i + 1,
                    next,
                );
            }

            assert_eq!(map.len(), 0, "Map is not empty");
        }
    }

    #[test]
    fn test_serialize_deserialize() {
        let mut rng = rand::thread_rng();
        let mut map = FnkBSet::new();

        let mut values = HashSet::new();

        for _ in 0..100 {
            let next = rng.gen_range(0..u32::MAX);
            map.insert(next);
            values.insert(next);
        }

        // Validate
        let height = map.validate();

        if height < 0 {
            panic!("(1) Incorrect tree state");
        }

        // (De)Serialize
        let serialize = map.try_to_vec().expect("Failed to serialize");
        let deserialize = FnkBSet::try_from_slice(&serialize).expect("Failed to deserialize");

        // Validate
        let height = map.validate();

        if height < 0 {
            panic!("(2) Incorrect tree state");
        }

        assert_eq!(map.len(), deserialize.len(), "Map length is not the same");

        for value in values {
            assert_eq!(
                map.contains(&value),
                deserialize.contains(&value),
                "Value is not the same"
            );
        }
    }

    #[test]
    fn test_equal() {
        let mut map1 = FnkBSet::new();
        let mut map2 = FnkBSet::new();

        assert_eq!(map1, map2);

        map1.insert(1);
        assert_ne!(map1, map2);

        map2.insert(1);
        assert_eq!(map1, map2);

        map2.insert(2);
        assert_ne!(map1, map2);

        map1.insert(2);
        assert_eq!(map1, map2);

        let mut map1 = FnkBSet::new();
        let mut map2 = FnkBSet::new();

        map1.insert(1);
        map1.insert(2);
        map2.insert(2);
        map2.insert(1);
        assert_eq!(map1, map2);
    }
}
