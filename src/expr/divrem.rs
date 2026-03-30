use super::*;

/// `SubIfGe(L, R) := if L >= R { L - R } else { L }`
type _SubIfGe<L, R> = If<
    _Lt<L, R>, // Ge is implemented on top of Lt, use Lt directly
    L,
    _SubUnchecked<L, R>,
>;

/// ```text
/// NaiveRem(L, R) := 2 * (H % R) + P, where R > 0
///
/// H := H(L), P := P(L)
/// ```
///
/// # Motivation
/// ```text
/// L % R = (2 * H + P) % R
///       = (H + H + P) % R
///       = ((H % R) + (H % R) + P) % R
///       = (2 * (H % R) + P) % R
///       = NaiveRem(L, R) % R
/// ```
#[apply(base_case! 0 == L => N0)] // NaiveRem(0, R) = 2 * (0 % R) + 0 = 0
#[apply(lazy)]
pub type _NaiveRem<L, R> = PushBit<
    _RemUnchecked<_H<L>, R>, //
    _P<L>,
>;

/// ```text
/// RemUnchecked(L, R) := L % R, where R > 0
///
/// H := H(L), P := P(L), R > 0
///
/// - H % R <= R - 1
/// - NaiveRem(L, R) = 2 * (H % R) + P <= 2 * (H % R) + 1 <= 2 * R - 1
/// => RemUnchecked(L, R) = L % R = NaiveRem(L, R) % R = SubIfGe(NaiveRem(L, R), R)
/// ```
pub(crate) type _RemUnchecked<L, R> = _SubIfGe<_NaiveRem<L, R>, R>;

/// DivUnchecked(L, R) := L / R, where R > 0
///
/// H := H(L), P := P(L)
///
/// Note that `H = (H / R) * R + H % R`, and
/// For any `X`, `Y`: `(X * R + Y) / R = X + Y / R`
///
/// ```text
/// L / R = (2 * H + P) / R
///       = (2 * ((H / R) * R + H % R) + P) / R
///       = (2 * (H / R) * R + 2 * (H % R) + P) / R
///       = (2 * (H / R) * R + 2 * (H % R) + P) / R
///       = 2 * (H / R) + (2 * (H % R) + P) / R
///       = 2 * (H / R) + NaiveRem(L, R) / R
/// ```
///
/// Since we still have NaiveRem(L, R) <= 2 * R - 1 (See [`_RemUnchecked`]),
/// `NaiveRem(L, R) / R = if NaiveRem(L, R) >= R { 1 } else { 0 }`
#[apply(base_case! 0 == L => N0)] // 0 / R = 0
#[apply(lazy)]
pub type _DivUnchecked<L, R> = PushBit<
    _DivUnchecked<_H<L>, R>,
    IsZero<_Lt<_NaiveRem<L, R>, R>>, //
>;

#[apply(lazy)]
pub type _Rem<L, R> = If<
    R,
    _RemUnchecked<L, R>,
    // Return the dividend for division by zero
    L,
>;

/// Type-level [`%`](std::ops::Rem).
///
/// The remainder `Rem(L, R)` is always given by `L - (L / R) * R` (but calculated more efficiently).
///
/// # Examples
#[doc = op_examples!(
    Rem,
    (5, 2) == 1,
    (11, 4) == 3,
)]
/// Since division by zero always yields zero, the remainder is equal to the dividend.
#[doc = op_examples!(
    Rem,
    (0, 0) == 0,
    (7, 0) == 7,
)]
#[doc(alias = "%")]
#[doc(alias = "modulo")]
#[apply(opaque)]
#[apply(test_op!
    test_rem,
    L % R,
    ..,
    1..
)]
pub type Rem<L, R> = _Rem;

#[apply(lazy)]
pub type _Div<L, R> = If<
    R,
    _DivUnchecked<L, R>,
    // Division by zero is defined as zero
    N0,
>;

/// Type-level [`/`](std::ops::Div)
///
///
/// # Examples
#[doc = op_examples!(
    Div,
    (5, 2) == 2,
    (11, 4) == 2,
)]
/// Division by zero is defined to yield zero.
#[doc = op_examples!(
    Div,
    (0, 0) == 0,
    (7, 0) == 0,
)]
#[doc(alias = "/")]
#[apply(opaque)]
#[apply(test_op!
    test_div,
    L / R,
    ..,
    1..,
)]
pub type Div<L, R> = _Div;
