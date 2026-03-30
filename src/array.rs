//! Drop-in replacement for builtin `[T; N]` arrays, using [`Nat`] for the length
//!
//! TODO: Examples
//!
//! # Oversized arrays
//! Because [`Nat`] is not restricted to the values of a [`usize`], it is possible to have array
//! types that have a [`Length`] exceeding [`usize::MAX`]. These arrays can only exist when the
//! item type is zero-sized, as they would otherwise exceed the object size limit of [`isize::MAX`].
//!
//! While these arrays can exist, they are only partially supported and many methods/impls will panic
//! when interacting with them. These methods/impls have a line in their documentation stating that
//! oversized arrays are unsupported.
//!
//! [`Nat`]: crate::Nat
//! [`Length`]: Array::Length

use crate::internals;

/// Trait for arrays whose length is measured as a [`Nat`](crate::Nat).
///
/// # Safety
/// Currently, this trait is sealed.
///
/// The guarantees made by types implementing [`Array`] with `Item = T` and `Length = N` include the following:
/// - Must have the same layout as an equivalent `[T; N]` builtin array. If `N > usize::MAX` (only
///   relevant for ZSTs), they still act as if there was such an array type.
/// - Arrays have no additional safety requirements over builtin arrays whatsoever. In particular:
///     - They have the same semantics as the equivalent builtin array with respect to arbitrary auto traits,
///       assuming there is no manual implementation from the crate declaring the trait.
///     - They also have the same semantics as the equivalent builtin array with respect to drop glue.
///       They never have a [`Drop`] implementation and only have the drop glue from their item type.
///       When the array is dropped, exactly `N` instances of `T` are dropped
///       ([in order](https://doc.rust-lang.org/reference/destructors.html)), even if `N > usize::MAX`.
///     - Note that together with the point about the layout, this is sufficient to perform arbitrary
///       casts and transmutes between equivalent array types. See the [`arr_api`] module.
/// - `MaybeUninit<[T; N]>` and `[MaybeUninit<T>; N]` are considered equivalent for the purposes of
///   this trait.
/// - Arrays of arrays are equivalent to their flattened versions, e.g. `[[i32; 4]; 3]` is
///   equivalent to `[i32; 12]`, which is equivalent to `[[i32; 3]; 4]`.
pub unsafe trait Array: Sized + internals::ArraySealed {
    /// The item type of the array.
    type Item;
    /// The length of the array as a type-level integer.
    type Length: crate::Nat;
}

pub use crate::internals::array_types::*;

/// A newtype adapter for an array implementor that the API relating to arrays.
///
/// The struct has a second generic parameter which is always the item of the array.
/// This gives better lifetime inference for the item type. Some methods, such as
/// [`Self::each_ref`] and the [`Index`](core::ops::Index) impl would not compile
/// the way they are written without it.
#[repr(transparent)]
pub struct ArrApi<A: Array<Item = T>, T = <A as Array>::Item> {
    /// The array being wrapped.
    ///
    /// If you are getting errors trying to move out of this in `const` contexts, try using
    /// [`Self::into_inner`].
    pub inner: A,
}

/// Adapter that turns two arrays with the same item type into one long array.
///
/// This is just a `repr(C)` pair.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct ArrConcat<A, B>(pub A, pub B);

impl<A, B> ArrConcat<A, B> {
    /// Wraps the fields in [`ManuallyDrop`](core::mem::ManuallyDrop).
    ///
    /// This may make it easier to destructure the result in `const` contexts.
    ///
    /// Note that the result of this method has the same layout as `self`, but it is not an [`Array`].
    #[must_use = "The values returned by this function are wrapped in ManuallyDrop and may need to be dropped"]
    pub const fn manually_drop_parts(
        self,
    ) -> ArrConcat<core::mem::ManuallyDrop<A>, core::mem::ManuallyDrop<B>> {
        use core::mem::ManuallyDrop;
        // SAFETY: ArrConcat<A, B> ~ repr(C) (A, B) ~ repr(C) (ManuallyDrop<A>, ManuallyDrop<B>) ~ Concat<...>
        unsafe {
            crate::utils::union_transmute!(
                ArrConcat::<A, B>,
                ArrConcat::<ManuallyDrop<A>, ManuallyDrop<B>>,
                self,
            )
        }
    }
}

/// Adapter that turns an array of arrays into one long array of items.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ArrFlatten<A>(pub A);

impl<A> ArrFlatten<A> {
    /// Returns the field of this struct.
    pub const fn into_inner(self) -> A {
        // SAFETY: repr(transparent)
        unsafe {
            crate::utils::union_transmute!(
                ArrFlatten::<A>, //
                A,
                self,
            )
        }
    }
}

/// A wrapper for a [`MaybeUninit`](core::mem::MaybeUninit) array that acts as a [`Vec`]
/// (with limited capacity), as well as a drop guard for the initialized items.
///
/// # Drop implementation
/// This type currently has drop glue that does nothing except drop its elements, regardless
/// of whether the item type needs to be dropped.
/// This may be annoying in some `const` code as there is currently no way to make the `Drop`
/// implementation `const` for item types that can be dropped in `const`.
///
/// These workarounds exist:
/// - Using [`drop_items`]/[`assert_empty`](Self::assert_empty) if it's just a local variable
///   that needs to be dropped.
/// - Wrapping this type in [`ManuallyDrop`](core::mem::ManuallyDrop) if the item type is known
///   to have no drop glue. The contents of [`ManuallyDrop`](core::mem::ManuallyDrop) can be
///   accessed in `const` using [`const_util::mem::man_drop_mut`].
/// - Using [`Arr`]/[`CopyArr`] instead if the item type has a default value, or a layout niche
///   with [`Option`].
///
/// # Oversized arrays
/// [Oversized arrays](crate::array#oversized-arrays) are never supported. Attempting to create
/// such an [`ArrVecApi`] results in a panic at runtime. Note that this is not guaranteed and
/// may be relaxed in the future.
#[cfg_attr(not(doc), repr(transparent))]
pub struct ArrVecApi<A: Array<Item = T>, T = <A as Array>::Item>(
    /// Encapsulates the drop impl to allow future changes
    arr_vec::ArrVecDrop<A, T>,
);

/// Alias for [`ArrVecApi`] around [`Arr`].
pub type ArrVec<T, N> = ArrVecApi<Arr<T, N>>;

/// A wrapper for a [`MaybeUninit`](core::mem::MaybeUninit) array that acts as a
/// [`VecDeque`](std::collections::VecDeque) (with limited capacity), as well as
/// a drop guard for the initialized items.
///
/// Note that unlike [`ArrApi`], all methods on this type may panic if the array length
/// exceeds [`usize::MAX`], without explicitly mentioning this in their docs.
///
/// # Drop implementation
/// See [`ArrVecApi#drop-implementation`]
#[cfg_attr(not(doc), repr(transparent))]
pub struct ArrDeqApi<A: Array<Item = T>, T = <A as Array>::Item>(
    /// Encapsulates the drop impl to allow future changes
    arr_deq::ArrDeqDrop<A, T>,
);

/// Alias for [`ArrDeqApi`] around [`Arr`].
pub type ArrDeq<T, N> = ArrDeqApi<Arr<T, N>>;

/// Helper macro that drops an [`ArrApi`], [`ArrVecApi`] or [`ArrDeqApi`], including in
/// const contexts, by dropping each of its items.
///
/// Currently, dropping in const contexts is only possible if the item type does
/// not have any drop glue or implementation. This macro is preferrable over
/// [`core::mem::forget`] in that it will give a compile error if the item type
/// cannot be dropped in the current context.
///
/// Once `const Destruct` bounds become stabilized, this macro can be rewritten
/// to drop the items in place.
///
/// # Examples
/// ```
/// use gnat::array::*;
/// const fn double_each<A: Array<Item = i32>>(vec: ArrVecApi<A>) -> ArrVecApi<A> {
///     let mut out = ArrVecApi::new();
///     let mut input = vec.as_slice();
///     while let [next, rest @ ..] = input {
///         out.push(*next * 2);
///         input = rest;
///     }
///     drop_items!(vec);
///     out
/// }
/// ```
#[macro_export]
#[doc(hidden)]
macro_rules! __drop_items {
    [ $arr:expr ] => {{
        let mut __guard = $crate::__mac::arr::ArrDrop($arr).enter();
        if __guard.needs_drop() {
            while let Some(_) = __guard.pop_next() {}
        }
        __guard.discard();
    }};
}
pub use __drop_items as drop_items;

pub(crate) mod container;
pub(crate) mod helper;

mod arr_deq;
mod arr_vec;
mod impls;

pub mod arr_api;
