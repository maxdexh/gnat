//! Various [`Nat`] constants
//!
//! Note that some types in this module require a high recursion limit.

#[allow(unused_imports)] // for docs
use crate::{Nat, NatExpr};
use crate::{expr, nat, small::*};

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
    type Eval = N1;
}
impl NatExpr for Bool<false> {
    type Eval = N0;
}

/// [`usize::BITS`] as a [`Nat`]
pub type PtrBits = nat::Eval<expr::Shl<Usize<{ size_of::<usize>() }>, nat::lit!(3)>>;

/// [`usize::MAX`] as a [`Nat`]
pub type UsizeMax = nat::Eval<expr::SatSub<expr::Shl<N1, PtrBits>, N1>>;

/// [`isize::MAX`] as a [`Nat`]
pub type IsizeMax = nat::Eval<expr::PopBit<UsizeMax>>;

#[test]
fn test_usize_max() {
    assert_eq!(nat::to_usize::<PtrBits>(), Some(usize::BITS as usize));
    assert_eq!(nat::to_usize::<UsizeMax>(), Some(usize::MAX));
    assert_eq!(nat::to_usize::<IsizeMax>(), Some(isize::MAX as usize));
}

macro_rules! gen_maxes {
    [
        $([$name:ident, $bits:ty, $prim:ty $(,)? ],)*
    ] => {
        $(
            #[doc = concat!("[`", stringify!($prim), "::MAX`] as a [`Nat`]")]
            pub type $name = nat::Eval<
                crate::expr::_DecUnchecked<
                    crate::expr::Shl<N1, $bits>
                >
            >;
        )*
        #[test]
        fn test_generated_maxes() {
            $(assert_eq!(
                nat::to_u128::<$name>(),
                Some(<$prim>::MAX as u128),
            );)*
        }
    };
}
gen_maxes![
    [I8Max, nat::lit!(7), i8],
    [U8Max, nat::lit!(8), u8],
    [I16Max, nat::lit!(15), i16],
    [U16Max, nat::lit!(16), u16],
    [I32Max, nat::lit!(31), i32],
    [U32Max, nat::lit!(32), u32],
    [I64Max, nat::lit!(63), i64],
    [U64Max, nat::lit!(64), u64],
    [I128Max, nat::lit!(127), i128],
    [U128Max, nat::lit!(128), u128],
];
