# rust-node-map


Problems I am currently facing:

1) The biggest problem I am facing is that I can not get a mutable reference to a BLOCK or a PAGE. This is because the map storing the blocks or pages contains an Rc reference to each page, or nodes... How can I mutate a value in these maps, while the maps are still holding RO references to pages and nodes? Is this the proper application for a `Cell`? Should I do: `Rc<Cell<Page<ValueType>>>` for example? Would this enable me to have as many RO references as needed to the `Rc`, while only having a single entity access the `Cell` at any given time?
2) Because BLOCK and PAGE structs/code was repeated, I tried to make this code generic over `ItemType`... however, this forced me to create a "Chunkable" trait, and possibly made the code more complicated... I will likely just duplicate the code between a Block and a Page in the future. Any ideas on how to make this cleaner while keeping the code dry?
3) I eventually intend to make this NodeMap thread-safe... but that will come after I have gotten it working for a single thread.

Questions:

1) Why wouldn't you just use a BST or some other data structure to hold the index for the map? `Because I am attempting to make this efficient for writes as well as reads... and I haven't yet found a good way to write BSTs in an efficient way` (they are always inefficient for writes, or for reads... you need to choose one or the other).
2) Is this the final structure? `No, actually... this is just a start to help me learn Rust and profile the code. I may end up doing something entirely different by the time I am done`
3) What is this for? `When it is all complete, it will be a database index, to store values and their related document_ids... I wanted to try an implement it as a generic data structure however, mostly just to try and learn Rust using generics, and common data structures`.

Idea:

In simple terms, the file format for the persisted index will be as follows:

#### {HEADER}
#### {INDEX} - can be any size, usually from 2-1048575 u64s
    This index will store offsets in the file to "blocks". The index is calculated by "crunching" incoming values to be stored in the map (see crunching). The crunching opperation will take a value and "crunch" it into this index range, allowing us to offset into this index with the crunched value to find a block offset to start searching.
#### {PAGE} - Can be stored anywhere in the file (pages can be moved on-the-fly to keep storage space efficient). Each page contains an {INDEX}, but this index will only ever be a size between 2-256 (2 for booleans). This index, like the master block {INDEX}, contains node offsets ("nodes" are collections of items) into the current page. The offset into this index is calculated by "crunching" a value.

    {PAGE: SMALLEST_VALUE,LARGEST_VALUE,{INDEX}{NODES...} }
    {PAGE: SMALLEST_VALUE,LARGEST_VALUE,{INDEX}{NODES...} }
    {PAGE: SMALLEST_VALUE,LARGEST_VALUE,{INDEX}{NODES...} }

#### {BLOCK} - Same structure as a page, can be stored anywhere in the file.

    {BLOCK: SMALLEST_VALUE,LARGEST_VALUE,{PAGES...} }
    {BLOCK: SMALLEST_VALUE,LARGEST_VALUE,{PAGES...} }
    {BLOCK: SMALLEST_VALUE,LARGEST_VALUE,{PAGES...} }

#### {NODE} - A node belongs to a PAGE, and is a collection of items. Contained within the PAGE "chunk" in the file. A single node can only contain document_ids of an equal value... in short, a "node" contains document_ids for one (all equal) value.

     {NODE: VALUE:document_ids(items)...}
     {NODE: VALUE:document_ids(items)...}
     {NODE: VALUE:document_ids(items)...}

#### Crunching - Value crunching works as follows (see `index.rs`): A value is "crunched" by linerarly interpolating a value between `smallest_value`, and `largest_value`, such that the resulting "crunched offset" is an index into an INDEX part of the file. The crunched offset will always be between 0..INDEX.len(). The idea being that for a small stored range (for example, values between 0-10), the INDEX will contain higher resolution into the BLOCK offsets. As the range grows, the resolution shrinks, which will require more searching to find the correct insertion/retreival block. However, this is a payoff to save on index size. The index can not continually grow, or it would simply get too large, so instead this "value crunching" system is meant to keep it as efficient as possible when the range of values is small, and still keep it (mostly) efficient by still providing BLOCK offsets when the range is much larger.

The "crunching" algorithm lerps between Type::MIN and Type::MAX at its worst (offseting into the index range such that the crunched index offset is always contained within the actual index size). This might require more searching for the actual block needed, as the index will simply provide a block offset for the closests block that can be stored in the index.

#### Psuedo code

To give a better idea of how this is intended to work, read the following psuedo code:

```rust
impl Nodes {
  fn insert_into_items(document_id: u128, value: ValueType) {
    let items_found = self.find_items_containing(value);
    match items_found {
      Some(items) => {
        items.insert(document_id);
      },
      None => {
        let mut items = self.create_new_items();
        items.insert(document_id);
      }
    }
  }
}

impl Page { // "Chunk" struct in the code (generic over ItemType)
  fn insert_into_page(document_id: u128, value: ValueType) {
    let index_offset = ValueType::node_map_crunch(self.smallest_value, self.largest_value); // Lerp the incoming value into the range of the index so we can find a page offset
    let page_offset = self.index[index_offset]; // fetch the file offset out of the index for the closest matching page
    let page = load_page_at_offset(block_offset); // The page_offset is simply a starting point for searching for the correct page

    // Finally, insert into the page, which will find the nearest items and insert the document_id into those items.
    block.insert(document_id, value);
  }
}

impl Block { // "Chunk" struct in the code (generic over ItemType)
  // document_id (called "item" in the code) will be inserted for this value... multiple document_ids can be inserted into the map for a given value
  fn insert_into_block(document_id: u128, value: ValueType) {
    let index_offset = ValueType::node_map_crunch(self.smallest_value, self.largest_value); // Lerp the incoming value into the range of the index so we can find a block_offset
    let block_offset = self.index[index_offset]; // fetch the file offset out of the index for the closest block
    let block = load_block_at_offset(block_offset); // The block_offset is simply a starting point for searching for the correct block

    // Finally, insert into the block, which will find the nearest page (repeating the code above for the page, and using a smaller index per-page), and insert into the page... inserting into the page will finally insert into the items.
    block.insert(document_id, value);
  }
}

impl NodeMap {
  pub fn insert(document_id: u128, value: ValueType) {
    self.root_block.insert_into_block(document_id, value);
  }
}
```
