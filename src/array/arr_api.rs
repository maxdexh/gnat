//! Conversion functions for [`Array`] types.
//!
//! # `try_from_{ref, mut, ...}_slice`
//! Functions that try to convert slices into [`Array`] types.
//!
//! This is equivalent to the [`TryFrom`] impls for builtin arrays from borrow and smart pointer slices.
//!
//! # `retype_{ref, mut, ...}`
//! Functions that cast the [`Array`] type behind the input reference/smart pointer
//! to another [`Array`] type with the same item type and length.
//!
//! # `try_retype_{ref, mut, ...}`
//! Functions that try to case the [`Array`] type behind the input reference/smart pointer
//! to another [`Array`] type with the same item type.
//!
//! #### Errors
//! The conversion succeeds if the length is the same. Otherwise, the input is returned in a
//! [`CondResult`], even for references, since [`CondResult`] does not need extra space for a
//! discriminant, so there is no niche optimization benifit from using an option (or ZST error
//! type).

use crate::{
    array::{Array, helper::*},
    condty::CondResult,
};

macro_rules! cast_by_raw {
    ($name:ident, $param:expr) => {
        $name!(from_raw, $name!(into_raw, $param).cast())
    };
}

macro_rules! decl_ptr {
    (
        $name:ident,
        modifiers! $modifiers:tt,
        $($input:tt)*
    ) => {
        decl_ptr! {
            @[$]
            $name,
            modifiers! $modifiers,
            modifiers! $modifiers,
            $($input)*
        }
    };
    (
        @[$DOLLAR_SIGN:tt]
        $name:ident,
        modifiers! $modifiers:tt,
        modifiers! { $($mods:tt)* },
        typ! { $tparam:ident => $($typ:tt)* },
        doc = ($docname_lhs:expr, $docname_rhs:expr),
        $(into_raw = |$into_raw_par:pat_param| $into_raw:expr,)?
        $(from_raw = |$from_raw_par:pat_param| $from_raw:expr,)?
        cast = |$cast_par:pat_param| $cast:expr,
        fns {
            $($fn:ident: $impl:tt),* $(,)?
        }
        $(,)?
    ) => {
        macro_rules! $name {
            (typ, $DOLLAR_SIGN$tparam:ty) => { $($typ)* };
            (docname, $inner_name:expr) => { core::concat!($docname_lhs, $inner_name, $docname_rhs) };
            $( (into_raw, $ptr:expr) => {{ let $into_raw_par = $ptr; $into_raw }}; )?
            $( (from_raw, $ptr:expr) => {{ let $from_raw_par = $ptr; $from_raw }}; )?
            (cast, $src:expr) => {{ let $cast_par = $src; $cast }};
            $( (fn $fn, $cb:ident) => { $cb! { $name $modifiers $impl } }; )*
            (fn $unknown:ident, $cb:ident) => {};
        }
    };
}
decl_ptr![
    Ref,
    modifiers! { pub const },
    typ! { inner => &$inner },
    doc = ("`&", "`"),
    into_raw = |r| core::ptr::from_ref(r).cast_mut(),
    from_raw = |r| &*r,
    cast = |r| cast_by_raw!(Ref, r),
    fns {
        retype: (retype_ref, try_retype_ref),
        unsize: unsize_ref,
        try_from_slice: (try_from_ref_slice, FromSliceOption),
    },
];
decl_ptr![
    RefMut,
    modifiers! { pub const },
    typ! { inner => &mut $inner },
    doc = ("`&mut ", "`"),
    into_raw = |r| core::ptr::from_mut(r),
    from_raw = |r| &mut *r,
    cast = |r| cast_by_raw!(RefMut, r),
    fns {
        retype: (retype_mut, try_retype_mut),
        unsize: unsize_mut,
        try_from_slice: (try_from_mut_slice, FromSliceOption),
    },
];
decl_ptr![
    Owned,
    modifiers! { pub const },
    typ! { inner => $inner },
    doc = ("`", "`"),
    cast = |r| crate::utils::_union_transmute(r),
    fns {
        retype: (retype, try_retype),
    },
];
decl_ptr![
    Box,
    modifiers! {
        #[cfg(feature = "alloc")]
        #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
        pub
    },
    typ! { inner => alloc::boxed::Box<$inner> },
    doc = ("[`Box<", ">`](std::boxed::Box)"),
    into_raw = |r| alloc::boxed::Box::into_raw(r),
    from_raw = |r| alloc::boxed::Box::from_raw(r),
    cast = |r| cast_by_raw!(Box, r),
    fns {
        retype: (retype_box, try_retype_box),
        unsize: unsize_box,
        try_from_slice: (try_from_boxed_slice, FromSliceResult),
    },
];
decl_ptr![
    Rc,
    modifiers! {
        #[cfg(feature = "alloc")]
        #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
        pub
    },
    typ! { inner => alloc::rc::Rc<$inner> },
    doc = ("[`Rc<", ">`](std::boxed::Box)"),
    into_raw = |r| alloc::rc::Rc::into_raw(r).cast_mut(),
    from_raw = |r| alloc::rc::Rc::from_raw(r),
    cast = |r| cast_by_raw!(Rc, r),
    fns {
        retype: (retype_rc, try_retype_rc),
        unsize: unsize_rc,
        try_from_slice: (try_from_rc_slice, FromSliceResult),
    },
];
decl_ptr![
    Arc,
    modifiers! {
        #[cfg(feature = "alloc")]
        #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
        pub
    },
    typ! { inner => alloc::sync::Arc<$inner> },
    doc = ("[`Rc<", ">`](std::boxed::Box)"),
    into_raw = |r| alloc::sync::Arc::into_raw(r).cast_mut(),
    from_raw = |r| alloc::sync::Arc::from_raw(r),
    cast = |r| cast_by_raw!(Arc, r),
    fns {
        retype: (retype_arc, try_retype_arc),
        unsize: unsize_arc,
        try_from_slice: (try_from_arc_slice, FromSliceResult),
    },
];

macro_rules! for_each_ptr {
    ($fn:ident, $cb:ident) => {
        Ref! { fn $fn, $cb }
        RefMut! { fn $fn, $cb }
        Box! { fn $fn, $cb }
        Rc! { fn $fn, $cb }
        Arc! { fn $fn, $cb }
        Owned! { fn $fn, $cb }
    };
}

macro_rules! decl_retype {
    ($ty:ident { $($mods:tt)* } ($retype:ident, $try_retype:ident)) => {
        #[doc = core::concat!(
            "Converts between ",
            $ty!(docname, "impl Array"),
            ".",
        )]
        ///
        /// The operation always succeeds because the source and destination array types are required
        /// to have the same item and length type.
        $($mods)* fn $retype<Src, Dst>(src: $ty!(typ, Src)) -> $ty!(typ, Dst)
        where
            Src: Array,
            Dst: Array<Item = Src::Item, Length = Src::Length>,
        {
            arr_impl_ubcheck::<Src>();
            arr_impl_ubcheck::<Dst>();

            // SAFETY: N == Dst::Length, `Array` invariant
            unsafe { $ty!(cast, src) }
        }

        #[doc = core::concat!(
            "Converts between ",
            $ty!(docname, "impl Array"),
            ".",
        )]
        ///
        /// The operation does not require the source and destination array types to have the same
        /// length.
        ///
        /// # Errors
        /// The conversion succeeds if the length is the same. Otherwise, the input is returned in a
        /// [`CondResult`], even for references, since [`CondResult`] does not need extra space for a
        /// discriminant, so there is no niche optimization benifit from using an option (or ZST error
        /// type).
        $($mods)* fn $try_retype<Src, Dst>(src: $ty!(typ, Src)) -> CondResult<
            crate::uops::Eq<Src::Length, Dst::Length>,
            $ty!(typ, Dst),
            $ty!(typ, Src),
        >
        where
            Src: Array,
            Dst: Array<Item = Src::Item>,
        {
            arr_impl_ubcheck::<Src>();
            arr_impl_ubcheck::<Dst>();

            crate::condty::ctx!(
                |c| c.new_ok(
                    // SAFETY: Src::Length == Dst::Length
                    unsafe { $ty!(cast, src) },
                ),
                |c| c.new_err(src),
            )
        }
    };
}
for_each_ptr!(retype, decl_retype);

macro_rules! decl_unsize {
    ($ty:ident { $($mods:tt)* } $name:ident) => {
        #[doc = core::concat!(
            "Converts ",
            $ty!(docname, "impl Array<Item = T>"),
            " to ",
            $ty!(docname, "[T]"),
            ".",
        )]
        ///
        /// This is equivalent to the implicit unsize coercion of builtin arrays.
        ///
        /// # Panics
        /// If the length of the input array exceeds `usize::MAX` (only possible for ZSTs)
        #[track_caller]
        $($mods)* fn $name<A: Array>(arr: $ty!(typ, A)) -> $ty!(typ, [A::Item]) {
            arr_impl_ubcheck::<A>();

            // SAFETY: `Array` to slice cast
            unsafe { $ty!(from_raw, crate::array::helper::unsize_raw_mut($ty!(into_raw, arr))) }
        }
    };
}
for_each_ptr!(unsize, decl_unsize);

#[cfg(feature = "alloc")]
macro_rules! FromSliceResult {
    (type, $T:ty, $E:ty) => { Result<$T, $E> };
    (Ok, $expr:expr) => { Ok($expr) };
    (Err, $expr:expr) => { Err($expr) };
}
macro_rules! FromSliceOption {
    (type, $T:ty, $E:ty) => { Option<$T> };
    (Ok, $expr:expr) => { Some($expr) };
    (Err, $expr:expr) => { None };
}
macro_rules! decl_from_slice {
    ($ty:ident { $($mods:tt)* } ($name:ident, $Res:ident)) => {
        #[doc = core::concat!(
            "Converts ",
            $ty!(docname, "[T]"),
            " to ",
            $ty!(docname, "impl Array<Item = T>"),
            ".",
        )]
        ///
        /// This is equivalent to the [`TryFrom`] impls for builtin arrays from borrow and smart pointer slices.
        ///
        /// # Errors
        $($mods)* fn $name<A: Array>(slice: $ty!(typ, [A::Item])) -> $Res!(type, $ty!(typ, A), $ty!(typ, [A::Item])) {
            arr_impl_ubcheck::<A>();

            match crate::uint::to_usize::<A::Length>() {
                Some(arr_len) if arr_len == slice.len() => {
                    // SAFETY:
                    // - Pointer cast with same item and length.
                    // - Ownership is transferred through into_raw followed by from_raw.
                    // - This is the same as the `$ptr<[T; N]> as TryFrom<$ptr<[T]>>` impl
                    let slice: $ty!(typ, A) = unsafe { $ty!(from_raw, $ty!(into_raw, slice).cast()) };
                    $Res!(Ok, slice)
                },
                _ => $Res!(Err, slice),
            }
        }
    };
}
for_each_ptr!(try_from_slice, decl_from_slice);
