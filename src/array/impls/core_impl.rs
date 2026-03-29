use crate::{Nat, array::*, nat};

impl<T, N: Nat, A> ArrApi<A>
where
    A: Array<Item = T, Length = N>,
{
    /// Converts the array to a slice.
    ///
    /// Equivalent to [`<[T; N]>::as_slice`](primitive.slice.method.as_slice).
    ///
    /// # Panics
    /// If `Length > usize::MAX`. This is guaranteed behavior.
    pub const fn as_slice(&self) -> &[T] {
        arr_api::unsize_ref(self)
    }

    /// Converts the array to a mutable slice.
    ///
    /// Equivalent to [`<[T; N]>::as_mut_slice`](primitive.slice.method.as_mut_slice).
    ///
    /// # Panics
    /// If `Length > usize::MAX`. This is guaranteed behavior.
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        arr_api::unsize_mut(self)
    }

    /// Equivalent of [`<[T; N]>::each_ref`](array::each_ref).
    ///
    /// Note that this method does not compile for `Length > usize::MAX` because the returned
    /// array will be too large for the architecture.
    pub const fn each_ref(&self) -> ArrApi<impl Array<Item = &T, Length = N> + Copy> {
        let mut out = ArrVecApi::<super::CopyArr<_, _>>::new();
        let mut this = self.as_slice();
        while let [first, rest @ ..] = this {
            out.push(first);
            this = rest;
        }
        out.assert_full()
    }

    /// Equivalent of [`<[T; N]>::each_mut`](array::each_mut).
    pub const fn each_mut(&mut self) -> ArrApi<impl Array<Item = &mut T, Length = N>> {
        let mut out = ArrVec::new();
        let mut this = self.as_mut_slice();
        while let [first, rest @ ..] = this {
            out.push(first);
            this = rest;
        }
        out.assert_full()
    }

    /// Equivalent of [`<[T; N]>::map`](array::map).
    pub fn map<B>(self, mut f: impl FnMut(T) -> B::Item) -> ArrApi<B>
    where
        B: Array<Length = N>,
    {
        let mut src = container::ArrConsumer::new(self);
        let mut dst = container::ArrBuilder::new();
        while let Some(item) = src.pop_front() {
            // SAFETY: `src` only returns up to `Length` items
            unsafe { dst.push_unchecked(f(item)) }
        }
        // SAFETY: `src` is empty, so this is full
        unsafe { dst.into_full_unchecked() }
    }
}

impl<A, T, N: Nat> Clone for ArrApi<A>
where
    A: Array<Item = T, Length = N>,
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut src = container::ArrRefConsumer::new(self);
        let mut dst = container::ArrBuilder::new();
        while let Some(item) = src.pop_front() {
            // SAFETY: `src` only returns up to `Length` items
            unsafe { dst.push_unchecked(item.clone()) }
        }
        // SAFETY: `src` is empty, so this is full
        unsafe { dst.into_full_unchecked() }
    }
}
impl<A> Copy for ArrApi<A> where A: Array<Item: Copy> + Copy {}

impl<A, T, N: Nat> Default for ArrApi<A>
where
    A: Array<Item = T, Length = N>,
    T: Default,
{
    fn default() -> Self {
        Arr::of(()).map(|()| Default::default())
    }
}

const _: () = {
    use core::fmt::{self, Debug};

    impl<A, T, N: Nat> Debug for ArrApi<A>
    where
        A: Array<Item = T, Length = N>,
        T: Debug,
    {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if nat::to_usize::<A::Length>().is_some() {
                write!(f, "{:?}", self.as_slice())
            } else {
                write!(f, "[...]")
            }
        }
    }
};

const _: () = {
    use core::hash::{Hash, Hasher};

    impl<A> Hash for ArrApi<A>
    where
        A: Array<Item: Hash>,
    {
        fn hash<H: Hasher>(&self, state: &mut H) {
            if const { nat::to_usize::<A::Length>().is_some() } {
                self.as_slice().hash(state)
            } else {
                fn hash_zst<A, H>(a: &A, h: &mut H)
                where
                    A: Array<Item: Hash>,
                    H: Hasher,
                {
                    let mut src = container::ArrRefConsumer::new(a);
                    while let Some(item) = src.pop_front() {
                        item.hash(h)
                    }
                }
                hash_zst(self, state);
            }
        }
    }
};
