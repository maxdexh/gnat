//! Utilities related to [`NatExpr`] implementors.

use core::cmp::Ordering;

use crate::{NatExpr, Uint, expr, maxint::Umax, uint};

/// Alias for [`NatExpr::Eval`].
pub type From<N> = <N as NatExpr>::Eval;

/// Turns an integer literal into a [`Uint`].
///
/// If you have a small constant value that is not a literal, use [`uint::FromU128`].
///
/// # Examples
/// ```
/// #![recursion_limit = "1024"] // `lit!` doesn't recurse, the type is just long
///
/// use gnat::uint;
/// assert_eq!(uint::to_u128::<uint::lit!(1)>(), Some(1));
/// assert_eq!(
///     uint::to_u128::<uint::lit!(100000000000000000000000000000)>(),
///     Some(100000000000000000000000000000),
/// )
/// ```
#[macro_export]
#[doc(hidden)]
macro_rules! __lit {
    ($l:literal) => {
        $crate::__mac::proc::__lit! {
            ($l)
            ($crate::__mac::lit::_DirectAppend)
            ($crate::small::U0)
            ($crate::small::U1)
        }
    };
}
pub use __lit as lit;

const fn to_umax_overflowing<N: Uint>() -> (Umax, bool) {
    const {
        if is_nonzero::<N>() {
            let (h, o1) = to_umax_overflowing::<uint::From<expr::PopBit<N>>>();
            let (t, o2) = h.overflowing_mul(2);
            let (n, o3) = t.overflowing_add(is_nonzero::<expr::LastBit<N>>() as _);
            (n, o1 || o2 || o3)
        } else {
            (0, false)
        }
    }
}
const fn to_umax<N: Uint>() -> Option<Umax> {
    match to_umax_overflowing::<N>() {
        (n, false) => Some(n),
        (_, true) => None,
    }
}

/// Returns whether a [`Uint`] is nonzero.
pub const fn is_nonzero<N: NatExpr>() -> bool {
    crate::internals::InternalOp!(N::Eval, IS_NONZERO)
}
/// Returns whether a [`Uint`] is zero.
pub const fn is_zero<N: NatExpr>() -> bool {
    !is_nonzero::<N>()
}

/// Returns the decimal representation of a [`Uint`] for arbitrarily large `N`.
pub const fn to_str<N: NatExpr>() -> &'static str {
    const fn to_byte_str_naive<N: Uint>() -> &'static [u8] {
        struct ConcatBytes<N>(N);
        impl<N: Uint> type_const::Const for ConcatBytes<N> {
            type Type = &'static [&'static [u8]];
            const VALUE: Self::Type = &[
                // Recursively append the last digit
                doit::<
                    uint::From<
                        // Pop a digit
                        expr::Div<N, uint::lit!(10)>,
                    >,
                >(),
                &[b'0' + to_usize::<expr::Rem<N, uint::lit!(10)>>().unwrap() as u8],
            ];
        }
        const fn doit<N: Uint>() -> &'static [u8] {
            const {
                if is_nonzero::<N>() {
                    const_util::concat::concat_bytes::<ConcatBytes<N>>()
                } else {
                    b""
                }
            }
        }
        match doit::<N>() {
            b"" => b"0",
            val => val,
        }
    }

    // try to stringify the primitive representation if there is any
    const fn shortcut_umax<N: Uint>() -> &'static str {
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

/// Converts [`N::Eval`](NatExpr) to a `usize` with overflow and reutrns whether any wrapping
/// occurred.
pub const fn to_usize_overflowing<N: NatExpr>() -> (usize, bool) {
    let (n, o1) = to_umax_overflowing::<N::Eval>();
    (n as _, o1 || n > usize::MAX as Umax)
}

/// Converts [`N::Eval`](NatExpr) to a `usize` or returns `None` if it doesn't fit.
pub const fn to_usize<N: NatExpr>() -> Option<usize> {
    match to_usize_overflowing::<N>() {
        (n, false) => Some(n),
        (_, true) => None,
    }
}

/// Converts [`N::Eval`](NatExpr) to a `u128` with overflow and reutrns whether any wrapping
/// occurred.
pub const fn to_u128_overflowing<N: NatExpr>() -> (u128, bool) {
    let (n, o1) = to_umax_overflowing::<N::Eval>();
    (n as _, o1 || n > u128::MAX as Umax)
}

/// Converts [`N::Eval`](NatExpr) to a `u128` or returns `None` if it doesn't fit.
pub const fn to_u128<N: NatExpr>() -> Option<u128> {
    match to_u128_overflowing::<N>() {
        (n, false) => Some(n),
        (_, true) => None,
    }
}

/// Compares [`L::Eval`](NatExpr) and [`R::Eval`](NatExpr).
///
/// If this function returns [`Equal`](core::cmp::Ordering::Equal), it is guaranteed that
/// [`L::Eval`](NatExpr) and [`R::Eval`](NatExpr) are exactly the same type.
pub const fn cmp<L: NatExpr, R: NatExpr>() -> Ordering {
    const fn doit<L: Uint, R: Uint>() -> Ordering {
        const {
            if !is_nonzero::<L>() {
                match is_nonzero::<R>() {
                    true => Ordering::Less,
                    false => Ordering::Equal,
                }
            } else {
                match doit::<From<expr::PopBit<L>>, From<expr::PopBit<R>>>() {
                    it @ (Ordering::Less | Ordering::Greater) => it,
                    Ordering::Equal => {
                        match (
                            is_nonzero::<expr::LastBit<L>>(),
                            is_nonzero::<expr::LastBit<R>>(),
                        ) {
                            (true, true) | (false, false) => Ordering::Equal,
                            (true, false) => Ordering::Greater,
                            (false, true) => Ordering::Less,
                        }
                    }
                }
            }
        }
    }
    doit::<L::Eval, R::Eval>()
}

const fn cmp_umax<Lhs: Uint>(rhs: Umax) -> Ordering {
    if let Some(lhs) = to_umax::<Lhs>() {
        if lhs < rhs {
            Ordering::Less
        } else if lhs == rhs {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    } else {
        Ordering::Greater
    }
}

/// Compares a [`Uint`] (lhs) to a [`u128`] (rhs).
pub const fn cmp_u128<Lhs: NatExpr>(rhs: u128) -> Ordering {
    cmp_umax::<Lhs::Eval>(rhs as _)
}

/// Compares a [`Uint`] (lhs) to a [`usize`] (rhs).
pub const fn cmp_usize<Lhs: NatExpr>(rhs: usize) -> Ordering {
    cmp_umax::<Lhs::Eval>(rhs as _)
}
