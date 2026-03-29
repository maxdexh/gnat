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

// Uint Implementation internals
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
pub mod small;
pub mod uint;
pub mod uops;

/// A type-level non-negative integer
///
/// See the [crate level documentation](crate).
///
/// It is guaranteed (including to unsafe code) that there is a one-to-one correspondence between
/// the non-negative integers and the set of types that can be observed to implement this trait.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a `Uint`",
    label = "`{Self}` was expected to implement `Uint` directly",
    note = "Consider using `uint::From<{Self}>` if `{Self}: NatExpr`"
)]
pub trait Uint: Sized + 'static + internals::UintSealed + NatExpr<Eval = Self> {}

/// A type that can be turned into a [`Uint`]
///
/// This is not only a conversion trait, but forms an important part in how most operations are
/// implemented. See the [`uops`] module.
#[diagnostic::on_unimplemented(
    message = "Cannot convert `{Self}` to a `Uint`",
    label = "To be used like a `Uint`, `{Self}` must implement `NatExpr`"
)]
pub trait NatExpr {
    /// Performs the conversion to [`Uint`].
    type Eval: Uint;
}
