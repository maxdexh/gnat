use super::*;

/// Pops the last bit of a [`Nat`], thereby getting half its value.
///
/// This is a primitive operation.
///
/// Effectively a more efficient implementation of [`Div<N, _2>`] or [`Shr<N, _1>`].
/// This, together with [`LastBit`] and [`If`], can be used to implement operations that
/// recursively destructure a number and accumulate a result.
///
/// See the [module level documentation](crate::lazy) for details on how to combine
/// primitive operations.
#[apply(lazy)]
pub type PopBit<N> = InternalOp!(nat::Eval<N>, PopBit);

/// Gets the last bit of a [`Nat`], thereby getting its [parity](https://en.wikipedia.org/wiki/Parity_(mathematics)).
///
/// This is a primitive operation.
///
/// Effectively a more efficient implementation of [`Rem<N, _2>`] or [`BitAnd<N, _1>`].
/// This, together with [`PopBit`] and [`If`], can be used to implement operations that
/// recursively destructure a number and accumulate a result.
///
/// See the [module level documentation](crate::lazy) for details on how to combine
/// primitive operations.
#[apply(lazy)]
pub type LastBit<N> = InternalOp!(nat::Eval<N>, LastBit);

/// Pushes a single bit to the end of a [`Nat`].
///
/// This is a primitive operation.
///
/// Effectively a more efficient implementation of `Add<Mul<N, _2>, IsNonzero<P>>`,
/// or `BitOr<Shl<N, _1>, IsNonzero<P>>`. It is meant to be used for building the
/// output of an operation recursively bit-by-bit.
///
/// See the [module level documentation](crate::lazy) for details on how to combine
/// primitive operations.
#[apply(lazy)]
pub type PushBit<N, P> = InternalOp!(nat::Eval<P>, PushSelfAsBit<nat::Eval<N>>);

/// Conditionally evaluates to one of its arguments.
///
/// This is a primitive operation.
///
/// If `Cond` is truthy (nonzero), then `nat::Eval<If<Cond, Then, Else>>` is the same as
/// `nat::Eval<Then>`. Otherwise, it is the same as `nat::Eval<Else>`.
/// Only the resulting argument has its [`NatExpr::Eval`] implementation accessed,
/// i.e. the other branch is not evaluated and thus cannot lead to cycles. This allows
/// breaking out of recursively implemented operations.
///
/// See the [module level documentation](crate::lazy) for details on how to combine
/// primitive operations.
///
/// # Opaqueness
/// This operation is not opaque in `Then` and `Else`. If `Cond` is known, then
/// `nat::Eval<If<Cond, Then, Else>>` normalizes as specified above.
#[apply(lazy)]
pub type If<C, T, F> = InternalOp!(nat::Eval<C>, If<T, F>);

/// Makes a [`NatExpr`] opaque with respect to the value of a parameter.
///
/// This is a primitive operation.
///
/// `Opaque<P, Out>` evaluates to the same [`Nat`] as `Out`, but only after
/// going through a projection via an internal associated type on
/// [`P::Eval`](NatExpr).
///
/// See the [module level documentation](crate::lazy) for details on opaqueness.
#[apply(lazy)]
pub type Opaque<P, Out> = nat::Eval<InternalOp!(nat::Eval<P>, Opaque<Out>)>;

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
        check_proven_identical!(true, nat::Eval<If<N1, A, B>>, nat::Eval<A>);
        check_proven_identical!(true, nat::Eval<If<N0, A, B>>, nat::Eval<B>);
        check_proven_identical!(true, nat::Eval<Opaque<N0, A>>, nat::Eval<A>);

        // types that are not provably the same
        check_proven_identical!(false, nat::Eval<Opaque<B, A>>, nat::Eval<A>);
        check_proven_identical!(false, nat::Eval<Opaque<B, A>>, Opaque<A, A>);
        check_proven_identical!(false, nat::Eval<PopBit<PushBit<N0, A>>>, N0);
    }
    accept::<nat::lit!(3), nat::lit!(7)>();
}
