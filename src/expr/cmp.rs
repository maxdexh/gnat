use super::*;

// Eq(L, R) := (L == R) = if L == R { 1 } else { 0 }
//
// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
//
//     L = R
// iff 2 * HL + PL = 2 * HR + PR
// iff PL = PR  and  HL = HR
// iff 1 = Xnor(PL, PR)  and  1 = Eq(HL, HR)
// iff 1 = if Xnor(PL, PR) == 1 { Eq(HL, HR) } else { 0 }
#[apply(nat_expr)]
pub type _Eq<L: NatExpr, R: NatExpr> = If<
    L,
    If<
        _Xnor<_P<L>, _P<R>>,
        _Eq<_H<R>, _H<L>>, // X == Y iff Y == X
        crate::lit!(0),
    >,
    // 0 == R
    IsZero<R>,
>;

/// Type-level [`==`](core::cmp::PartialEq)
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "==")]
#[apply(opaque)]
#[apply(test_op! test_eq, (L == R) as _)]
pub type Eq<L, R> = _Eq;

#[apply(nat_expr)]
pub type _Ne<L: NatExpr, R: NatExpr> = IsZero<Eq<L, R>>;

/// Type-level [`!=`](core::cmp::PartialEq)
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "!=")]
#[apply(opaque)]
pub type Ne<L, R> = _Ne;

/// LtByLast(L, R) := H(L) == H(R) and P(L) == 0 and P(R) == 1
type _LtByLast<L, R> = _And<
    If<_P<L>, crate::lit!(0), _P<R>>, //
    _Eq<_H<L>, _H<R>>,
>;

// Lt(L, R) := (L < R) = if L < R { 1 } else { 0 }
//
// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
//
//     L < R
// iff 2 * HL + PL < 2 * HR + PR
// iff HL < HR or HL = HR and PL = 0 and PR = 1
// iff Lt(HL, HR) = 1 or LtByLast(L, R) = 1
#[apply(nat_expr)]
pub type _Lt<L: NatExpr, R: NatExpr> = If<
    R,
    If<
        L,
        If<
            _Lt<_H<L>, _H<R>>, //
            crate::lit!(1),
            _LtByLast<L, R>,
        >,
        crate::lit!(1), // 0 < R is true since L != 0
    >,
    crate::lit!(0), // L < 0 is false
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

#[apply(nat_expr)]
pub type _Le<L: NatExpr, R: NatExpr> = IsZero<_Lt<R, L>>;

/// Type-level [`<=`](core::cmp::PartialOrd)
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "<=")]
#[apply(opaque)]
pub type Le<L, R> = _Le;

#[apply(nat_expr)]
pub type _Min<L: NatExpr, R: NatExpr> = If<_Lt<L, R>, R, L>;

/// Type-level [`min`](core::cmp::min)
#[apply(opaque)]
pub type Min<L, R> = _Min;

#[apply(nat_expr)]
pub type _Max<L: NatExpr, R: NatExpr> = If<_Lt<L, R>, L, R>;

/// Type-level [`max`](core::cmp::max)
#[apply(opaque)]
pub type Max<L, R> = _Max;
