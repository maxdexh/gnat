use crate::{NatExpr, array::*, consts::Usize};

fn partial_eq_impl<A: Array, U>(lhs: &ArrApi<A>, rhs: &[U]) -> bool
where
    A::Item: PartialEq<U>,
{
    (const { crate::to_usize::<A::Length>().is_some() } && lhs.as_slice() == rhs)
}

impl<A, U> PartialEq<[U]> for ArrApi<A>
where
    A: Array,
    A::Item: PartialEq<U>,
{
    fn eq(&self, other: &[U]) -> bool {
        partial_eq_impl(self, other)
    }
}
impl<A, U> PartialEq<&[U]> for ArrApi<A>
where
    A: Array,
    A::Item: PartialEq<U>,
{
    fn eq(&self, &other: &&[U]) -> bool {
        partial_eq_impl(self, other)
    }
}
impl<A, U, const N: usize> PartialEq<[U; N]> for ArrApi<A>
where
    A: Array,
    A::Item: PartialEq<U>,
    Usize<N>: NatExpr<Eval = A::Length>,
{
    fn eq(&self, other: &[U; N]) -> bool {
        partial_eq_impl(self, other)
    }
}
impl<A, B> PartialEq<ArrApi<B>> for ArrApi<A>
where
    A: Array,
    B: Array,
    A::Item: PartialEq<B::Item>,
{
    fn eq(&self, other: &ArrApi<B>) -> bool {
        if const { crate::cmp::<A::Length, B::Length>().is_ne() } {
            false
        } else if const { crate::to_usize::<A::Length>().is_some() } {
            self.as_slice() == other.as_slice()
        } else {
            let mut lhs = container::ArrRefConsumer::new(self);
            let mut rhs = container::ArrRefConsumer::new(other);
            while let (Some(l), Some(r)) = (lhs.pop_front(), rhs.pop_front()) {
                if l != r {
                    return false;
                }
            }
            true
        }
    }
}
impl<A> Eq for ArrApi<A>
where
    A: Array,
    A::Item: Eq,
{
}

impl<A> PartialOrd for ArrApi<A>
where
    A: Array,
    A::Item: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        // FIXME: oversize
        self.as_slice().partial_cmp(other.as_slice())
    }
}
impl<A> Ord for ArrApi<A>
where
    A: Array,
    A::Item: Ord,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // FIXME: oversize
        self.as_slice().cmp(other.as_slice())
    }
}
