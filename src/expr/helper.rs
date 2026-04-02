use super::*;

pub type _And<L, R> = If<L, R, crate::lit!(0)>;
pub type _Or<L, R> = If<L, crate::lit!(1), R>;
pub type _Xor<L, R> = crate::Eval<If<L, IsZero<R>, R>>;
pub type _Xnor<L, R> = crate::Eval<If<L, R, IsZero<R>>>;
pub type _Xor3<A, B, C> = crate::Eval<If<A, _Xnor<B, C>, _Xor<B, C>>>;

/// Eager version of `PopBit`.
pub type _H<N> = crate::Eval<PopBit<N>>;
/// Eager version of `LastBit`.
pub type _P<N> = crate::Eval<LastBit<N>>;

#[apply(nat_expr)]
// H := H(N), P := P(N), N > 0.
// Result is unspecified for malformed input.
//
// DecUnchecked(N) := N - 1
//
// N - 1 = 2 * H + P - 1
//       = if P { 2 * H + 1 - 1 } else { 2 * H + 0 - 1 }
//       = if P { 2 * H + 0 } else { 2 * (H - 1) + 1 }
//
// because H - 1 < 0 iff H = 0, which with N > 0 gives N = P = 1, that branch is never
// taken and we can assume H - 1 = DecUnchecked(H) >= 0, so PushBit is valid for this.
pub type _DecUnchecked<N: NatExpr> = If<
    //
    _P<N>,
    PushBit<_H<N>, crate::lit!(0)>,
    PushBit<_DecUnchecked<_H<N>>, crate::lit!(1)>,
>;
