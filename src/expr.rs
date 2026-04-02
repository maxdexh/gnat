//! Module with operations for [`Nat`]s.
//!
//! # Laziness
//! Operations implemented as a struct implementing [`NatExpr`] are lazy in the sense that
//! they will only be evaluated to a [`Nat`] instance if the associated [`NatExpr::Eval`]
//! is accessed.
//!
//! Most importantly, [`If`] is implemented such that only the necessary branch is evaluated,
//! which means it is possible to do recursion with it.
//!
//! All operations in this module are lazy. In order to get a [`Nat`] from them, e.g. for use
//! with [arrays](crate::array), use [`crate::Eval`] or [`crate::eval!`] to evaluate them.
//!
//! # Primitive operations
//! Operations that are implemented through an (internal) associated type on [`Nat`] are called primitive.
//!
//! All operations in this module are implemented on top of the following primitive operations:
//! - [`PopBit<N>`] removes the last bit of [`N::Eval`](NatExpr).
//!     - Evaluates like `N.eval() / 2`
//! - [`LastBit<N>`] gets the last bit of [`N::Eval`](NatExpr).
//!     - Evaluates like `N.eval() % 2`
//! - [`PushBit<N, B>`] pushes [`B::Eval`](NatExpr) as a bit to the end of [`N::Eval`](NatExpr)
//!     - Evaluates like `2 * N.eval() + (B.eval() != 0) as _`
//! - [`If<C, T, F>`] evaluates to [`T::Eval`](NatExpr) if `C` is nonzero, otherwise
//!   to [`F::Eval`](NatExpr). Only the necessary [`NatExpr::Eval`] projection is performed.
//!     - Evaluates like `if C != 0 { T.eval() } else { F.eval() }`
//!
//! These primitives, together with recursive [`NatExpr`] implementations,
//! form a [Turing-complete](https://en.wikipedia.org/wiki/Turing_completeness) system.
//! Any computable function on the natural numbers can be implemented.
//!
//! # Recursion
//! The way to implement an operation where the output requires looking at the entire number is to
//! do it recursively. Regular type aliases do not support recursion since they are eagerly
//! expanded (see error E0391 "cycle detected when expanding type alias").
//!
//! Instead, one has to go through [`NatExpr`] to make the operation lazy and use [`If`] to exit the
//! recursion. For example, consider this implementation of [`BitAnd`]:
//! ```
//! use gnat::{NatExpr, expr::*};
//!
//! #[gnat::nat_expr]
//! type MyBitAnd<L: NatExpr, R: NatExpr> = gnat::expr! {
//!     if L {
//!         PushBit(
//!             // recurse on the tails and append the head
//!             MyBitAnd(
//!                 gnat::Eval(PopBit(R)),
//!                 gnat::Eval(PopBit(L)),
//!             ),
//!             if LastBit(L) { LastBit(R) } else { 0 }, // logical AND
//!         )
//!     } else {
//!         0 // base case, 0 & R = 0
//!     }
//! };
//! ```
//! Because `MyBitAnd` and [`PushBit`] are lazy and [`If`] only accesses
//! [`NatExpr::Eval`] in the required branch, this will exit when `L = 0`,
//! without getting stuck in an infinite loop.
//!
//! Note the application of [`crate::Eval`] to [`PopBit`]. This is not strictly
//! necessary, but without it, the input to `MyBitAnd` becomes more deeply nested
//! on each recursive evaluation (`PopBit<PopBit<...>>`), which causes `MyBitAnd`
//! to take longer to compute (longer compile times). Evaluating in each step causes
//! the level of nesting to decrease, since the number becomes smaller.
//!
//! Similarly, switching the order of `R` and `L` on each recursion also terminates
//! faster (for `R` much larger than `L`), at no extra cost.
//!
//! # Opaqueness
//! There is another primitive operation, [`Opaque<P, Out>`].
//! It is implemented to always return `Out::Eval`, but it still goes through a projection on `P::Eval`.
//!
//! This means that something like `Eval<Opaque<A, Opaque<B, Func<A, B>>>>` will always be the same
//! as `Eval<Func<A, B>>`, except that the compiler won't know this until it actually knows the
//! value of both `A` and `B`. The benefits of this are:
//! - If `Func` is recursive over only one of its arguments, then if we do something like
//!   `Eval<Func<UsizeMax, B>>`, where `B` is a generic parameter, then the compiler will try to
//!   normalize all the recursions away, since it knows how to evaluate `If<A, ...>`, since `A` is
//!   known. This can cause unexpected "overflow while evaluating" errors.
//!
//!   Wrapping `Func` in `Opaque` causes the evaluation to be deferred until both arguments are
//!   known, which prevents this.
//! - Making `Func` public API risks exposing implementation details due to type inferrence.
//!   Wrapping a hidden [`NatExpr`] implementor in `Opaque` minimizes the amount of information
//!   about the [`Nat`] that is returned, which means that the implementation can be changed after
//!   the fact.
//!
//! In the example from before, the following is an almost exact reimplementation of [`BitAnd`]:
//! ```
//! # use gnat::expr::BitAnd as MyBitAnd;
//! use gnat::expr::Opaque;
//! #[gnat::nat_expr]
//! type MyBitAndFinal<L: gnat::NatExpr, R: gnat::NatExpr> = Opaque<
//!     L,
//!     Opaque<
//!         R,
//!         MyBitAnd<L, R>,
//!     >,
//! >;
//! ```

#[expect(unused_imports)] // for docs
use crate::{Nat, NatExpr};
use crate::{internals::InternalOp, utils::apply};

/// Input format:
/// ```compile_fail
/// #[apply(nat_expr)]
/// pub type A<P1: NatExpr, P2: NatExpr, ...> = $Val;
/// ```
///
/// Output format:
/// ```compile_fail
/// pub struct A<P1, P2, ...>(P1, P2, ...);
/// impl<P1: NatExpr, P2: NatExpr, ...> NatExpr for A<P1, P2, ...> {
///     type Eval = gnat::Eval<$Val>;
/// }
/// ```
macro_rules! nat_expr {
    (
        (
            $(eval_attrs = { $(#[$eval_attrs:meta])* },)?
        )
        $(#[$attr:meta])*
        $v:vis type $Name:ident<$($P:ident: $Bound:path $(= $Def:ty)?),* $(,)?> = $Val:ty;
    ) => {
        $(#[$attr])*
        $v struct $Name<$($P $(= $Def)?),*>($($P),*);
        impl<$($P: $Bound),*> crate::NatExpr for $Name<$($P),*> {
            $($(#[$eval_attrs])*)?
            type Eval = crate::Eval<$Val>;
        }
    };
}
pub(crate) use nat_expr;

/// Variadic [`Opaque`]
macro_rules! VarOpaque {
    ($($LazyBase:ident)::+<$($P:ident),* $(,)?>) => {
        crate::expr::VarOpaque!(
            @$($P)*,
            $($LazyBase)::+<$($P),*>
        )
    };
    (@$P:ident $($Ps:ident)*, $Out:ty) => {
        crate::expr::Opaque<$P, crate::expr::VarOpaque!(@$($Ps)*, $Out)>
    };
    (@, $Out:ty) => {
        $Out
    };
}
pub(crate) use VarOpaque;

/// Like [`nat_expr`], but wraps the result in [`VarOpaque`].
/// For this, another [`nat_expr`] type `$LazyBase` is declared in the
/// module to holds the implementation to be wrapped by [`VarOpaque`].
///
/// Recursive implementations should use that name when recursing,
/// not the opaque wrapper.
///
/// Additionally, when an additional `pub(...)` visibility is passed
/// to the attribute, the non-opaque base type is exported at that
/// visibility, for internal use elsewhere.
macro_rules! opaque {
    (
        ()
        $(#[$attr:meta])*
        $v:vis type $Name:ident<$($P:ident $(= $Def:ty)?),* $(,)?> $(: $OutBound:path)? = $LazyBase:ident;
    ) => {
        #[cfg(test)]
        #[allow(unused)] // Ensure that LazyBase is spanned for LSP
        const _: () = { use $LazyBase; };

        $(#[$attr])*
        $v struct $Name<$($P $(= $Def)?),*>($($P),*);
        impl<$($P: crate::NatExpr),*> crate::NatExpr for $Name<$($P),*> {
            type Eval = $crate::Eval<$crate::expr::VarOpaque!($LazyBase<$($P),*>)>;
        }
    };
}

macro_rules! test_op {
    (
        ($test_name:ident, $($args:tt)*)
        $(#[$attr:meta])*
        $v:vis $kw:ident $TypeName:ident<$($P:ident $(= $Def:ty)?),* $(,)?> $($rest:tt)*
    ) => {
        #[cfg(test)]
        crate::expr::testing::test_op! { $test_name: $($P)*, $TypeName<$($P),*>, $($args)* }

        $(#[$attr])*
        $v $kw $TypeName<$($P $(= $Def)?),*> $($rest)*
    };
}

macro_rules! op_examples {
    (
        $opname:ident
        $(, ($farg:literal $(, $arg:literal)* $(,)?) == $res:literal )* $(,)?
    ) => {
        core::concat!(
            "```\nuse gnat::expr;\n# macro_rules! assert_nat_eq { ($nat:ty, $val:expr) => { assert_eq!(gnat::to_u128::<$nat>(), Some($val)) } }\n",
            $(
                core::concat!(
                    "assert_nat_eq!(expr::",
                    core::stringify!($opname),
                    "<gnat::lit!(",
                    $farg,
                    $("), gnat::lit!(", $arg,)*
                    ")>, ",
                    $res,
                    ");\n",
                ),
            )*
            "```",
        )
    };
}

mod primitives;
pub use primitives::{If, LastBit, Opaque, PopBit, PushBit};

mod helper;
pub(crate) use helper::*;

mod trivial;
pub use trivial::{IsNonzero, IsZero};

mod testing;

mod bitmath;
pub use bitmath::{BitAnd, BitOr, BitXor};

mod log;
pub use log::{BaseLen, Log};

mod add;
pub use add::Add;
pub(crate) use add::*;

mod mul;
pub use mul::Mul;
pub(crate) use mul::*;

mod cmp;
pub(crate) use cmp::*;
pub use cmp::{Eq, Ge, Gt, Le, Lt, Max, Min, Ne};

mod sub;
pub(crate) use sub::*;
pub use sub::{AbsDiff, SatSub};

mod divrem;
pub(crate) use divrem::*;
pub use divrem::{Div, Rem};

mod shift;
pub(crate) use shift::*;
pub use shift::{Shl, Shr};

mod pow;
pub use pow::Pow;
