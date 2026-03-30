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
//! with [arrays](crate::array), use [`crate::Eval`] to evaluate them.
//!
//! # Primitive operations
//! Operations that are implemented through a dedicated associated type are called primitive.
//!
//! Currently, there are the following primitive operations.
//! They are implemented using associated types of [`Nat`] that are not public API.
//! - [`PopBit<N>`] removes the last bit of [`N::Eval`](NatExpr).
//!     - Evaluates like `N.eval() / 2`
//! - [`LastBit<N>`] gets the last bit of [`N::Eval`](NatExpr).
//!     - Evaluates like `N.eval() % 2`
//! - [`PushBit<N, B>`] pushes [`B::Eval`](NatExpr) as a bit to the end of [`N::Eval`](NatExpr)
//!     - Evaluates like `2 * N.eval() + (B.eval() != 0) as _`
//! - [`If<C, T, F>`] evaluates to [`T::Eval`](NatExpr) if `C` is nonzero, otherwise
//!   to [`F::Eval`](NatExpr). Only the necessary [`NatExpr::Eval`] projection is accessed.
//!     - Evaluates like `if C != 0 { T.eval() } else { F.eval() }`
//!
//! These primitives, together with [`NatExpr`] implementations based on them (and [`crate::Eval`]),
//! are sufficient for a [Turing-complete](https://en.wikipedia.org/wiki/Turing_completeness)
//! system, and all other operations in this module are just implemented on top of them. The way to do this is
//! described in the following sections.
//!
//! # Recursion
//! The way to implement an operation where the output requires looking at the entire number is to
//! do it recursively. However, regular type aliases do not support recursion, see error E0391
//! "cycle detected when expanding type alias".
//!
//! Instead, one has to go through [`NatExpr`] to make the operation "lazy", as in its value is only
//! computed when it is projected to [`NatExpr::Eval`]. For example, consider the following
//! implementation of [`BitAnd`]:
//! ```
//! use gnat::{NatExpr, small::*, expr::*};
//! // MyBitAnd is a struct implementing NatExpr, i.e. a lazy operation
//! pub struct MyBitAnd<L, R>(L, R);
//! impl<L: NatExpr, R: NatExpr> NatExpr for MyBitAnd<L, R> {
//!     // SUBOPTIMAL IMPLEMENTATION, SEE BELOW
//!     type Eval = gnat::Eval<If<
//!         L,
//!         // take the bitand of the previous bits and append the and of the last bit
//!         PushBit<
//!             MyBitAnd<PopBit<L>, PopBit<R>>,
//!             If<LastBit<L>, LastBit<R>, N0>, // boolean AND
//!         >,
//!         N0, // 0 & R = 0
//!     >>;
//! }
//! fn check_input<L: NatExpr, R: NatExpr>() {
//!     assert_eq!(
//!         gnat::to_u128::<MyBitAnd<L, R>>().unwrap(),
//!         gnat::to_u128::<L>().unwrap() & gnat::to_u128::<R>().unwrap(),
//!     )
//! }
//! check_input::<N3, N5>();
//! check_input::<N59, N122>();
//! check_input::<gnat::lit!(0b10101000110111111), gnat::lit!(0b11110111011111)>()
//! ```
//! Because `MyBitAnd` is [`NatExpr`] here and [`If`] works by only evaluating
//! [`NatExpr::Eval`] for the branch that is needed for the output, this will
//! properly exit when `L` becomes 0 and will not get stuck in an infinite loop.
//!
//! #### Evaluating recursive arguments
//! Because [`PopBit`] is itself lazy, the above definition of `MyBitAnd` will
//! result in the arguments to `MyBitAnd` accumulating `PopBit<PopBit<...>>`
//! for every recusive step. This can be fixed by applying [`crate::Eval`] to
//! the recursive arguments; e.g. in the example above, it is preferrable to
//! use `MyBitAnd<gnat::Eval<PopBit<L>>, gnat::Eval<PopBit<R>>>`.
//!
//! This is almost always beneficial for compile times.
//!
//! # Opaqueness
//! Note: This section is only relevant if the operation in question is public API or when
//! experiencing weird recursion limit errors from normalization of large inputs.
//!
//! The reason this is useful is that because types are heavily normalized
//! by the compiler, it is easy to accidentally leak implementation details about
//! them in a public API, which would make them impossible to normalize in the future,
//! as someone could rely on them behaving a certain way in generic contexts.
//! An example of this would be `LastBit<PushBit<N, B>> = B` where the arguments are generic.
//!
//! Furthermore, when using things like `crate::Eval<Min<UsizeMax, N>>` where `N` is generic,
//! the compiler might try to normalize the entire recursive `Min` operation, which may cause
//! spurious "overflow while ..." errors.
//!
//! These things can be guarded against using [`Opaque`]. `Opaque<P, Out>` always evaluates
//! to `Out`, but only after projecting through an internal associated type of `P`, like
//!`<P as Nat>::_Opaque<Out>`.
//!
//! This means that the compiler can only determine the value of [`Opaque<P, Out>`]
//! after it has determined the value of `P`, and it cannot do any normalization
//! specific to the implementation of `Out::Eval` before that.
//!
//! The way to use this when implementing a public operation `Op<A, B>` is as follows:
//! - The actual implementation is moved to a seperate lazy operation `OpImpl<A, B>`. Recursive
//!   evaluations use `OpImpl` rather than `Op`.
//! - `Op` should be a lazy operation that evaluates to `crate::Eval<Opaque<A, Opaque<B, OpImpl<A, B>>>>`
//!
//! # Complete example implementation of [`BitAnd`]
//! ```
//! use gnat::{NatExpr, small::*, expr::*};
//! pub struct _MyBitAnd<L, R>(L, R); // hide this in a private module
//! impl<L: NatExpr, R: NatExpr> NatExpr for _MyBitAnd<L, R> {
//!     type Eval = gnat::Eval<If<
//!         L,
//!         // take the bitand of the previous bits and append the and of the last bit
//!         PushBit<
//!             _MyBitAnd<
//!                 gnat::Eval<PopBit<L>>,
//!                 gnat::Eval<PopBit<R>>,
//!             >,
//!             If<LastBit<L>, LastBit<R>, N0>, // boolean AND
//!         >,
//!         N0, // 0 & R = 0
//!     >>;
//! }
//! pub type MyBitAnd<L, R> = Opaque<L, Opaque<R, _MyBitAnd<L, R>>>;
//! fn check_input<L: NatExpr, R: NatExpr>() {
//!     assert_eq!( // works fully generically!
//!         gnat::to_u128::<MyBitAnd<L, R>>().unwrap(),
//!         gnat::to_u128::<L>().unwrap() & gnat::to_u128::<R>().unwrap(),
//!     )
//! }
//! check_input::<N3, N5>();
//! check_input::<N59, N122>();
//! check_input::<gnat::lit!(0b10101000110111111), gnat::lit!(0b11110111011111)>()
//! ```

#[expect(unused_imports)] // for docs
use crate::{Nat, NatExpr};
use crate::{internals::InternalOp, small::*, utils::apply};

macro_rules! lazy_impl {
    (
        $(())?
        type $Name:ident<$($P:ident $(= $_:ty)?),* $(,)?> = $Val:ty;
    ) => {
        impl<$($P: crate::NatExpr),*> crate::NatExpr for $Name<$($P),*> {
            #[doc(hidden)]
            type Eval = crate::Eval<$Val>;
        }
    };
    (
        $(())?
        $(#[$attr:meta])*
        type $Name:ident<$($P:ident: $Bound:path $(= $_:ty)?),* $(,)?>: $OutBound:path = $Val:ty;
    ) => {
        $(#[$attr])*
        impl<$($P: $Bound),*> $OutBound for $Name<$($P),*> {
            #[doc(hidden)]
            type Eval = crate::Eval<$Val>;
        }
    };
}
pub(crate) use lazy_impl;

/// Input format:
/// ```compile_fail
/// #[apply(lazy)]
/// pub type A<P1, P2, ...> = $Val;
/// ```
///
/// Output format:
/// ```compile_fail
/// #[apply(lazy)]
/// pub struct A<P1, P2, ...>(P1, P2, ...);
/// impl<P1: NatExpr, P2: NatExpr, ...> NatExpr for A<P1, P2, ...> {
///     type Eval = gnat::Eval<$Val>;
/// }
/// ```
macro_rules! lazy {
    (
        $(())?
        $(#[$attr:meta])*
        pub type $Name:ident<$($P:ident $(= $Def:ty)?),* $(,)?> = $Val:ty;
    ) => {
        $(#[$attr])*
        pub struct $Name<$($P $(= $Def)?),*>($($P),*);
        crate::expr::lazy_impl! {
            type $Name<$($P),*> = $Val;
        }
    };
}
pub(crate) use lazy;

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

/// Like [`lazy`], but wraps the result in [`VarOpaque`].
/// For this, another [`lazy`] type `$LazyBase` is declared in the
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
        pub type $Name:ident<$($P:ident $(: $Bound:path)? $(= $Def:ty)?),* $(,)?> $(: $OutBound:path)? = $LazyBase:ident;
    ) => {
        #[cfg(test)]
        #[allow(unused)] // Ensure that LazyBase is spanned for LSP
        const _: () = { use $LazyBase; };

        $crate::expr::lazy! {
            $(#[$attr])*
            pub type $Name<$($P $(: $Bound)? $(= $Def)?),*> $(: $OutBound)? = $crate::Eval<$crate::expr::VarOpaque!($LazyBase<$($P),*>)>;
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
            "```\nuse gnat::{expr, small::*};\n# macro_rules! assert_nat_eq { ($nat:ty, $val:expr) => { assert_eq!(gnat::to_u128::<$nat>(), Some($val)) } }\n",
            $(
                core::concat!(
                    "assert_nat_eq!(expr::",
                    core::stringify!($opname),
                    "<N",
                    $farg,
                    $(", N", $arg,)*
                    ">, ",
                    $res,
                    ");\n",
                ),
            )*
            "```",
        )
    };
}

macro_rules! base_case {
    (
        (0 == $CheckZero:ty => $IfZero:ty)
        $(#[$attr:meta])*
        $v:vis type $Name:ident<$($P:ident $(: $Bound:path)? $(= $Def:ty)?),* $(,)?> $(: $OutBound:path)? = $Val:ty;
    ) => {
        $(#[$attr])*
        $v type $Name<$($P $(: $Bound)? $(= $Def)?),*> $(: $OutBound)? = crate::expr::If<
            $CheckZero,
            $Val,
            $IfZero,
        >;
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
