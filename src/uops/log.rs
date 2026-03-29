use super::*;

#[apply(lazy)]
pub type _ILogUncheckedNormRec<B, N> = _ILogUnchecked<
    B,
    // Normalize recursive argument
    uint::From<_DivUnchecked<N, B>>,
>;
#[apply(lazy)]
pub type _ILogUnchecked<B, N> = If<
    //
    _Lt<N, B>,
    U0,
    _Inc<_ILogUncheckedNormRec<B, N>>,
>;
#[apply(lazy)]
pub type _ILog<B, N> = If<
    // Check B > 1 and N > 0
    _And<_H<B>, N>,
    _ILogUnchecked<B, N>,
    // Recurse infinitely
    _ILog<B, N>,
>;

/// Type-level [`ilog`](usize::ilog) (fallible)
///
/// # Errors
/// Using `B <= 1` or `N == 0` gives an "overflow while evaluating" error.
/// ```compile_fail,E0275
/// use gnat::{uops::ILog, uint, small::*};
/// const _: fn(uint::From<ILog<U1, U0>>) = |_| {};
/// ```
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
    _H<B>, // H<B> = 0 iff B <= 1
    If<
        N,
        // If B > 1 and N > 0, length in base B is just ILog + 1
        _Inc<_ILogUnchecked<B, N>>,
        // The length of 0 is 1
        U1,
    >,
    // Recurse infinitely
    _BaseLen<B, N>,
>;

/// Calculates `to_string().len()` in base `B` (fallible).
///
/// # Errors
/// Using `B <= 1` gives an "overflow while evaluating" error.
/// ```compile_fail,E0275
/// use gnat::{uops::BaseLen, uint, small::*};
/// const _: fn(uint::From<BaseLen<U1, U0>>) = |_| {};
/// ```
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
