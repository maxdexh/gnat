use super::*;

// DoubleIf(N, C) := if C { 2 * N } else { N }
type _DoubleIf<N, C> = If<C, PushBit<N, crate::lit!(0)>, N>;

// Shl(L, R) := L << R := L * Pow(2, R)
//
// Let H := H(R), P := P(R). Then R = 2 * H + P = H + H + P.
//
//   Shl(L, X + Y)
// = L * Pow(2, X + Y)
// = L * Pow(2, X) + Pow(2, Y)
// = Shl(Shl(L, X), Y)
//
// B <= 1.
//   Shl(L, B)
// = L * Pow(2, B)
// = if B { L * Pow(2, 1) } else { L * Pow(2, 0) }
// = if B { 2 * L } else { L }
// = DoubleIf(L, B)
//
//   Shl(L, R)
// = Shl(L, H + H + P)
// = Shl(Shl(Shl(L, H), H), P)
// = DoubleIf(Shl(Shl(L, H), H), P)
#[apply(nat_expr)]
pub type _Shl<L: NatExpr, R: NatExpr> = If<
    R,
    _DoubleIf<
        // NOTE: From testing, this is the fastest known way to write this recursion
        // The inner Shl is normalized only on the next iteration by gnat::Eval<L>
        _Shl<_Shl<crate::Eval<L>, _H<R>>, _H<R>>,
        _P<R>,
    >,
    // L << 0 = L
    L,
>;

/// Type-level [`<<`](core::ops::Shl)
#[doc(alias = "<<")]
#[apply(opaque)]
#[apply(test_op!
    test_shl,
    L << R,
    ..,
    ..=15,
)]
pub type Shl<L, R> = _Shl;

// HalfIf(N, C) := if C { H(N) } else { N }
type _HalfIf<N, C> = If<C, PopBit<N>, N>;

// This implementation works the same as that of `_Shl`, except whenever
// we double there, we halve here
#[apply(nat_expr)]
pub type _Shr<L: NatExpr, R: NatExpr> = If<
    _And<L, R>,
    _HalfIf<
        // NOTE: See note on _Shl
        _Shr<_Shr<crate::Eval<L>, _H<R>>, _H<R>>,
        _P<R>,
    >,
    // L << 0 = L; 0 << R = 0
    L,
>;

/// Type-level [`>>`](core::ops::Shr)
#[doc(alias = ">>")]
#[apply(opaque)]
#[apply(test_op!
    test_shr,
    L >> R,
    ..,
    ..=15,
)]
pub type Shr<L, R> = _Shr;
