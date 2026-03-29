#![cfg(test)]

use crate::{Uint, small::*, uint, expr};

pub(crate) type SatDec<N> = uint::From<expr::If<N, expr::_DecUnchecked<N>, U0>>;

#[test]
/// Make sure the test runner is actually testing anything, since it uses SatDec to traverse ranges.
fn test_satdec() {
    fn doit<const N: u128, V: Uint>()
    where
        crate::consts::ConstU128<N>: crate::NatExpr<Eval = V>,
    {
        assert_eq!(uint::to_u128::<SatDec<V>>(), Some(N.saturating_sub(1)),)
    }
    macro_rules! tests {
        ($($val:literal)*) => {$(
            doit::<$val, _>();
        )*};
    }
    tests! { 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 }
}

const MORE_TESTS: bool = option_env!("more_uint_tests").is_some();
const SKIP_TESTS: bool = option_env!("skip_uint_tests").is_some();
pub(crate) type DefaultHi = uint::From<
    expr::If<
        crate::consts::ConstBool<SKIP_TESTS>,
        U0,
        expr::If<
            crate::consts::ConstBool<MORE_TESTS>, //
            uint::lit!(50),
            uint::lit!(10),
        >,
    >,
>;
pub(crate) type DefaultLo = crate::small::U0;

/// A type-level linked list of `Uint`s
pub(crate) trait UintList: Sized {
    const EMPTY: bool;
    type First: Uint;
    type Tail: UintList;
    type Len: Uint;

    type ReduceTestsArgs<T: Tests<RangesLo = Self>>: Tests<RangesLo = ()>;
}

pub(crate) type ListLen<L> = <L as UintList>::Len;
pub(crate) type InputLen<T> = ListLen<<T as Tests>::RangesLo>;
pub(crate) trait Tests: Sized {
    // The n-dimensional input range
    type RangesLo: UintList;
    type RangesHi: UintList;
    fn run_tests_on<L: UintList<Len = InputLen<Self>>>();
}

// The empty list must be cyclical for `Tail` and `ReduceTests` when recursing over it,
// so that we don't need to monomorphize infinitely many functions.
impl UintList for () {
    const EMPTY: bool = true;
    type First = U0;
    type Tail = Self;
    type Len = U0;

    type ReduceTestsArgs<T: Tests<RangesLo = Self>> = T;
}
impl<N: Uint, L: UintList> UintList for (N, L) {
    const EMPTY: bool = false;
    type First = N;
    type Tail = L;
    type Len = uint::From<expr::_Inc<L::Len>>;

    type ReduceTestsArgs<T: Tests<RangesLo = Self>> =
        <L as UintList>::ReduceTestsArgs<FirstArgTestsTraverser<T>>;
}

/// Recursively apply `ReduceTest` until we have no parameters left.
pub(crate) fn run_tests<T: Tests>() {
    const fn get_dispatch<T: Tests>() -> fn() {
        <T::RangesLo as UintList>::ReduceTestsArgs::<T>::run_tests_on::<()>
    }
    let dispatch = const {
        if SKIP_TESTS {
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
    Lo: Uint,
    LoTail: UintList,
{
    type RangesLo = LoTail;
    type RangesHi = <T::RangesHi as UintList>::Tail;

    fn run_tests_on<L: UintList<Len = InputLen<Self>>>() {
        Self::good_traverse::<L, <T::RangesHi as UintList>::First>()
    }
}
impl<T, Len, Lo, LoTail> FirstArgTestsTraverser<T>
where
    T: Tests<RangesLo = (Lo, LoTail)>,
    Lo: Uint,
    Len: Uint,
    LoTail: UintList<Len = Len>,
{
    const fn next_good_traverse<L: UintList<Len = Len>, N: Uint>() -> fn() {
        Self::good_traverse::<L, SatDec<N>>
    }
    fn good_traverse<L: UintList<Len = Len>, N: Uint>() {
        let (test, next) = const {
            let cmp = uint::cmp::<N, <T::RangesLo as UintList>::First>();
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
                fn run_tests_on<L: crate::expr::testing::UintList<Len = LeafInputLen>>() {
                    Flattener::<L>::doit()
                }
            }
            struct Flattener<L>(L);
            impl<
                // Name a list using each param. The tail of the list
                // is the parameter after it. For the last parameter,
                // the tail doesn't matter, so use an extra dummy param.
                $first: crate::expr::testing::UintList<
                    Tail = $fshifted
                >
                $(, $param: crate::expr::testing::UintList<
                    Tail = $shifted
                >)*
                , __Extra: crate::expr::testing::UintList
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
            fn doit<$first: crate::Uint $(, $param: crate::Uint)*>() {
                let $first = crate::uint::to_u128::<$first>().unwrap();
                $(let $param = crate::uint::to_u128::<$param>().unwrap();)*
                assert_eq!(
                    crate::uint::to_u128::<$got>(),
                    Some($expect),
                    "params={:?}",
                    ($($param),*)
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
    (@bound $n:expr) => { crate::uint::From<crate::consts::ConstU128<{$n}>> };
    (@select lo $lo:ty, $_:ty $(,)?) => { $lo };
    (@select hi $_:ty, $hi:ty $(,)?) => { $hi };
}
pub(crate) use test_op;
