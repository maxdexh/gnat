//! Types conditional on a [`Nat`](crate::Nat)
//!
//! This module provides conditional types based on `Nat`s.

macro_rules! ctx {
    (|$ctxt:pat_param| $true:expr, |$ctxf:pat_param| $false:expr $(, $($C:ty $(,)?)?)?) => {{
        match $crate::__mac::cond::hold::<$($($C)?)?>() {
            Ok($ctxt) => $true,
            Err($ctxf) => $false,
        }
    }};
}
pub(crate) use ctx;

pub mod direct;

use core::mem::ManuallyDrop;

use crate::NatExpr;

/// Direct conditional type based on a [`Nat`](crate::Nat).
///
/// "Direct" in this context refers to the fact that it is not newtype wrapped (unlike [`CondResult`]),
/// so if `Cond` is nonzero, then `CondTy<Cond, T, F>` is exactly the same type as `T`.
/// Otherwise it is the same type as `F`.
///
/// As a consequence, any generic `TFun<CondTy<C, T, F>>` is exactly the same type as `TFun<T>` or
/// `TFun<F>` and therefore is valid to transmute if the value of `C` is known (can be checked at
/// runtime) or `T = F` (which may follow from other invariants, such as [`Nat`](crate::Nat) uniqueness).
/// This applies even to types with unspecified layout such as `TFun<X> = Vec<X>` or type
/// projections like `TFun<X> = <X as Tr>::Assoc`.
///
/// This type's disadvantage compared to [`CondResult`] are the usual use cases for a newtype wrapper:
/// It is not possible to use impls of `T` and `F` if `C` is generic, it does not play nicely with
/// type inference (especially of `C`) and it can't have methods. Its "methods" are defined as
/// functions in the [`direct`] module.
#[allow(type_alias_bounds)]
pub type CondTy<Cond: NatExpr, True, False> = crate::internals::CondTy<Cond::Eval, True, False>;

/// A [`Result`]-like wrapper for [`CondTy`]
///
/// If `Cond` is nonzero, instances of this type are always `Ok` instances with inner type `T`,
/// otherwise they are always `Err` instances with inner type `E`.
///
/// This type is a [`repr(transparent)`](https://doc.rust-lang.org/reference/type-layout.html#r-layout.repr.transparent)
/// wrapper around the corresponding instance kind. This means that `Ok` instances have the same layout as
/// `T` and `Err` instances have the same layout as `E`. No space is required to store the instance
/// kind.
#[repr(transparent)]
#[must_use]
pub struct CondResult<Cond: NatExpr, T, E> {
    /// The underlying [`CondTy`]. The struct is `repr(transparent)` around this
    /// field.
    pub inner: CondTy<Cond, T, E>,
}
impl<C: NatExpr, T, E> CondResult<C, T, E> {
    /// Turns this result into its wrapped [`CondTy`] by moving out of
    /// [`self.inner`](Self::inner).
    ///
    /// Also works in const contexts, even when generics or drop impls are involved.
    pub const fn into_inner(self) -> CondTy<C, T, E> {
        // SAFETY: repr(transparent)
        unsafe {
            crate::utils::union_transmute!(
                CondResult::<C, T, E>, //
                CondTy::<C, T, E>,
                self,
            )
        }
    }

    /// Whether instances of this type are `Ok`
    pub const IS_OK: bool = !crate::is_zero::<C>();

    /// Whether instances of this type are `Err`
    pub const IS_ERR: bool = !Self::IS_OK;

    /// Whether this result is `Ok`
    pub const fn is_ok(&self) -> bool {
        Self::IS_OK
    }

    /// Whether this result is `Err`
    pub const fn is_err(&self) -> bool {
        Self::IS_ERR
    }

    /// Equivalent of [`Result::as_ref`].
    pub const fn as_ref(&self) -> CondResult<C, &T, &E> {
        CondResult {
            inner: direct::as_ref::<C, _, _>(&self.inner),
        }
    }

    /// Equivalent of [`Result::as_mut`].
    pub const fn as_mut(&mut self) -> CondResult<C, &mut T, &mut E> {
        CondResult {
            inner: direct::as_mut::<C, _, _>(&mut self.inner),
        }
    }

    /// Turns this result into a regular builtin [`Result`].
    #[expect(clippy::missing_errors_doc)]
    pub const fn into_std(self) -> Result<T, E> {
        ctx!(
            //
            |c| Ok(c.unwrap_ok(self)),
            |c| Err(c.unwrap_err(self))
        )
    }

    /// Creates an `Ok` instance, assuming [`Self::IS_OK`]
    ///
    /// # Panics
    /// If [`Self::IS_ERR`]
    #[track_caller]
    pub const fn new_ok(ok: T) -> Self {
        ctx!(
            //
            |c| c.new_ok(ok),
            |_| panic!("Call to `new_ok` on Err type")
        )
    }

    /// Equivalent of [`Result::unwrap`], but `const` and without the [`Debug`] bound.
    ///
    /// # Panics
    /// If [`Self::IS_ERR`]
    pub const fn unwrap(self) -> T {
        ctx!(
            //
            |c| c.unwrap_ok(self),
            |_| panic!("Call to `unwrap` on Err type")
        )
    }

    /// Creates an `Err` instance, assuming [`Self::IS_ERR`]
    ///
    /// # Panics
    /// If [`Self::IS_OK`]
    #[track_caller]
    pub const fn new_err(err: E) -> Self {
        ctx!(
            //
            |_| panic!("Call to `new_err` on Ok type"),
            |c| c.new_err(err),
        )
    }

    /// Equivalent of [`Result::unwrap_err`], but `const` and without the [`Debug`] bound.
    ///
    /// # Panics
    /// If [`Self::IS_OK`]
    pub const fn unwrap_err(self) -> E {
        ctx!(
            //
            |_| panic!("Call to `unwrap_err` on Ok type"),
            |c| c.unwrap_err(self),
        )
    }

    /// Like [`Self::into_std`], but wraps the variants in [`ManuallyDrop`].
    ///
    /// This allows destructive matching in `const` contexts, even when generics or [`Drop`] impls are involved.
    #[expect(clippy::missing_errors_doc)]
    #[must_use = "The contents of this `Result` may need to be dropped, and it may be an `Err` variant, which needs to be handled."]
    pub const fn into_manual_drop_std(self) -> Result<ManuallyDrop<T>, ManuallyDrop<E>> {
        ctx!(
            //
            |c| Ok(ManuallyDrop::new(c.unwrap_ok(self))),
            |c| Err(ManuallyDrop::new(c.unwrap_err(self))),
        )
    }
}

impl<C: NatExpr, T> CondResult<C, T, T> {
    /// Creates a result where both instance kinds have the same type.
    pub const fn new_trivial(inner: T) -> Self {
        Self {
            inner: direct::new_trivial::<C, _>(inner),
        }
    }

    /// Unwraps a result where both instance kinds have the same type.
    pub const fn unwrap_trivial(self) -> T {
        direct::unwrap_trivial::<C, _>(self.into_inner())
    }
}

/// An [`Option`]-like wrapper for [`CondTy`]
///
/// This struct is a `repr(transparent)` newtype wrapper for [`CondTy<Cond, T, ()>`].
/// If `Cond` is zero, then this struct is a `repr(transparent)` wrapper around `E`.
/// Otherwise, it is a `repr(transparent)` wrapper around `()`.
#[repr(transparent)]
pub struct CondOption<Cond: NatExpr, T> {
    /// The underlying [`CondTy`]. The struct is `repr(transparent)` around this
    /// field.
    pub inner: CondTy<Cond, T, ()>,
}

impl<C: NatExpr, T> CondOption<C, T> {
    /// Turns this option into its wrapped [`CondTy`] by moving out of
    /// [`self.inner`](Self::inner).
    ///
    /// Also works in const contexts, even when generics or drop impls are involved.
    pub const fn into_inner(self) -> CondTy<C, T, ()> {
        // SAFETY: repr(transparent)
        unsafe {
            crate::utils::union_transmute!(
                CondOption::<C, T>, //
                CondTy::<C, T, ()>,
                self,
            )
        }
    }

    /// Whether instances of this type are `Some`
    pub const IS_SOME: bool = !crate::is_zero::<C>();

    /// Whether instances of this type are `None`
    pub const IS_NONE: bool = !Self::IS_SOME;

    /// Whether this option is `Some`
    pub const fn is_some(&self) -> bool {
        Self::IS_SOME
    }

    /// Whether this result is `None`
    pub const fn is_none(&self) -> bool {
        Self::IS_NONE
    }

    /// Turns this option into a regular builtin [`Option`].
    pub const fn into_std(self) -> Option<T> {
        ctx!(
            //
            |c| Some(c.unwrap_some(self)),
            |c| {
                c.drop_none(self);
                None
            },
        )
    }

    /// Equivalent of [`Option::as_ref`]
    pub const fn as_ref(&self) -> CondOption<C, &T> {
        ctx!(
            //
            |c| c.new_some(c.unwrap_true(direct::as_ref::<C, _, _>(&self.inner))),
            |c| c.new_none(),
        )
    }

    /// Equivalent of [`Option::as_mut`]
    pub const fn as_mut(&mut self) -> CondOption<C, &mut T> {
        ctx!(
            //
            |c| c.new_some(c.unwrap_true(direct::as_mut::<C, _, _>(&mut self.inner))),
            |c| c.new_none(),
        )
    }

    /// Equivalent of [`Option::unwrap`]
    ///
    /// # Panics
    /// If [`Self::IS_NONE`]
    pub const fn unwrap(self) -> T {
        ctx!(
            //
            |c| c.unwrap_some(self),
            |_| panic!("Call to `unwrap` on None instance"),
        )
    }

    /// Discards the value in a `const` context, assuming that [`Self::IS_NONE`]
    ///
    /// # Panics
    /// If [`Self::IS_SOME`]
    #[track_caller]
    pub const fn assert_none(self) {
        ctx!(
            //
            |_| panic!("Call to `assert_none` on Some instance"),
            |c| c.drop_none(self),
        )
    }
}
