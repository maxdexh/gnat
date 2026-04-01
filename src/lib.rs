//! TODO: Docs go here
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(any(test, doc, feature = "std")), no_std)]
#![cfg_attr(test, recursion_limit = "1024")]
#![warn(
    clippy::nursery,
    clippy::missing_panics_doc,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::undocumented_unsafe_blocks,
    missing_docs
)]
#![allow(clippy::redundant_pub_crate, clippy::use_self)]

#[cfg(feature = "alloc")]
extern crate alloc;

extern crate self as gnat;

// Nat Implementation internals
mod internals;
mod uimpl;

// Macro implementation details
#[doc(hidden)]
pub mod __mac;

// internal utils
mod const_fmt;
mod maxint;
mod utils;

// Public API
pub mod array;
pub mod condty;
pub mod consts;
pub mod expr;

mod nat_api;
pub use nat_api::*;

/// Trait for type-level natural numbers.
///
/// See the [crate level documentation](crate).
///
/// It is guaranteed that there is a one-to-one correspondence between
/// the natural numbers including zero and the types that implement this trait.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a `Nat`",
    label = "`{Self}` was expected to implement `Nat` directly",
    note = "Consider using `crate::Eval<{Self}>` if `{Self}: NatExpr`"
)]
pub trait Nat: Sized + 'static + internals::NatSealed + NatExpr<Eval = Self> {}

/// Trait for deferred [`Nat`] expressions.
///
/// This is not only a conversion trait, but forms an important part in how most operations are
/// implemented. See the [`mod@expr`] module.
#[diagnostic::on_unimplemented(
    message = "Cannot convert `{Self}` to a `Nat`",
    label = "To be used like a `Nat`, `{Self}` must implement `NatExpr`"
)]
pub trait NatExpr {
    /// Performs the conversion to [`Nat`].
    type Eval: Nat;
}

/// Turns an integer literal into a [`Nat`].
///
/// If you have a small constant value that is not a literal, use [`consts::Usize`].
///
/// # Examples
/// ```
/// #![recursion_limit = "1024"] // `lit!` doesn't recurse, the type is just long
///
/// assert_eq!(gnat::to_u128::<gnat::lit!(1)>(), Some(1));
/// assert_eq!(
///     gnat::to_u128::<gnat::lit!(100000000000000000000000000000)>(),
///     Some(100000000000000000000000000000),
/// )
/// ```
#[macro_export]
macro_rules! lit {
    (0) => {
        $crate::__mac::lit::_0
    };
    (1) => {
        $crate::__mac::lit::_1
    };
    ($l:literal) => {
        $crate::__mac::proc::__lit! {
            ($l)
            ($crate::__mac::lit::_DirectAppend)
            ($crate::__mac::lit::_0)
            ($crate::__mac::lit::_1)
        }
    };
}

/// Converts expression syntax into [`NatExpr`] type expressions.
///
/// # Examples
/// ```
/// # macro_rules! chk_same_type {
/// #     ($l:ty, $r:ty $(,)?) => { let _: $l = panic!() as $r; };
/// # }
/// fn with_exprs<A: gnat::NatExpr, B: gnat::NatExpr>() {
///     // Unsuffixed literals are translated to `Nat`s
///     chk_same_type!(
///         gnat::expr! { 2 },
///         gnat::lit!(2),
///     );
///     // Most operators map to their correspondingly named operation
///     chk_same_type!(
///         gnat::expr! { A + B },
///         gnat::expr::Add<A, B>,
///     );
///     chk_same_type!(
///         gnat::expr! { A == B },
///         gnat::expr::Eq<A, B>,
///     );
///     // Subtraction is saturating
///     chk_same_type!(
///         gnat::expr! { A - B },
///         gnat::expr::SatSub<A, B>,
///     );
///     // `!` uses logical negation
///     chk_same_type!(
///         gnat::expr! { !A },
///         gnat::expr::IsZero<A>,
///     );
///     // Function calls translate to type paths
///     chk_same_type!(
///         gnat::expr! { gnat::expr::AbsDiff(A, B) },
///         gnat::expr::AbsDiff<A, B>,
///     );
///     // Expression paths also translate to type paths
///     chk_same_type!(
///         gnat::expr! { gnat::expr::AbsDiff::<A, B> },
///         gnat::expr::AbsDiff<A, B>,
///     );
///     // Conditionals are also supported
///     chk_same_type!(
///         gnat::expr! { if A { B } else { 2 * B } },
///         gnat::expr::If<
///             A,
///             B,
///             gnat::expr::Mul<gnat::lit!(2), B>,
///         >,
///     );
/// }
/// ```
#[macro_export]
#[cfg(feature = "macros")]
macro_rules! expr {
    { $($t:tt)* } => { $crate::__mac::proc::expr!($($t)*) };
}

/// Same as [`expr!`] wrapped in [`Eval`]. Useful for use with [`mod@array`].
///
/// # Examples
/// The [`mod@array`] uses [`Nat`] instead of [`NatExpr`] (for better type inference), so
/// this is more convenient than [`expr!`]+[`Eval`]:
/// ```
/// use gnat::{Nat, array::*};
/// fn concat<T, M: Nat, N: Nat>(a: Arr<T, M>, b: Arr<T, N>) -> Arr<T, gnat::eval! { M + N }> {
///     a.concat(b).retype()
/// }
/// ```
#[macro_export]
#[cfg(feature = "macros")]
macro_rules! eval {
    { $($t:tt)* } => { $crate::Eval<$crate::expr!($($t)*)> };
}

/// Converts type alias syntax into a [`NatExpr`] impl.
///
/// # Examples
/// ```
/// #[gnat::nat_expr]
/// type Factorial<N: gnat::NatExpr> = gnat::expr! {
///     if N {
///         Factorial(gnat::Eval(N - 1)) * N
///     } else {
///         1
///     }
/// };
/// assert_eq!(
///     gnat::to_u128::<Factorial<gnat::lit!(5)>>(),
///     Some(120),
/// );
/// ```
/// Equivalent code without this attribute:
/// ```
/// struct Factorial<N>(N);
/// impl<N: gnat::NatExpr> gnat::NatExpr for Factorial<N> {
///     type Eval = gnat::eval! {
///         if N {
///             Factorial(gnat::Eval(N - 1)) * N
///         } else {
///             1
///         }
///     };
/// }
/// ```
#[cfg(feature = "macros")]
pub use gnat_proc::nat_expr;
