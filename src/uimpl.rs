//! Internal types implementing the `Nat` trait
//!
//! Names are kept short to keep them
//! readable as binary numbers in error messages.
//!
//! These are private because they may be changed in the future.
pub struct _1(());
pub struct _0(());
pub struct _U<H, P>(H, P);
