use crate::{
    Nat,
    array::{helper::*, *},
    nat,
};
use crate::{condty, expr, utils};
use core::marker::PhantomData;
use core::mem::MaybeUninit;

// NOTE: Mutable access to fields and construction of this struct requires a safety comment.
pub(crate) struct DoubleEndedBuffer<A: Array> {
    /// # Safety
    /// See below.
    start: usize,
    /// # Safety
    /// See below.
    end: usize,
    /// # Safety
    /// start <= end and arr[start..end] must be initialized to valid instances. The buffer has ownership over them
    /// and is responsible for dropping them.
    arr: ArrApi<MaybeUninit<A>>,
}
impl<A: Array> Drop for DoubleEndedBuffer<A> {
    fn drop(&mut self) {
        // SAFETY: These elements are valid and the buffer has ownership over them.
        unsafe { core::ptr::drop_in_place(self.as_mut_slice()) }
    }
}
impl<A, T> DoubleEndedBuffer<A>
where
    A: Array<Item = T>,
{
    pub const fn new(arr: A) -> Self {
        // SAFETY: `arr` contains `arr_len` valid instances of `T`, which spans the entire array.
        Self {
            start: 0,
            end: arr_len::<A>(),
            arr: ArrApi {
                inner: MaybeUninit::new(arr),
            },
        }
    }
    pub const fn from_vec(vec: ArrVecApi<A>) -> Self {
        let (arr, end) = vec.into_uninit_parts();
        // SAFETY: arr[..end] is initalized by vector invariant
        Self { start: 0, end, arr }
    }
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: arr[start..end] is valid and only valid instances can be written into the
        // returned slice.
        unsafe {
            utils::assume_init_mut_slice(utils::subslice!(
                &mut self.arr.as_mut_slice(),
                self.start, //
                self.end,
            ))
        }
    }
    #[inline]
    pub const fn len(&self) -> usize {
        // SAFETY: By invariant.
        unsafe { core::hint::assert_unchecked(self.start <= self.end) }
        self.end - self.start
    }
    pub const fn pop_front(&mut self) -> Option<T> {
        if self.len() == 0 {
            return None;
        }

        let old_start = self.start;

        // SAFETY:
        // self.end - self.start > 0, thus self.start < self.end,
        // thus incrementing doesn't violate start <= end.
        // The array validity variant only gets weaker from doing this.
        self.start += 1;

        // SAFETY: This is the last time we remember the validity of the first item,
        // so we can safely move out of it.
        Some(unsafe { self.arr.as_slice()[old_start].assume_init_read() })
    }
    pub const fn pop_back(&mut self) -> Option<T> {
        if self.len() == 0 {
            return None;
        }
        // SAFETY:
        // self.end - self.start > 0, thus self.end > self.start,
        // thus decrementing doesn't violate start <= end.
        // The array validity variant only gets weaker from doing this.
        self.end -= 1;

        // SAFETY: This is the last time we remember the validity of the first item,
        // so we can safely move out of it.
        Some(unsafe { self.arr.as_slice()[self.end].assume_init_read() })
    }
}

pub type PopDigit<N> = nat::Eval<expr::_Shr<N, crate::consts::PtrBits>>;

#[utils::apply(expr::lazy)]
pub type _DigitLenRec<N> = _DigitLen<PopDigit<N>>;
#[utils::apply(expr::lazy)]
pub type _DigitLen<N> = expr::If<
    N,
    expr::_Inc<_DigitLenRec<N>>, //
    nat::lit!(0),
>;
pub type DigitLen<N> = nat::Eval<_DigitLen<N>>;

pub(crate) struct BigCounter<N: Nat> {
    /// # Safety
    /// Represents a number in base `usize::MAX + 1`.
    /// Little-endian, i.e. least significant digit at index 0.
    ///
    /// Must be less than or equal to N
    digits: CopyArr<usize, DigitLen<N>>,
}
const fn all_zeros(mut digits: &[usize]) -> bool {
    while let &[ref rest @ .., last] = digits {
        digits = rest;
        if last != 0 {
            return false;
        }
    }
    true
}
impl<N: Nat> BigCounter<N> {
    pub const fn dec(&mut self) -> bool {
        if self.is_zero() {
            return false;
        }
        // SAFETY: self > 0, so decrementing is ok
        let mut digits = self.digits.as_mut_slice();
        while let [lsd, rest @ ..] = digits {
            digits = rest;
            let ovfl;
            (*lsd, ovfl) = lsd.overflowing_sub(1);
            if !ovfl {
                break;
            }
        }
        true
    }
    /// # Safety
    /// self < N
    pub const unsafe fn inc_unchecked(&mut self) {
        // SAFETY: self < N, so incrementing is ok
        let mut digits = self.digits.as_mut_slice();
        while let [lsd, rest @ ..] = digits {
            digits = rest;
            let ovfl;
            (*lsd, ovfl) = lsd.overflowing_add(1);
            if !ovfl {
                return;
            }
        }
    }
    pub const fn is_zero(&self) -> bool {
        all_zeros(self.digits.as_slice())
    }
    pub const fn zero() -> Self {
        Self {
            digits: CopyArr::of(0),
        }
    }
    pub const fn max() -> Self {
        const {
            // SAFETY: This construction ensures self == N
            Self {
                digits: if nat::is_nonzero::<N>() {
                    BigCounter::<PopDigit<N>>::max()
                        .digits
                        .concat([nat::to_usize_overflowing::<N>().0])
                        .try_retype()
                        .unwrap()
                } else {
                    CopyArr::of(0)
                },
            }
        }
    }
    pub const fn to_usize(&self) -> Option<usize> {
        match self.digits.as_slice() {
            [] => Some(0),
            [lsd, rest @ ..] => match all_zeros(rest) {
                true => Some(*lsd),
                false => None,
            },
        }
    }
}

/// # Safefty
/// It must be safe to conceive the ZST `T` from nothing.
const unsafe fn conjure_zst<T>() -> T {
    debug_assert!(const { size_of::<T>() == 0 });

    // SAFETY: By definition. Reading ZSTs from dangling is legal.
    unsafe { core::ptr::dangling::<T>().read() }
}
pub(crate) struct InstanceCounter<T, N: Nat> {
    /// # Safety
    /// - This type owns as many instances as indicated by the value represented by `digits`
    /// - `T` must be a ZST
    counter: BigCounter<N>,
    _p: PhantomData<T>,
}
impl<T, N: Nat> Drop for InstanceCounter<T, N> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}
impl<T, N: Nat> InstanceCounter<T, N> {
    pub const fn full(arr: impl Array<Item = T, Length = N>) -> Self {
        assert!(size_of::<T>() == 0);
        core::mem::forget(arr);
        // SAFETY: Array of `N` instances was forgotten, so this is logically
        // equivalent to moving them into a new container.
        Self {
            counter: BigCounter::max(),
            _p: PhantomData,
        }
    }
    pub const fn empty() -> Self {
        assert!(size_of::<T>() == 0);
        // SAFETY: An empty container is trivially safe to create.
        Self {
            counter: BigCounter::zero(),
            _p: PhantomData,
        }
    }
    pub const fn pop(&mut self) -> Option<T> {
        match self.counter.dec() {
            // SAFETY: Counter was decremented, so creating one instance from nothing
            // is logically equivalent to moving it out of the container.
            true => Some(unsafe { conjure_zst() }),
            false => None,
        }
    }
    /// # Safety
    /// Counter must be smaller than `N`.
    pub const unsafe fn push_unchecked(&mut self, item: T) {
        // SAFETY:
        // - `inc_unchecked` is safe because the counter is smaller than `N`
        // - incrementing the instance count is safe because an instance was
        //   because it is logically equivalent to moving the forgotten
        //   instance into the container.
        unsafe {
            core::mem::forget(item);
            self.counter.inc_unchecked()
        }
    }
    pub const fn len(&self) -> Option<usize> {
        self.counter.to_usize()
    }
}

pub(crate) struct ArrBuilder<A: Array> {
    /// # Safety
    /// If BigCounter, then this is a container with up to N instances
    /// of T, where the value of the counter is the number of free slots.
    #[allow(clippy::complexity)]
    inner: condty::CondResult<
        PopDigit<A::Length>,                 // if N is oversized
        InstanceCounter<A::Item, A::Length>, // use a counter
        ArrVecApi<A>,                        // else a vec
    >,
}
impl<A: Array> ArrBuilder<A> {
    pub const fn new() -> Self {
        Self {
            inner: condty::ctx!(
                |c| c.new_ok(InstanceCounter::empty()),
                |c| c.new_err(ArrVecApi::new()), //
            ),
        }
    }
    /// # Safety
    /// This builder must have `A::Length` elements.
    pub unsafe fn into_full_unchecked(self) -> A {
        condty::ctx!(
            // SAFETY: The counter is maxed out, so this is logically equivalent
            // to moving the instances out of the container.
            |_| unsafe {
                core::mem::forget(self);
                conjure_zst()
            },
            |c| c.unwrap_err(self.inner).assert_full(),
        )
    }
    /// # Safety
    /// This builder must have fewer than `A::Length` elements.
    pub const unsafe fn push_unchecked(&mut self, item: A::Item) {
        let inner = self.inner.as_mut();
        condty::ctx!(
            // SAFETY:
            |c| unsafe { c.unwrap_ok(inner).push_unchecked(item) },
            |c| c.unwrap_err(inner).push(item), //
        )
    }
}

pub(crate) struct ArrConsumer<A: Array> {
    #[allow(clippy::complexity)]
    inner: condty::CondResult<
        PopDigit<A::Length>,                 // if Length is oversized
        InstanceCounter<A::Item, A::Length>, // use a counter
        DoubleEndedBuffer<A>,                // else a buffer
    >,
}
impl<A: Array> ArrConsumer<A> {
    pub const fn new(arr: A) -> Self {
        Self {
            inner: condty::ctx!(
                |c| c.new_ok(InstanceCounter::full(arr)),
                |c| c.new_err(DoubleEndedBuffer::new(arr)), //
            ),
        }
    }
    pub const fn pop_front(&mut self) -> Option<A::Item> {
        let inner = self.inner.as_mut();
        condty::ctx!(
            |c| c.unwrap_ok(inner).pop(), //
            |c| c.unwrap_err(inner).pop_front(),
        )
    }
    pub const fn pop_back(&mut self) -> Option<A::Item> {
        let inner = self.inner.as_mut();
        condty::ctx!(
            |c| c.unwrap_ok(inner).pop(), //
            |c| c.unwrap_err(inner).pop_back(),
        )
    }
    pub const fn len(&self) -> Option<usize> {
        let inner = self.inner.as_ref();
        condty::ctx!(
            |c| c.unwrap_ok(inner).len(), //
            |c| Some(c.unwrap_err(inner).len())
        )
    }
    pub const fn from_vec(vec: ArrVecApi<A>) -> Self {
        Self {
            inner: condty::ctx!(
                |_| unreachable!(), // currently unsupported
                |c| c.new_err(DoubleEndedBuffer::from_vec(vec))
            ),
        }
    }
}

pub(crate) struct ArrRefConsumer<'a, T, N: Nat> {
    inner: condty::CondResult<
        PopDigit<N>,            // if oversized
        (BigCounter<N>, &'a T), // yield the same reference N times
        &'a [T],                // else yield from a slice
    >,
}
impl<'a, T, N: Nat> ArrRefConsumer<'a, T, N> {
    pub const fn new<A>(arr: &'a A) -> Self
    where
        A: Array<Item = T, Length = N>,
    {
        const { arr_impl_ubcheck::<A>() }

        Self {
            inner: condty::ctx!(
                |c| c.new_ok((
                    BigCounter::max(),
                    // SAFETY: array length is nonzero, so this points to the first item.
                    // (which has the same address as all the other items, because T is a ZST)
                    unsafe { &*core::ptr::from_ref(arr).cast() },
                )),
                |c| c.new_err(arr_api::unsize_ref(arr)),
            ),
        }
    }
    pub const fn pop_front(&mut self) -> Option<&'a T> {
        let inner = self.inner.as_mut();
        condty::ctx!(
            |c| {
                let (count, r) = c.unwrap_ok(inner);
                match count.is_zero() {
                    true => None,
                    false => Some(r),
                }
            },
            |c| {
                let inner = c.unwrap_err(inner);
                match inner {
                    [] => None,
                    [next, rest @ ..] => {
                        *inner = rest;
                        Some(next)
                    }
                }
            }
        )
    }
}
