// All bitwise operations are implemented the same way:
//
// Let Bitwise(Op, L, R) be the result of all Op(L(i), R(i)) appended to each other.
//
// Then we observe
// Bitwise(Op, L, R) = Bitwise(Op, Append(H(L), P(L)), Append(H(R), P(R)))
//                   = Append(Bitwise(Op, H(L), H(R)), Op(P(L), P(R))
//
// Because H and P split the number into the part before and after the last bit.

use super::*;

// BitAnd(L, R) := Bitwise(And, L, R), see module level
#[apply(lazy)]
pub type _BitAnd<L, R> = If<
    L,
    PushBit<
        _BitAnd<_H<R>, _H<L>>, // A & B = B & A, switching will terminate faster
        _And<_P<L>, _P<R>>,
    >,
    // 0 & R = 0
    N0,
>;

/// Type-level [`&`](https://en.wikipedia.org/wiki/Bitwise_operation#AND)
#[doc(alias = "&")]
#[apply(opaque)]
#[apply(test_op! test_bit_and, L & R)]
pub type BitAnd<L, R> = _BitAnd;

// BitOr(L, R) := Bitwise(Or, L, R), see module level
#[apply(lazy)]
pub type _BitOr<L, R> = If<
    L,
    PushBit<
        _BitOr<_H<R>, _H<L>>, // A | B = B | A
        _Or<_P<L>, _P<R>>,
    >,
    // 0 | R = R
    R,
>;

/// Type-level [`|`](https://en.wikipedia.org/wiki/Bitwise_operation#OR)
#[doc(alias = "|")]
#[apply(opaque)]
#[apply(test_op! test_bit_or, L | R)]
// BitOr(L, R) := Bitwise(Or, L, R), see above
pub type BitOr<L, R> = _BitOr;

// BitXor(L, R) = Bitwise(Xor, L, R), see module level
#[apply(lazy)]
pub type _BitXor<L, R> = If<
    L,
    PushBit<
        _BitXor<_H<R>, _H<L>>, // A ^ B = B ^ A
        _Xor<_P<L>, _P<R>>,
    >,
    // 0 ^ R = R
    R,
>;

/// Type-level [`^`](https://en.wikipedia.org/wiki/Bitwise_operation#XOR)
#[doc(alias = "^")]
#[apply(opaque)]
#[apply(test_op! test_bit_xor, L ^ R)]
pub type BitXor<L, R> = _BitXor;
