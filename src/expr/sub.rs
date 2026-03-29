use super::*;

/// ```text
/// SubUnchecked(L, R, C) := L - R - C, where C <= 1, L <= R + C
///
/// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
/// ```
///
/// This is a variation of binary addition.
/// ```text
///   L - R - C
/// = 2 * HL + PL - (2 * HR + PR) - C
/// = 2 * (HL - HR) + PL - PR - C
///
/// X := PL - PR - C, so -2 <= X <= 1.
/// ```
///
/// Using euclidian or floor divmod, which are identical for positive divisors,
/// - `0 <= X % 2 <= 1` because euclidian mod is nonnegative
///   - `X % 2 = (PL - PR - C) % 2 = (PL + PR + C) % 2 = Xor(PL, PR, C)`
/// - `CC := -(X / 2)` has `0 <= CC <= 1`
///   - Thus, `CC = 1  iff  CC > 0  iff  X / 2 < 0  iff  X < 0  iff  PL < PR + C`
///   - For `PL = 1`, `CC = 1  iff  1 < PR + C  iff  PR = 1 and C = 1  iff  And(PR, C) = 1`
///   - For `PL = 0`, `CC = 1  iff  0 < PR + C  iff  PR = 1  or C = 1  iff   Or(PR, C) = 1`
///   - Hence `CC = if PL { And(PR, C) } else { Or(PR, C) }`
///
/// Then `X = 2 * (X / 2) + X % 2 = - 2 * CC + X % 2` gives:
/// ```text
///   L - R - C
/// = 2 * (HL - HR) - 2 * CC + X % 2
/// = 2 * (HL - HR - CC) + X % 2
/// = Append(SubUnchecked(HL, HR, CC), X % 2)
/// ```
#[apply(base_case! 0 == R => If<C, _DecUnchecked<L>, L>)] // L - 0 - C = L - C = if C { L - 1 } else { L }
#[apply(lazy)]
pub type _SubUnchecked<L, R, C = U0> = PushBit<
    _SubUnchecked<
        _H<L>,
        _H<R>,
        // Normalize recursive argument
        uint::From<
            If<
                _P<L>, //
                _And<_P<R>, C>,
                _Or<_P<R>, C>,
            >,
        >,
    >,
    _Xor3<_P<L>, _P<R>, C>,
>;

/// `AbsDiff(L, R) := |L - R| = if L < R { R - L } else { L - R }`
#[apply(lazy)]
pub type _AbsDiff<L, R> = If<
    _Lt<L, R>, //
    _SubUnchecked<R, L>,
    _SubUnchecked<L, R>,
>;

/// Type-level [`abs_diff`](u128::abs_diff)
#[apply(test_op! test_abs_diff, L.abs_diff(R))]
#[apply(opaque)]
pub type AbsDiff<L, R> = _AbsDiff;

/// `SatSub(L, R) := if L < R { 0 } else { L - R }`
#[apply(lazy)]
pub type _SatSub<L, R> = If<
    _Lt<R, L>, //
    _SubUnchecked<L, R>,
    U0,
>;

/// Type-level [`saturating_sub`](u128::saturating_sub)
#[apply(opaque)]
#[apply(test_op! test_sat_sub, L.saturating_sub(R))]
pub type SatSub<L, R> = _SatSub;
