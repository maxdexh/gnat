use super::*;

/// Pops the last bit of a [`Nat`], thereby getting half its value.
///
/// This is a primitive operation.
///
/// Effectively a more efficient implementation of [`Div<N, _2>`] or [`Shr<N, _1>`].
/// This, together with [`LastBit`] and [`If`], can be used to implement operations that
/// recursively destructure a number and accumulate a result.
///
/// See the [module level documentation](crate::expr) for details on how to combine
/// primitive operations.
#[apply(lazy)]
pub type PopBit<N> = InternalOp!(crate::Eval<N>, PopBit);

/// Gets the last bit of a [`Nat`], thereby getting its parity.
///
/// This is a primitive operation.
///
/// Effectively a more efficient implementation of [`Rem<N, _2>`] or [`BitAnd<N, _1>`].
/// This, together with [`PopBit`] and [`If`], can be used to implement operations that
/// recursively destructure a number and accumulate a result.
///
/// See the [module level documentation](crate::expr) for details on how to combine
/// primitive operations.
#[apply(lazy)]
pub type LastBit<N> = InternalOp!(crate::Eval<N>, LastBit);

/// Pushes a single bit to the end of a [`Nat`].
///
/// This is a primitive operation.
///
/// Effectively a more efficient implementation of `Add<Mul<N, _2>, IsNonzero<P>>`,
/// or `BitOr<Shl<N, _1>, IsNonzero<P>>`. It is meant to be used for building the
/// output of an operation recursively bit-by-bit.
///
/// See the [module level documentation](crate::expr) for details on how to combine
/// primitive operations.
#[apply(lazy)]
pub type PushBit<N, P> = InternalOp!(crate::Eval<P>, PushSelfAsBit<crate::Eval<N>>);

/// Ternary operation
///
/// This is a primitive operation.
///
/// If `Cond` is truthy (nonzero), then `crate::Eval<If<Cond, Then, Else>>` is the same as
/// `crate::Eval<Then>`. Otherwise, it is the same as `crate::Eval<Else>`.
/// Only the resulting argument has its [`NatExpr::Eval`] implementation accessed,
/// i.e. the other branch is not evaluated and thus cannot lead to cycles. This allows
/// breaking out of recursively implemented operations.
///
/// See the [module level documentation](crate::expr) for details on how to combine
/// primitive operations.
///
/// # Opaqueness
/// This operation is not opaque in `Then` and `Else`. If `Cond` is known, then
/// `crate::Eval<If<Cond, Then, Else>>` normalizes as specified above.
#[apply(lazy)]
pub type If<C, T, F> = InternalOp!(crate::Eval<C>, If<T, F>);

/// Makes a [`NatExpr`] opaque with respect to the value of a parameter.
///
/// This is a primitive operation.
///
/// `Opaque<P, Out>` evaluates to the same [`Nat`] as `Out`, but only after
/// going through a projection via an internal associated type on
/// [`P::Eval`](NatExpr).
///
/// See the [module level documentation](crate::expr) for details on opaqueness.
#[apply(lazy)]
pub type Opaque<P, Out> = crate::Eval<InternalOp!(crate::Eval<P>, Opaque<Out>)>;

#[test]
fn opaqueness_tests() {
    struct Namespace<L, R>(L, R);
    trait Fallback {
        const PROVEN_IDENTICAL: bool;
    }
    // `Wat` has a trait const
    impl<X> Namespace<X, X> {
        const PROVEN_IDENTICAL: bool = true;
    }
    impl<L, R> Fallback for Namespace<L, R> {
        const PROVEN_IDENTICAL: bool = false;
    }
    macro_rules! check_proven_identical {
        ($expect:expr, $lhs:ty, $rhs:ty) => {
            assert!($expect == Namespace::<$lhs, $rhs>::PROVEN_IDENTICAL)
        };
    }
    fn accept<A: NatExpr, B: NatExpr>() {
        // types that are provably the same
        check_proven_identical!(true, crate::Eval<If<N1, A, B>>, crate::Eval<A>);
        check_proven_identical!(true, crate::Eval<If<N0, A, B>>, crate::Eval<B>);
        check_proven_identical!(true, crate::Eval<Opaque<N0, A>>, crate::Eval<A>);

        // types that are not provably the same
        check_proven_identical!(false, crate::Eval<Opaque<B, A>>, crate::Eval<A>);
        check_proven_identical!(false, crate::Eval<Opaque<B, A>>, Opaque<A, A>);
        check_proven_identical!(false, crate::Eval<PopBit<PushBit<N0, A>>>, N0);
    }
    accept::<crate::lit!(3), crate::lit!(7)>();
}
