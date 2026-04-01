//! Various [`Nat`] constants
//!
//! Note that some types in this module require a high recursion limit.

use crate::expr;
#[allow(unused_imports)] // for docs
use crate::{Nat, NatExpr};

/// Holds a const [`u128`]
///
/// Implements [`NatExpr`] for [`small`](crate::small) values.
pub struct U128<const N: u128>;

/// Holds a const [`usize`]
///
/// Implements [`NatExpr`] for [`small`](crate::small) values.
pub struct Usize<const N: usize>;

/// Holds a const [`bool`]
///
/// Implements [`NatExpr`], using seperate impls for `true` and `false`.
pub struct Bool<const B: bool>;
impl NatExpr for Bool<true> {
    type Eval = crate::lit!(1);
}
impl NatExpr for Bool<false> {
    type Eval = crate::lit!(0);
}

/// [`usize::BITS`] as a [`Nat`]
pub type PtrBits = crate::Eval<expr::Shl<Usize<{ size_of::<usize>() }>, crate::lit!(3)>>;

/// [`usize::MAX`] as a [`Nat`]
pub type UsizeMax = crate::Eval<expr::SatSub<expr::Shl<crate::lit!(1), PtrBits>, crate::lit!(1)>>;

/// [`isize::MAX`] as a [`Nat`]
pub type IsizeMax = crate::Eval<expr::PopBit<UsizeMax>>;

#[test]
fn test_usize_max() {
    assert_eq!(crate::to_usize::<PtrBits>(), Some(usize::BITS as usize));
    assert_eq!(crate::to_usize::<UsizeMax>(), Some(usize::MAX));
    assert_eq!(crate::to_usize::<IsizeMax>(), Some(isize::MAX as usize));
}

macro_rules! gen_maxes {
    [
        $([$name:ident, $bits:ty, $prim:ty $(,)? ],)*
    ] => {
        $(
            #[doc = concat!("[`", stringify!($prim), "::MAX`] as a [`Nat`]")]
            pub type $name = crate::Eval<
                crate::expr::_DecUnchecked<
                    crate::expr::Shl<crate::lit!(1), $bits>
                >
            >;
        )*
        #[test]
        fn test_generated_maxes() {
            $(assert_eq!(
                crate::to_u128::<$name>(),
                Some(<$prim>::MAX as u128),
            );)*
        }
    };
}
gen_maxes![
    [I8Max, crate::lit!(7), i8],
    [U8Max, crate::lit!(8), u8],
    [I16Max, crate::lit!(15), i16],
    [U16Max, crate::lit!(16), u16],
    [I32Max, crate::lit!(31), i32],
    [U32Max, crate::lit!(32), u32],
    [I64Max, crate::lit!(63), i64],
    [U64Max, crate::lit!(64), u64],
    [I128Max, crate::lit!(127), i128],
    [U128Max, crate::lit!(128), u128],
];

macro_rules! new_alias {
    ($name:ident, $val:literal, $ty:ty) => {
        #[doc = core::concat!("`", $val, "` as a [`Nat`](crate::Nat)")]
        type $name = $ty;
        #[diagnostic::do_not_recommend]
        impl crate::NatExpr for crate::consts::U128<$val> {
            type Eval = $name;
        }
        #[diagnostic::do_not_recommend]
        impl crate::NatExpr for crate::consts::Usize<$val> {
            type Eval = $name;
        }
    };
}

new_alias!(N0, 0, crate::uimpl::_0);
new_alias!(N1, 1, crate::uimpl::_1);

macro_rules! bisect {
    ($name:ident, $val:expr, $half:ty, $parity:ty) => {
        new_alias! { $name, $val, crate::uimpl::_U<$half, $parity> }
    };
}
include!(concat!(env!("OUT_DIR"), "/consts.rs"));
