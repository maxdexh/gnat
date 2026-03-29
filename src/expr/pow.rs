use super::*;

/// Quad(N) := 4 * N
type _Quad<N> = PushBit<PushBit<N, U0>, U0>;

/// ```text
/// Square(N) := Pow(N, 2) = N * N
///
/// N = 2 * H + P, H = H(N), P = P(N)
///
/// If P = 1: Pow(N, 2) = Pow(2 * H + 1, 2) = 4 * Pow(H, 2) + 4 * H + 1
/// If P = 0: Pow(N, 2) = Pow(2 * H, 2) = 4 * Pow(H, 2)
/// ```
#[apply(base_case! 0 == N => U0)] // 0 * 0 = 0
#[apply(lazy)]
pub type _Square<N> = If<
    _P<N>,
    _CarryAdd<
        _Quad<_Square<_H<N>>>,
        _Quad<_H<N>>, //
        U1,
    >,
    _Quad<_Square<_H<N>>>,
>;

/// MulIf(N, F, C) := if C { N * F } else { N }
type _MulIf<N, F, C> = If<C, _Mul<F, N>, N>;

/// Fast Pow
///
/// ```text
/// H := H(E), P := P(E), E = 2 * H + P
///
///   Pow(B, E)
/// = Pow(B, 2 * H + P)
/// = Pow(Pow(B, H), 2) * Pow(B, P)
/// = Square(Pow(B, H)) * if P { B } else { 1 }
/// = if P { Square(Pow(B, H)) * B } else { Square(Pow(B, H)) }
/// = MulIf(Square(Pow(B, H)), B, P)
/// ```
#[apply(base_case! 0 == E => U1)] // Pow(B, 0) = 1 (including if B = 0)
#[apply(lazy)]
pub type _Pow<B, E> = _MulIf<
    _Square<_Pow<B, _H<E>>>, //
    B,
    _P<E>,
>;

/// Type-level [`pow`](usize::pow)
#[apply(opaque)]
#[apply(test_op!
    test_pow,
    B.pow(E.try_into().unwrap()),
    ..,
    // Cap the exponent at 10 for tests
    ..=10,
)]
pub type Pow<B, E> = _Pow;
