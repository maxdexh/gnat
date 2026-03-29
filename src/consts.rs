//! Various [`Uint`] constants
//!
//! Note that some types in this module require a high recursion limit.

#[allow(unused_imports)] // for docs
use crate::{NatExpr, Uint};
use crate::{small::*, uint, uops};

/// Holds a const [`u128`]
///
/// Implements [`NatExpr`] for [`small`](crate::small) values.
pub struct ConstU128<const N: u128>;

/// Holds a const [`usize`]
///
/// Implements [`NatExpr`] for [`small`](crate::small) values.
pub struct ConstUsize<const N: usize>;

/// Holds a const [`bool`]
///
/// Implements [`NatExpr`], using seperate impls for `true` and `false`.
pub struct ConstBool<const B: bool>;
impl NatExpr for ConstBool<true> {
    type Eval = U1;
}
impl NatExpr for ConstBool<false> {
    type Eval = U0;
}

/// [`usize::BITS`] as a [`Uint`]
pub type PtrBits = uint::From<uops::Shl<ConstUsize<{ size_of::<usize>() }>, uint::lit!(3)>>;

/// [`usize::MAX`] as a [`Uint`]
pub type UsizeMax = uint::From<uops::SatSub<uops::Shl<U1, PtrBits>, U1>>;

/// [`isize::MAX`] as a [`Uint`]
pub type IsizeMax = uint::From<uops::PopBit<UsizeMax>>;

#[test]
fn test_usize_max() {
    assert_eq!(uint::to_usize::<PtrBits>(), Some(usize::BITS as usize));
    assert_eq!(uint::to_usize::<UsizeMax>(), Some(usize::MAX));
    assert_eq!(uint::to_usize::<IsizeMax>(), Some(isize::MAX as usize));
}

macro_rules! gen_maxes {
    [
        $([$name:ident, $bits:ty, $prim:ty $(,)? ],)*
    ] => {
        $(
            #[doc = concat!("[`", stringify!($prim), "::MAX`] as a [`Uint`]")]
            pub type $name = uint::From<
                crate::uops::_DecUnchecked<
                    crate::uops::Shl<U1, $bits>
                >
            >;
        )*
        #[test]
        fn test_generated_maxes() {
            $(assert_eq!(
                uint::to_u128::<$name>(),
                Some(<$prim>::MAX as u128),
            );)*
        }
    };
}
gen_maxes![
    [I8Max, uint::lit!(7), i8],
    [U8Max, uint::lit!(8), u8],
    [I16Max, uint::lit!(15), i16],
    [U16Max, uint::lit!(16), u16],
    [I32Max, uint::lit!(31), i32],
    [U32Max, uint::lit!(32), u32],
    [I64Max, uint::lit!(63), i64],
    [U64Max, uint::lit!(64), u64],
    [I128Max, uint::lit!(127), i128],
    [U128Max, uint::lit!(128), u128],
];
