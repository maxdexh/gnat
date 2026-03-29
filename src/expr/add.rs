use super::*;

/// ```text
/// Inc(N) := N + 1
///
/// H := H(N), P := P(N).
///
/// N + 1 = 2 * H + P + 1
///       = if P { 2 * H + 2 } else { 2 * H + 1 }
///       = if P { 2 * (H + 1) } else { 2 * H + 1 }
///       = if P { PushBit(H + 1, 0) } else { PushBit(H, 1) }
/// ```
#[apply(lazy)]
pub type _Inc<N> = If<
    _P<N>, //
    PushBit<_Inc<_H<N>>, U0>,
    PushBit<_H<N>, U1>,
>;

pub(crate) type _PlusBit<N, C> = If<C, _Inc<N>, N>;

/// This is just binary addition.
///
/// ```text
/// Add(L, R, C) := L + R + C, where C <= 1.
///
/// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
///
///   L + R + C
/// = (2 * LH + LP) + (2 * RH + RP) + C
/// = 2 * (LH + RH) + (LP + RP + C)
///
/// X := LP + RP + C. Since X = 2 * (X / 2) + X % 2, we get
///   L + R + C
/// = 2 * (LH + RH + X / 2) + X % 2
/// = Append(LH + RH + X / 2, X % 2)
/// ```
#[apply(base_case! 0 == L => _PlusBit<R, C>)] // 0 + R + C = R + C
#[apply(lazy)]
pub type _CarryAdd<L, R, C = U0> = PushBit<
    // LH + RH + X / 2
    _CarryAdd<
        _H<R>, // swap args to converge faster
        _H<L>,
        // Normalize recursive argument
        uint::From<
            // Since X = LP + RP + C <= 3, we have X / 2 being either 0 or 1,
            // and therefore X / 2 = 1 iff LP + RP + C >= 2, else X / 2 = 0.
            If<
                _P<L>,
                // LP = 1, so LP + RP + C >= 2 iff RP + C >= 1 iff RP = 1 or  C = 1
                _Or<_P<R>, C>,
                // LP = 0, so LP + RP + C >= 2 iff RP + C >= 2 iff RP = 1 and C = 1
                _And<_P<R>, C>,
            >,
        >,
    >,
    // X % 2 is 1 iff X is odd, i.e. if an odd number of LP, RP, C are 1.
    // Hence X % 2 = Xor(LP, RP, C).
    _Xor3<_P<L>, _P<R>, C>,
>;

/// Type-level [`+`](Add)
#[apply(opaque)]
#[apply(test_op! test_add, L + R)]
pub type Add<L, R> = _CarryAdd;
