extern crate num_traits;

use std::{collections::HashMap, ops::Range, rc::Rc};

mod indexer;
mod node_map;

// naming
// index -> static size, indexes (records offsets) to pages
// chunk -> A contiguous block of data in the file, usually containing an index
// node -> a sub-item of a "chunk", can be a page, a block, or a list of item ids
// page -> a chunk cotaining items
// block -> a chunk containing pages

// file format
// HEADER
// u32  - magic_header: Magic header identifier
// u16  - version: Version of this index file
// u8   - item_size: Value size in bits. A item_size of zero means values have a dynamic size (i.e. strings).
// u64  - block_header_offset: Offset to block header. Can be stored anywhere in the file. Usually stored at the end of the file.
// 113 bytes reserved: HEADER = 128 bytes total

// INDEX
// u32  - magic_header: Magic header identifier for index
// u32  - index_size: Index size
// {value}  - smallest_value: Smallest value stored in entire index
// {value}  - largest_value: Largest value stored in entire index
// index_size * u64: Index into blocks where to start searching (relative to block_header_offset)

// NODES
// u32 - magic_header: Magic header identifier for nodes
// u32 - node_id_count: Number of node IDs
// u128 * node_id_count: Node IDs...

// PAGE: store offsets to nodes
// u32  - magic_header: Magic header identifier for page
// u32  - nodes_count: number of nodes in this page
// u16  - nodes_size: Size in bytes of each nodes entry
// {
// u32  - start_offset: Relative Start offset in file for nodes
// u32  - end_offset: Relative End offset in file for nodes
// {value} - value: Value for these nodes
// }... * nodes_count

// PAGES (blocks): store offsets to pages
// u32  - magic_header: Magic header identifier
// u32  - page_count: Number of pages
// {
// u64  - start_offset: Start offset in file for this page
// u64  - end_offset: End offset in file for this page
// {value}  - smallest_value: Smallest value stored in this page
// {value}  - largest_value: Largest value stored in this page
// 256 * u32 - page_offset_index: Offset into pages
// } ... * page_count

// PAGE [ items, items, items ]
// BLOCK [ page, page, page ]
// INDEX [ block/page, block/page, block/page ]

// HEADER
// INDEX
// PAGE
// BLOCKS

trait Chunkable<ValueType, ItemType>
where
  ValueType: indexer::NodeMapIndexSize<ValueType> + Copy + PartialOrd,
{
  fn new_node(value: ValueType) -> Self;
  fn smallest_value(&self) -> ValueType;
  fn largest_value(&self) -> ValueType;
  fn insert(&mut self, item: Item, value: ValueType);
}

type Item = u128;
type Offset = u64;
type Index = u32;

const NO_OFFSET: Offset = Offset::MAX;
const NO_INDEX: Index = Index::MAX;

struct Node<ValueType> {
  start_offset: Offset,
  end_offset: Offset,
  items_value: ValueType,
  item_count: Index,
  items: HashMap<Index, Item>,
}

type NodePtr<ValueType> = Rc<Node<ValueType>>;

struct Chunk<ValueType, ItemType>
where
  ValueType: indexer::NodeMapIndexSize<ValueType> + Copy + PartialOrd,
  ItemType: Chunkable<ValueType, ItemType>,
{
  start_offset: Offset,
  end_offset: Offset,
  index: Vec<Index>,
  smallest_value: ValueType,
  largest_value: ValueType,
  node_count: Index,
  nodes: HashMap<Index, Rc<ItemType>>,
  _need_index_rebuild: bool,
  _need_blocks_rebuild: bool,
}

type Page<ValueType> = Chunk<ValueType, Node<ValueType>>;
type PagePtr<ValueType> = Rc<Page<ValueType>>;

type Block<ValueType> = Chunk<ValueType, Page<ValueType>>;
type BlockPtr<ValueType> = Rc<Block<ValueType>>;

struct NodeMap<'a, ValueType>
where
  ValueType: indexer::NodeMapIndexSize<ValueType> + Copy + PartialOrd,
{
  storage_path: &'a str,
  root_block: BlockPtr<ValueType>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {}
}
