//! Provide some extensions of `alloc::collections`.

// -----------------------------------------------------------------------------
// Modules

mod array_deque;
mod block_list;
mod bloom_filter;
mod page_pool;
mod typeid_map;

// -----------------------------------------------------------------------------
// Exports

pub use array_deque::ArrayDeque;
pub use block_list::BlockList;
pub use bloom_filter::BloomFilter;
pub use page_pool::PagePool;
pub use typeid_map::TypeIdMap;
