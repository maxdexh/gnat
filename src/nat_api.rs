//! Utilities related to [`NatExpr`] implementors.

use core::cmp::Ordering;

use crate::{Nat, NatExpr, expr, maxint::Umax};

/// Alias for [`NatExpr::Eval`].
pub type Eval<N> = <N as NatExpr>::Eval;

const fn to_umax_overflowing<N: Nat>() -> (Umax, bool) {
    const {
        if is_zero::<N>() {
            (0, false)
        } else {
            let (h, o1) = to_umax_overflowing::<Eval<expr::PopBit<N>>>();
            let (t, o2) = h.overflowing_mul(2);
            let (n, o3) = t.overflowing_add(!is_zero::<expr::LastBit<N>>() as _);
            (n, o1 || o2 || o3)
        }
    }
}
const fn to_umax<N: Nat>() -> Option<Umax> {
    match to_umax_overflowing::<N>() {
        (n, false) => Some(n),
        (_, true) => None,
    }
}

/// Checks whether a [`Nat`] is zero.
pub const fn is_zero<N: NatExpr>() -> bool {
    crate::internals::InternalOp!(N::Eval, IS_ZERO)
}

/// Returns the decimal representation of a [`Nat`].
pub const fn to_str<N: NatExpr>() -> &'static str {
    const fn to_byte_str_naive<N: Nat>() -> &'static [u8] {
        struct ConcatBytes<N>(N);
        impl<N: Nat> type_const::Const for ConcatBytes<N> {
            type Type = &'static [&'static [u8]];
            const VALUE: Self::Type = &[
                // Recursively append the last digit
                doit::<
                    Eval<
                        // Pop a digit
                        expr::Div<N, crate::lit!(10)>,
                    >,
                >(),
                &[b'0' + to_usize::<expr::Rem<N, crate::lit!(10)>>().unwrap() as u8],
            ];
        }
        const fn doit<N: Nat>() -> &'static [u8] {
            const {
                if !is_zero::<N>() {
                    b""
                } else {
                    const_util::concat::concat_bytes::<ConcatBytes<N>>()
                }
            }
        }
        match doit::<N>() {
            b"" => b"0",
            val => val,
        }
    }

    // try to stringify the primitive representation if there is any
    const fn shortcut_umax<N: Nat>() -> &'static str {
        const {
            let fast_eval = const {
                const MAXLEN: usize = crate::maxint::umax_strlen(Umax::MAX);

                if let Some(n) = to_umax::<N>() {
                    let len = crate::maxint::umax_strlen(n);
                    let mut out = [0; MAXLEN];
                    crate::maxint::umax_write(n, &mut out);
                    Some((&{ out }, len))
                } else {
                    None
                }
            };
            let byte_str = match fast_eval {
                Some((out, len)) => out.split_at(len).0,
                None => to_byte_str_naive::<N>(),
            };
            match core::str::from_utf8(byte_str) {
                Ok(s) => s,
                Err(_) => unreachable!(),
            }
        }
    }

    shortcut_umax::<N::Eval>()
}

/// Converts a [`Nat`] to a `usize` with overflow and reutrns whether any wrapping
/// occurred.
pub const fn to_usize_overflowing<N: NatExpr>() -> (usize, bool) {
    let (n, o1) = to_umax_overflowing::<N::Eval>();
    (n as _, o1 || n > usize::MAX as Umax)
}

/// Converts a [`Nat`] to a `usize` or returns `None` if it doesn't fit.
pub const fn to_usize<N: NatExpr>() -> Option<usize> {
    match to_usize_overflowing::<N>() {
        (n, false) => Some(n),
        (_, true) => None,
    }
}

/// Converts a [`Nat`] to a `u128` with overflow and reutrns whether any wrapping
/// occurred.
pub const fn to_u128_overflowing<N: NatExpr>() -> (u128, bool) {
    let (n, o1) = to_umax_overflowing::<N::Eval>();
    (n as _, o1 || n > u128::MAX as Umax)
}

/// Converts a [`Nat`] to a `u128` or returns `None` if it doesn't fit.
pub const fn to_u128<N: NatExpr>() -> Option<u128> {
    match to_u128_overflowing::<N>() {
        (n, false) => Some(n),
        (_, true) => None,
    }
}

/// Compares two [`Nat`]s.
///
/// If this function returns [`Equal`](core::cmp::Ordering::Equal), it is guaranteed that
/// [`L::Eval`](NatExpr) and [`R::Eval`](NatExpr) are exactly the same type.
pub const fn cmp<L: NatExpr, R: NatExpr>() -> Ordering {
    const fn doit<L: Nat, R: Nat>() -> Ordering {
        const {
            if is_zero::<L>() {
                if is_zero::<R>() {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            } else {
                match doit::<Eval<expr::PopBit<L>>, Eval<expr::PopBit<R>>>() {
                    it @ (Ordering::Less | Ordering::Greater) => it,
                    Ordering::Equal => {
                        match (is_zero::<expr::LastBit<L>>(), is_zero::<expr::LastBit<R>>()) {
                            (false, false) | (true, true) => Ordering::Equal,
                            (false, true) => Ordering::Greater,
                            (true, false) => Ordering::Less,
                        }
                    }
                }
            }
        }
    }
    doit::<L::Eval, R::Eval>()
}
