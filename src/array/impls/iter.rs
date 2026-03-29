use crate::{
    Nat,
    array::{container::*, *},
};

// TODO: override default impls, add methods like as_slice
pub struct IntoIter<A: Array> {
    pub(crate) items: ArrConsumer<A>,
}

impl<T, N: Nat, A> Iterator for IntoIter<A>
where
    A: Array<Item = T, Length = N>,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.items.pop_front()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        #[allow(clippy::option_if_let_else)]
        match self.items.len() {
            Some(len) => (len, Some(len)),
            None => (usize::MAX, None),
        }
    }
}
impl<T, N: Nat, A> DoubleEndedIterator for IntoIter<A>
where
    A: Array<Item = T, Length = N>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.items.pop_back()
    }
}

impl<A: Array> IntoIterator for ArrApi<A> {
    type Item = A::Item;
    type IntoIter = IntoIter<Self>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            items: ArrConsumer::new(self),
        }
    }
}
impl<A: Array> IntoIterator for ArrVecApi<A> {
    type Item = A::Item;
    type IntoIter = IntoIter<A>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            items: ArrConsumer::from_vec(self),
        }
    }
}

pub struct IntoIterDeq<A: Array> {
    pub(crate) deq: ArrDeqApi<A>,
}
impl<A: Array> Iterator for IntoIterDeq<A> {
    type Item = A::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.deq.pop_front()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.deq.len();
        (len, Some(len))
    }
}
impl<T, N: Nat, A> DoubleEndedIterator for IntoIterDeq<A>
where
    A: Array<Item = T, Length = N>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.deq.pop_back()
    }
}
impl<T, N: Nat, A> ExactSizeIterator for IntoIterDeq<A> where A: Array<Item = T, Length = N> {}
impl<A: Array> IntoIterator for ArrDeqApi<A> {
    type Item = A::Item;
    type IntoIter = IntoIterDeq<A>;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { deq: self }
    }
}
