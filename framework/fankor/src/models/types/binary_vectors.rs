use crate::prelude::AccountSize;
use borsh::{BorshDeserialize, BorshSerialize};
use std::cmp::Ordering;
use std::fmt::Debug;
use std::io::Write;
use std::mem;
use std::mem::size_of;

// Based on https://github.com/oliver-anhuth/avl/blob/d53a6e006a5f4a6df04755703c10dca763028fcf/src/map.rs#L1039

pub(crate) const MAX_HEIGHT: usize = 23; // 1,44 * log2(MAX_NODES) / MAX_NODES = 2 ^ 16 bits

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
pub struct FnkBVec<K, V> {
    nodes: Vec<Node<K, V>>,

    /// The index of the root element. If it is 0, it means that the vector
    /// is empty, otherwise the actual index is `root_index - 1`.
    root_position: u16,
}

impl<K, V> FnkBVec<K, V> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root_position: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            root_position: 0,
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns true if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the total capacity of the map.
    pub fn capacity(&self) -> usize {
        self.nodes.capacity()
    }

    /// Returns a reference to the root value.
    pub fn root_entry(&self) -> Option<(&K, &V)> {
        if self.root_position == 0 {
            None
        } else {
            let data = &self.nodes[self.root_position as usize - 1];
            Some((&data.key, &data.value))
        }
    }

    /// Returns a mutable reference to the root value.
    pub fn root_entry_mut(&mut self) -> Option<(&K, &mut V)> {
        if self.root_position == 0 {
            None
        } else {
            let data = &mut self.nodes[self.root_position as usize - 1];
            Some((&data.key, &mut data.value))
        }
    }

    // METHODS ----------------------------------------------------------------

    /// Clears the map, deallocating all memory.
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.root_position = 0;
    }
}

#[cfg(test)]
impl<K: Debug, V: Debug> FnkBVec<K, V> {
    // GETTERS ----------------------------------------------------------------

    /// Returns the height of the map.
    pub fn height(&self) -> u8 {
        if self.root_position == 0 {
            0
        } else {
            let node = &self.nodes[self.root_position as usize - 1];
            node.height
        }
    }

    // METHODS ----------------------------------------------------------------

    /// Validates the tree is ok.
    pub fn validate(&self) -> i16 {
        self._validate(self.root_position)
    }

    fn _validate(&self, node_position: u16) -> i16 {
        if node_position == 0 {
            return 0;
        }

        let node = &self.nodes[node_position as usize - 1];
        let left_height = self._validate(node.left_child_at);
        if left_height < 0 {
            return -1;
        }

        let right_height = self._validate(node.right_child_at);
        if right_height < 0 {
            return -1;
        }

        if (left_height - right_height).abs() > 1 {
            -1
        } else {
            left_height.max(right_height) + 1
        }
    }
}

impl<K: Ord + Copy, V: Copy> FnkBVec<K, V> {
    // METHODS ----------------------------------------------------------------

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: &K) -> Option<&V> {
        if self.root_position == 0 {
            return None;
        }

        let mut current_element = &self.nodes[self.root_position as usize - 1];
        loop {
            match key.cmp(&current_element.key) {
                Ordering::Less => {
                    if current_element.left_child_at == 0 {
                        break;
                    }

                    current_element = &self.nodes[current_element.left_child_at as usize - 1];
                }
                Ordering::Greater => {
                    if current_element.right_child_at == 0 {
                        break;
                    }

                    current_element = &self.nodes[current_element.right_child_at as usize - 1];
                }
                Ordering::Equal => {
                    return Some(&current_element.value);
                }
            }
        }

        None
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if self.root_position == 0 {
            return None;
        }

        let mut next_position = self.root_position;
        loop {
            let current_element = &self.nodes[next_position as usize - 1];

            match key.cmp(&current_element.key) {
                Ordering::Less => {
                    if current_element.left_child_at == 0 {
                        return None;
                    }

                    next_position = current_element.left_child_at;
                }
                Ordering::Greater => {
                    if current_element.right_child_at == 0 {
                        return None;
                    }

                    next_position = current_element.right_child_at;
                }
                Ordering::Equal => {
                    break;
                }
            }
        }

        let current_element = &mut self.nodes[next_position as usize - 1];
        Some(&mut current_element.value)
    }

    /// Returns true if the key is in the map, else false.
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Inserts a new element into the vector. It will panic if the maximum
    /// number of nodes is exceeded. If the key already exists, it will
    /// overwrite the value and return the old one.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        assert!(
            self.len() <= u16::MAX as usize,
            "Maximum number of nodes exceeded"
        );

        if self.root_position == 0 {
            self.nodes.push(Node::new(key, value));
            self.root_position = self.nodes.len() as u16;

            return None;
        }

        let old_value = None;
        let mut parents = [0u16; MAX_HEIGHT];
        let mut parent_left_direction = [false; MAX_HEIGHT];
        let mut parent_index = 0;

        parents[0] = self.root_position;
        parent_left_direction[0] = false;

        // Go down finding the position where to insert the new node.
        loop {
            let node_position = parents[parent_index] as usize;
            let node = &self.nodes[node_position - 1];

            match node.key.cmp(&key) {
                Ordering::Greater => {
                    if node.left_child_at == 0 {
                        // Insert node and update parent.
                        self.nodes.push(Node::new(key, value));

                        let new_index = self.nodes.len() as u16;
                        let node = &mut self.nodes[node_position - 1];
                        node.left_child_at = new_index;

                        break;
                    }

                    parent_index += 1;
                    parents[parent_index] = node.left_child_at;
                    parent_left_direction[parent_index] = true;
                }
                Ordering::Less => {
                    if node.right_child_at == 0 {
                        // Insert node and update parent.
                        self.nodes.push(Node::new(key, value));

                        let new_index = self.nodes.len() as u16;
                        let node = &mut self.nodes[node_position - 1];
                        node.right_child_at = new_index;

                        break;
                    }

                    parent_index += 1;
                    parents[parent_index] = node.right_child_at;
                    parent_left_direction[parent_index] = false;
                }
                Ordering::Equal => {
                    let node = &mut self.nodes[node_position - 1];
                    let old_value = mem::replace(&mut node.value, value);

                    // We do not need to rebalance the tree.
                    return Some(old_value);
                }
            }
        }

        // Go up balancing nodes and adjusting sizes.
        loop {
            let node_position = parents[parent_index];
            let (subtree_root, rebalanced) = self.rebalance_node(node_position);

            if !rebalanced {
                if parent_index == 0 {
                    break;
                }

                parent_index -= 1;
                continue;
            }

            // Update parent.
            if parent_index == 0 {
                self.root_position = subtree_root;
            } else {
                let parent = &mut self.nodes[parents[parent_index - 1] as usize - 1];

                if parent_left_direction[parent_index] {
                    parent.left_child_at = subtree_root;
                } else {
                    parent.right_child_at = subtree_root;
                }
            }

            break;
        }

        old_value
    }

    /// Removes the entry from the map and returns its value.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if self.root_position == 0 {
            return None;
        }

        let mut parents = [0u16; MAX_HEIGHT];
        let mut parent_left_direction = [false; MAX_HEIGHT];
        let mut parent_index = 0;
        let to_remove_position;

        parents[0] = self.root_position;
        parent_left_direction[0] = false;

        // Go down finding the position of the element to remove.
        loop {
            let node_position = parents[parent_index];
            let node = &self.nodes[node_position as usize - 1];

            match node.key.cmp(key) {
                Ordering::Greater => {
                    if node.left_child_at == 0 {
                        return None;
                    }

                    parent_index += 1;
                    parents[parent_index] = node.left_child_at;
                    parent_left_direction[parent_index] = true;
                }
                Ordering::Less => {
                    if node.right_child_at == 0 {
                        return None;
                    }

                    parent_index += 1;
                    parents[parent_index] = node.right_child_at;
                    parent_left_direction[parent_index] = false;
                }
                Ordering::Equal => {
                    to_remove_position = node_position;

                    break;
                }
            }
        }

        // Unlink node to remove.
        {
            // Check if node to-unlink has right sub tree
            let node_to_remove = &self.nodes[to_remove_position as usize - 1];
            if node_to_remove.right_child_at != 0 {
                let node_to_remove_parent_index = parent_index;
                let node_to_remove_direction = parent_left_direction[parent_index];
                let right_child_position = node_to_remove.right_child_at;
                let right_node = &self.nodes[right_child_position as usize - 1];

                if right_node.left_child_at == 0 {
                    // Replace node by smallest child in right sub tree
                    //   |             |
                    //   *             R
                    //  / \           / \
                    // L   R         L   RR
                    //    / \    =>
                    //       RR

                    // Replace node to remove in parents.
                    parents[parent_index] = right_child_position;

                    // Update right node.
                    let node_to_remove = &self.nodes[to_remove_position as usize - 1];
                    let node_to_remove_left_child_at = node_to_remove.left_child_at;
                    let right_node = &mut self.nodes[right_child_position as usize - 1];
                    right_node.left_child_at = node_to_remove_left_child_at;
                } else {
                    // Replace node by smallest child in right sub tree
                    //   |             |
                    //   *             RL
                    //  / \           /  \
                    // L   R         L    R
                    //    / \    =>      / \
                    //  RL   RR       RLR   RR
                    //    \
                    //     RLR

                    // Add right node to parents.
                    parent_index += 1;
                    parents[parent_index] = right_child_position;
                    parent_left_direction[parent_index] = false;

                    // Get min node.
                    let mut min_node = &self.nodes[node_to_remove.right_child_at as usize - 1];
                    while min_node.left_child_at != 0 {
                        parent_index += 1;
                        parents[parent_index] = min_node.left_child_at;
                        parent_left_direction[parent_index] = true;

                        min_node = &self.nodes[min_node.left_child_at as usize - 1];
                    }

                    // Replace node to remove in parents by min node.
                    parents[node_to_remove_parent_index] = parents[parent_index];

                    // Unlink min node from parent.
                    let min_node_position = parents[parent_index];
                    let min_node_parent_position = parents[parent_index - 1];
                    let min_node = &self.nodes[min_node_position as usize - 1];
                    let min_node_right_child_at = min_node.right_child_at;
                    debug_assert_eq!(min_node.left_child_at, 0);

                    let min_node_parent = &mut self.nodes[min_node_parent_position as usize - 1];
                    if parent_left_direction[parent_index] {
                        min_node_parent.left_child_at = min_node_right_child_at;
                    } else {
                        min_node_parent.right_child_at = min_node_right_child_at;
                    }

                    // Link min node to node-to-remove's children.
                    let node_to_remove = &self.nodes[to_remove_position as usize - 1];
                    let node_to_remove_left_child_at = node_to_remove.left_child_at;
                    let node_to_remove_right_child_at = node_to_remove.right_child_at;
                    let min_node = &mut self.nodes[min_node_position as usize - 1];

                    min_node.left_child_at = node_to_remove_left_child_at;
                    min_node.right_child_at = node_to_remove_right_child_at;

                    // Remove min node from parents at the last position.
                    parent_index -= 1;
                }

                // Update parent.
                if node_to_remove_parent_index == 0 {
                    self.root_position = parents[node_to_remove_parent_index];
                } else {
                    let parent =
                        &mut self.nodes[parents[node_to_remove_parent_index - 1] as usize - 1];
                    if node_to_remove_direction {
                        parent.left_child_at = parents[node_to_remove_parent_index];
                    } else {
                        parent.right_child_at = parents[node_to_remove_parent_index];
                    }
                }
            } else {
                // Node to-unlink is stem or leaf, unlink from tree.
                //   |        |
                //   *   =>   L
                //  /
                // L
                let left_child_at = node_to_remove.left_child_at;

                if parent_index == 0 {
                    self.root_position = left_child_at;
                } else {
                    let parent = &mut self.nodes[parents[parent_index - 1] as usize - 1];
                    if parent_left_direction[parent_index] {
                        parent.left_child_at = left_child_at;
                    } else {
                        parent.right_child_at = left_child_at;
                    }
                }

                parent_index = parent_index.saturating_sub(1);
            }
        }

        // Rebalance parents.
        {
            for parent_index in (0..=parent_index).rev() {
                let (subtree_root, rebalanced) = self.rebalance_node(parents[parent_index]);

                if !rebalanced {
                    continue;
                }

                // Update parent.
                if parent_index == 0 {
                    self.root_position = subtree_root;
                } else {
                    let parent = &mut self.nodes[parents[parent_index - 1] as usize - 1];

                    if parent_left_direction[parent_index] {
                        parent.left_child_at = subtree_root;
                    } else {
                        parent.right_child_at = subtree_root;
                    }
                }
            }
        }

        // Remove node and swap it with last node.
        let last_node_position = self.nodes.len() as u16;
        let old_node = self.nodes.swap_remove(to_remove_position as usize - 1);

        if self.root_position == last_node_position {
            self.root_position = to_remove_position;
        } else if last_node_position != to_remove_position {
            let last_node_key = self.nodes[to_remove_position as usize - 1].key;
            let mut current_position = self.root_position;

            loop {
                let node = &mut self.nodes[current_position as usize - 1];

                match node.key.cmp(&last_node_key) {
                    Ordering::Greater => {
                        if node.left_child_at == last_node_position {
                            node.left_child_at = to_remove_position;
                            break;
                        }

                        debug_assert_ne!(node.left_child_at, 0);

                        current_position = node.left_child_at;
                    }
                    Ordering::Less => {
                        if node.right_child_at == last_node_position {
                            node.right_child_at = to_remove_position;
                            break;
                        }

                        debug_assert_ne!(node.right_child_at, 0);

                        current_position = node.right_child_at;
                    }
                    Ordering::Equal => {
                        unreachable!("Node should have been found before");
                    }
                }
            }
        }

        Some(old_node.value)
    }

    // ------------------------------------------------------------------------
    // Auxiliary methods ------------------------------------------------------
    // ------------------------------------------------------------------------

    /// Rotate given node to the left and returns the new subtree's root.
    /// ```none
    ///     |                |
    ///     *                R
    ///    / \              / \
    ///   L   R      =>    *   RR
    ///      / \          / \
    ///    RL   RR       L   RL
    /// ```
    /// Note `node_position` is always correct.
    fn rotate_left(&mut self, node_position: u16) -> u16 {
        let node = &self.nodes[node_position as usize - 1];

        let right_position = node.right_child_at;
        let right = &mut self.nodes[right_position as usize - 1];
        let node_right_child_at = right.left_child_at;

        right.left_child_at = node_position;

        let node = &mut self.nodes[node_position as usize - 1];
        node.right_child_at = node_right_child_at;

        self.adjust_height(node_position);
        self.adjust_height(right_position);

        right_position
    }

    /// Rotate given node to the right and returns the new subtree's root.
    /// ```none
    ///      |                |
    ///      *                L
    ///     / \              / \
    ///    L   R      =>   LL   *
    ///   / \                  / \
    /// LL   LR              LR   R
    /// ```
    /// Note `node_position` is always correct.
    fn rotate_right(&mut self, node_position: u16) -> u16 {
        let node = &self.nodes[node_position as usize - 1];

        let left_position = node.left_child_at;
        let left = &mut self.nodes[left_position as usize - 1];
        let node_left_child_at = left.right_child_at;

        left.right_child_at = node_position;

        let node = &mut self.nodes[node_position as usize - 1];
        node.left_child_at = node_left_child_at;

        self.adjust_height(node_position);
        self.adjust_height(left_position);

        left_position
    }

    /// Rotate given node to the right and returns the new subtree's root
    /// plus a flag indicating whether rebalancing had been necessary.
    /// ```none
    ///      |                |
    ///      *                L
    ///     / \              / \
    ///    L   R      =>   LL   *
    ///   / \                  / \
    /// LL   LR              LR   R
    /// ```
    /// Note `node_position` is always correct.
    fn rebalance_node(&mut self, node_position: u16) -> (u16, bool) {
        let left_child_height = self.left_height(node_position);
        let right_child_height = self.right_height(node_position);

        if left_child_height > right_child_height + 1 {
            // Rebalance right.
            let left_child_at = self.nodes[node_position as usize - 1].left_child_at;

            if self.right_height(left_child_at) > self.left_height(left_child_at) {
                let left_child_at = self.rotate_left(left_child_at);
                self.nodes[node_position as usize - 1].left_child_at = left_child_at;
            }

            (self.rotate_right(node_position), true)
        } else if right_child_height > left_child_height + 1 {
            // Rebalance left.
            let right_child_at = self.nodes[node_position as usize - 1].right_child_at;

            if self.left_height(right_child_at) > self.right_height(right_child_at) {
                let right_child_at = self.rotate_right(right_child_at);
                self.nodes[node_position as usize - 1].right_child_at = right_child_at;
            }

            (self.rotate_left(node_position), true)
        } else {
            // Adjust balance.
            let mut parent = &mut self.nodes[node_position as usize - 1];
            parent.height = left_child_height.max(right_child_height);

            (node_position, false)
        }
    }

    fn left_height(&self, node_position: u16) -> u8 {
        let node = &self.nodes[node_position as usize - 1];

        if node.left_child_at == 0 {
            0
        } else {
            let left = &self.nodes[node.left_child_at as usize - 1];
            left.height + 1
        }
    }

    fn right_height(&self, node_position: u16) -> u8 {
        let node = &self.nodes[node_position as usize - 1];

        if node.right_child_at == 0 {
            0
        } else {
            let right = &self.nodes[node.right_child_at as usize - 1];
            right.height + 1
        }
    }

    fn adjust_height(&mut self, node_position: u16) {
        let left_child_height = self.left_height(node_position);
        let right_child_height = self.right_height(node_position);

        let mut node = &mut self.nodes[node_position as usize - 1];
        node.height = left_child_height.max(right_child_height);
    }
}

impl<K, V> FnkBVec<K, V> {
    /// Returns an iterator over the map.
    pub fn iter(&self) -> Iter<K, V> {
        if self.is_empty() {
            return Iter {
                data: self,
                parents: [0; 23],
                parent_index: 0,
            };
        }

        let mut parents = [0u16; 23];
        parents[0] = self.root_position;

        let mut parent_index = 1u8;

        // Get left most node.
        let mut min_node = &self.nodes[self.root_position as usize - 1];
        while min_node.left_child_at != 0 {
            parent_index += 1;
            parents[parent_index as usize - 1] = min_node.left_child_at;

            min_node = &self.nodes[min_node.left_child_at as usize - 1];
        }

        Iter {
            data: self,
            parents,
            parent_index,
        }
    }
}

impl<K, V> Default for FnkBVec<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: BorshSerialize, V: BorshSerialize> BorshSerialize for FnkBVec<K, V> {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        (self.nodes.len() as u16).serialize(writer)?;
        self.root_position.serialize(writer)?;

        for node in &self.nodes {
            node.serialize(writer)?;
        }

        Ok(())
    }
}

impl<K: BorshDeserialize, V: BorshDeserialize> BorshDeserialize for FnkBVec<K, V> {
    #[inline]
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let len = u16::deserialize(buf)?;
        let root_position = u16::deserialize(buf)?;

        let mut nodes = Vec::with_capacity(len as usize);
        for _ in 0..len {
            nodes.push(Node::deserialize(buf)?);
        }

        Ok(Self {
            nodes,
            root_position,
        })
    }
}

impl<K: Copy, V: Copy> AccountSize for FnkBVec<K, V> {
    fn min_account_size() -> usize {
        size_of::<u16>() * 2
    }

    fn actual_account_size(&self) -> usize {
        size_of::<u16>() * 2 + self.nodes.len() * Node::<K, V>::byte_size()
    }
}

impl<K: PartialEq, V: PartialEq> PartialEq for FnkBVec<K, V> {
    fn eq(&self, other: &Self) -> bool {
        if self.nodes.len() != other.len() {
            return false;
        }

        for (a, b) in self.iter().zip(other.iter()) {
            if a != b {
                return false;
            }
        }

        true
    }
}

impl<K: Eq, V: Eq> Eq for FnkBVec<K, V> {}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub(crate) struct Node<K, V> {
    pub key: K,
    pub value: V,
    pub left_child_at: u16,
    pub right_child_at: u16,
    pub height: u8,
}

impl<K, V> Node<K, V> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            left_child_at: 0,
            right_child_at: 0,
            height: 0,
        }
    }

    // GETTERS ----------------------------------------------------------------

    pub const fn byte_size() -> usize {
        size_of::<K>() + size_of::<V>() + size_of::<u16>() * 2 + size_of::<u8>()
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct Iter<'a, K, V> {
    pub(crate) data: &'a FnkBVec<K, V>,
    pub(crate) parents: [u16; MAX_HEIGHT],
    /// Zero means empty.
    /// One means the first parent.
    pub(crate) parent_index: u8,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.parent_index == 0 {
            return None;
        }

        let node_position = self.parents[self.parent_index as usize - 1];
        let node = &self.data.nodes[node_position as usize - 1];
        self.parent_index -= 1;

        if node.right_child_at != 0 {
            self.parent_index += 1;
            self.parents[self.parent_index as usize - 1] = node.right_child_at;

            // Get left most node.
            let mut min_node = &self.data.nodes[node.right_child_at as usize - 1];
            while min_node.left_child_at != 0 {
                self.parent_index += 1;
                self.parents[self.parent_index as usize - 1] = min_node.left_child_at;

                min_node = &self.data.nodes[min_node.left_child_at as usize - 1];
            }
        }

        Some((&node.key, &node.value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.data.len();

        (size, Some(size))
    }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> {}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use rand::Rng;
    use std::collections::HashSet;

    /// This test can take some time to complete.
    #[test]
    fn test_iterator() {
        for _ in 0..100 {
            let mut rng = rand::thread_rng();
            let mut map = FnkBVec::new();

            let mut keys = HashSet::with_capacity(100);

            for i in 0..100 {
                let next = rng.gen_range(0..u32::MAX);
                map.insert(next, i);
                keys.insert(next);
            }

            let mut sorted_keys = keys.iter().copied().collect::<Vec<_>>();
            sorted_keys.sort();

            let map_keys = map.iter().map(|(k, _)| *k).collect::<Vec<_>>();

            assert_eq!(sorted_keys, map_keys);
        }
    }

    /// This test can take some time to complete.
    #[test]
    fn test_insert_get_and_remove_random() {
        for _ in 0..100 {
            let mut rng = rand::thread_rng();
            let mut map = FnkBVec::new();

            assert_eq!(map.validate(), 0, "(0) Invalid height");

            let bits = 10;
            let combinations = 2u32.pow(bits);
            let mut values = HashSet::with_capacity(combinations as usize);
            let max_height = 1.44 * (combinations as f64).log2();

            for i in 0..combinations {
                let next = rng.gen_range(0..u32::MAX);
                map.insert(next, i);
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

                assert_eq!(
                    map.get(&next),
                    Some(&i),
                    "(I{}) Map does not contain {}",
                    i + 1,
                    next,
                );
            }

            assert_eq!(map.len(), values.len(), "Invalid map length");

            for (i, next) in values.iter().enumerate() {
                assert!(
                    map.remove(next).is_some(),
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

                assert_eq!(
                    map.get(next),
                    None,
                    "(R{}) Map still contains {}",
                    i + 1,
                    next,
                );
            }

            assert_eq!(map.root_position, 0, "Root position is not 0");
            assert_eq!(map.len(), 0, "Map is not empty");
        }
    }

    #[test]
    fn test_serialize_deserialize() {
        let mut rng = rand::thread_rng();
        let mut map = FnkBVec::new();

        let mut values = HashSet::new();

        for i in 0..100 {
            let next = rng.gen_range(0..u32::MAX);
            map.insert(next, i);
            values.insert(next);
        }

        // Validate
        let height = map.validate();

        if height < 0 {
            panic!("(1) Incorrect tree state");
        }

        // (De)Serialize
        let serialize = map.try_to_vec().expect("Failed to serialize");
        let deserialize = FnkBVec::try_from_slice(&serialize).expect("Failed to deserialize");

        // Validate
        let height = map.validate();

        if height < 0 {
            panic!("(2) Incorrect tree state");
        }

        assert_eq!(
            map.root_position, deserialize.root_position,
            "Root position is not the same"
        );
        assert_eq!(map.len(), deserialize.len(), "Map length is not the same");

        for value in values {
            assert_eq!(
                map.get(&value),
                deserialize.get(&value),
                "Value is not the same"
            );
        }
    }

    #[test]
    fn test_equal() {
        let mut map1 = FnkBVec::new();
        let mut map2 = FnkBVec::new();

        assert_eq!(map1, map2);

        map1.insert(1, 1);
        assert_ne!(map1, map2);

        map2.insert(1, 1);
        assert_eq!(map1, map2);

        map2.insert(1, 2);
        assert_ne!(map1, map2);

        let mut map2 = FnkBVec::new();
        map2.insert(2, 1);
        assert_ne!(map1, map2);

        let mut map1 = FnkBVec::new();
        let mut map2 = FnkBVec::new();

        map1.insert(1, 1);
        map1.insert(2, 2);

        map2.insert(2, 2);
        map2.insert(1, 1);
        assert_eq!(map1, map2);
    }
}
