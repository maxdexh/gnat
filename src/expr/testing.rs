#![cfg(test)]

use crate::{Nat, expr};

pub(crate) type SatDec<N> = crate::Eval<expr::If<N, expr::_DecUnchecked<N>, crate::lit!(0)>>;

/// The test runner for all operations uses [`SatDec`] to traverse a range of inputs.
/// This test is there to ensure that it behaves correctly.
#[test]
fn test_satdec() {
    fn doit<const N: u128, V: Nat>()
    where
        crate::consts::U128<N>: crate::NatExpr<Eval = V>,
    {
        assert_eq!(crate::to_u128::<SatDec<V>>(), Some(N.saturating_sub(1)),)
    }
    macro_rules! tests {
        ($($val:literal)*) => {$(
            doit::<$val, _>();
        )*};
    }
    tests! { 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 }
}

const TEST_COUNT: u128 = if cfg!(test) {
    match option_env!("GNAT_TEST_COUNT") {
        Some(val) => match u128::from_str_radix(val, 10) {
            Ok(count) => count,
            Err(_) => panic!("Invalid test count"),
        },
        None => 10,
    }
} else {
    0
};
pub(crate) type DefaultHi = crate::Eval<crate::consts::U128<TEST_COUNT>>;
pub(crate) type DefaultLo = crate::lit!(0);

/// A type-level linked list of `Nat`s
pub(crate) trait NatList: Sized {
    const EMPTY: bool;
    type First: Nat;
    type Tail: NatList;
    type Len: Nat;

    type ReduceTestsArgs<T: Tests<RangesLo = Self>>: Tests<RangesLo = ()>;
}

pub(crate) type ListLen<L> = <L as NatList>::Len;
pub(crate) type InputLen<T> = ListLen<<T as Tests>::RangesLo>;
pub(crate) trait Tests: Sized {
    // The n-dimensional input range
    type RangesLo: NatList;
    type RangesHi: NatList;
    fn run_tests_on<L: NatList<Len = InputLen<Self>>>();
}

// The empty list must be cyclical for `Tail` and `ReduceTests` when recursing over it,
// so that we don't need to monomorphize infinitely many functions.
impl NatList for () {
    const EMPTY: bool = true;
    type First = crate::lit!(0);
    type Tail = Self;
    type Len = crate::lit!(0);

    type ReduceTestsArgs<T: Tests<RangesLo = Self>> = T;
}
impl<N: Nat, L: NatList> NatList for (N, L) {
    const EMPTY: bool = false;
    type First = N;
    type Tail = L;
    type Len = crate::Eval<expr::_Inc<L::Len>>;

    type ReduceTestsArgs<T: Tests<RangesLo = Self>> =
        <L as NatList>::ReduceTestsArgs<FirstArgTestsTraverser<T>>;
}

/// Recursively apply `ReduceTest` until we have no parameters left.
pub(crate) fn run_tests<T: Tests>() {
    const fn get_dispatch<T: Tests>() -> fn() {
        <T::RangesLo as NatList>::ReduceTestsArgs::<T>::run_tests_on::<()>
    }
    let dispatch = const {
        if TEST_COUNT == 0 {
            None
        } else {
            Some(get_dispatch::<T>())
        }
    };
    if let Some(dispatch) = dispatch {
        dispatch()
    }
}
pub(crate) struct FirstArgTestsTraverser<T>(T);
impl<T, Lo, LoTail> Tests for FirstArgTestsTraverser<T>
where
    T: Tests<RangesLo = (Lo, LoTail)>,
    Lo: Nat,
    LoTail: NatList,
{
    type RangesLo = LoTail;
    type RangesHi = <T::RangesHi as NatList>::Tail;

    fn run_tests_on<L: NatList<Len = InputLen<Self>>>() {
        Self::good_traverse::<L, <T::RangesHi as NatList>::First>()
    }
}
impl<T, Len, Lo, LoTail> FirstArgTestsTraverser<T>
where
    T: Tests<RangesLo = (Lo, LoTail)>,
    Lo: Nat,
    Len: Nat,
    LoTail: NatList<Len = Len>,
{
    const fn next_good_traverse<L: NatList<Len = Len>, N: Nat>() -> fn() {
        Self::good_traverse::<L, SatDec<N>>
    }
    fn good_traverse<L: NatList<Len = Len>, N: Nat>() {
        let (test, next) = const {
            let cmp = crate::cmp::<N, <T::RangesLo as NatList>::First>();
            (
                match cmp.is_ge() {
                    true => Some(T::run_tests_on::<(N, L)>),
                    false => None,
                },
                match cmp.is_gt() {
                    true => Some(Self::next_good_traverse::<L, N>()),
                    false => None,
                },
            )
        };
        if let Some(next) = next {
            next()
        }
        if let Some(test) = test {
            test()
        }
    }
}

macro_rules! test_op {
    (
        $name:ident:
        $first:ident $($param:ident)*,
        $got:ty,
        $expect:expr
        $(, $( $range:tt )* )?
    ) => {
        crate::expr::testing::test_op! {
            @shift
            $name
            [$first $($param)*],
            // Shift the params left and add an extra param.
            [$($param)* __Extra],
            $got,
            $expect,
            [$(, $( $range )*)?]
        }
    };
    (
        @shift
        $name:ident
        [$first:ident $($param:ident)*],
        [$fshifted:ident $($shifted:ident)*],
        $got:ty,
        $expect:expr,
        [$($range:tt)*]
    ) => {
        #[test]
        fn $name() {
            struct Leaf;
            type LeafInputLen = crate::expr::testing::InputLen<Leaf>;
            impl crate::expr::testing::Tests for Leaf {
                type RangesLo = crate::expr::testing::test_op!(
                    @ranges lo
                    [ $first $($param)* ]
                    $($range)*
                );
                type RangesHi = crate::expr::testing::test_op!(
                    @ranges hi
                    [ $first $($param)* ]
                    $($range)*
                );
                fn run_tests_on<L: crate::expr::testing::NatList<Len = LeafInputLen>>() {
                    Flattener::<L>::doit()
                }
            }
            struct Flattener<L>(L);
            impl<
                // Name a list using each param. The tail of the list
                // is the parameter after it. For the last parameter,
                // the tail doesn't matter, so use an extra dummy param.
                $first: crate::expr::testing::NatList<
                    Tail = $fshifted
                >
                $(, $param: crate::expr::testing::NatList<
                    Tail = $shifted
                >)*
                , __Extra: crate::expr::testing::NatList
            > Flattener<$first> {
                fn doit() {
                    // By generating code that has an explicit name for each
                    // tail list, we can now directly name all items of the
                    // list. As a bonus, we can use the dummy param to check
                    // that the input list has the correct length.
                    const {
                        debug_assert!(__Extra::EMPTY);
                        debug_assert!(!$first::EMPTY);
                        $(debug_assert!(!$param::EMPTY);)*
                    }
                    doit::<$first::First $(, $param::First)*>()
                }
            }
            #[expect(non_snake_case)]
            fn doit<$first: crate::Nat $(, $param: crate::Nat)*>() {
                let $first = crate::to_u128::<$first>().unwrap();
                $(let $param = crate::to_u128::<$param>().unwrap();)*
                assert_eq!(
                    crate::to_u128::<$got>(),
                    Some($expect),
                    "params={:?}",
                    ($first, $($param),*)
                );
            }

            crate::expr::testing::run_tests::<Leaf>()
        }
    };
    (
        @ranges $what:ident
        []
        $(,)?
    ) => {
        ()
    };
    (
        @ranges $what:ident
        []
        $($rest:tt)+
    ) => {
        core::compile_error! { core::concat!("Leftover ranges: ", stringify!($($rest)+)) }
    };
    (
        @ranges $what:ident
        [ $_:ident $($rest:ident)* ]
    ) => {
        (
            crate::expr::testing::test_op!(@select $what crate::expr::testing::DefaultLo, crate::expr::testing::DefaultHi),
            crate::expr::testing::test_op!(@ranges $what [$($rest)*]),
        )
    };
    (
        @ranges $what:ident
        [ $_:ident $($rest:ident)* ]
        , ..
        $(, $($range_rest:tt)*)?
    ) => {
        (
            crate::expr::testing::test_op!(@select $what crate::expr::testing::DefaultLo, crate::expr::testing::DefaultHi),
            crate::expr::testing::test_op!(@ranges $what [$($rest)*] $(, $($range_rest)*)?),
        )
    };
    (
        @ranges $what:ident
        [ $_:ident $($rest:ident)* ]
        , $lo:tt..
        $(, $($range_rest:tt)*)?
    ) => {
        (
            crate::expr::testing::test_op!(@select $what crate::expr::testing::test_op!(@bound $lo), crate::expr::testing::DefaultHi),
            crate::expr::testing::test_op!(@ranges $what [$($rest)*] $(, $($range_rest)*)?),
        )
    };
    (
        @ranges $what:ident
        [ $_:ident $($rest:ident)* ]
        , ..=$hi:tt
        $(, $($range_rest:tt)*)?
    ) => {
        (
            crate::expr::testing::test_op!(@select $what crate::expr::testing::DefaultLo, crate::expr::testing::test_op!(@bound $hi)),
            crate::expr::testing::test_op!(@ranges $what [$($rest)*] $(, $($range_rest)*)?),
        )
    };
    (
        @ranges $what:ident
        [ $_:ident $($rest:ident)* ]
        , $lo:tt..=$hi:tt
        $(, $($range_rest:tt)*)?
    ) => {
        (
            crate::expr::testing::test_op!(@select $what crate::expr::testing::test_op!(@bound $lo), crate::expr::testing::test_op!(@bound $hi)),
            crate::expr::testing::test_op!(@ranges $what [$($rest)*] $(, $($range_rest)*)?),
        )
    };
    (@bound $n:ty) => { $n };
    (@bound $n:expr) => { crate::Eval<crate::consts::U128<{$n}>> };
    (@select lo $lo:ty, $_:ty $(,)?) => { $lo };
    (@select hi $_:ty, $hi:ty $(,)?) => { $hi };
}
pub(crate) use test_op;
