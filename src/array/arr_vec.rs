use core::{marker::PhantomData, mem::MaybeUninit};

mod core_impl;

use const_util::result::expect_ok;

use crate::{
    Nat,
    array::{helper::*, *},
    const_fmt, utils,
};

/// Wraps the drop impl so it isn't exposed as a trait bound
// NOTE: Mutable access to fields and construction of this struct requires a safety
// comment. Prefer using as_mut_repr and from_uninit_parts for this.
#[repr(transparent)]
pub(crate) struct ArrVecDrop<A: Array<Item = T>, T = <A as Array>::Item> {
    /// # Safety
    /// See [`ArrVecRepr`].
    repr: ArrVecRepr<A>,
    _p: PhantomData<T>,
}
impl<A: Array<Item = T>, T> Drop for ArrVecDrop<A, T> {
    fn drop(&mut self) {
        let (owned, ..) = self.split_at_spare_mut();

        // SAFETY:
        // The vector has ownership over the first `len` items, so they are considered to stop
        // existing after this drop runs.
        // Since they are behind `MaybeUninit`, which can hold invalid values and does not have
        // drop glue itself, it is safe to drop them here.
        unsafe { core::ptr::drop_in_place(owned) }
    }
}

pub(crate) struct ArrVecRepr<A: Array> {
    /// # Safety
    /// See below
    len: usize,
    /// # Safety
    /// When this struct is wrapped in ArrVecApi/ArrVecDrop, the first `len` items of `arr` must be initialized
    arr: ArrApi<MaybeUninit<A>>,
}

// Defer the impl of ArrVecApi here because we need it for the Drop impl
impl<A: Array<Item = T>, T> ArrVecDrop<A> {
    const fn split_at_spare_mut(&mut self) -> (&mut [T], &mut [MaybeUninit<T>]) {
        let Self { repr, .. } = self;
        // SAFETY: Invariants are upheld, see below
        let &mut ArrVecRepr { ref mut arr, len } = repr;
        let (init, spare) = arr.as_mut_slice().split_at_mut(len);
        // SAFETY: The first `len` items are valid by invariant. It is not safely
        // possible to write invalid values into them since they are behind `&mut [T]`.
        (unsafe { crate::utils::assume_init_mut_slice(init) }, spare)
    }
}

// Methods that directly use the fields
impl<A: Array<Item = T, Length = N>, T, N: Nat> ArrVecApi<A> {
    const fn as_repr(&self) -> &ArrVecRepr<A> {
        let Self(ArrVecDrop { repr, .. }) = self;
        repr
    }

    /// # Safety
    /// The invariants of the vector must be upheld. It must also not be possible to break them
    /// through returned references created from this.
    const unsafe fn as_mut_repr(&mut self) -> &mut ArrVecRepr<A> {
        let Self(ArrVecDrop { repr, .. }) = self;
        repr
    }

    /// Creates a vector from its components.
    ///
    /// Equivalent of [`Vec::from_raw_parts`].
    ///
    /// # Safety
    /// The first `len` elements of `arr`, i.e. `arr[..len]`, must be initialized to valid values of `T`.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    /// use core::mem::MaybeUninit;
    ///
    /// let mut arr = ArrApi::new(MaybeUninit::<[_; 3]>::uninit());
    /// arr.as_mut_slice()[0].write(1);
    /// // SAFETY: The first element of `arr` is initialized to a valid i32
    /// let vec = unsafe { ArrVecApi::from_uninit_parts(arr, 1) };
    /// assert_eq!(vec, [1]);
    /// ```
    pub const unsafe fn from_uninit_parts(arr: ArrApi<MaybeUninit<A>>, len: usize) -> Self {
        let _ = arr_len::<A>(); // Ensure array non-oversized.

        let repr = ArrVecRepr { arr, len };
        Self(ArrVecDrop {
            repr,
            _p: PhantomData,
        })
    }

    const fn into_repr(self) -> ArrVecRepr<A> {
        // SAFETY: repr(transparent)
        unsafe {
            utils::union_transmute!(
                ArrVecApi<A>,
                ArrVecRepr<A>,
                self, //
            )
        }
    }

    /// Returns the vector's elements as mutable slices.
    ///
    /// The tuple is divided into the valid and potentially invalid elements of the vector.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    /// use core::mem::MaybeUninit;
    ///
    /// let mut arr = ArrApi::new(MaybeUninit::<[_; 3]>::uninit());
    /// arr.as_mut_slice()[1].write(3);
    /// let mut vec = ArrVecApi::from_uninit_array(arr);
    /// vec.push(0);
    ///
    /// let (init, spare) = vec.split_at_spare_mut() else { unreachable!() };
    /// assert_eq!((init.len(), spare.len()), (1, 2));
    /// init[0] = 1;
    /// spare[1].write(2);
    /// spare.reverse();
    ///
    /// unsafe { vec.set_len(3) }
    /// assert_eq!(vec, [1, 2, 3]);
    /// ```
    pub const fn split_at_spare_mut(&mut self) -> (&mut [T], &mut [MaybeUninit<T>]) {
        let Self(drop) = self;
        drop.split_at_spare_mut()
    }
}

impl<A, T, N: Nat> ArrVecApi<A>
where
    A: Array<Item = T, Length = N>,
{
    /// Creates a vector from a backing array.
    ///
    /// The initial length of the vector will be zero.
    /// When combined with [`set_len`](Self::set_len), this method has the same effect as
    /// [`from_uninit_parts`](Self::from_uninit_parts).
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    /// use core::mem::MaybeUninit;
    ///
    /// let mut arr = ArrApi::new(MaybeUninit::<[_; 3]>::uninit());
    /// arr.as_mut_slice()[0].write(1);
    /// let vec = ArrVecApi::from_uninit_array(arr);
    /// assert_eq!(vec, []); // length is always 0
    /// ```
    pub const fn from_uninit_array(arr: ArrApi<MaybeUninit<A>>) -> Self {
        // SAFETY: arr[0..0] is empty and thus trivially valid
        unsafe { Self::from_uninit_parts(arr, 0) }
    }

    /// Turns the vector into its components.
    ///
    /// The first `len` elements of the array are guaranteed to be initialized to valid values of `T`.
    ///
    /// Equivalent of [`Vec::into_raw_parts`].
    #[must_use = "The returned array may contain valid items previously owned by the vec that may need to be dropped"]
    pub const fn into_uninit_parts(self) -> (ArrApi<MaybeUninit<A>>, usize) {
        let ArrVecRepr { len, arr } = self.into_repr();
        (arr, len)
    }

    /// Moves the backing array out of the vector.
    ///
    /// The first [`self.len()`] elements of the array are guaranteed to be initialized to valid values of `T`.
    ///
    /// Together with [`self.len()`], this is the same as [`Self::into_uninit_parts`],
    ///
    /// [`self.len()`]: Self::len
    #[must_use = "The returned array may contain valid items previously owned by the vec that may need to be dropped"]
    pub const fn into_backing_array(self) -> ArrApi<MaybeUninit<A>> {
        self.into_repr().arr
    }

    /// Creates an empty vector.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// type A = Arr<i32, gnat::lit!(10)>;
    /// assert_eq!(ArrVecApi::<A>::new(), []);
    /// ```
    pub const fn new() -> Self {
        Self::from_uninit_array(ArrApi::new(MaybeUninit::uninit()))
    }

    /// Creates a full vector from an instance of the underlying array.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// assert_eq!(ArrVecApi::new_full([1, 2, 3]), [1, 2, 3]);
    /// ```
    pub const fn new_full(arr: A) -> Self {
        let arr = ArrApi::new(MaybeUninit::new(arr));
        // SAFETY: `arr` comes from a regular array, which means it has `arr_len::<A>()` valid items.
        unsafe { Self::from_uninit_parts(arr, arr_len::<A>()) }
    }

    /// Returns the vector's elements as mutable slices.
    ///
    /// The tuple is divided into the valid and potentially invalid elements of the vector.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    /// use core::mem::MaybeUninit;
    ///
    /// let mut arr = ArrApi::new(MaybeUninit::<[_; 3]>::uninit());
    /// arr.as_mut_slice()[1].write(2);
    /// let mut vec = ArrVecApi::from_uninit_array(arr);
    /// vec.push(1);
    /// let (init, spare) = vec.split_at_spare();
    /// assert_eq!(init, [1]);
    /// assert_eq!(unsafe { spare[0].assume_init_read() }, 2);
    /// ```
    pub const fn split_at_spare(&self) -> (&[T], &[MaybeUninit<T>]) {
        let &ArrVecRepr { ref arr, len } = self.as_repr();
        let (init, spare) = arr.as_slice().split_at(len);
        // SAFETY: The first `len` elements are valid by invariant
        (unsafe { crate::utils::assume_init_slice(init) }, spare)
    }

    /// Returns the vector's known valid elements as a slice.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// let mut vec = ArrVecApi::new_full([1, 2, 3]);
    /// vec.pop();
    /// assert_eq!(vec.as_slice()[1..], [2]);
    /// ```
    pub const fn as_slice(&self) -> &[T] {
        self.split_at_spare().0
    }

    /// Returns the vector potentially invalid elements as a slice.
    ///
    /// See [`Self::split_at_spare`].
    pub const fn spare_capacity(&self) -> &[MaybeUninit<T>] {
        self.split_at_spare().1
    }

    /// Returns the vector potentially invalid elements as a mutable slice.
    ///
    /// See [`Self::split_at_spare`].
    pub const fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        self.split_at_spare_mut().1
    }

    /// Returns the vector's known valid elements as a mutable slice.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// let mut vec = ArrVecApi::new_full([1, 2, 3]);
    /// vec.as_mut_slice().reverse();
    /// assert_eq!(vec, [3, 2, 1]);
    /// ```
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        self.split_at_spare_mut().0
    }

    /// Returns the length of the vector.
    ///
    /// The length is the number of elements known to be valid.
    pub const fn len(&self) -> usize {
        self.as_repr().len
    }

    /// Checks whether the vector is empty, i.e. whether [`len`](Self::len) is zero.
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length of the vector's backing array, i.e. [`ArrApi::<A>::length`].
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// assert_eq!(ArrVecApi::<[i32; 20]>::new().capacity(), 20);
    /// ```
    pub const fn capacity(&self) -> usize {
        arr_len::<A>()
    }

    /// Checks whether the vector is full.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// assert_eq!(ArrVecApi::<[i32; 20]>::new().is_full(), false);
    /// assert_eq!(ArrVecApi::new_full([1; 20]).is_full(), true);
    /// ```
    pub const fn is_full(&self) -> bool {
        self.spare_len() == 0
    }

    /// Returns the number of elements that can be pushed into the vector until it is full.
    ///
    /// This is the same as the length of the potentially invalid segment, i.e.
    /// `self.capacity() - self.len()`
    pub const fn spare_len(&self) -> usize {
        let diff = self.capacity().checked_sub(self.len());
        // SAFETY: We cannot have more than `capacity` valid elements by invariant
        // and it is a safety invariant of this type that the first `self.len()` elements
        // are valid
        unsafe { diff.unwrap_unchecked() }
    }

    /// Moves the elements of the vector into a full array.
    ///
    /// # Panics
    /// If the vector is full.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// let mut vec = ArrVecApi::<[i32; 2]>::new();
    /// vec.push(1);
    /// vec.push(2);
    /// let [a, b] = vec.assert_full();
    /// assert_eq!((a, b), (1, 2));
    /// ```
    #[track_caller]
    pub const fn assert_full(self) -> A {
        match self.is_full() {
            // SAFETY: The vec is full, hence all elements of the backing array are valid
            // and owned, so we may move `A` out of them.
            true => unsafe { self.into_uninit_parts().0.inner.assume_init() },
            false => const_fmt::fmt![
                "Call to `assert_full` on `ArrVecApi` with length ",
                self.len(),
                " out of ",
                self.capacity()
            ]
            .panic(),
        }
    }

    /// Discards the vector by asserting that it is empty and using [`core::mem::forget`].
    ///
    /// See the info about the [Drop implementation](crate::array::ArrVecApi#drop-implementation).
    ///
    /// # Panics
    /// If the vector is non-empty.
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    /// const fn works_in_const<A: Array<Item = i32>>(arr: A) -> i32 {
    ///     let mut vec = ArrVecApi::new_full(arr);
    ///     let mut sum = 0;
    ///     while let Some(item) = vec.pop() {
    ///         sum += item;
    ///     }
    ///     vec.assert_empty();
    ///     sum
    /// }
    /// assert_eq!(works_in_const([1; 20]), 20);
    /// ```
    #[track_caller]
    pub const fn assert_empty(self) {
        if self.is_empty() {
            // No items, so drop is a noop
            core::mem::forget(self)
        } else {
            const_fmt::fmt![
                "Call to `assert_empty` on `ArrVecApi` with length ",
                self.len()
            ]
            .panic()
        }
    }

    /// Equivalent of [`Vec::push`].
    ///
    /// # Panics
    /// If the vector is full.
    ///
    /// ```
    /// use gnat::array::*;
    ///
    /// let mut vec = ArrVecApi::<[_; 20]>::new();
    /// vec.push(1);
    /// vec.push(2);
    /// assert_eq!(vec, [1, 2]);
    /// ```
    #[track_caller]
    pub const fn push(&mut self, item: T) {
        expect_ok(self.try_push(item), "Call to `push` on full `ArrVecApi`")
    }

    /// Like [`push`](Self::push), but returns [`Err`] on full vecs.
    ///
    /// ```
    /// use gnat::array::*;
    ///
    /// let mut vec = ArrVecApi::<[_; 2]>::new();
    /// assert_eq!(vec.try_push(1), Ok(()));
    /// assert_eq!(vec.try_push(2), Ok(()));
    /// assert_eq!(vec.try_push(3), Err(3));
    /// assert_eq!(vec, [1, 2]);
    /// ```
    ///
    /// # Errors
    /// Returns back the input as an error if the vector was full.
    pub const fn try_push(&mut self, item: T) -> Result<(), T> {
        if self.is_full() {
            return Err(item);
        }

        // SAFETY: See below
        let ArrVecRepr { arr, len } = unsafe { self.as_mut_repr() };

        // We now own `len + 1` valid items
        arr.as_mut_slice()[*len].write(item);

        // So we can increment. No overflow because we had !self.is_full()
        *len += 1;

        Ok(())
    }

    /// Equivalent of [`Vec::pop`].
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// assert_eq!(
    ///     ArrVecApi::<[i32; 20]>::new().pop(),
    ///     None
    /// );
    /// assert_eq!(
    ///     ArrVecApi::new_full(Arr::<_, gnat::lit!(20)>::from_fn(|i| i)).pop(),
    ///     Some(19)
    /// );
    /// ```
    pub const fn pop(&mut self) -> Option<T> {
        // SAFETY: See below
        let ArrVecRepr { arr, len } = unsafe { self.as_mut_repr() };

        if *len == 0 {
            return None;
        }

        // No overflow because !self.is_empty(). At the end of this method, we no longer own an
        // item at `len`.
        *len -= 1;

        // SAFETY: This is the last time we remember our ownership of the valid item at `len`.
        // After this, while we do leave the bytes of the value in place, it is considered moved
        // out of and therefore invalid.
        Some(unsafe { arr.as_slice()[*len].assume_init_read() })
    }

    /// Sets the length of the vector.
    ///
    /// Equivalent of [`Vec::set_len`].
    ///
    /// # Safety
    /// The first `new_len` items in the backing array must be valid instances of `T`.
    /// After calling this method, the vector has length and ownership of `new_len` items.
    pub const unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());
        // SAFETY: The caller guarantees that the first `new_len` items are valid and that this
        // vec may take ownership of them.
        unsafe { self.as_mut_repr().len = new_len };
    }

    /// Transfers the elements from the vector into a contiguous [`ArrDeqApi`].
    ///
    /// # Examples
    /// ```
    /// use gnat::array::*;
    ///
    /// let mut vec = ArrVecApi::<[_; 20]>::new();
    /// vec.push(1);
    /// vec.push(2);
    /// let mut deq = vec.into_deque();
    /// assert_eq!(deq.as_slices(), ([1, 2].as_slice(), [].as_slice()));
    /// ```
    pub const fn into_deque(self) -> crate::array::ArrDeqApi<A> {
        ArrDeqApi::from_vec_impl(self)
    }

    /// Changes the backing array type of the vector.
    ///
    /// Only the item type is required to stay the same.
    ///
    /// # Errors
    /// If `A::Length < vec.len()` or `A::Length > usize::MAX`, the original vector is returned.
    pub const fn try_retype<Dst>(self) -> Result<ArrVecApi<Dst>, Self>
    where
        Dst: Array<Item = T>,
    {
        match crate::to_usize::<Dst::Length>() {
            Some(cap) if cap < self.len() => Err(self),
            _ => {
                let (arr, len) = self.into_uninit_parts();
                // SAFETY: new cap >= len, so we must still have `len` valid elements.
                Ok(unsafe { ArrVecApi::from_uninit_parts(arr.retype_uninit(), len) })
            }
        }
    }
}
