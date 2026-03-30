use crate::array::*;

macro_rules! tuple_gen_impl {
    ($count:expr, $P:ident $($T:ident)*) => {
        const _: () = {
            const COUNT: usize = $count;
            type Tuple<$P> = ($P, $($T),*);
            impl<A, T> From<crate::array::ArrApi<A>> for Tuple<T>
            where
                A: Array<Item = T, Length = crate::Eval<crate::consts::Usize<COUNT>>>,
            {
                fn from(value: crate::array::ArrApi<A>) -> Self {
                    crate::array::arr_api::retype::<_, [_; COUNT]>(value).into()
                }
            }
            impl<A, T> From<Tuple<T>> for ArrApi<A>
            where
                A: Array<Item = T, Length = crate::Eval<crate::consts::Usize<COUNT>>>,
            {
                fn from(value: Tuple<T>) -> Self {
                    crate::array::arr_api::retype::<[_; COUNT], _>(value.into())
                }
            }

            const _: () = {
                #[allow(unused)]
                const COUNT_COPY: usize = COUNT;
                tuple_gen_impl! { COUNT_COPY.checked_sub(1).unwrap(), $($T)* }
            };
        };
    };
    ($_:expr,) => {};
}
tuple_gen_impl! {
    12,
    T T T T
    T T T T
    T T T T
}
