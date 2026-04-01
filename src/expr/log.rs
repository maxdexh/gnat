use super::*;

#[apply(lazy)]
pub type _LogUncheckedNormRec<B, N> = _LogUnchecked<
    B,
    // Normalize recursive argument
    crate::Eval<_DivUnchecked<N, B>>,
>;
#[apply(lazy)]
pub type _LogUnchecked<B, N> = If<
    //
    _Lt<N, B>,
    crate::lit!(0),
    _Inc<_LogUncheckedNormRec<B, N>>,
>;
#[apply(lazy)]
pub type _Log<B, N> = If<
    // Check B > 1 and N > 0
    _And<_H<B>, N>,
    _LogUnchecked<B, N>,
    // Fallback value
    crate::lit!(0),
>;

/// Type-level [`ilog`](u128::ilog)
///
/// The base is taken as the first argument.
///
/// # Examples
#[doc = op_examples!(
    Log,
    (5, 6) == 1,
    (5, 4) == 0,
)]
/// If the logarithm is undefined, the result is 0:
#[doc = op_examples!(
    Log,
    (0, 10) == 0,
    (1, 10) == 0,
    (5, 0) == 0,
)]
#[apply(opaque)]
#[apply(test_op!
    test_ilog,
    N.ilog(B).into(),
    2..,
    1..,
)]
pub type Log<B, N> = _Log;

#[apply(lazy)]
pub type _BaseLen<B, N> = If<
    // Half of B is zero iff B <= 1
    _H<B>,
    If<
        N,
        // If B > 1 and N > 0, length in base B is just ILog + 1
        _Inc<_LogUnchecked<B, N>>,
        // The length of 0 is 1
        crate::lit!(1),
    >,
    // If B = 1 then return unary length, if B = 0 then return fallback
    If<B, N, crate::lit!(0)>,
>;

/// Calculates the length of a number in an arbitrary base.
///
/// The base is taken as the first argument.
#[apply(opaque)]
#[apply(test_op! test_base_len, {
    let mut n = N;
    let mut r = 1;
    while n >= B {
        r += 1;
        n /= B;
    }
    r
}, 2..)]
/// # Examples
/// Calculating the length of `to_string`:
#[doc = op_examples!(
    BaseLen,
    (10, 0) == 1,
    (10, 10) == 2,
    (10, 99) == 2,
)]
/// Base 1 uses unary length:
#[doc = op_examples!(
    BaseLen,
    (1, 3) == 3,
    (1, 10) == 10,
)]
/// Base 0 always gives 0:
#[doc = op_examples!(
    BaseLen,
    (0, 0) == 0,
    (0, 5) == 0,
)]
pub type BaseLen<B, N> = _BaseLen;
