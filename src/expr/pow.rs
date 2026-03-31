use super::*;

// Quad(N) := 4 * N
type _Quad<N> = PushBit<PushBit<N, N0>, N0>;

// Square(N) := Pow(N, 2) = N * N
//
// N = 2 * H + P, H = H(N), P = P(N)
//
// If P = 1: Pow(N, 2) = Pow(2 * H + 1, 2) = 4 * Pow(H, 2) + 4 * H + 1
// If P = 0: Pow(N, 2) = Pow(2 * H, 2) = 4 * Pow(H, 2)
#[apply(lazy)]
pub type _Square<N> = If<
    N,
    If<
        _P<N>,
        _Add<
            _Quad<_Square<_H<N>>>,
            _Quad<_H<N>>,
            N1, // Use internal carry arg for +1
        >,
        _Quad<_Square<_H<N>>>,
    >,
    // 0 * 0 = 0
    N0,
>;

// MulIf(N, F, C) := if C { N * F } else { N }
type _MulIf<N, F, C> = If<C, _Mul<F, N>, N>;

// Fast Pow algorithm
//
// H := H(E), P := P(E), E = 2 * H + P
//
//   Pow(B, E)
// = Pow(B, 2 * H + P)
// = Pow(Pow(B, H), 2) * Pow(B, P)
// = Square(Pow(B, H)) * if P { B } else { 1 }
// = if P { Square(Pow(B, H)) * B } else { Square(Pow(B, H)) }
// = MulIf(Square(Pow(B, H)), B, P)
#[apply(lazy)]
pub type _Pow<B, E> = If<
    E,
    _MulIf<
        _Square<_Pow<B, _H<E>>>, //
        B,
        _P<E>,
    >,
    // Pow(B, 0) = 1 (including when B = 0)
    N1,
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
