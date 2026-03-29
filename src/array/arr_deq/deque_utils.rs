use core::{mem::MaybeUninit, ops::Range, ptr::NonNull};

use crate::array::{Array, arr_deq::ArrDeqRepr, helper::*};

/// Returns the ranges from the invariant.
pub(crate) const fn slice_ranges(
    head: usize,
    len: usize,
    cap: usize,
) -> (Range<usize>, Range<usize>) {
    debug_assert!(head <= cap);
    debug_assert!(len <= cap);

    let after_head = cap - head;
    if after_head >= len {
        (head..head + len, 0..0)
    } else {
        let tail_len = len - after_head;
        (head..cap, 0..tail_len)
    }
}

pub(crate) const fn wrapping_idx(logical: usize, cap: usize) -> usize {
    debug_assert!(logical == 0 || logical < 2 * cap);
    let phys = if logical >= cap {
        logical - cap
    } else {
        logical
    };
    debug_assert!(phys == 0 || phys < cap);
    phys
}

pub(crate) const fn phys_idx_of(idx: usize, head: usize, cap: usize) -> usize {
    // FIXME: This needs explanation for its correctness, especially for ZSTs
    wrapping_idx(head.wrapping_add(idx), cap)
}

/// # Safety
/// - `start <= end <= slice.len()`
/// - `slice` is valid for reads. The returned pointers are too.
/// - `slice[start..end]` is initalized
/// - If `slice` is valid for writes, then so are the returned pointers
const unsafe fn subslice_init_nonnull<T>(
    slice: NonNull<[MaybeUninit<T>]>,
    Range { start, end }: Range<usize>,
) -> NonNull<[T]> {
    extern crate self as gnat;
    debug_assert!(start <= end);
    debug_assert!(end <= slice.len());
    // SAFETY: Must be
    NonNull::slice_from_raw_parts(unsafe { slice.cast().add(start) }, end - start)
}

/// Combined implementation akin to `VecDeque::as_(mut)_slices`.
///
/// # Safety
/// Must be called with valid fields of an ArrDeqApi/ArrDeqDrop,
/// the buf's len being the backing array's.
/// The returned slices are those mentioned in the safety invariants.
pub(crate) const unsafe fn as_nonnull_slices<T>(
    buf: NonNull<[MaybeUninit<T>]>,
    head: usize,
    len: usize,
) -> (NonNull<[T]>, NonNull<[T]>) {
    debug_assert!(len <= buf.len());
    debug_assert!(head <= buf.len());
    let (lhs, rhs) = slice_ranges(head, len, buf.len());
    // SAFETY: `slice_ranges` always returns ranges in the initialized parts of the deque.
    unsafe {
        (
            subslice_init_nonnull(buf, lhs),
            subslice_init_nonnull(buf, rhs),
        )
    }
}

impl<A: Array> ArrDeqRepr<A> {
    pub(crate) const fn phys_idx_of(&self, idx: usize) -> usize {
        phys_idx_of(idx, self.head, arr_len::<A>())
    }
    pub(crate) const fn tail(&self) -> usize {
        self.phys_idx_of(self.len)
    }
    pub(crate) const fn phys_idx_before_head(&self, idx: usize) -> usize {
        let cap = arr_len::<A>();
        wrapping_idx(self.head.wrapping_sub(idx).wrapping_add(cap), cap)
    }

    /// # Safety
    /// The repr must come from an ArrDeqApi.
    /// The element at `self.arr[idx]` must be initialized and never used again
    /// until overwritten (including drops)
    pub(crate) const unsafe fn phys_read(&self, idx: usize) -> A::Item {
        // SAFETY: `idx` is initialized and never used again
        unsafe { self.arr.as_slice()[idx].assume_init_read() }
    }
}
