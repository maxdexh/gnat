//! Method impls using array invariants.
//!
//! As these methods make heavy use of transmutes, they need to be tested the most.
//! Also see [`arr_api`].

use core::mem::MaybeUninit;

use crate::{
    Nat,
    array::{helper::*, *},
    nat, utils,
};

impl<T, N: Nat, A> ArrApi<A>
where
    A: Array<Item = T, Length = N>,
{
    /// Tries to turn the array into a builtin `[T; M]` array of the same size.
    ///
    /// This is different from [`Self::try_retype`] in that it works even when `[T; M]` does not
    /// implement [`Array`] due to `M` being too large.
    ///
    /// # Errors
    /// If `Self::Length != M`.
    ///
    /// # Examples
    /// ```
    /// use gnat::{array::*, nat};
    ///
    /// let arr = CopyArr::<_, nat::lit!(20_000)>::from_fn(|i| i);
    /// assert_eq!(arr.try_into_builtin::<19_999>(), Err(arr));
    /// assert_eq!(arr.try_into_builtin::<20_001>(), Err(arr));
    /// let builtin: [_; 20_000] = arr.try_into_builtin().unwrap();
    /// assert_eq!(builtin, core::array::from_fn::<_, 20_000, _>(|i| i));
    /// ```
    pub const fn try_into_builtin<const M: usize>(self) -> Result<[T; M], Self> {
        if const { nat::cmp_usize::<N>(M).is_eq() } {
            Ok(
                // SAFETY: `Array` invariant
                unsafe { utils::union_transmute!(Self, [T; M], self) },
            )
        } else {
            Err(self)
        }
    }
}

impl<T, N: Nat, A> ArrApi<A>
where
    A: Array<Item = T, Length = N>,
{
    /// Equivalent to `[x; N]` with `x` of a copyable type.
    ///
    /// # Examples
    /// Creating an array of integers.
    /// ```
    /// use gnat::{array::*, nat};
    /// let arr = Arr::<_, nat::lit!(4)>::of(1);
    /// assert_eq!(arr, [1; 4]);
    /// ```
    ///
    /// Creating an oversized array of `()`
    /// ```
    /// #![recursion_limit = "1024"]
    /// use gnat::{array::*, nat, lazy, consts::{PtrBits, UsizeMax}};
    /// type LargeSize = nat::Eval<lazy::Shl<nat::lit!(1), PtrBits>>;
    /// assert!(nat::to_usize::<LargeSize>().is_none());
    /// let arr = Arr::<_, LargeSize>::of(());
    /// let ArrConcat(most, [()]): ArrConcat<CopyArr<_, UsizeMax>, _> = arr.retype();
    /// assert_eq!(most.as_slice().len(), usize::MAX);
    /// ```
    pub const fn of(item: T) -> Self
    where
        T: Copy,
    {
        arr_impl_ubcheck::<A>();

        let mut out = ArrApi::new(MaybeUninit::uninit());
        // Skip the noop loop for ZSTs to avoid panicking
        if const { size_of::<T>() != 0 } {
            let mut buf = out.as_mut_slice(); // pure
            while let [first, rest @ ..] = buf {
                *first = MaybeUninit::new(item); // Guaranteed noop for ZSTs
                buf = rest;
            }
        }
        // SAFETY:
        // If `T` is not a ZST: All elements have been initialized by the loop
        //
        // If `T` is a ZST: It is valid to construct an array of `N` instances
        // because safe code *could* have generated any number of instances of
        // `T` by copying `item`. The loop was a noop, so we can skip it.
        unsafe { out.inner.assume_init() }
    }
}

impl<T, N: Nat, A> ArrApi<A>
where
    A: Array<Item = MaybeUninit<T>, Length = N>,
{
    /// Moves the items from another array of [`MaybeUninit<T>`] items with minimal loss.
    ///
    /// If `B::Length < Self::Length`, the extra items will be forgotten.
    /// If `B::Length > Self::Length`, the missing items will be left uninitialized.
    /// Otherwise, the output is as if by [`try_retype`](Self::try_retype).
    pub const fn retype_uninit<B>(self) -> B
    where
        B: Array<Item = MaybeUninit<T>>,
    {
        // SAFETY: M := B::Length
        // - if M <= N, then transmuting through a union forgets `M - N` elements,
        //   which is safe.
        // - if M >= N, then transmuting through a union fills the rest of the array with
        //   uninitialized memory, which is valid in this context.
        unsafe {
            utils::union_transmute!(
                ArrApi<A>,
                B, //
                self,
            )
        }
    }

    /// Moves the items into `[MaybeUninit<T>; M]` with minimal loss.
    ///
    /// If `M > Self::Length`, the extra items will be forgotten.
    /// If `M < Self::Length`, the missing items will be left uninitialized.
    /// Otherwise, the output is as if by [`try_into_builtin`](Self::try_into_builtin).
    ///
    /// This method is useful for promoting recursively defined [`Array`]s like [`Arr`]
    /// if an upper bound for the length can be acquired as a const generic usize, e.g.
    /// from the [`generic_upper_bound`](https://docs.rs/generic-upper-bound/3.1.1/generic_upper_bound/)
    /// crate.
    ///
    /// # Examples
    /// Converting a [`Nat`] to a string in binary, at compile time, with arbitrary length.
    /// ```
    /// extern crate generic_upper_bound as gub;
    /// use gnat::{NatExpr, Nat, nat, lazy, array::{Arr, ArrApi}};
    /// use core::mem::MaybeUninit;
    ///
    /// type BinaryLen<N> = nat::Eval<lazy::BaseLen<nat::lit!(2), N>>;
    /// const fn to_binary_arr<N: Nat>() -> Arr<u8, BinaryLen<N>> {
    ///     let last_bit = [
    ///         b'0' + !nat::is_zero::<lazy::LastBit::<N>>() as u8
    ///     ];
    ///     if nat::is_zero::<lazy::PopBit<N>>() {
    ///         ArrApi::new(last_bit).try_retype().unwrap()
    ///     } else {
    ///         to_binary_arr::<nat::Eval<lazy::PopBit<N>>>()
    ///             .concat(last_bit)
    ///             .try_retype()
    ///             .unwrap()
    ///     }
    /// }
    /// pub const fn to_str_binary<N: NatExpr>() -> &'static str {
    ///     struct Doit<N, const ARRLEN: usize = 0>(N);
    ///     impl<N: Nat, const ARRLEN: usize> gub::Const for Doit<N, ARRLEN> {
    ///         type Type = &'static [MaybeUninit<u8>];
    ///         const VALUE: Self::Type = &{
    ///             let arr = to_binary_arr::<N>();
    ///             ArrApi::new(MaybeUninit::new(arr))
    ///                 .into_uninit_builtin::<ARRLEN>()
    ///         };
    ///     }
    ///     impl<N: Nat> gub::AcceptUpperBound for Doit<N> {
    ///         type Output = &'static [MaybeUninit<u8>];
    ///         const DESIRED_GENERIC: usize = nat::to_usize::<BinaryLen<N>>().unwrap();
    ///         type Eval<const ARRLEN: usize> = Doit<N, ARRLEN>;
    ///     }
    ///     let slice: &'static [MaybeUninit<u8>] = gub::eval_with_upper_bound::<Doit<N::Eval>>();
    ///     let (init, _) = slice.split_at(gub::desired_generic::<Doit<N::Eval>>());
    ///     // SAFETY: The first BinaryLen<N> items were initialized in to_binary_arr and this
    ///     // casts &[MaybeUninit<u8>] to &[u8], which is valid for initialized data
    ///     let init = unsafe {
    ///         core::slice::from_raw_parts(
    ///             init.as_ptr().cast::<u8>(),
    ///             init.len(),
    ///         )
    ///     };
    ///     match core::str::from_utf8(init) {
    ///         Ok(s) => s,
    ///         Err(_) => unreachable!(),
    ///     }
    /// }
    /// assert_eq!(to_str_binary::<nat::lit!(0b100100010100)>(), "100100010100");
    /// ```
    pub const fn into_uninit_builtin<const M: usize>(self) -> [MaybeUninit<T>; M] {
        // SAFETY:
        // - if M >= N, then transmuting through a union forgets `M - N` elements,
        //   which is safe.
        // - if M <= N, then transmuting through a union fills the rest of the array with
        //   uninitialized memory, which is valid in this context.
        unsafe {
            utils::union_transmute!(
                ArrApi::<A>, //
                [MaybeUninit::<T>; M],
                self
            )
        }
    }
}
