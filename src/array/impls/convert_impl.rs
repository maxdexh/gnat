use core::array::TryFromSliceError;

use crate::array::*;

impl<T, A> AsRef<[T]> for ArrApi<A>
where
    A: Array<Item = T>,
{
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}
impl<T, A> AsMut<[T]> for ArrApi<A>
where
    A: Array<Item = T>,
{
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}
impl<T, A> core::borrow::Borrow<[T]> for ArrApi<A>
where
    A: Array<Item = T>,
{
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}
impl<T, A> core::borrow::BorrowMut<[T]> for ArrApi<A>
where
    A: Array<Item = T>,
{
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

fn try_from_slice_error() -> TryFromSliceError {
    enum Never {}
    const EMPTY: &[Never] = &[];
    type A = &'static [Never; 1];
    match A::try_from(EMPTY) {
        Err(err) => err,

        // https://github.com/rust-lang/rust-clippy/issues/11984
        #[allow(clippy::uninhabited_references)]
        Ok([a]) => match *a {},
    }
}
impl<'a, T, A> TryFrom<&'a [T]> for &'a ArrApi<A>
where
    A: Array<Item = T>,
{
    type Error = TryFromSliceError;
    fn try_from(value: &'a [T]) -> Result<Self, Self::Error> {
        arr_api::try_from_ref_slice(value).ok_or_else(try_from_slice_error)
    }
}
impl<'a, T, A> TryFrom<&'a mut [T]> for &'a mut ArrApi<A>
where
    A: Array<Item = T>,
{
    type Error = TryFromSliceError;
    fn try_from(value: &'a mut [T]) -> Result<Self, Self::Error> {
        arr_api::try_from_mut_slice(value).ok_or_else(try_from_slice_error)
    }
}
impl<T, A> TryFrom<&[T]> for ArrApi<A>
where
    A: Array<Item = T>,
    T: Copy,
{
    type Error = TryFromSliceError;
    fn try_from(value: &[T]) -> Result<Self, Self::Error> {
        <&crate::array::CopyArr<_, _>>::try_from(value)
            .copied()
            .map(ArrApi::retype)
    }
}
impl<T, A> TryFrom<&mut [T]> for ArrApi<A>
where
    A: Array<Item = T>,
    T: Copy,
{
    type Error = TryFromSliceError;
    fn try_from(value: &mut [T]) -> Result<Self, Self::Error> {
        (value as &[T]).try_into()
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a, T, A> From<&'a ArrApi<A>> for alloc::borrow::Cow<'a, [T]>
where
    A: Array<Item = T>,
    T: Clone,
{
    fn from(value: &'a ArrApi<A>) -> Self {
        value.as_slice().into()
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a, T, A> From<&'a ArrApi<A>> for alloc::vec::Vec<T>
where
    A: Array<Item = T>,
    T: Clone,
{
    fn from(value: &'a ArrApi<A>) -> Self {
        value.as_slice().into()
    }
}
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<T, A> From<&mut ArrApi<A>> for alloc::vec::Vec<T>
where
    A: Array<Item = T>,
    T: Clone,
{
    fn from(value: &mut ArrApi<A>) -> Self {
        value.as_slice().into()
    }
}
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<T, A> From<ArrApi<A>> for alloc::sync::Arc<[T]>
where
    A: Array<Item = T>,
{
    fn from(value: ArrApi<A>) -> Self {
        arr_api::unsize_arc(alloc::sync::Arc::new(value))
    }
}
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<T, A> From<ArrApi<A>> for alloc::rc::Rc<[T]>
where
    A: Array<Item = T>,
{
    fn from(value: ArrApi<A>) -> Self {
        arr_api::unsize_rc(alloc::rc::Rc::new(value))
    }
}
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<T, A> From<ArrApi<A>> for alloc::boxed::Box<[T]>
where
    A: Array<Item = T>,
{
    fn from(value: ArrApi<A>) -> Self {
        arr_api::unsize_box(alloc::boxed::Box::new(value))
    }
}
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<T, A> From<ArrApi<A>> for alloc::vec::Vec<T>
where
    A: Array<Item = T>,
{
    fn from(value: ArrApi<A>) -> Self {
        <[T]>::into_vec(value.into())
    }
}
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<T, A> From<ArrApi<A>> for alloc::collections::VecDeque<T>
where
    A: Array<Item = T>,
{
    fn from(value: ArrApi<A>) -> Self {
        alloc::vec::Vec::from(value).into()
    }
}
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<K, V, A> From<ArrApi<A>> for std::collections::HashMap<K, V>
where
    A: Array<Item = (K, V)>,
    K: core::hash::Hash + Eq,
{
    fn from(value: ArrApi<A>) -> Self {
        Self::from_iter(value)
    }
}
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<T, A> From<ArrApi<A>> for std::collections::HashSet<T>
where
    A: Array<Item = T>,
    T: core::hash::Hash + Eq,
{
    fn from(value: ArrApi<A>) -> Self {
        Self::from_iter(value)
    }
}
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<K, V, A> From<ArrApi<A>> for alloc::collections::BTreeMap<K, V>
where
    A: Array<Item = (K, V)>,
    K: Ord,
{
    fn from(value: ArrApi<A>) -> Self {
        Self::from_iter(value)
    }
}
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<T, A> From<ArrApi<A>> for alloc::collections::BTreeSet<T>
where
    A: Array<Item = T>,
    T: Ord,
{
    fn from(value: ArrApi<A>) -> Self {
        Self::from_iter(value)
    }
}
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<T, A> From<ArrApi<A>> for alloc::collections::BinaryHeap<T>
where
    A: Array<Item = T>,
    T: Ord,
{
    fn from(value: ArrApi<A>) -> Self {
        alloc::vec::Vec::from(value).into()
    }
}
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<T, A> From<ArrApi<A>> for alloc::collections::LinkedList<T>
where
    A: Array<Item = T>,
    T: Ord,
{
    fn from(value: ArrApi<A>) -> Self {
        Self::from_iter(value)
    }
}
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<T, A> TryFrom<alloc::boxed::Box<[T]>> for alloc::boxed::Box<ArrApi<A>>
where
    A: Array<Item = T>,
{
    type Error = alloc::boxed::Box<[T]>;
    fn try_from(value: alloc::boxed::Box<[T]>) -> Result<Self, Self::Error> {
        arr_api::try_from_boxed_slice(value)
    }
}
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<T, A> TryFrom<alloc::vec::Vec<T>> for ArrApi<A>
where
    A: Array<Item = T>,
{
    type Error = alloc::vec::Vec<T>;
    fn try_from(mut value: alloc::vec::Vec<T>) -> Result<Self, Self::Error> {
        if crate::nat::to_usize::<A::Length>() == Some(value.len()) {
            // SAFETY: set_len(0) is always safe and effectively forgets the elements,
            // ensuring that the drop of `Vec` only frees the allocation.
            unsafe { value.set_len(0) }
            // SAFETY: Transfer ownership of the still initialized elements
            Ok(unsafe { core::ptr::read(value.as_ptr().cast::<Self>()) })
        } else {
            Err(value)
        }
    }
}
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<T, A> TryFrom<alloc::vec::Vec<T>> for alloc::boxed::Box<ArrApi<A>>
where
    A: Array<Item = T>,
{
    type Error = alloc::vec::Vec<T>;
    fn try_from(value: alloc::vec::Vec<T>) -> Result<Self, Self::Error> {
        if crate::nat::to_usize::<A::Length>() == Some(value.len()) {
            value
                .into_boxed_slice()
                .try_into()
                .map_err(|_| unreachable!())
        } else {
            Err(value)
        }
    }
}
