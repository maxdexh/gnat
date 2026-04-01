use core::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

use crate::{const_fmt, utils};

use super::{ArrApi, ArrDeqApi, Array, helper::*};

/// Wraps the drop impl so it isn't exposed as a trait bound
// NOTE: Mutable access to fields and construction of this struct requires a safety
// comment. Prefer using as_mut_repr and from_repr for this.
#[repr(transparent)]
pub(crate) struct ArrDeqDrop<A: Array<Item = T>, T = <A as Array>::Item> {
    /// # Safety
    /// See [`ArrDeqRepr`].
    repr: ArrDeqRepr<A>,
    _p: PhantomData<T>,
}
impl<A: Array<Item = T>, T> Drop for ArrDeqDrop<A, T> {
    fn drop(&mut self) {
        let (lhs, rhs) = self.as_mut_slices();

        // SAFETY:
        // `as_mut_slices` returns `lhs` and `rhs` from the safety invariants.
        // The deque has ownership over the contained items, so they are considered to stop
        // existing after this drop runs.
        // Since they are behind `MaybeUninit`, which can hold invalid values and does not have
        // drop glue itself, it is safe to drop them here.
        unsafe {
            core::ptr::drop_in_place(lhs);
            core::ptr::drop_in_place(rhs);
        }
    }
}
impl<T, A: Array<Item = T>> ArrDeqDrop<A> {
    const fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        // SAFETY: See below
        let Self { repr, .. } = self;
        let &mut ArrDeqRepr {
            head,
            len,
            ref mut arr,
        } = repr;

        let buf = NonNull::from_mut(arr.as_mut_slice());

        // SAFETY: The invariants of `as_nonnull_slices` are such that it it safe to call with the
        // fields of a `ArrDeqApi`. The returned pointers are derived from the mutable slice such
        // that it is valid to turn them back into mutable references.
        // Also the returned slices can only be used to write valid elements into the valid part of
        // the array.
        unsafe {
            let (mut lhs, mut rhs) = as_nonnull_slices(buf, head, len);
            (lhs.as_mut(), rhs.as_mut())
        }
    }
}

struct ArrDeqRepr<A: Array> {
    /// # Safety
    /// See below
    head: usize,
    /// # Safety
    /// See below
    len: usize,
    /// # Safety
    /// Let `cap := A::Length`. When this struct is wrapped in ArrDeqDrop/ArrDeqApi, the following must hold:
    /// - `len <= cap`
    /// - `head < cap`
    /// - Let (lhs, rhs) be defined as the following places:
    ///   - If `len <= cap - head`, then `lhs = arr[head .. head + len]`, `rhs = arr[0..0]`
    ///   - If `len > cap - head`, then `lhs = arr[head .. cap]`, `rhs = arr[0 .. len - (cap - head)]`
    ///   - Alternatively, `lhs = arr[head .. min(len, cap - head) + head]`, `rhs = arr[0 .. len.saturating_sub(cap - head)]`
    /// - `lhs` and `rhs` must be initialized with valid instances of `A::Item`. The deque is considered to
    ///   have ownership over these.
    arr: ArrApi<MaybeUninit<A>>,
}
mod deque_utils;
use const_util::result::expect_ok;
use deque_utils::*;

// Methods dealing directly with the fields
impl<A: Array<Item = T>, T> ArrDeqApi<A> {
    const fn as_repr(&self) -> &ArrDeqRepr<A> {
        let Self(ArrDeqDrop { repr, .. }, ..) = self;
        repr
    }

    /// # Safety
    /// The invariant of the deque must be upheld. It must also not be possible to break them
    /// through returned references created from this.
    const unsafe fn as_mut_repr(&mut self) -> &mut ArrDeqRepr<A> {
        let Self(ArrDeqDrop { repr, .. }, ..) = self;
        repr
    }

    /// # Safety
    /// `head, len < A::Length`, `len` items are initialized, starting at `head` and wrapping
    /// around the end of the array.
    const unsafe fn from_repr(repr: ArrDeqRepr<A>) -> Self {
        let _ = arr_len::<A>(); // Ensure array non-oversized.

        Self(ArrDeqDrop {
            repr,
            _p: PhantomData,
        })
    }

    const fn into_repr(self) -> ArrDeqRepr<A> {
        // SAFETY: repr(transparent)
        unsafe {
            utils::union_transmute!(
                ArrDeqApi<A>,
                ArrDeqRepr<A>,
                self, //
            )
        }
    }

    /// Returns a mutable reference to the elements as a pair of slices.
    ///
    /// If this deque is contiguous, then the right slice will be empty.
    /// The left slice is empty only when the entire deque is empty.
    ///
    /// Note that the exact distribution between left and right are not guaranteed
    /// unless the length of this deque is 1 or less, or this deque is contiguous
    /// according to an API.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// let mut deq = ArrDeqApi::<[_; 20]>::new();
    /// deq.push_front(1);
    /// deq.push_back(2);
    /// let (lhs, rhs) = deq.as_mut_slices();
    /// assert_eq!(lhs.iter_mut().chain(&mut *rhs).next(), Some(&mut 1));
    /// assert_eq!(lhs.iter_mut().chain(rhs).next_back(), Some(&mut 2));
    /// ```
    pub const fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        let Self(drop, ..) = self;
        drop.as_mut_slices()
    }
}

impl<A: Array<Item = T>, T> ArrDeqApi<A> {}

impl<A: Array<Item = T>, T> ArrDeqApi<A> {
    pub(super) const fn from_vec_impl(vec: crate::array::ArrVecApi<A>) -> Self {
        let (arr, len) = vec.into_uninit_parts();
        // SAFETY: ArrVec is contiguous at the beginning of the array
        unsafe { Self::from_repr(ArrDeqRepr { head: 0, len, arr }) }
    }

    /// Creates a new empty [`ArrDeqApi`].
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// assert_eq!(ArrDeqApi::<[i32; 20]>::new(), []);
    /// ```
    pub const fn new() -> Self {
        let repr = ArrDeqRepr {
            arr: ArrApi::new(MaybeUninit::uninit()),
            head: 0,
            len: 0,
        };
        // SAFETY: 0 elements are initialized
        unsafe { Self::from_repr(repr) }
    }

    /// Creates a full [`ArrDeqApi<A>`] from an instance of `A`.
    ///
    /// The resulting deque is contiguous.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// assert_eq!(ArrDeqApi::new_full([1; 20]), [1; 20]);
    /// ```
    pub const fn new_full(full: A) -> Self {
        let repr = ArrDeqRepr {
            arr: ArrApi::new(MaybeUninit::new(full)),
            head: 0,
            len: arr_len::<A>(),
        };

        // SAFETY: All elements are initialized because we have a fully initialized array
        unsafe { Self::from_repr(repr) }
    }

    /// Returns [`ArrApi::<A>::length`]
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// assert_eq!(ArrDeqApi::<[i32; 20]>::new().capacity(), 20);
    /// ```
    pub const fn capacity(&self) -> usize {
        arr_len::<A>()
    }

    /// Returns the current number of elements in this deque.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// assert_eq!(ArrDeqApi::<[i32; 20]>::new().len(), 0);
    /// assert_eq!(ArrDeqApi::new_full([1; 20]).len(), 20);
    /// ```
    pub const fn len(&self) -> usize {
        self.as_repr().len
    }

    /// Checks whether this deque is empty.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// assert_eq!(ArrDeqApi::<[i32; 20]>::new().is_empty(), true);
    /// assert_eq!(ArrDeqApi::new_full([1; 20]).is_empty(), false);
    /// ```
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks whether this deque is full.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// assert_eq!(ArrDeqApi::<[i32; 20]>::new().is_full(), false);
    /// assert_eq!(ArrDeqApi::new_full([1; 20]).is_full(), true);
    /// ```
    pub const fn is_full(&self) -> bool {
        self.len() >= self.capacity()
    }

    /// Equivalent of [`VecDeque::pop_front`](std::collections::VecDeque::pop_front).
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// assert_eq!(
    ///     ArrDeqApi::<[i32; 20]>::new().pop_front(),
    ///     None
    /// );
    /// assert_eq!(
    ///     ArrDeqApi::new_full(Arr::<_, gnat::lit!(20)>::from_fn(|i| i)).pop_front(),
    ///     Some(0)
    /// );
    /// ```
    pub const fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        // SAFETY: See below
        let repr = unsafe { self.as_mut_repr() };

        // SAFETY: This is the last time we remember that the element at this index was initialized.
        // After this, when we decrement len, the element is treated as invalid and not read from
        // until overwritten.
        let popped = unsafe { repr.phys_read(repr.head) };

        // No overflow because !self.is_empty()
        repr.len -= 1;

        // Does not depend on `len`, can be reordered.
        // By decrementing len and shifting head one forward (wrapping if needed), we have removed
        // the first item from the deque.
        repr.head = repr.phys_idx_of(1);

        Some(popped)
    }

    /// Equivalent of [`VecDeque::pop_back`](std::collections::VecDeque::pop_back).
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// assert_eq!(
    ///     ArrDeqApi::<[i32; 20]>::new().pop_back(),
    ///     None
    /// );
    /// assert_eq!(
    ///     ArrDeqApi::new_full(Arr::<_, gnat::lit!(20)>::from_fn(|i| i)).pop_back(),
    ///     Some(19)
    /// );
    /// ```
    pub const fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        // SAFETY: See below
        let repr = unsafe { self.as_mut_repr() };

        // Surrender ownership of the last item. No overflow because !self.is_empty()
        repr.len -= 1;

        // SAFETY: `tail` now points to what used to be the last item in the deque.
        // This is the last time we remember that the element at this index was initialized.
        // After this, the element is treated as invalid and not read from until overwritten.
        Some(unsafe { repr.phys_read(repr.tail()) })
    }

    /// Equivalent of [`VecDeque::push_front`](std::collections::VecDeque::push_front).
    ///
    /// # Panics
    /// If this deque is full.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// let mut deq = ArrDeqApi::<[_; 20]>::new();
    /// deq.push_front(1);
    /// deq.push_front(2);
    /// assert_eq!(deq, [2, 1]);
    /// ```
    #[track_caller]
    pub const fn push_front(&mut self, item: T) {
        expect_ok(
            self.try_push_front(item),
            "Call to `push_front` on full `ArrDeqApi`",
        )
    }

    /// Equivalent of [`VecDeque::push_back`](std::collections::VecDeque::push_back).
    ///
    /// # Panics
    /// If this deque is full.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// let mut deq = ArrDeqApi::<[_; 20]>::new();
    /// deq.push_back(1);
    /// deq.push_back(2);
    /// assert_eq!(deq, [1, 2]);
    /// ```
    #[track_caller]
    pub const fn push_back(&mut self, item: T) {
        expect_ok(
            self.try_push_back(item),
            "Call to `push_back` on full `ArrDeqApi`",
        )
    }

    /// Like [`push_front`](Self::push_front), but returns [`Err`] on full deques.
    ///
    /// # Errors
    /// Returns back the input if the deque is full.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// let mut deq = ArrDeqApi::<[_; 2]>::new();
    /// assert_eq!(deq.try_push_front(1), Ok(()));
    /// assert_eq!(deq.try_push_front(2), Ok(()));
    /// assert_eq!(deq.try_push_front(3), Err(3));
    /// assert_eq!(deq, [2, 1]);
    /// ```
    pub const fn try_push_front(&mut self, item: T) -> Result<(), T> {
        if self.is_full() {
            return Err(item);
        }

        // SAFETY: See below
        let repr = unsafe { self.as_mut_repr() };

        // Move the head back by one
        repr.head = repr.phys_idx_before_head(1);

        // Add the new item at the head
        repr.arr.as_mut_slice()[repr.head].write(item);

        // No overflow because !self.is_full()
        repr.len += 1;

        Ok(())
    }

    /// Like [`push_back`](Self::push_back), but returns [`Err`] on full deques.
    ///
    /// # Errors
    /// Returns back the input if the deque is full.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// let mut deq = ArrDeqApi::<[_; 2]>::new();
    /// assert_eq!(deq.try_push_back(1), Ok(()));
    /// assert_eq!(deq.try_push_back(2), Ok(()));
    /// assert_eq!(deq.try_push_back(3), Err(3));
    /// assert_eq!(deq, [1, 2]);
    /// ```
    pub const fn try_push_back(&mut self, item: T) -> Result<(), T> {
        if self.is_full() {
            return Err(item);
        }

        // SAFETY: See below
        let repr = unsafe { self.as_mut_repr() };

        // Write a new last item into the array.
        let tail = repr.tail();
        repr.arr.as_mut_slice()[tail].write(item);

        // No overflow because !self.is_full()
        repr.len += 1;

        Ok(())
    }

    /// Returns a reference to the elements as a pair of slices.
    ///
    /// If this deque is contiguous, then the right slice will be empty.
    /// The left slice is empty only when the entire deque is empty.
    ///
    /// Note that the exact distribution between left and right are not guaranteed
    /// unless the length of this deque is 1 or less, or this deque is contiguous
    /// according to an API.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// let mut deq = ArrDeqApi::<[_; 20]>::new();
    /// deq.push_front(1);
    /// deq.push_back(2);
    /// let (lhs, rhs) = deq.as_slices();
    /// assert_eq!(lhs.iter().chain(rhs).next(), Some(&1));
    /// assert_eq!(lhs.iter().chain(rhs).next_back(), Some(&2));
    /// ```
    pub const fn as_slices(&self) -> (&[T], &[T]) {
        let &ArrDeqRepr { head, len, ref arr } = self.as_repr();
        // SAFETY: The invariants of `as_nonnull_slices` are such that it it safe to call with the
        // fields of a `ArrDeqApi`. The returned pointers are valid for reads.
        unsafe {
            let (lhs, rhs) = as_nonnull_slices(
                crate::utils::nonnull_from_const_ref(arr.as_slice()),
                head,
                len,
            );
            (lhs.as_ref(), rhs.as_ref())
        }
    }

    /// Rotates the underlying array to make the initialized part contiguous.
    /// Also returns a mutable slice of the now contiguous elements.
    ///
    /// After calling this method, [`Self::as_slices`] and [`Self::as_mut_slices`] are guaranteed
    /// to return an empty slice as the right tuple element.
    ///
    /// ```
    /// use gnat::array::*;
    /// let mut deq = ArrDeqApi::<[i32; 20]>::new();
    /// for i in 0..3 {
    ///     deq.push_back(i);
    ///     deq.push_front(10 - i);
    /// }
    /// assert_eq!(deq.make_contiguous(), [8, 9, 10, 0, 1, 2]);
    /// assert_eq!(deq.as_slices(), ([8, 9, 10, 0, 1, 2].as_slice(), [].as_slice()));
    /// ```
    #[inline]
    pub const fn make_contiguous(&mut self) -> &mut [T] {
        /// This is way less performant than <[T]>::rotate_left, which is not const.
        const fn rotate_left<T>(slice: &mut [T], dist: usize) {
            const fn reverse<T>(slice: &mut [T]) {
                let mut i = 0;
                while i < slice.len() / 2 {
                    slice.swap(i, slice.len() - i - 1);
                    i += 1;
                }
            }
            let (lhs, rhs) = slice.split_at_mut(dist);
            // EFGHIJKLMN^ABCD
            reverse(lhs);
            // NMLKJIHGFE^ABCD
            reverse(rhs);
            // NMLKJIHGFE^DBCA
            reverse(slice);
            // ABCDEFGHIJ^KLMN
        }

        // SAFETY: Left rotation by `head`, i.e. right rotation by cap - head, which means:
        // - Start of the first range: `head`
        //   -> `0`
        // - End of the first range: `min(len, cap - head) + head`
        //   -> `min(len, cap - head)`
        // - Start of the second range: `0`
        //   -> `cap - head`
        // - End of the second range: `len.saturating_sub(cap - head)`
        //   = `len - min(len, cap - head)`
        //   = max(0, len - cap + head)
        //   -> max(0, len - cap + head) + cap - head
        //   = max(cap - head, len)
        //
        // So we get `0 .. min(len, cap - head)`, `cap - head .. max(len, cap - head)`
        // - If `len > cap - head`: `0 .. cap - head`, `cap - head .. len`
        // - If `len <= cap - head`: `0 .. len`, `cap - head .. cap - head`
        //
        // Since the second range is empty in case 2, we always get that the valid elements are
        // contiguous in `arr[0..len]`. Hence we can set head to 0.
        let repr = unsafe { self.as_mut_repr() };
        rotate_left(repr.arr.as_mut_slice(), repr.head);
        repr.head = 0;

        self.as_mut_slices().0
    }

    /// Transfers the elements into an [`ArrVecApi`](crate::array::ArrVecApi).
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// let mut deq = ArrDeqApi::<[_; 20]>::new();
    /// deq.push_front(2);
    /// deq.push_front(1);
    /// let mut vec = deq.into_contiguous();
    /// assert_eq!(vec.as_slice(), [1, 2]);
    /// ```
    #[doc(alias = "retype_vec")]
    pub const fn into_contiguous(mut self) -> crate::array::ArrVecApi<A> {
        use crate::array::*;

        self.make_contiguous();
        let ArrDeqRepr { len, arr, head: _ } = self.into_repr();
        // SAFETY: `head == 0`, so the first `len` elements are initialized.
        unsafe { ArrVecApi::from_uninit_parts(arr, len) }
    }

    /// Makes this deque contiguous and then returns the elements as a full array.
    ///
    /// # Panics
    /// If [`!self.is_full()`](Self::is_full)
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// let mut deq = ArrDeqApi::<[i32; 2]>::new();
    /// deq.push_back(2);
    /// deq.push_front(1);
    /// let [a, b] = deq.assert_full();
    /// assert_eq!((a, b), (1, 2));
    /// ```
    #[track_caller]
    pub const fn assert_full(mut self) -> A {
        if self.is_full() {
            self.make_contiguous();
            // SAFETY: The deque is full, hence all elements of the backing array are initialized
            unsafe { self.into_repr().arr.inner.assume_init() }
        } else {
            const_fmt::fmt![
                "Call to `assert_full` on `ArrDeqApi` with length ",
                self.len(),
                " out of ",
                self.capacity()
            ]
            .panic()
        }
    }

    /// Discards an empty deque by asserting that it is empty and using
    /// [`core::mem::forget`] if it is.
    ///
    /// See the info about the [Drop implementation](crate::array::ArrVecApi#drop-implementation).
    /// ```
    /// use gnat::array::*;
    /// const fn works_in_const<A: Array<Item = i32>>(arr: A) -> i32 {
    ///     let mut deq = ArrDeqApi::new_full(arr);
    ///     let mut sum = 0;
    ///     while let Some(item) = deq.pop_front() {
    ///         sum += item;
    ///     }
    ///     deq.assert_empty();
    ///     sum
    /// }
    /// assert_eq!(works_in_const([1; 20]), 20);
    /// ```
    #[track_caller]
    pub const fn assert_empty(self) {
        if !self.is_empty() {
            const_fmt::fmt![
                "Call to `assert_empty` on `ArrDeqApi` with length ",
                self.len()
            ]
            .panic()
        }
        core::mem::forget(self);
    }
}

mod core_impl;
