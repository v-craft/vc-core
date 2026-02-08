# Platform-agnostic Utilities

*Platform-agnostic: No dependencies on atomic variables, sync primitives, or OS APIs like threads and timing.*

## Hash Containers and Extensions

- Re-exports `hashbrown`, `foldhash`, and `indexmap`.
- Provides three fixed hash builders:
    1. `FixedHashState`: Based on `foldhash::fast::FixedHasher`, offers fast and stable hashing.
    2. `SparseHashState`: Designed for "sparse" data, typically integers uniformly distributed
       from `0` (e.g., `EntityId`). The lower bits of the hash match the ID, while the upper bits
       undergo a single multiplication to support SIMD optimizations in *SwissTable*.
    3. `NoOpHashState`: A no-operation hash function that passes through the underlying bytes directly.
       Designed for pre-hashed keys and `TypeId`.
- Provides pre-packaged hash containers based on the above hash builders:
    1. `HashMap` and `HashSet`: Default to `FixedHashState`, interchangeable, built on `hashbrown`.
    2. `SparseHashMap` and `SparseHashSet`: Fixed `SparseHashState`, built on `hashbrown`.
    3. `NoOpHashMap` and `NoOpHashSet`: Fixed `NoOpHashState`, built on `hashbrown`.
    4. `IndexMap` and `IndexSet`: Default to `FixedHashState`, interchangeable, built on `indexmap`.
    5. `SparseIndexMap` and `SparseIndexSet`: Fixed `SparseHashState`, built on `indexmap`.
- Note: The `Index` series maintains insertion order, ensuring deterministic iteration at the cost of
  additional overhead.
- Provides a pre-hashed wrapper `Hashed<T>` that computes the hash once at creation and stores it,
  typically used with `NoOpHashState`.

## Stack-Optimized Linear Collections

- Re-exports `fastvec`, containing three container types:
    1. `StackVec`: A dynamic array stored on the stack with fixed capacity but variable element count.
    2. `FastVec`: Prioritizes stack storage, can fall back to heap allocation. Marked as `!Sync`,
       typically used for temporary data processing.
    3. `AutoVec`: Prioritizes stack storage, can fall back to heap allocation, suitable for long-term storage.

## Additional Extensions

- `TypeIdMap`: A map keyed by `TypeId`.
- `ArrayDeque`: A fixed-capacity circular array stored on the stack.
- `BlockList`: A block-based singly linked list that optimizes cache locality through data blocking,
  with limited free block reuse.
- `BloomFilter`: A simple [Bloom-filter](https://en.wikipedia.org/wiki/Bloom_filter).
- `PagePool`: A simple memory pool supporting insertion but not deletion (except for bulk clearing).
  Manages only memory allocation, not `Drop` semantics for contained elements. Typically used for
  simple data that does not require `Drop`.

## Helper Utilities

- `range_invoke`: A macro that expands and invokes an inner macro multiple times.
- `default`: A convenience function that simplifies `Default::default()`.
- `UnsafeCellDeref`: Provides supplementary methods for `UnsafeCell` to simplify interior mutability
  implementations.

## Thread-Safe Containers

For thread-safe and platform-dependent containers, see `vc_os::utils`.
