use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::{Zc, ZeroCopyType};
use crate::prelude::{CopyType, FnkBVec, Node, MAX_HEIGHT};
use crate::utils::bpf_writer::BpfWriter;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::mem::size_of;

pub struct ZcFnkBVec<'info, K: CopyType<'info>, V: CopyType<'info>> {
    info: &'info AccountInfo<'info>,
    offset: usize,
    _data: PhantomData<(K, V)>,
}

impl<'info, K: CopyType<'info>, V: CopyType<'info>> ZcFnkBVec<'info, K, V> {
    // GETTERS ----------------------------------------------------------------

    /// Returns the number of elements in the map.
    pub fn len(&self) -> FankorResult<u16> {
        let bytes =
            self.info
                .data
                .try_borrow()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;

        let mut bytes = &bytes[self.offset..];
        Ok(u16::deserialize(&mut bytes)?)
    }

    /// Returns true if the map contains no elements.
    pub fn is_empty(&self) -> FankorResult<bool> {
        self.len().map(|len| len == 0)
    }

    /// Returns the root position.
    fn root_position(&self) -> FankorResult<u16> {
        let bytes =
            self.info
                .data
                .try_borrow()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;

        let mut bytes = &bytes[self.offset..];
        bytes = &bytes[size_of::<u16>()..];
        Ok(u16::deserialize(&mut bytes)?)
    }

    /// Returns the offset of the content vector.
    fn content_offset(&self) -> usize {
        self.offset + size_of::<u16>() * 2
    }

    // METHODS ----------------------------------------------------------------

    /// Writes the number of elements in the map.
    fn write_len(&self, len: u16) -> FankorResult<()> {
        let mut bytes = self.info.data.try_borrow_mut().map_err(|_| {
            FankorErrorCode::ZeroCopyPossibleDeadlock {
                type_name: std::any::type_name::<Self>(),
            }
        })?;

        let bytes = &mut bytes[self.offset..];
        let mut writer = BpfWriter::new(bytes);
        u16::serialize(&len, &mut writer)?;

        Ok(())
    }

    /// Writes the root position in the map.
    fn write_root_position(&self, root_position: u16) -> FankorResult<()> {
        let mut bytes = self.info.data.try_borrow_mut().map_err(|_| {
            FankorErrorCode::ZeroCopyPossibleDeadlock {
                type_name: std::any::type_name::<Self>(),
            }
        })?;

        let bytes = &mut bytes[self.offset + size_of::<u16>()..];
        let mut writer = BpfWriter::new(bytes);
        u16::serialize(&root_position, &mut writer)?;

        Ok(())
    }

    /// Clears the map, deallocating all memory.
    pub fn clear(&self) -> FankorResult<()> {
        let actual_length = self.len()?;
        let length = Zc::<u16>::new_unchecked(self.info, self.offset);
        length.try_write_value_unchecked(&0)?;

        let root_position = Zc::<u16>::new_unchecked(self.info, self.offset + size_of::<u16>());
        root_position.try_write_value_unchecked(&0)?;

        let content = Zc::<u8>::new_unchecked(self.info, self.offset + size_of::<u16>() * 2);
        content.remove_bytes_unchecked(actual_length as usize * size_of::<Node<K, V>>())?;

        Ok(())
    }
}

impl<
        'info,
        K: Ord + Copy + BorshSerialize + BorshDeserialize + CopyType<'info>,
        V: Copy + BorshSerialize + BorshDeserialize + CopyType<'info>,
    > ZcFnkBVec<'info, K, V>
{
    // GETTERS ----------------------------------------------------------------

    /// Returns a reference to the root value.
    pub fn root_entry(&self) -> FankorResult<Option<(K, V)>> {
        let root_position = self.root_position()?;

        if root_position == 0 {
            Ok(None)
        } else {
            let data = self.get_node(root_position)?;
            Ok(Some((data.key, data.value)))
        }
    }

    /// Returns a mutable reference to the root value.
    pub fn root_entry_mut(&self) -> FankorResult<Option<(K, Zc<'info, V>)>> {
        let root_position = self.root_position()?;

        if root_position == 0 {
            Ok(None)
        } else {
            let data = self.get_node(root_position)?;
            Ok(Some((
                data.key,
                Zc::new_unchecked(self.info, self.content_offset() + size_of::<K>()),
            )))
        }
    }

    // METHODS ----------------------------------------------------------------

    /// Returns the node placed at `index`.
    ///
    /// # Safety
    /// This method is unsafe because it does not check if the index is in bounds.
    fn get_node(&self, index: u16) -> FankorResult<Node<K, V>> {
        let mut offset = self.content_offset();
        offset += index as usize * size_of::<Node<K, V>>();

        let bytes =
            self.info
                .data
                .try_borrow()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;

        Ok(<Node<K, V>>::deserialize(&mut &bytes[offset..])?)
    }

    /// Returns the value of the node placed at `index`.
    ///
    /// # Safety
    /// This method is unsafe because it does not check if the index is in bounds.
    fn get_node_key(&self, index: u16) -> FankorResult<K> {
        let mut offset = self.content_offset();
        offset += index as usize * size_of::<Node<K, V>>();

        let bytes =
            self.info
                .data
                .try_borrow()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;

        Ok(K::deserialize(&mut &bytes[offset..])?)
    }

    /// Returns the value of the node placed at `index`.
    ///
    /// # Safety
    /// This method is unsafe because it does not check if the index is in bounds.
    fn get_node_value(&self, index: u16) -> FankorResult<V> {
        let mut offset = self.content_offset();
        offset += index as usize * size_of::<Node<K, V>>();
        offset += size_of::<K>();

        let bytes =
            self.info
                .data
                .try_borrow()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;

        Ok(V::deserialize(&mut &bytes[offset..])?)
    }

    /// Returns the left child of the node placed at `index`.
    ///
    /// # Safety
    /// This method is unsafe because it does not check if the index is in bounds.
    fn get_node_left_child_at(&self, index: u16) -> FankorResult<u16> {
        let mut offset = self.content_offset();
        offset += index as usize * size_of::<Node<K, V>>();
        offset += size_of::<K>() + size_of::<V>();

        let bytes =
            self.info
                .data
                .try_borrow()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;

        Ok(u16::deserialize(&mut &bytes[offset..])?)
    }

    /// Returns the right child of the node placed at `index`.
    ///
    /// # Safety
    /// This method is unsafe because it does not check if the index is in bounds.
    fn get_node_right_child_at(&self, index: u16) -> FankorResult<u16> {
        let mut offset = self.content_offset();
        offset += index as usize * size_of::<Node<K, V>>();
        offset += size_of::<K>() + size_of::<V>() + size_of::<u16>();

        let bytes =
            self.info
                .data
                .try_borrow()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;

        Ok(u16::deserialize(&mut &bytes[offset..])?)
    }

    /// Returns the height of the node placed at `index`.
    ///
    /// # Safety
    /// This method is unsafe because it does not check if the index is in bounds.
    fn get_node_height(&self, index: u16) -> FankorResult<u8> {
        let mut offset = self.content_offset();
        offset += index as usize * size_of::<Node<K, V>>();
        offset += size_of::<K>() + size_of::<V>() + size_of::<u16>() * 2;

        let bytes =
            self.info
                .data
                .try_borrow()
                .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                    type_name: std::any::type_name::<Self>(),
                })?;

        Ok(u8::deserialize(&mut &bytes[offset..])?)
    }

    /// Returns a node's value ZC placed at `index`.
    ///
    /// # Safety
    /// This method is unsafe because it does not check if the index is in bounds.
    fn get_node_value_zc(&self, index: u16) -> FankorResult<Zc<'info, V>> {
        let mut offset = self.content_offset();
        offset += index as usize * size_of::<Node<K, V>>();
        Ok(Zc::new_unchecked(self.info, offset + size_of::<K>()))
    }

    /// Writes a node at `index`.
    ///
    /// # Safety
    /// This method is unsafe because it does not check if the index is in bounds.
    fn write_node(&self, index: u16, node: &Node<K, V>) -> FankorResult<()> {
        let mut offset = self.content_offset();
        offset += index as usize * size_of::<Node<K, V>>();

        let mut bytes = self.info.data.try_borrow_mut().map_err(|_| {
            FankorErrorCode::ZeroCopyPossibleDeadlock {
                type_name: std::any::type_name::<Self>(),
            }
        })?;

        let bytes = &mut bytes[offset..];
        let mut writer = BpfWriter::new(bytes);
        node.key.serialize(&mut writer)?;
        node.value.serialize(&mut writer)?;
        node.left_child_at.serialize(&mut writer)?;
        node.right_child_at.serialize(&mut writer)?;
        node.height.serialize(&mut writer)?;

        Ok(())
    }

    /// Writes the value of a node at `index`.
    ///
    /// # Safety
    /// This method is unsafe because it does not check if the index is in bounds.
    fn write_node_value(&self, index: u16, value: &V) -> FankorResult<()> {
        let mut offset = self.content_offset();
        offset += index as usize * size_of::<Node<K, V>>();
        offset += size_of::<K>();

        let mut bytes = self.info.data.try_borrow_mut().map_err(|_| {
            FankorErrorCode::ZeroCopyPossibleDeadlock {
                type_name: std::any::type_name::<Self>(),
            }
        })?;

        let bytes = &mut bytes[offset..];
        let mut writer = BpfWriter::new(bytes);
        value.serialize(&mut writer)?;

        Ok(())
    }

    /// Writes the left child of a node at `index`.
    ///
    /// # Safety
    /// This method is unsafe because it does not check if the index is in bounds.
    fn write_node_left_child_at(&self, index: u16, left_child_at: u16) -> FankorResult<()> {
        let mut offset = self.content_offset();
        offset += index as usize * size_of::<Node<K, V>>();
        offset += size_of::<K>() + size_of::<V>();

        let mut bytes = self.info.data.try_borrow_mut().map_err(|_| {
            FankorErrorCode::ZeroCopyPossibleDeadlock {
                type_name: std::any::type_name::<Self>(),
            }
        })?;

        let bytes = &mut bytes[offset..];
        let mut writer = BpfWriter::new(bytes);
        left_child_at.serialize(&mut writer)?;

        Ok(())
    }

    /// Writes the right child of a node at `index`.
    ///
    /// # Safety
    /// This method is unsafe because it does not check if the index is in bounds.
    fn write_node_right_child_at(&self, index: u16, right_child_at: u16) -> FankorResult<()> {
        let mut offset = self.content_offset();
        offset += index as usize * size_of::<Node<K, V>>();
        offset += size_of::<K>() + size_of::<V>() + size_of::<u16>();

        let mut bytes = self.info.data.try_borrow_mut().map_err(|_| {
            FankorErrorCode::ZeroCopyPossibleDeadlock {
                type_name: std::any::type_name::<Self>(),
            }
        })?;

        let bytes = &mut bytes[offset..];
        let mut writer = BpfWriter::new(bytes);
        right_child_at.serialize(&mut writer)?;

        Ok(())
    }

    /// Writes the height of a node at `index`.
    ///
    /// # Safety
    /// This method is unsafe because it does not check if the index is in bounds.
    fn write_node_height(&self, index: u16, height: u8) -> FankorResult<()> {
        let mut offset = self.content_offset();
        offset += index as usize * size_of::<Node<K, V>>();
        offset += size_of::<K>() + size_of::<V>() + size_of::<u16>() * 2;

        let mut bytes = self.info.data.try_borrow_mut().map_err(|_| {
            FankorErrorCode::ZeroCopyPossibleDeadlock {
                type_name: std::any::type_name::<Self>(),
            }
        })?;

        let bytes = &mut bytes[offset..];
        let mut writer = BpfWriter::new(bytes);
        height.serialize(&mut writer)?;

        Ok(())
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: &K) -> FankorResult<Option<V>> {
        let root_position = self.root_position()?;
        if root_position == 0 {
            return Ok(None);
        }

        let mut current_element = self.get_node(root_position - 1)?;
        loop {
            match key.cmp(&current_element.key) {
                Ordering::Less => {
                    if current_element.left_child_at == 0 {
                        break;
                    }

                    current_element = self.get_node(current_element.left_child_at - 1)?;
                }
                Ordering::Greater => {
                    if current_element.right_child_at == 0 {
                        break;
                    }

                    current_element = self.get_node(current_element.right_child_at - 1)?;
                }
                Ordering::Equal => {
                    return Ok(Some(current_element.value));
                }
            }
        }

        Ok(None)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut(&self, key: &K) -> FankorResult<Option<Zc<'info, V>>> {
        let root_position = self.root_position()?;
        if root_position == 0 {
            return Ok(None);
        }

        let mut next_position = root_position;
        loop {
            let current_element = self.get_node(next_position - 1)?;

            match key.cmp(&current_element.key) {
                Ordering::Less => {
                    if current_element.left_child_at == 0 {
                        return Ok(None);
                    }

                    next_position = current_element.left_child_at;
                }
                Ordering::Greater => {
                    if current_element.right_child_at == 0 {
                        return Ok(None);
                    }

                    next_position = current_element.right_child_at;
                }
                Ordering::Equal => {
                    break;
                }
            }
        }

        Ok(Some(self.get_node_value_zc(next_position - 1)?))
    }

    /// Returns true if the key is in the map, else false.
    pub fn contains_key(&self, key: &K) -> FankorResult<bool> {
        Ok(self.get(key)?.is_some())
    }

    /// Inserts a new element into the vector. It will panic if the maximum
    /// number of nodes is exceeded. If the key already exists, it will
    /// overwrite the value and return the old one.
    pub fn insert(&self, key: K, value: V) -> FankorResult<Option<V>> {
        let length = self.len()?;

        let root_position = self.root_position()?;
        if root_position == 0 {
            // Insert node.
            self.write_node(0, &Node::new(key, value))?;
            self.write_len(length + 1)?;
            self.write_root_position(length + 1)?;

            return Ok(None);
        }

        let old_value = None;
        let mut parents = [0u16; MAX_HEIGHT];
        let mut parent_left_direction = [false; MAX_HEIGHT];
        let mut parent_index = 0;

        parents[0] = root_position;
        parent_left_direction[0] = false;

        // Go down finding the position where to insert the new node.
        loop {
            let node_position = parents[parent_index];
            let node = self.get_node(node_position - 1)?;

            match node.key.cmp(&key) {
                Ordering::Greater => {
                    if node.left_child_at == 0 {
                        // Insert node and update parent.
                        self.write_node(0, &Node::new(key, value))?;
                        self.write_len(length + 1)?;

                        let new_index = length + 1;
                        self.write_node_left_child_at(node_position - 1, new_index)?;

                        break;
                    }

                    parent_index += 1;
                    parents[parent_index] = node.left_child_at;
                    parent_left_direction[parent_index] = true;
                }
                Ordering::Less => {
                    if node.right_child_at == 0 {
                        // Insert node and update parent.
                        self.write_node(0, &Node::new(key, value))?;
                        self.write_len(length + 1)?;

                        let new_index = length + 1;
                        self.write_node_right_child_at(node_position - 1, new_index)?;

                        break;
                    }

                    parent_index += 1;
                    parents[parent_index] = node.right_child_at;
                    parent_left_direction[parent_index] = false;
                }
                Ordering::Equal => {
                    self.write_node_value(node_position - 1, &value)?;

                    // We do not need to rebalance the tree.
                    return Ok(Some(node.value));
                }
            }
        }

        // Go up balancing nodes and adjusting sizes.
        loop {
            let node_position = parents[parent_index];
            let (subtree_root, rebalanced) = self.rebalance_node(node_position)?;

            if !rebalanced {
                if parent_index == 0 {
                    break;
                }

                parent_index -= 1;
                continue;
            }

            // Update parent.
            if parent_index == 0 {
                self.write_root_position(subtree_root)?;
            } else {
                let parent_position = parents[parent_index - 1] - 1;
                if parent_left_direction[parent_index] {
                    self.write_node_left_child_at(parent_position, subtree_root)?;
                } else {
                    self.write_node_right_child_at(parent_position, subtree_root)?;
                }
            }

            break;
        }

        Ok(old_value)
    }

    /// Removes the entry from the map and returns its value.
    pub fn remove(&self, key: &K) -> FankorResult<Option<V>> {
        let root_position = self.root_position()?;
        if root_position == 0 {
            return Ok(None);
        }

        let mut parents = [0u16; MAX_HEIGHT];
        let mut parent_left_direction = [false; MAX_HEIGHT];
        let mut parent_index = 0;
        let to_remove_position;

        parents[0] = root_position;
        parent_left_direction[0] = false;

        // Go down finding the position of the element to remove.
        loop {
            let node_position = parents[parent_index];
            let node = self.get_node(node_position - 1)?;

            match node.key.cmp(key) {
                Ordering::Greater => {
                    if node.left_child_at == 0 {
                        return Ok(None);
                    }

                    parent_index += 1;
                    parents[parent_index] = node.left_child_at;
                    parent_left_direction[parent_index] = true;
                }
                Ordering::Less => {
                    if node.right_child_at == 0 {
                        return Ok(None);
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
            let node_to_remove = self.get_node(to_remove_position - 1)?;
            if node_to_remove.right_child_at != 0 {
                let node_to_remove_parent_index = parent_index;
                let node_to_remove_direction = parent_left_direction[parent_index];
                let right_child_position = node_to_remove.right_child_at;
                let right_node = self.get_node(right_child_position - 1)?;

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
                    let node_to_remove = self.get_node(to_remove_position - 1)?;
                    let node_to_remove_left_child_at = node_to_remove.left_child_at;
                    self.write_node_left_child_at(
                        right_child_position - 1,
                        node_to_remove_left_child_at,
                    )?;
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
                    let mut min_node = self.get_node(node_to_remove.right_child_at - 1)?;
                    while min_node.left_child_at != 0 {
                        parent_index += 1;
                        parents[parent_index] = min_node.left_child_at;
                        parent_left_direction[parent_index] = true;

                        min_node = self.get_node(min_node.left_child_at - 1)?;
                    }

                    // Replace node to remove in parents by min node.
                    parents[node_to_remove_parent_index] = parents[parent_index];

                    // Unlink min node from parent.
                    let min_node_position = parents[parent_index];
                    let min_node_parent_position = parents[parent_index - 1];
                    let min_node = self.get_node(min_node_position - 1)?;
                    let min_node_right_child_at = min_node.right_child_at;
                    debug_assert_eq!(min_node.left_child_at, 0);

                    if parent_left_direction[parent_index] {
                        self.write_node_left_child_at(
                            min_node_parent_position - 1,
                            min_node_right_child_at,
                        )?;
                    } else {
                        self.write_node_right_child_at(
                            min_node_parent_position - 1,
                            min_node_right_child_at,
                        )?;
                    }

                    // Link min node to node-to-remove's children.
                    let node_to_remove = self.get_node(to_remove_position - 1)?;
                    let node_to_remove_left_child_at = node_to_remove.left_child_at;
                    let node_to_remove_right_child_at = node_to_remove.right_child_at;
                    self.write_node_left_child_at(
                        min_node_position - 1,
                        node_to_remove_left_child_at,
                    )?;
                    self.write_node_right_child_at(
                        min_node_position - 1,
                        node_to_remove_right_child_at,
                    )?;

                    // Remove min node from parents at the last position.
                    parent_index -= 1;
                }

                // Update parent.
                if node_to_remove_parent_index == 0 {
                    self.write_root_position(parents[node_to_remove_parent_index])?;
                } else {
                    let parent_index = parents[node_to_remove_parent_index - 1] - 1;

                    if node_to_remove_direction {
                        self.write_node_left_child_at(
                            parent_index,
                            parents[node_to_remove_parent_index],
                        )?;
                    } else {
                        self.write_node_right_child_at(
                            parent_index,
                            parents[node_to_remove_parent_index],
                        )?;
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
                    self.write_root_position(left_child_at)?;
                } else {
                    let parent_position = parents[parent_index - 1] - 1;
                    if parent_left_direction[parent_index] {
                        self.write_node_left_child_at(parent_position, left_child_at)?;
                    } else {
                        self.write_node_right_child_at(parent_position, left_child_at)?;
                    }
                }

                parent_index = parent_index.saturating_sub(1);
            }
        }

        // Rebalance parents.
        {
            for parent_index in (0..=parent_index).rev() {
                let (subtree_root, rebalanced) = self.rebalance_node(parents[parent_index])?;

                if !rebalanced {
                    continue;
                }

                // Update parent.
                if parent_index == 0 {
                    self.write_root_position(subtree_root)?;
                } else {
                    let parent_position = parents[parent_index - 1] - 1;
                    if parent_left_direction[parent_index] {
                        self.write_node_left_child_at(parent_position, subtree_root)?;
                    } else {
                        self.write_node_right_child_at(parent_position, subtree_root)?;
                    }
                }
            }
        }

        // Remove node and swap it with last node.
        let last_node_position = self.len()?;
        let old_node_value = self.get_node_value(to_remove_position - 1)?;
        let last_node = self.get_node(last_node_position - 1)?;
        self.write_node(to_remove_position - 1, &last_node)?;

        // Remove bytes from the last element.
        let node_size = Node::<K, V>::byte_size();
        let last_element_offset =
            self.content_offset() + (last_node_position as usize - 1) * node_size;
        let zc = Zc::<()>::new_unchecked(self.info, last_element_offset);
        zc.remove_bytes_unchecked(node_size)?;

        // Fix position of content.
        if root_position == last_node_position {
            self.write_root_position(to_remove_position)?;
        } else if last_node_position != to_remove_position {
            let last_node_key = self.get_node_key(to_remove_position - 1)?;
            let mut current_position = root_position;

            loop {
                let node = self.get_node(current_position - 1)?;

                match node.key.cmp(&last_node_key) {
                    Ordering::Greater => {
                        if node.left_child_at == last_node_position {
                            self.write_node_left_child_at(
                                current_position - 1,
                                to_remove_position,
                            )?;
                            break;
                        }

                        debug_assert_ne!(node.left_child_at, 0);

                        current_position = node.left_child_at;
                    }
                    Ordering::Less => {
                        if node.right_child_at == last_node_position {
                            self.write_node_right_child_at(
                                current_position - 1,
                                to_remove_position,
                            )?;
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

        Ok(Some(old_node_value))
    }

    /// Returns an iterator over the map.
    pub fn iter(&self) -> FankorResult<Iter<'info, K, V>> {
        if self.is_empty()? {
            return Ok(Iter {
                info: self.info,
                offset: self.offset,
                parents: [0; 23],
                parent_index: 0,
                _data: PhantomData,
            });
        }

        let mut parents = [0u16; 23];
        parents[0] = self.root_position()?;

        let mut parent_index = 1u8;

        // Get left most node.
        let mut left_child_at = self.get_node_left_child_at(parents[0] - 1)?;
        while left_child_at != 0 {
            parent_index += 1;
            parents[parent_index as usize - 1] = left_child_at;

            left_child_at = self.get_node_left_child_at(left_child_at - 1)?;
        }

        Ok(Iter {
            info: self.info,
            offset: self.offset,
            parents,
            parent_index,
            _data: PhantomData,
        })
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
    fn rotate_left(&self, node_position: u16) -> FankorResult<u16> {
        let right_position = self.get_node_right_child_at(node_position - 1)?;
        let node_right_child_at = self.get_node_left_child_at(right_position - 1)?;

        self.write_node_left_child_at(right_position - 1, node_position)?;
        self.write_node_right_child_at(node_position - 1, node_right_child_at)?;

        self.adjust_height(node_position)?;
        self.adjust_height(right_position)?;

        Ok(right_position)
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
    fn rotate_right(&self, node_position: u16) -> FankorResult<u16> {
        let left_position = self.get_node_left_child_at(node_position - 1)?;
        let node_left_child_at = self.get_node_right_child_at(left_position - 1)?;

        self.write_node_right_child_at(left_position - 1, node_position)?;
        self.write_node_left_child_at(node_position - 1, node_left_child_at)?;

        self.adjust_height(node_position)?;
        self.adjust_height(left_position)?;

        Ok(left_position)
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
    fn rebalance_node(&self, node_position: u16) -> FankorResult<(u16, bool)> {
        let left_child_height = self.left_height(node_position)?;
        let right_child_height = self.right_height(node_position)?;

        if left_child_height > right_child_height + 1 {
            // Rebalance right.
            let left_child_at = self.get_node_left_child_at(node_position - 1)?;

            if self.right_height(left_child_at)? > self.left_height(left_child_at)? {
                let left_child_at = self.rotate_left(left_child_at)?;
                self.write_node_left_child_at(node_position - 1, left_child_at)?;
            }

            Ok((self.rotate_right(node_position)?, true))
        } else if right_child_height > left_child_height + 1 {
            // Rebalance left.
            let right_child_at = self.get_node_right_child_at(node_position - 1)?;

            if self.left_height(right_child_at)? > self.right_height(right_child_at)? {
                let right_child_at = self.rotate_right(right_child_at)?;
                self.write_node_right_child_at(node_position - 1, right_child_at)?;
            }

            Ok((self.rotate_left(node_position)?, true))
        } else {
            // Adjust balance.
            self.write_node_height(node_position - 1, left_child_height.max(right_child_height))?;

            Ok((node_position, false))
        }
    }

    fn left_height(&self, node_position: u16) -> FankorResult<u8> {
        let left_child_at = self.get_node_left_child_at(node_position - 1)?;

        if left_child_at == 0 {
            Ok(0)
        } else {
            let height = self.get_node_height(left_child_at - 1)?;
            Ok(height + 1)
        }
    }

    fn right_height(&self, node_position: u16) -> FankorResult<u8> {
        let right_child_at = self.get_node_right_child_at(node_position - 1)?;

        if right_child_at == 0 {
            Ok(0)
        } else {
            let height = self.get_node_height(right_child_at - 1)?;
            Ok(height + 1)
        }
    }

    fn adjust_height(&self, node_position: u16) -> FankorResult<()> {
        let left_child_height = self.left_height(node_position)?;
        let right_child_height = self.right_height(node_position)?;

        self.write_node_height(node_position - 1, left_child_height.max(right_child_height))?;

        Ok(())
    }
}

impl<'info, K: CopyType<'info>, V: CopyType<'info>> ZeroCopyType<'info> for ZcFnkBVec<'info, K, V> {
    fn new(info: &'info AccountInfo<'info>, offset: usize) -> FankorResult<(Self, Option<usize>)> {
        Ok((
            ZcFnkBVec {
                info,
                offset,
                _data: PhantomData,
            },
            None,
        ))
    }

    fn read_byte_size_from_bytes(bytes: &[u8]) -> FankorResult<usize> {
        let mut bytes2 = bytes;
        let size = u16::deserialize(&mut bytes2)?;

        // Size field + root position + nodes
        let final_size =
            size_of::<u16>() + size_of::<u16>() + size as usize * Node::<K, V>::byte_size();

        Ok(final_size)
    }
}

impl<'info, K: CopyType<'info>, V: CopyType<'info>> CopyType<'info> for FnkBVec<K, V> {
    type ZeroCopyType = ZcFnkBVec<'info, K, V>;

    fn byte_size_from_instance(&self) -> usize {
        // Size field + root position + nodes
        size_of::<u16>() + size_of::<u16>() + self.len() * Node::<K, V>::byte_size()
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub struct Iter<'info, K, V> {
    pub(crate) info: &'info AccountInfo<'info>,
    pub(crate) offset: usize,
    pub(crate) parents: [u16; MAX_HEIGHT],
    /// Zero means empty.
    /// One means the first parent.
    pub(crate) parent_index: u8,
    pub(crate) _data: PhantomData<(K, V)>,
}

impl<
        'info,
        K: Ord + Copy + BorshSerialize + BorshDeserialize + CopyType<'info>,
        V: Copy + BorshSerialize + BorshDeserialize + CopyType<'info>,
    > Iterator for Iter<'info, K, V>
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.parent_index == 0 {
            return None;
        }

        let (zc, _) = ZcFnkBVec::<K, V>::new(self.info, self.offset).unwrap();
        let node_position = self.parents[self.parent_index as usize - 1];
        let node = zc
            .get_node(node_position - 1)
            .expect("Cannot read node from FnkBVec iterator");
        self.parent_index -= 1;

        if node.right_child_at != 0 {
            self.parent_index += 1;
            self.parents[self.parent_index as usize - 1] = node.right_child_at;

            // Get left most node.
            let mut left_child_at = zc
                .get_node_left_child_at(node.right_child_at - 1)
                .expect("Cannot read node from FnkBVec iterator");

            while left_child_at != 0 {
                self.parent_index += 1;
                self.parents[self.parent_index as usize - 1] = left_child_at;

                left_child_at = zc
                    .get_node_left_child_at(left_child_at - 1)
                    .expect("Cannot read node from FnkBVec iterator");
            }
        }

        Some((node.key, node.value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (zc, _) = ZcFnkBVec::<K, V>::new(self.info, self.offset).unwrap();
        let size = zc.len().expect("Cannot read size from FnkBVec iterator");

        (size as usize, Some(size as usize))
    }
}

impl<
        'info,
        K: Ord + Copy + BorshSerialize + BorshDeserialize + CopyType<'info>,
        V: Copy + BorshSerialize + BorshDeserialize + CopyType<'info>,
    > ExactSizeIterator for Iter<'info, K, V>
{
}
