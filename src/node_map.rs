use super::*;

use super::indexer::NodeMapIndexSize;
use std::{borrow::BorrowMut, collections::HashMap, rc::Rc};

fn set_within(vec: &mut Vec<Index>, range: Range<usize>, index: Index) {
  for i in range {
    vec[i] = index;
  }
}

impl<ValueType, ItemType> Chunkable<ValueType, ItemType> for Node<ValueType>
where
  ValueType: indexer::NodeMapIndexSize<ValueType> + Copy + PartialOrd,
{
  fn new_node(value: ValueType) -> Self {
    Node {
      start_offset: 0,
      end_offset: 0,
      items_value: value,
      item_count: 0,
      items: HashMap::<Index, Item>::new(),
    }
  }

  fn smallest_value(&self) -> ValueType {
    return self.items_value;
  }

  fn largest_value(&self) -> ValueType {
    return self.items_value;
  }

  fn insert(&mut self, item: Item, value: ValueType) {
    self.items.insert(self.item_count, item);
  }
}

impl<ValueType, ItemType> Chunkable<ValueType, ItemType> for Page<ValueType>
where
  ValueType: indexer::NodeMapIndexSize<ValueType> + Copy + PartialOrd,
{
  fn new_node(value: ValueType) -> Self {
    Page {
      start_offset: 0,
      end_offset: 0,
      index: vec![NO_INDEX; 256],
      smallest_value: value,
      largest_value: value,
      node_count: 0,
      nodes: HashMap::<Index, Rc<Node<ValueType>>>::new(),
      _need_index_rebuild: false,
      _need_blocks_rebuild: false,
    }
  }

  fn smallest_value(&self) -> ValueType {
    return self.smallest_value;
  }

  fn largest_value(&self) -> ValueType {
    return self.largest_value;
  }

  fn insert(&mut self, item: Item, value: ValueType) {
    let mut node = self.find_or_create_node_for_value(value);
    Chunkable::<ValueType, Node<ValueType>>::insert(
      Rc::<Node<ValueType>>::get_mut(&mut node).unwrap(),
      item,
      value,
    );
  }
}

impl<ValueType> Node<ValueType> {}

impl<ValueType, ItemType> Chunk<ValueType, ItemType>
where
  ValueType: NodeMapIndexSize<ValueType> + Copy + PartialOrd,
  ItemType: Chunkable<ValueType, ItemType>,
{
  fn new(value: ValueType, index_size: usize) -> Self {
    Chunk {
      start_offset: 0,
      end_offset: 0,
      index: vec![NO_INDEX; index_size],
      smallest_value: value,
      largest_value: value,
      node_count: 0,
      nodes: HashMap::<Index, Rc<ItemType>>::new(),
      _need_index_rebuild: false,
      _need_blocks_rebuild: false,
    }
  }

  fn new_blank(index_size: usize) -> Self {
    Chunk {
      start_offset: 0,
      end_offset: 0,
      index: vec![NO_INDEX; index_size],
      smallest_value: ValueType::node_map_largest_value(),
      largest_value: ValueType::node_map_smallest_value(),
      node_count: 0,
      nodes: HashMap::<Index, Rc<ItemType>>::new(),
      _need_index_rebuild: false,
      _need_blocks_rebuild: false,
    }
  }

  fn new_node(value: ValueType) -> Rc<ItemType> {
    Rc::<_>::new(ItemType::new_node(value))
  }

  fn get_or_load_node(&mut self, index: Index) -> Option<Rc<ItemType>> {
    match self.nodes.get(&index) {
      Some(block) => Some(Rc::clone(block)),
      None => {
        // TODO: Load page
        None
      }
    }
  }

  fn rebuild_index(&mut self) {
    let mut new_index = self.index.clone();
    let smallest_value = self.smallest_value;
    let largest_value = self.largest_value;

    for i in 0..self.index.len() {
      let index = *self.index.get(i).unwrap();
      if index == NO_INDEX {
        continue;
      }

      let node_option = self.get_or_load_node(index);

      match node_option {
        Some(node) => {
          let start_offset = node
            .smallest_value()
            .node_map_crunch(smallest_value, largest_value);

          let end_offset = node
            .largest_value()
            .node_map_crunch(smallest_value, largest_value);

          if start_offset == end_offset {
            new_index[start_offset as usize] = index;
          } else {
            set_within(
              &mut new_index,
              start_offset as usize..(end_offset + 1) as usize,
              index,
            );
          }
        }
        None => {
          new_index[i] = NO_INDEX;
        }
      }
    }
  }

  fn crunch_and_update(&mut self, value: ValueType) -> Index {
    let mut need_index_rebuild = false;

    if value < self.smallest_value {
      self.smallest_value = value;
      need_index_rebuild = true;
    }

    if value > self.largest_value {
      self.largest_value = value;
      need_index_rebuild = true;
    }

    if need_index_rebuild {
      self.rebuild_index()
    }

    value.node_map_crunch(self.smallest_value, self.largest_value)
  }

  fn create_node_for_value(&mut self, value: ValueType) -> Rc<ItemType> {
    let node = Chunk::new_node(value);

    let node_index = self.node_count;
    self.nodes.insert(node_index, Rc::clone(&node));

    self.node_count += 1;
    self._need_blocks_rebuild = true;

    node
  }

  //fn find_page_index_for_value(&self, value: ValueType) -> Index {}

  fn find_or_create_node_for_value(&mut self, value: ValueType) -> Rc<ItemType> {
    let index = self.crunch_and_update(value);
    if index == NO_INDEX {
      return self.create_node_for_value(value);
    }

    match self.get_or_load_node(index) {
      Some(page) => page.clone(),
      None => self.create_node_for_value(value),
    }
  }

  pub fn insert(&mut self, item: Item, value: ValueType) {
    let mut node = self.find_or_create_node_for_value(value);
    Rc::<ItemType>::get_mut(&mut node)
      .unwrap()
      .insert(item, value);
  }
}

impl<'a, ValueType> NodeMap<'a, ValueType>
where
  ValueType: NodeMapIndexSize<ValueType> + Copy + PartialOrd,
{
  pub fn new(storage_path: &'a str) -> Self {
    NodeMap::<ValueType> {
      storage_path,
      root_block: Rc::<_>::new(Block::<ValueType>::new_blank(
        ValueType::node_map_index_size(),
      )),
    }
  }

  pub fn insert(&mut self, item: Item, value: ValueType) {
    Rc::<Chunk<ValueType, Chunk<ValueType, Node<ValueType>>>>::get_mut(&mut self.root_block)
      .unwrap()
      .insert(item, value);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let mut node_map = NodeMap::<u32>::new("test");
    node_map.insert(234, 10);
  }
}
