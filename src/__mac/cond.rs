use core::marker::PhantomData;

use crate::{NatExpr, condty::*, utils};

pub use Result;

// SAFETY INVARIANT: `COND = !gnat::is_zero::<C>()`
// Conversely, the existence of an instance of this type proves the above statement!
pub struct CtxCond<C, const COND: bool> {
    _p: PhantomData<C>,
}

pub const fn hold<C: crate::NatExpr>() -> Result<CtxCond<C, true>, CtxCond<C, false>> {
    if crate::is_zero::<C>() {
        Err(CtxCond::<C, false> { _p: PhantomData })
    } else {
        Ok(CtxCond::<C, true> { _p: PhantomData })
    }
}

impl<C: NatExpr> CtxCond<C, true> {
    #[inline(always)]
    pub const fn new_true<T, F>(&self, t: T) -> CondTy<C, T, F> {
        // SAFETY: C is nonzero, so `CondTy<C, T, F> = T`
        unsafe { utils::same_type_transmute!(T, CondTy::<C, T, F>, t) }
    }
    #[inline(always)]
    pub const fn new_ok<T, E>(&self, ok: T) -> CondResult<C, T, E> {
        CondResult {
            inner: self.new_true(ok),
        }
    }
    #[inline(always)]
    pub const fn new_some<T>(&self, some: T) -> CondOption<C, T> {
        CondOption {
            inner: self.new_true(some),
        }
    }
    #[inline(always)]
    pub const fn unwrap_true<T, F>(&self, t: CondTy<C, T, F>) -> T {
        // SAFETY: C is nonzero, so `CondTy<C, T, F> = T`
        unsafe { utils::same_type_transmute!(CondTy::<C, T, F>, T, t) }
    }
    #[inline(always)]
    pub const fn unwrap_ok<T, E>(&self, ok: CondResult<C, T, E>) -> T {
        self.unwrap_true(ok.into_inner())
    }
    #[inline(always)]
    pub const fn unwrap_some<T>(&self, some: CondOption<C, T>) -> T {
        self.unwrap_true(some.into_inner())
    }
}
impl<C: NatExpr> CtxCond<C, false> {
    #[inline(always)]
    pub const fn new_false<T, F>(&self, f: F) -> CondTy<C, T, F> {
        // SAFETY: C is zero, so `CondTy<C, T, F> = F`
        unsafe { utils::same_type_transmute!(F, CondTy::<C, T, F>, f) }
    }
    #[inline(always)]
    pub const fn new_err<T, E>(&self, err: E) -> CondResult<C, T, E> {
        CondResult {
            inner: self.new_false(err),
        }
    }
    #[inline(always)]
    pub const fn new_none<T>(&self) -> CondOption<C, T> {
        CondOption {
            inner: self.new_false(()),
        }
    }
    #[inline(always)]
    pub const fn unwrap_false<T, F>(&self, f: CondTy<C, T, F>) -> F {
        // SAFETY: C is zero, so `CondTy<C, T, F> = F`
        unsafe { utils::same_type_transmute!(CondTy::<C, T, F>, F, f) }
    }
    #[inline(always)]
    pub const fn unwrap_err<T, E>(&self, err: CondResult<C, T, E>) -> E {
        self.unwrap_false(err.into_inner())
    }
    #[inline(always)]
    pub const fn drop_none<T>(&self, none: CondOption<C, T>) {
        self.unwrap_false(none.into_inner())
    }
}
