use super::*;

// AddIf(C, L, R) := if C { L + R } else { R }
//                 = L + if C { R } else { 0 }
type _AddIf<C, L, R> = If<C, add::_CarryAdd<L, R>, L>;

// Double(N) := 2 * N
type _Double<N> = PushBit<N, U0>;

/// ```text
/// Mul(L, R) := L * R
///
/// H := H(L), P := P(L)
///
/// L * R = (2 * H + P) * R
///       = 2 * (H * R) + P * R
///       = 2 * (H * R) + if P { R } else { 0 }
///       = AddIf(P, Double(H * R), R)
/// ```
#[apply(base_case! 0 == L => U0)] // 0 * R = 0
#[apply(lazy)]
pub type _Mul<L, R> = _AddIf<
    _P<L>,
    //
    _Double<_Mul<_H<L>, R>>,
    R,
>;

/// Type-level [`*`](core::ops::Mul)
#[doc(alias = "*")]
#[apply(opaque)]
#[apply(test_op! test_mul, L * R)]
pub type Mul<L, R> = _Mul;
