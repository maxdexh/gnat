use crate::{Nat, NatExpr, array::Array, uimpl::*};

// NOTE: items from this module with names starting with _
// are not meant to be used from anywhere but this module.

pub trait _Cond {
    type _CondTy<T, F>;
}
impl<C: Nat> _Cond for C {
    type _CondTy<T, F> = InternalOp!(C, _CondTy<T, F>);
}
pub type CondTy<C, T, F> = <C as _Cond>::_CondTy<T, F>;

pub type _Internals<N> = <N as NatSealed>::__Nat;
macro_rules! InternalOp {
    ($N:ty, $($item:tt)*) => {
        <crate::internals::_Internals<$N> as crate::internals::_Nat>::$($item)*
    };
}
pub(crate) use InternalOp;

pub trait ArraySealed {}

// Map the internal API to the public one using an
// undocumented associated type.
pub trait NatSealed: 'static {
    /// Not public API
    #[doc(hidden)]
    type __Nat: _Nat;
}
pub trait _Nat: _NatArrs + 'static {
    const IS_ZERO: bool;

    // This needs to evaluate directly to `T` or `F` because it is observable
    // for generic `T` and `F` (not that one could do anything else, since there
    // are no trait bounds)
    type _CondTy<T, F>;

    // These are exposed only through structs implementing NatExpr, so we can
    // do the NatExpr conversion on the result here directly. This has the
    // advantage of making errors more readable, since if this was `: NatExpr`,
    // then `crate::Eval<If<C, T, F>>` would normalize to
    // <<< C as NatSealed>::__Nat
    //       as _Nat>::IfImpl<T, F>
    //       as NatExpr>::Eval
    // Converting to `Nat` here removes the final `NatExpr` conversion.
    type If<T: NatExpr, F: NatExpr>: Nat;
    type Opaque<N: NatExpr>: Nat;

    // Opaque in all arguments, including `Self`.
    type PopBit: Nat;
    type LastBit: Nat;
    type PushSelfAsBit<N: Nat>: Nat;

    // PushBit<N, P> has to project through N and P to make the operation
    // opaque with respect to both, so simply implementing with a helper
    // `_ToBit: _Bit` doesn't work, because e.g.
    // `crate::Eval<PopBit<PushBit<N, _1>>>` would normalize to `w`.
    type _DirectAppend<B: _Bit>: _Nat;
}

pub trait _Pint: _Nat {}
pub trait _Bit: _Nat {}

#[diagnostic::do_not_recommend]
impl<N: _Nat> NatSealed for N {
    type __Nat = N;
}
#[diagnostic::do_not_recommend]
impl<N: _Nat> NatExpr for N {
    type Eval = N;
}
#[diagnostic::do_not_recommend]
impl<N: _Nat> Nat for N {}

// 0
impl _Bit for _0 {}
impl _Nat for _0 {
    const IS_ZERO: bool = true;

    type _CondTy<T, F> = F;

    type If<T: NatExpr, F: NatExpr> = F::Eval;
    type Opaque<N: NatExpr> = N::Eval;

    type PopBit = _0;
    type LastBit = _0;

    type PushSelfAsBit<N: Nat> = InternalOp!(N, _DirectAppend<Self>);
    type _DirectAppend<B: _Bit> = B;
}

// 1
impl _Bit for _1 {}
impl _Pint for _1 {}
impl _Nat for _1 {
    const IS_ZERO: bool = false;

    type _CondTy<T, F> = T;

    type If<T: NatExpr, F: NatExpr> = T::Eval;
    type Opaque<N: NatExpr> = N::Eval;

    type PopBit = _0;
    type LastBit = _1;

    type PushSelfAsBit<N: Nat> = InternalOp!(N, _DirectAppend<Self>);
    type _DirectAppend<B: _Bit> = _U<Self, B>;
}

// 2 * N + B where N > 0, B <= 1. Together with 0 and 1, this covers
// all non-negative integers.
impl<Pre: _Pint, Last: _Bit> _Pint for _U<Pre, Last> {}
impl<Pre: _Pint, Last: _Bit> _Nat for _U<Pre, Last> {
    const IS_ZERO: bool = false;

    type _CondTy<T, F> = T;

    type If<T: NatExpr, F: NatExpr> = T::Eval;
    type Opaque<N: NatExpr> = N::Eval;

    type PopBit = Pre;
    type LastBit = Last;

    type PushSelfAsBit<N: Nat> = InternalOp!(N, _DirectAppend<_1>);
    type _DirectAppend<B: _Bit> = _U<Self, B>;
}

#[derive(Clone, Copy)]
#[repr(C)]
// NOTE: repr(C) (H, H, P) is equivalent but slows down miri. https://github.com/fizyk20/generic-array/issues/157
pub struct ArrBisect<H, P> {
    halves: [H; 2],
    parity: P,
}

// Ideally, every useful combination of traits that cannot be implemented properly on `ArrApi`
// should have an array type. For now, the only such trait is `Copy`, until `Freeze`/`NoCell`
// might become stable.
macro_rules! gen_arr_internals {
    [
        $ArrsTrait:ident,
        [$(
            [
                $bound_name:ident,
                ($($bound:tt)*),

                $out_inner:ident,

                $doc:expr,
                $out:ident,
            ]
        ),* $(,)?],
        $wrap:ident,
    ] => {
        pub trait $ArrsTrait {$(
            type $bound_name<T: $($bound)*>: $($bound)*;
        )*}
        $(type $bound_name<T, N> = <_Internals<N> as crate::internals::$ArrsTrait>::$bound_name<T>;)*

        macro_rules! impl_body_zero { () => {$(
            type $bound_name<T: $($bound)*> = [T; 0];
        )*}}
        macro_rules! impl_body_one { () => {$(
            type $bound_name<T: $($bound)*> = [T; 1];
        )*}}
        macro_rules! impl_body_bisect { ($Pre:ident, $Pop:ident) => {$(
            type $bound_name<T: $($bound)*> = ArrBisect<Pre::$bound_name<T>, Pop::$bound_name<T>>;
        )*}}

        $(
            #[doc = core::concat!("The inner [`Array`] type of ", core::stringify!($out), ".")]
            #[cfg_attr(not(doc), repr(transparent))]
            pub struct $out_inner<T: $($bound)*, N: crate::Nat>($bound_name<T, N>);

            // SAFETY: repr(transparent), array was recursively constructed to be a valid implementor
            unsafe impl<T: $($bound)*, N: crate::Nat> Array for $out_inner<T, N> {
                type Item = T;
                type Length = N;
            }
            impl<T: $($bound)*, N: crate::Nat> ArraySealed for $out_inner<T, N> {}

            impl<T: $($bound)*, N: crate::Nat> Copy for $out_inner<T, N>
            where
                T: Copy,
                $bound_name<T, N>: Copy
            {
            }
            impl<T: $($bound)*, N: crate::Nat> Clone for $out_inner<T, N>
            where
                T: Copy,
                $bound_name<T, N>: Copy
            {
                fn clone(&self) -> Self {
                    *self
                }
            }

            #[doc = $doc]
            pub type $out<T, N> = $wrap<$out_inner<T, N>>;
        )*

        pub mod array_types { pub use super::{$($out_inner, $out),*}; }
    };
}
use crate::array::ArrApi;
gen_arr_internals![
    _NatArrs,
    [
        [
            _Arr,
            (Sized),
            ArrInner,
            crate::utils::docexpr! {
                /// General [`Array`] implementation.
                ///
                /// See the [module level documentation](crate::array).
            },
            Arr,
        ],
        [
            _CopyArr,
            (Copy),
            CopyArrInner,
            crate::utils::docexpr! {
                /// [`Array`] implementation that implements [`Copy`] but requires `T: Copy`.
                ///
                /// See the [module level documentation](crate::array).
            },
            CopyArr,
        ],
    ],
    ArrApi,
];
impl _NatArrs for _0 {
    impl_body_zero!();
}
impl _NatArrs for _1 {
    impl_body_one!();
}
impl<Pre: _Pint, Pop: _Bit> _NatArrs for _U<Pre, Pop> {
    impl_body_bisect!(Pre, Pop);
}
