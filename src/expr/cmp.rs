use super::*;

/// ```text
/// Eq(L, R) := (L == R) = if L == R { 1 } else { 0 }
///
/// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
///
///     L = R
/// iff 2 * HL + PL = 2 * HR + PR
/// iff PL = PR  and  HL = HR
/// iff 1 = Xnor(PL, PR)  and  1 = Eq(HL, HR)
/// iff 1 = if Xnor(PL, PR) == 1 { Eq(HL, HR) } else { 0 }
/// ```
#[apply(base_case! 0 == L => IsZero<R>)] // L = R  iff  R == 0
#[apply(lazy)]
pub type _Eq<L, R> = If<
    _Xnor<_P<L>, _P<R>>,
    _Eq<_H<R>, _H<L>>, // X == Y iff Y == X
    U0,
>;

/// Type-level [`==`](core::cmp::PartialEq)
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "==")]
#[apply(opaque)]
#[apply(test_op! test_eq, (L == R) as _)]
pub type Eq<L, R> = _Eq;

#[apply(lazy)]
pub type _Ne<L, R> = IsZero<Eq<L, R>>;

/// Type-level [`!=`](core::cmp::PartialEq)
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "!=")]
#[apply(opaque)]
pub type Ne<L, R> = _Ne;

/// ```text
/// LtByLast(L, R) := (H(L) == H(R) and P(L) == 0 and P(R) == 1)
/// ```
type _LtByLast<L, R> = _And<
    If<_P<L>, U0, _P<R>>, //
    _Eq<_H<L>, _H<R>>,
>;

/// ```text
/// Lt(L, R) := (L < R) = if L < R { 1 } else { 0 }
///
/// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
///
///     L < R
/// iff 2 * HL + PL < 2 * HR + PR
/// iff HL < HR or HL = HR and PL = 0 and PR = 1
/// iff Lt(HL, HR) = 1 or LtByLast(L, R) = 1
/// ```
#[apply(lazy)]
pub type _Lt<L, R> = If<
    R,
    If<
        L,
        If<
            _Lt<_H<L>, _H<R>>, //
            U1,
            _LtByLast<L, R>,
        >,
        U1, // 0 < R is true since L != 0
    >,
    U0, // L < 0 is false
>;

/// Type-level [`<`](core::cmp::PartialOrd)
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "<")]
#[apply(opaque)]
#[apply(test_op! test_lt, (L < R) as _)]
pub type Lt<L, R> = _Lt;

/// Type-level [`>`](core::cmp::PartialOrd)
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = ">")]
pub type Gt<L, R> = Lt<R, L>;

/// Type-level [`>=`](core::cmp::PartialOrd)
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = ">=")]
pub type Ge<L, R> = Le<R, L>;

#[apply(lazy)]
pub type _Le<L, R> = IsZero<_Lt<R, L>>;

/// Type-level [`<=`](core::cmp::PartialOrd)
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "<=")]
#[apply(opaque)]
pub type Le<L, R> = _Le;

#[apply(lazy)]
pub type _Min<L, R> = If<_Lt<L, R>, R, L>;

/// Type-level [`min`](core::cmp::min)
#[apply(opaque)]
pub type Min<L, R> = _Min;

#[apply(lazy)]
pub type _Max<L, R> = If<_Lt<L, R>, L, R>;

/// Type-level [`max`](core::cmp::max)
#[apply(opaque)]
pub type Max<L, R> = _Max;
