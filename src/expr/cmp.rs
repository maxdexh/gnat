use super::*;

// Cmp(L, R) := if L < R { 0 } else if L > R { 1 } else { 2 }
//
// HL := H(L), PL := P(L), HR := H(R), PR := P(R)
// CmpHalf := Cmp(HL, HR)
//
// Cmp(L, R) = 0
// iff L < R
// iff 2 * HL + PL < 2 * HR + PR
// iff HL < HR or (HL = HR and PL < PR)
// iff Cmp(HL, HR) = 0 or (Cmp(HL, HR) = 2 and PL = 0 and PR = 1)
//
// Cmp(L, R) = 1
// iff L > R
// iff 2 * HL + PL > 2 * HR + PR
// iff Cmp(HL, HR) = 1 or (Cmp(HL, HR) = 2 and PL = 1 and PR = 0)
//
// Cmp(L, R) = 2
// iff L = R
// iff HL = HR and PL = PR
// iff Cmp(HL, HR) = 2 and PL = PR
//
// - If Cmp(HL, HR) != 2, then Cmp(L, R) = Cmp(HL, HR)
// - If Cmp(HL, HR)  = 2, then decide Cmp(L, R) based on PL, PR
#[apply(nat_expr)]
#[apply(test_op! test_cmp, {
    match L.cmp(&R) {
        core::cmp::Ordering::Less => 0,
        core::cmp::Ordering::Greater => 1,
        core::cmp::Ordering::Equal => 2,
    }
})]
pub type _Cmp<L: Nat, R: Nat> = If<
    L,
    // L != 0. If R = 0, then L > R, else recurse
    If<R, _CmpRecExpr<L, R>, crate::lit!(1)>,
    // L = 0. If R = 0, then L = R, else L < R
    If<R, crate::lit!(0), crate::lit!(2)>,
>;
// Cmp without base case
#[apply(nat_expr)]
pub type _CmpRecExpr<L: Nat, R: Nat> = _CmpInterp<
    crate::Eval<
        _Cmp<
            _H<L>, //
            _H<R>,
        >,
    >,
    _P<L>,
    _P<R>,
>;
// CmpInterp(Cmp(HL, HR), PL, PR) := Cmp(L, R)
#[apply(nat_expr)]
pub type _CmpInterp<CmpHalf: Nat, PL: Nat, PR: Nat> = If<
    // H(CmpHalf) != 0 iff Cmp(HL, HR) = 2
    _H<CmpHalf>,
    // Cmp(HL, HR) = 2
    If<
        PL,
        // PL = 1, thus
        // - If PR = 1, then PL = PR, then Cmp(L, R) = 2
        // - If PR = 0, then Cmp(L, R) = 1
        If<PR, crate::lit!(2), crate::lit!(1)>,
        // PL = 0, thus
        // - If PR = 1, then Cmp(L, R) = 0
        // - If PR = 0, then PL = PR, then Cmp(L, R) = 2
        If<PR, crate::lit!(0), crate::lit!(2)>,
    >,
    // Cmp(HL, HR) != 2, thus Cmp(L, R) = Cmp(HL, HR)
    CmpHalf,
>;

/// Type-level [`==`](core::cmp::PartialEq)
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "==")]
#[apply(opaque)]
#[apply(test_op! test_eq, (L == R) as _)]
pub type Eq<L, R> = _Eq;

#[apply(nat_expr)] // L = R iff Cmp(L, R) = 2; H(2)=1, H(0)=H(1)=0
pub type _Eq<L: NatExpr, R: NatExpr> = _H<_Cmp<L::Eval, R::Eval>>;

/// Type-level [`!=`](core::cmp::PartialEq)
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "!=")]
#[apply(opaque)]
pub type Ne<L, R> = _Ne;

#[apply(nat_expr)]
pub type _Ne<L: NatExpr, R: NatExpr> = IsZero<_Eq<L, R>>;

/// Type-level [`<`](core::cmp::PartialOrd)
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "<")]
#[apply(opaque)]
#[apply(test_op! test_lt, (L < R) as _)]
pub type Lt<L, R> = _Lt;

#[apply(nat_expr)] // L < R iff Cmp(L, R) = 0
pub type _Lt<L: NatExpr, R: NatExpr> = IsZero<_Cmp<L::Eval, R::Eval>>;

/// Type-level [`>`](core::cmp::PartialOrd)
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = ">")]
pub type Gt<L, R> = Lt<R, L>;

/// Type-level [`<=`](core::cmp::PartialOrd)
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = "<=")]
#[apply(opaque)]
pub type Le<L, R> = _Le;
#[apply(nat_expr)]
pub type _Le<L: NatExpr, R: NatExpr> = IsZero<Gt<L, R>>;

/// Type-level [`>=`](core::cmp::PartialOrd)
///
/// The result of this operation is either `0` or `1`.
#[doc(alias = ">=")]
pub type Ge<L, R> = Le<R, L>;

/// Type-level [`min`](core::cmp::min)
#[apply(opaque)]
pub type Min<L, R> = _Min;

#[apply(nat_expr)]
pub type _Min<L: NatExpr, R: NatExpr> = If<_Lt<L, R>, R, L>;

#[apply(nat_expr)]
pub type _Max<L: NatExpr, R: NatExpr> = If<_Lt<L, R>, L, R>;

/// Type-level [`max`](core::cmp::max)
#[apply(opaque)]
pub type Max<L, R> = _Max;
