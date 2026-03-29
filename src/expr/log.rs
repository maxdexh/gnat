use super::*;

#[apply(lazy)]
pub type _ILogUncheckedNormRec<B, N> = _ILogUnchecked<
    B,
    // Normalize recursive argument
    nat::Eval<_DivUnchecked<N, B>>,
>;
#[apply(lazy)]
pub type _ILogUnchecked<B, N> = If<
    //
    _Lt<N, B>,
    N0,
    _Inc<_ILogUncheckedNormRec<B, N>>,
>;
#[apply(lazy)]
pub type _ILog<B, N> = If<
    // Check B > 1 and N > 0
    _And<_H<B>, N>,
    _ILogUnchecked<B, N>,
    // Fallback value
    N0,
>;

/// Type-level [`ilog`](u128::ilog)
///
/// The base is taken as the first argument.
/// Returns 0 for inputs where the logarithm is not defined.
#[apply(opaque)]
#[apply(test_op!
    test_ilog,
    N.ilog(B).into(),
    2..,
    1..,
)]
pub type ILog<B, N> = _ILog;

#[apply(lazy)]
pub type _BaseLen<B, N> = If<
    // Half of B is zero iff B <= 1
    _H<B>,
    If<
        N,
        // If B > 1 and N > 0, length in base B is just ILog + 1
        _Inc<_ILogUnchecked<B, N>>,
        // The length of 0 is 1
        N1,
    >,
    // If B = 1 then return unary length, if B = 0 then return fallback
    If<B, N, N0>,
>;

/// Calculates the length of the number in an arbitrary base.
///
/// The base is taken as the first argument.
/// Returns unary length for base 1 or 0 for base 0.
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
pub type BaseLen<B, N> = _BaseLen;
