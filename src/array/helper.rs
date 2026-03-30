use core::marker::PhantomData;

use crate::{Nat, array::*, const_fmt};

#[track_caller]
pub(crate) const fn arr_len<A: Array>() -> usize {
    const fn doit<N: Nat>() -> usize {
        let precalc = const {
            match crate::to_usize::<N>() {
                Some(n) => Ok(n),
                None => Err(const_fmt::fmt![
                    "Array length ",
                    PhantomData::<N>,
                    " exceeds the maximum value for a usize",
                ]),
            }
        };
        match precalc {
            Ok(n) => n,
            Err(err) => err.panic(),
        }
    }
    doit::<A::Length>()
}

/// Checks some invariants of an array type.
///
/// Note that the array's [`size_of`] is used by this function. If the array type in question
/// exceeds the maximum size for the architecture, this will by itself cause a
/// post-monomorphization error.
pub(crate) const fn arr_impl_ubcheck<A: Array>() {
    #[cfg(debug_assertions)]
    const {
        assert!(
            align_of::<A>() == align_of::<A::Item>(),
            "UB: Array alignment must be the same as that of item"
        );
        let item_size = size_of::<A::Item>();
        let arr_size = size_of::<A>();
        if let Some(arr_len) = crate::to_usize::<A::Length>() {
            let calc_size = arr_len.checked_mul(item_size);
            assert!(
                calc_size.is_some() && arr_size == calc_size.unwrap(),
                "UB: Array size must be equal to item size multiplied by length"
            )
        } else {
            assert!(
                item_size == 0 && arr_size == 0,
                "UB: Array with length exceeding usize::MAX must be ZST"
            )
        }
    }
}

/// # Panics
/// If `A::Length > usize::MAX`
///
/// # Safety
/// This operation is strictly the same as [`core::ptr::slice_from_raw_parts_mut`] with `ptr.cast()` as
/// the first argument and [`ArrApi::<A>::length()`] as the second.
///
/// Due to the guarantees made by [`Array`], this should generally mean that the returned pointer
/// is valid for the same operations as `ptr`. In particular, if `ptr` is valid for some operation
/// on `A::Length`  values of `A::Item` with array layout, then the returned pointer is valid for
/// that operation on the corresponding slice.
#[track_caller]
pub(crate) const fn unsize_raw_mut<A: Array>(ptr: *mut A) -> *mut [A::Item] {
    core::ptr::slice_from_raw_parts_mut(ptr.cast(), arr_len::<A>())
}
