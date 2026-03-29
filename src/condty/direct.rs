//! Functions for [`CondTy`].

use crate::{NatExpr, condty::CondTy, utils};

/// Unwraps a `True` instance a [`CondTy`].
///
/// # Panics
/// If `C` is zero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn unwrap_true<C: NatExpr, T, F>(tern: CondTy<C, T, F>) -> T {
    ctx!(
        //
        |c| c.unwrap_true(tern),
        |_| panic!("Call to `unwrap_true` with false condition"),
        C,
    )
}

/// Creates a `True` instance of [`CondTy`]
///
/// # Panics
/// If `C` is zero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn new_true<C: NatExpr, T, F>(value: T) -> CondTy<C, T, F> {
    ctx!(
        //
        |c| c.new_true(value),
        |_| panic!("Call to `new_true` with false condition"),
        C,
    )
}

/// Unwraps a `False` instance a [`CondTy`].
///
/// # Panics
/// If `C` is nonzero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn unwrap_false<C: NatExpr, T, F>(tern: CondTy<C, T, F>) -> F {
    ctx!(
        //
        |_| panic!("Call to `unwrap_false` with true condition"),
        |c| c.unwrap_false(tern),
        C,
    )
}

/// Creates a `False` instance of [`CondTy`]
///
/// # Panics
/// If `C` is nonzero (even if `T` and `F` are the same type).
#[track_caller]
pub const fn new_false<C: NatExpr, T, F>(value: F) -> CondTy<C, T, F> {
    ctx!(
        //
        |_| panic!("Call to `new_false` with true condition"),
        |c| c.new_false(value),
        C,
    )
}

/// Turns reference to [`CondTy`] into [`CondTy`] of reference.
pub const fn as_ref<C: NatExpr, T, F>(tern: &CondTy<C, T, F>) -> CondTy<C, &T, &F> {
    // SAFETY: Same type under type map `X -> &'a X` for some 'a
    unsafe { utils::same_type_transmute!(&CondTy::<C, T, F>, CondTy::<C, &T, &F>, tern) }
}

/// Turns mutable reference to [`CondTy`] into [`CondTy`] of mutable reference.
pub const fn as_mut<C: NatExpr, T, F>(tern: &mut CondTy<C, T, F>) -> CondTy<C, &mut T, &mut F> {
    // SAFETY: Same type under type map `X -> &'a mut X` for some 'a
    unsafe {
        utils::same_type_transmute!(&mut CondTy::<C, T, F>, CondTy::<C, &mut T, &mut F>, tern)
    }
}

/// Turns `CondTy<C, T, T>` into `T`
///
/// This function is effectively the identity function.
pub const fn unwrap_trivial<C: NatExpr, T>(tern: CondTy<C, T, T>) -> T {
    // SAFETY: CondTy<C, T, T> is the same type type as T or T
    unsafe { crate::utils::same_type_transmute!(CondTy::<C, T, T>, T, tern) }
}

/// Turns `T` into `CondTy<C, T, T>`
///
/// This function is effectively the identity function.
pub const fn new_trivial<C: NatExpr, T>(inner: T) -> CondTy<C, T, T> {
    // SAFETY: CondTy<C, T, T> is the same type type as T or T
    unsafe { crate::utils::same_type_transmute!(T, CondTy::<C, T, T>, inner) }
}
