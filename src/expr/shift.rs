use super::*;

/// DoubleIf(N, C) := if C { 2 * N } else { N }
type _DoubleIf<N, C> = If<C, PushBit<N, U0>, N>;

/// `Shl(L, R) := L << R := L * Pow(2, R)`
///
/// Let `H := H(R), P := P(R)`. Then `R = 2 * H + P = H + H + P`.
///
/// ```text
///   Shl(L, X + Y)
/// = L * Pow(2, X + Y)
/// = L * Pow(2, X) + Pow(2, Y)
/// = Shl(Shl(L, X), Y)
/// ```
///
/// ```text
/// B <= 1.
///
///   Shl(L, B)
/// = L * Pow(2, B)
/// = if B { L * Pow(2, 1) } else { L * Pow(2, 0) }
/// = if B { 2 * L } else { L }
/// = DoubleIf(L, B)
/// ```
///
/// ```text
///   Shl(L, R)
/// = Shl(L, H + H + P)
/// = Shl(Shl(Shl(L, H), H), P)
/// = DoubleIf(Shl(Shl(L, H), H), P)
/// ```
#[apply(base_case! 0 == R => L)] // L << 0 = L
#[apply(lazy)]
pub type _Shl<L, R> = _DoubleIf<
    // NOTE: From testing, this is the fastest known way to write this recursion
    // The inner Shl is normalized only on the next iteration by uint::From<L>
    _Shl<_Shl<uint::From<L>, _H<R>>, _H<R>>,
    _P<R>,
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

/// This implementation works the same as that of [`_Shl`], except whenever
/// we double there, we halve here
#[apply(base_case! 0 == _And<L, R> => L)] // L << 0 = L; 0 << R = 0 (= L if L = 0)
#[apply(lazy)]
pub type _Shr<L, R> = _HalfIf<
    // NOTE: See note on _Shl
    _Shr<_Shr<uint::From<L>, _H<R>>, _H<R>>,
    _P<R>,
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
