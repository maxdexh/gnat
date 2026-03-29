//! implementation detail for some error messages

use core::marker::PhantomData;

use crate::{
    Nat,
    maxint::{self, Umax},
    utils::subslice,
};

/// ```rust_analyzer_prefer_brackets
/// fmt![]
/// ```
macro_rules! fmt {
    [] => {
        crate::const_fmt::ConstFmtWrap("").enter()
    };
    [ $f:expr $(, $($then:tt)* )? ] => {
        crate::const_fmt::ConstFmtVariants::Pair(
            crate::const_fmt::ConstFmtWrap($f).enter(),
            crate::const_fmt::fmt![$($($then)*)?],
        )
    };
}
pub(crate) use fmt;

pub(crate) enum Never {}

/// # Safety
/// Must be ReifyVariants<'a, Self> (or with compatible lifetime), or uninhabited.
pub(crate) unsafe trait DestructFmt<'a>: 'a {
    type Inner1: DestructFmt<'a>;
    type Inner2: DestructFmt<'a>;
}
// SAFETY: By definition
unsafe impl<'a> DestructFmt<'a> for Never {
    type Inner1 = Self;
    type Inner2 = Self;
}
// SAFETY: By definition
unsafe impl<'a, I1, I2> DestructFmt<'a> for ConstFmtVariants<'a, I1, I2>
where
    I1: DestructFmt<'a>,
    I2: DestructFmt<'a>,
{
    type Inner1 = I1;
    type Inner2 = I2;
}

#[allow(type_alias_bounds)]
type ReifyVariants<'a, Fmt: DestructFmt<'a>> = ConstFmtVariants<'a, Fmt::Inner1, Fmt::Inner2>;
const fn reify_variants<'a, Fmt>(fmt: &Fmt) -> &ReifyVariants<'a, Fmt>
where
    Fmt: DestructFmt<'a>,
{
    // SAFETY: Fmt is either ReifyVariants or uninhabited. Since we have a reference, it is the
    // former. ReifyVariants is covariant in its lifetime, so this is ok.
    unsafe { &*core::ptr::from_ref(fmt).cast() }
}

pub enum ConstFmtVariants<'a, Inner1 = Never, Inner2 = Never> {
    Str(&'a str),
    Umax(Umax),
    Pair(Inner1, Inner2),
}
impl<'a, I1, I2> ConstFmtVariants<'a, I1, I2>
where
    I1: DestructFmt<'a>,
    I2: DestructFmt<'a>,
{
    pub(crate) const fn fmt_write_impl<'b>(&self, out: &'b mut [u8]) -> Option<&'b mut [u8]> {
        match *self {
            ConstFmtVariants::Str(s) => {
                const fn doit_str<'a>(s: &str, mut out: &'a mut [u8]) -> Option<&'a mut [u8]> {
                    let s = s.as_bytes();
                    if out.len() < s.len() {
                        out.copy_from_slice(subslice![&s, _, out.len()]);
                        return None;
                    }
                    let s_out;
                    (s_out, out) = out.split_at_mut(s.len());
                    s_out.copy_from_slice(s);
                    Some(out)
                }
                doit_str(s, out)
            }
            ConstFmtVariants::Umax(n) => {
                const fn doit_umax(mut n: Umax, mut out: &mut [u8]) -> Option<&mut [u8]> {
                    if out.is_empty() {
                        return None;
                    }
                    let mut ran_out = false;
                    while out.len() < maxint::umax_strlen(n) {
                        ran_out = true;
                        n /= 10;
                    }
                    out = maxint::umax_write(n, out);
                    if ran_out {
                        debug_assert!(out.is_empty());
                        None
                    } else {
                        Some(out)
                    }
                }
                doit_umax(n, out)
            }
            ConstFmtVariants::Pair(ref inner1, ref inner2) => {
                if let Some(new_out) = reify_variants::<I1>(inner1).fmt_write_impl(out) {
                    reify_variants::<I2>(inner2).fmt_write_impl(new_out)
                } else {
                    None
                }
            }
        }
    }
    #[track_caller]
    pub(crate) const fn panic_with_bufsize<const BUFSIZE: usize>(&self) -> ! {
        #[track_caller]
        const fn doit(buf: &mut [u8], rest: Option<usize>) -> ! {
            let result = if let Some(rest) = rest {
                buf.split_at(buf.len() - rest).0
            } else {
                const ELLISPIS: &[u8] = b"...";
                buf.split_at_mut(buf.len() - ELLISPIS.len())
                    .1
                    .copy_from_slice(ELLISPIS);
                buf
            };
            panic!(
                "{}",
                match core::str::from_utf8(result) {
                    Ok(out) => out,
                    Err(_) => unreachable!(),
                }
            )
        }
        let mut buf = [0; BUFSIZE];
        let buf = buf.as_mut_slice();
        let rest = match self.fmt_write_impl(buf) {
            Some(rest) => Some(rest.len()),
            None => None,
        };
        doit(buf, rest)
    }
    #[track_caller]
    pub(crate) const fn panic(&self) -> ! {
        // Limit message size to 10KiB
        self.panic_with_bufsize::<{ 10 << 10 }>()
    }
}

#[repr(transparent)]
pub(crate) struct ConstFmtWrap<T>(pub T);
impl<T> ConstFmtWrap<T> {
    pub(crate) const fn into_inner(self) -> T {
        // SAFETY: repr(transparent)
        unsafe { crate::utils::union_transmute!(ConstFmtWrap<T>, T, self) }
    }
}
macro_rules! enter_impl {
    (
        [$($generics:tt)*],
        $Self:ty,
        $VariantsT:ty,
        |$self:pat_param| $to_variant:expr
    ) => {
        #[allow(dead_code)]
        impl<$($generics)*> ConstFmtWrap<$Self> {
            pub(crate) const fn enter(self) -> $VariantsT {
                let $self = self.into_inner();
                $to_variant
            }
        }
    };
}
enter_impl![
    [],
    usize, //
    ConstFmtVariants<'static>,
    |n| ConstFmtVariants::Umax(n as _)
];
enter_impl![
    [],
    u128, //
    ConstFmtVariants<'static>,
    |n| ConstFmtVariants::Umax(n as _)
];
enter_impl![
    [N: Nat],
    PhantomData<N>,
    ConstFmtVariants<'static>,
    |_| ConstFmtVariants::Str(crate::uint::to_str::<N>())
];
enter_impl![
    ['a],
    &'a str,
    ConstFmtVariants<'a>,
    |s| ConstFmtVariants::Str(s)
];
