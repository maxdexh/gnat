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
pub mod small;

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
/// implemented. See the [`expr`] module.
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
    ($l:literal) => {
        $crate::__mac::proc::__lit! {
            ($l)
            ($crate::__mac::lit::_DirectAppend)
            ($crate::small::N0)
            ($crate::small::N1)
        }
    };
}
