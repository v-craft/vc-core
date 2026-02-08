use alloc::vec::Vec;
use core::mem::ManuallyDrop;
use core::sync::atomic::Ordering;

use vc_os::sync::SyncUnsafeCell;
use vc_os::sync::atomic::AtomicPtr;

use crate::entity::Entity;
use crate::utils::DebugCheckedUnwrap;

struct Slot {
    inner: SyncUnsafeCell<Entity>,
}

impl Slot {
    const fn empty() -> Self {
        let source = Entity::PLACEHOLDER;
        Self {
            inner: SyncUnsafeCell::new(source),
        }
    }

    #[inline]
    const unsafe fn set(&self, entity: Entity) {
        unsafe {
            self.inner.get().write(entity);
        }
    }

    #[inline]
    const unsafe fn get(&self) -> Entity {
        unsafe { self.inner.get().read() }
    }
}

struct Chunk {
    first: AtomicPtr<Slot>,
}

impl Chunk {
    const fn new() -> Self {
        Self {
            first: AtomicPtr::new(core::ptr::null_mut()),
        }
    }
    #[inline]
    unsafe fn get(&self, index: u32) -> Entity {
        let head = self.first.load(Ordering::Relaxed);
        let target = unsafe { &*head.add(index as usize) };
        unsafe { target.get() }
    }

    #[inline]
    unsafe fn get_slice(&self, index: u32, ideal_len: u32, chunk_capacity: u32) -> &[Slot] {
        let after_index_slice_len = chunk_capacity - index;
        let len = after_index_slice_len.min(ideal_len) as usize;
        let head = self.first.load(Ordering::Relaxed);
        unsafe { core::slice::from_raw_parts(head.add(index as usize), len) }
    }

    #[inline]
    unsafe fn set(&self, index: u32, entity: Entity, chunk_capacity: u32) {
        let ptr = self.first.load(Ordering::Relaxed);

        let head = if ptr.is_null() {
            unsafe { self.init(chunk_capacity) }
        } else {
            ptr
        };

        let target = unsafe { &*head.add(index as usize) };

        unsafe {
            target.set(entity);
        }
    }

    #[cold]
    #[inline(never)]
    unsafe fn init(&self, chunk_capacity: u32) -> *mut Slot {
        let mut buff = ManuallyDrop::new(Vec::new());
        buff.reserve_exact(chunk_capacity as usize);
        buff.resize_with(chunk_capacity as usize, Slot::empty);

        let ptr = buff.as_mut_ptr();
        self.first.store(ptr, Ordering::Relaxed);
        ptr
    }

    unsafe fn dealloc(&mut self, chunk_capacity: u32) {
        let ptr = *self.first.get_mut();
        if !ptr.is_null() {
            // SAFETY: This was created in [`Self::init`] from a standard Vec.
            unsafe {
                Vec::from_raw_parts(ptr, 0, chunk_capacity as usize);
            }
        }
    }
}

const NUM_CHUNKS: u32 = 24;
const NUM_SKIPPED: u32 = u32::BITS - NUM_CHUNKS;

struct FreeBuffer([Chunk; NUM_CHUNKS as usize]);

impl FreeBuffer {
    const fn new() -> Self {
        Self([const { Chunk::new() }; NUM_CHUNKS as usize])
    }

    #[inline]
    const fn capacity_of_chunk(chunk_index: u32) -> u32 {
        // We do this because we're skipping the first `NUM_SKIPPED` powers, so we need to make up for them by doubling the first index.
        // This is why the first 2 indices both have a capacity of 512.
        let corrected = if chunk_index == 0 { 1 } else { chunk_index };
        // We add NUM_SKIPPED because the total capacity should be as if [`Self::NUM_CHUNKS`] were 32.
        // This skips the first NUM_SKIPPED powers.
        let corrected = corrected + NUM_SKIPPED;
        // This bit shift is just 2^corrected.
        1 << corrected
    }

    #[inline]
    const fn index_info(full_index: u32) -> (u32, u32, u32) {
        // We do a `saturating_sub` because we skip the first `NUM_SKIPPED` powers to make space for the first chunk's entity count.
        // The -1 is because this is the number of chunks, but we want the index in the end.
        // We store chunks in smallest to biggest order, so we need to reverse it.
        let chunk_index = (NUM_CHUNKS - 1).saturating_sub(full_index.leading_zeros());
        let chunk_capacity = Self::capacity_of_chunk(chunk_index);
        // We only need to cut off this particular bit.
        // The capacity is only one bit, and if other bits needed to be dropped, `leading` would have been greater
        let index_in_chunk = full_index & !chunk_capacity;

        (chunk_index, index_in_chunk, chunk_capacity)
    }

    #[inline]
    fn index_in_chunk(&self, full_index: u32) -> (&Chunk, u32, u32) {
        let (chunk_index, index_in_chunk, chunk_capacity) = Self::index_info(full_index);
        // SAFETY: The `index_info` is correct.
        let chunk = unsafe { self.0.get_unchecked(chunk_index as usize) };
        (chunk, index_in_chunk, chunk_capacity)
    }

    unsafe fn get(&self, full_index: u32) -> Entity {
        let (chunk, index, _) = self.index_in_chunk(full_index);
        // SAFETY: Ensured by caller.
        unsafe { chunk.get(index) }
    }

    #[inline]
    unsafe fn set(&self, full_index: u32, entity: Entity) {
        let (chunk, index, chunk_capacity) = self.index_in_chunk(full_index);
        // SAFETY: Ensured by caller and that the index is correct.
        unsafe { chunk.set(index, entity, chunk_capacity) }
    }

    #[inline]
    unsafe fn iter(&self, indices: core::ops::Range<u32>) -> FreeBufferIterator<'_> {
        FreeBufferIterator {
            buffer: self,
            future_buffer_indices: indices,
            current_chunk_slice: [].iter(),
        }
    }
}

impl Drop for FreeBuffer {
    fn drop(&mut self) {
        for index in 0..NUM_CHUNKS {
            let capacity = Self::capacity_of_chunk(index);
            // SAFETY: we have `&mut` and the capacity is correct.
            unsafe { self.0[index as usize].dealloc(capacity) };
        }
    }
}

struct FreeBufferIterator<'a> {
    buffer: &'a FreeBuffer,
    /// The indices in the buffer that are not yet in `current_chunk_slice`.
    future_buffer_indices: core::ops::Range<u32>,
    /// The part of the buffer we are iterating at the moment.
    current_chunk_slice: core::slice::Iter<'a, Slot>,
}

impl<'a> Iterator for FreeBufferIterator<'a> {
    type Item = Entity;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(found) = self.current_chunk_slice.next() {
            return Some(unsafe { found.get() });
        }

        let still_need = self.future_buffer_indices.len() as u32;
        if still_need == 0 {
            return None;
        }
        let next_index = self.future_buffer_indices.start;
        let (chunk, index, chunk_capacity) = self.buffer.index_in_chunk(next_index);

        // SAFETY: Assured by `FreeBuffer::iter`
        let slice = unsafe { chunk.get_slice(index, still_need, chunk_capacity) };
        self.future_buffer_indices.start += slice.len() as u32;
        self.current_chunk_slice = slice.iter();

        // SAFETY: Constructor ensures these indices are valid in the buffer; the buffer is not sparse, and we just got the next slice.
        // So the only way for the slice to be empty is if the constructor did not uphold safety.
        let next = unsafe { self.current_chunk_slice.next().debug_checked_unwrap() };
        // SAFETY: We have `&mut self`, so that memory order is certain.
        // The caller of `FreeBuffer::iter` ensures the memory order of this value's lifetime.
        Some(unsafe { next.get() })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.future_buffer_indices.len() + self.current_chunk_slice.len();
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for FreeBufferIterator<'a> {}
impl<'a> core::iter::FusedIterator for FreeBufferIterator<'a> {}
