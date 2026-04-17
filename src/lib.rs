//! This crate provides type-level natural numbers, similar to [`typenum`](https://docs.rs/typenum/latest/typenum/).
//!
//! A type-level number is a type that represents a number. The [`Nat`] trait functions as the
//! "meta-type" of type-level numbers, i.e. to accept a type-level number, use a generic
//! parameter `N: Nat`.
//!
//! The use cases are the same as those of generic consts.
//!
//! ## Why this crate?
//! This crate differs from `typenum` in that [`Nat`] is not just a marker trait.
//! To perform arbitrary operations on a generic `N: Nat`, no extra bounds are
//! required.
//!
//! As a result, this crate is more expressive than `typenum` or the
//! `generic_const_exprs` feature.
//!
//! ### Motivating examples
//!
//! #### Concatenating arrays at compile time
//! Using `generic_const_exprs` or `typenum`/`generic-array`:
//! ```
#![cfg_attr(doctest, doc = "```\n```compile_fail")]
//! #![feature(generic_const_exprs)]
//! const fn concat_arrays_gcex<T, const M: usize, const N: usize>(
//!     a: [T; M],
//!     b: [T; N],
//! ) -> [T; M + N]
//! where
//!     [T; M + N]:, // Required well-formedness bound
//! {
//!     todo!() // Possible with unsafe code
//! }
//!
//! use generic_array::{GenericArray, ArrayLength};
//! const fn concat_arrays_tnum<T, M: ArrayLength, N: ArrayLength>(
//!     a: GenericArray<T, M>,
//!     b: GenericArray<T, N>,
//! ) -> GenericArray<T, typenum::op!(M + N)>
//! where // ArrayLength is not enough, we also need to add a bound for `+`
//!     M: std::ops::Add<N, Output: ArrayLength>,
//! {
//!     todo!() // Possible with unsafe code
//! }
//! ```
//! Using this crate:
//! ```
//! use gnat::{Nat, array::Arr};
//! const fn concat_arrays_gnat<T, M: Nat, N: Nat>(
//!     a: Arr<T, M>,
//!     b: Arr<T, N>,
//! ) -> Arr<T, gnat::eval!(M + N)> { // No extra bounds!
//!     a.concat_arr(b).retype() // There is even a method for this :)
//! }
//! ```
//! #### Const Recursion
//! Naively writing a function that recurses over the const parameter is impossible in
//! `generic_const_exprs` and `typenum`, since the recursive argument needs the same
//! bounds as the parameter:
//! ```compile_fail
//! #![feature(generic_const_exprs)]
//! fn recursive_gcex<const N: usize>() -> u32
//! where
//!     [(); N / 2]:, // The argument must be well-formed
//!     [(); (N / 2) / 2]:, // The argument's argument must be well-formed
//!     // ... need infinitely many bounds, even though N converges to 0
//! {
//!     if N == 0 {
//!         0
//!     } else {
//!         // The bounds above for N need to imply the same bounds for N / 2
//!         recursive_gcex::<{ N / 2 }>() + 1
//!     }
//! }
//!
//! use {std::ops::Div, typenum::{P2, Unsigned}};
//! fn recursive_tnum<N>() -> u32
//! where
//!     N: Unsigned + Div<P2>,
//!     N::Output: Unsigned + Div<P2>,
//!     <N::Output as Div<P2>>::Output: Unsigned + Div<P2>,
//!     // ... again, we would need this to repeat infinitely often
//! {
//!     if N::USIZE == 0 { // (Pretend this correctly handles overflow)
//!         0
//!     } else {
//!         recursive_tnum::<typenum::op!(N / 2)>() + 1
//!     }
//! }
//! ```
//! While this can be expressed using a helper trait like `trait RecDiv2: Unsigned { type Output: RecDiv2;  }`,
//! it is cumbersome and leaks into the bounds of every other calling function.
//!
//! Using this crate, the naive implementation without bounds just works:
//! ```
//! fn recursive_gnat<N: gnat::Nat>() -> u32 {
//!     if gnat::is_zero::<N>() {
//!         0
//!     } else {
//!         recursive_gnat::<gnat::eval!(N / 2)>() + 1
//!     }
//! }
//! assert_eq!(recursive_gnat::<gnat::lit!(10)>(), 4); // 10 5 2 1 0
//! ```
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

// Nat Implementation internals
mod internals;
mod uimpl;

// Macro implementation details
#[doc(hidden)]
pub mod __mac;

// internal utils
mod const_fmt;
mod maxint;
mod num;
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
    message = "`{Self}` is not a `gnat::Nat`",
    label = "expected to implement `gnat::Nat` directly",
    note = "if `gnat::NatExpr` is implemented, consider using `gnat::Eval<{Self}>`"
)]
pub trait Nat: Sized + 'static + internals::NatSealed + NatExpr<Eval = Self> {}

/// Trait for deferred [`Nat`] expressions.
///
/// This is not only a conversion trait. It is essential to the implementation of most operations.
/// See the [`mod@expr`] module.
///
/// A common pattern is to define operations like this:
/// ```
#[cfg_attr(doctest, doc = "```\n```compile_fail")]
/// struct Add<L, R>(L, R);
/// impl<L: gnat::NatExpr, R: gnat::NatExpr> gnat::NatExpr for Add<L, R> {
///     type Eval = ...;
/// }
/// ```
/// This can be abbreviated with the [`nat_expr`] macro.
#[diagnostic::on_unimplemented(
    message = "Cannot convert `{Self}` to a `gnat::Nat`",
    label = "must implement `gnat::NatExpr` to be used like a `Nat`"
)]
pub trait NatExpr {
    /// Evaluates to [`Nat`].
    type Eval: Nat;
}

/// Turns an integer literal into a [`Nat`].
///
/// If you need to convert a value stored in a `const` that is small, you
/// can instead convert it using [`consts::Usize`]. Otherwise consider
/// declaring it as a [`Nat`] instead and using [`to_usize`] to get the
/// value.
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
            ($crate::__mac::lit::_U)
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

/// Same as [`expr!`] wrapped in [`Eval`]. Useful for [`mod@array`] lengths.
///
/// # Examples
/// The [`mod@array`] module only accepts [`Nat`] for lengths, rather than [`NatExpr`] (for better type inference).
/// This macro is more convenient than [`expr!`]+[`Eval`]:
/// ```
/// use gnat::{Nat, array::*};
/// fn concat<T, M: Nat, N: Nat>(
///     a: Arr<T, M>,
///     b: Arr<T, N>,
/// ) -> Arr<T, gnat::eval! { M + N }> { // Alternatively: gnat::Eval<gnat::expr!(M + N)>
///     a.concat_arr(b).retype()
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
