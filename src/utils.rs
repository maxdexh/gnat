use core::{
    mem::{ManuallyDrop, MaybeUninit},
    ptr::NonNull,
};
pub(crate) use gnat_proc::__apply as apply;

/// Performs the operation of moving the argument into a `repr(C)` union
/// of `Src` and `Dst` and reading out `Dst`.
///
/// In particular, the following operations are equivalent (`SRC := size_of::<Src>(), DST: size_of::<Dst>()`):
/// - If `SRC >= DST` (a shrinking transmute), this is equivalent to [`core::mem::transmute_copy`] from
///   `Src` to `Dst` (plus forgetting the input)
/// - If `SRC == DST`, it is equivalent to [`core::mem::transmute`]
/// - It is always equivalent to swapping `min(SRC, DST)` bytes of `MaybeUninit::new(src)` and
///   `MaybeUninit::<Dst>::uninit()` and calling `assume_init` on the latter.
///   I.e. if `SRC <= DST` (a growing transmute) does a regular transmute in addition to leaving
///   the remaining bytes of `Dst` uninitialized.
///
/// # Safety
/// The described operation must be safe.
pub(crate) const unsafe fn _union_transmute<Src, Dst>(src: Src) -> Dst {
    #[repr(C)]
    union Helper<Src, Dst> {
        src: ManuallyDrop<Src>,
        dst: ManuallyDrop<Dst>,
    }
    // SAFETY: By definition
    unsafe {
        ManuallyDrop::into_inner(
            Helper {
                src: ManuallyDrop::new(src),
            }
            .dst,
        )
    }
}
/// Wrapper for [`_union_transmute`] that forces explicit type args.
macro_rules! union_transmute {
    ($Src:ty, $Dst:ty, $src:expr $(,)?) => {
        crate::utils::_union_transmute::<$Src, $Dst>($src)
    };
}
pub(crate) use union_transmute;

/// Transmutes a type to itself.
///
/// # Safety
/// `Src` and `Dst` must be exactly the same type.
pub(crate) const unsafe fn _same_type_transmute<Src, Dst>(src: Src) -> Dst {
    if size_of::<Src>() != size_of::<Dst>() || align_of::<Src>() != align_of::<Dst>() {
        // SAFETY: `Src` and `Dst` are the same type, so they have the same size and alignment
        unsafe { core::hint::unreachable_unchecked() }
    }
    // SAFETY: Trivial transmute, since Src and Dst are the same
    unsafe { _union_transmute::<Src, Dst>(src) }
}
/// Wrapper for [`_same_type_transmute`] that forces explicit type args.
macro_rules! same_type_transmute {
    ($Src:ty, $Dst:ty, $src:expr $(,)?) => {
        crate::utils::_same_type_transmute::<$Src, $Dst>($src)
    };
}
pub(crate) use same_type_transmute;

/// # Safety
/// All elements must be initialized
pub(crate) const unsafe fn assume_init_slice<T>(slice: &[MaybeUninit<T>]) -> &[T] {
    // SAFETY: repr(transparent); All elements are initialized, so reading initialized values is safe
    unsafe { core::slice::from_raw_parts(slice.as_ptr().cast(), slice.len()) }
}

/// # Safety
/// All elements must be initialized
pub(crate) const unsafe fn assume_init_mut_slice<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    // SAFETY: repr(transparent); All elements are initialized, so reading initialized values is safe
    // Writing initialized elements (which may drop old values) is safe too.
    unsafe { core::slice::from_raw_parts_mut(slice.as_mut_ptr().cast(), slice.len()) }
}

/// Creates a [`NonNull`] from an immutable reference. The returned pointer is only valid for
/// reads.
pub(crate) const fn nonnull_from_const_ref<T: ?Sized>(r: &T) -> NonNull<T> {
    // SAFETY: References are never null
    unsafe { NonNull::new_unchecked(core::ptr::from_ref(r).cast_mut()) }
}

macro_rules! subslice {
    ( & $slice:expr, $($range:tt)* ) => {
        crate::utils::subslice!(@split_at $slice, $($range)*)
    };
    ( &mut $slice:expr, $($range:tt)* ) => {
        crate::utils::subslice!(@split_at_mut $slice, $($range)*)
    };
    ( @$method:ident $slice:expr, _, $right:expr $(,)? ) => {
        $slice.$method($right).0
    };
    ( @$method:ident $slice:expr, $left:expr, _ $(,)? ) => {
        $slice.$method($left).1
    };
    ( @$method:ident $slice:expr, $left:expr, $right:expr $(,)? ) => {
        $slice.$method($right).0.$method($left).1
    };
}
pub(crate) use subslice;

macro_rules! docexpr {
    [ $(#[doc = $doc:expr])* ] => {
        ::core::concat!($($doc, "\n"),*)
    };
}
pub(crate) use docexpr;
