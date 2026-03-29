use crate::{NatExpr, Uint, array::Array, uimpl::*};

// NOTE: items from this module with names starting with _,
// except the above, are not meant to be used from anywhere
// but this module. This includes associated items.

pub trait _Cond {
    type _CondTy<T, F>;
}
impl<C: Uint> _Cond for C {
    type _CondTy<T, F> = InternalOp!(C, _CondTy<T, F>);
}
pub type CondTy<C, T, F> = <C as _Cond>::_CondTy<T, F>;

pub type _Internals<N> = <N as UintSealed>::__Uint;
macro_rules! InternalOp {
    ($N:ty, $($item:tt)*) => {
        <crate::internals::_Internals<$N> as crate::internals::_Uint>::$($item)*
    };
}
pub(crate) use InternalOp;

pub trait ArraySealed {}

// Map the internal API to the public one using an
// undocumented associated type.
pub trait UintSealed: 'static {
    /// Not public API
    #[doc(hidden)]
    type __Uint: _Uint;
}
pub trait _Uint: _UintArrs + 'static {
    const IS_NONZERO: bool;

    // This needs to evaluate directly to `T` or `F` because it is observable
    // for generic `T` and `F` (not that one could do anything else, since there
    // are no trait bounds)
    type _CondTy<T, F>;

    // These are exposed only through structs implementing NatExpr, so we can
    // do the NatExpr conversion on the result here directly. This has the
    // advantage of making errors more readable, since if this was `: NatExpr`,
    // then `uint::From<If<C, T, F>>` would normalize to
    // <<< C as UintSealed>::__Uint
    //       as _Uint>::IfImpl<T, F>
    //       as NatExpr>::Eval
    // Converting to `Uint` here removes the final `NatExpr` conversion.
    type If<T: NatExpr, F: NatExpr>: Uint;
    type Opaque<N: NatExpr>: Uint;

    // Opaque in all arguments, including `Self`.
    type PopBit: Uint;
    type LastBit: Uint;
    type PushSelfAsBit<N: Uint>: Uint;

    // PushBit<N, P> has to project through N and P to make the operation
    // opaque with respect to both, so simply implementing with a helper
    // `_ToBit: _Bit` doesn't work, because e.g.
    // `uint::From<PopBit<PushBit<N, _1>>>` would normalize to `w`.
    type _DirectAppend<B: _Bit>: _Uint;
}

pub trait _Pint: _Uint {}
pub trait _Bit: _Uint {}

#[diagnostic::do_not_recommend]
impl<N: _Uint> UintSealed for N {
    type __Uint = N;
}
#[diagnostic::do_not_recommend]
impl<N: _Uint> NatExpr for N {
    type Eval = N;
}
#[diagnostic::do_not_recommend]
impl<N: _Uint> Uint for N {}

// 0
impl _Bit for _0 {}
impl _Uint for _0 {
    const IS_NONZERO: bool = false;

    type _CondTy<T, F> = F;

    type If<T: NatExpr, F: NatExpr> = F::Eval;
    type Opaque<N: NatExpr> = N::Eval;

    type PopBit = _0;
    type LastBit = _0;

    type PushSelfAsBit<N: Uint> = InternalOp!(N, _DirectAppend<Self>);
    type _DirectAppend<B: _Bit> = B;
}

// 1
impl _Bit for _1 {}
impl _Pint for _1 {}
impl _Uint for _1 {
    const IS_NONZERO: bool = true;

    type _CondTy<T, F> = T;

    type If<T: NatExpr, F: NatExpr> = T::Eval;
    type Opaque<N: NatExpr> = N::Eval;

    type PopBit = _0;
    type LastBit = _1;

    type PushSelfAsBit<N: Uint> = InternalOp!(N, _DirectAppend<Self>);
    type _DirectAppend<B: _Bit> = _U<Self, B>;
}

// 2 * N + B where N > 0, B <= 1. Together with 0 and 1, this covers
// all non-negative integers.
impl<Pre: _Pint, Last: _Bit> _Pint for _U<Pre, Last> {}
impl<Pre: _Pint, Last: _Bit> _Uint for _U<Pre, Last> {
    const IS_NONZERO: bool = true;

    type _CondTy<T, F> = T;

    type If<T: NatExpr, F: NatExpr> = T::Eval;
    type Opaque<N: NatExpr> = N::Eval;

    type PopBit = Pre;
    type LastBit = Last;

    type PushSelfAsBit<N: Uint> = InternalOp!(N, _DirectAppend<_1>);
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
            pub struct $out_inner<T: $($bound)*, N: crate::Uint>($bound_name<T, N>);

            // SAFETY: repr(transparent), array was recursively constructed to be a valid implementor
            unsafe impl<T: $($bound)*, N: crate::Uint> Array for $out_inner<T, N> {
                type Item = T;
                type Length = N;
            }
            impl<T: $($bound)*, N: crate::Uint> ArraySealed for $out_inner<T, N> {}

            impl<T: $($bound)*, N: crate::Uint> Copy for $out_inner<T, N>
            where
                T: Copy,
                $bound_name<T, N>: Copy
            {
            }
            impl<T: $($bound)*, N: crate::Uint> Clone for $out_inner<T, N>
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
    _UintArrs,
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
impl _UintArrs for _0 {
    impl_body_zero!();
}
impl _UintArrs for _1 {
    impl_body_one!();
}
impl<Pre: _Pint, Pop: _Bit> _UintArrs for _U<Pre, Pop> {
    impl_body_bisect!(Pre, Pop);
}
