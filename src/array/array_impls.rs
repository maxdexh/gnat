use crate::array::{Array, ArrayLayout};

// SAFETY: Allowed by definition
unsafe impl<T, const L: usize> ArrayLayout for [T; L] {
    type Item = T;
}
// SAFETY: Allowed by definition
unsafe impl<T, N: crate::Nat, const L: usize> Array for [T; L]
where
    crate::consts::Usize<L>: crate::NatExpr<NatExpr = N>,
{
    type Length = N;
}

// SAFETY: Allowed by definition
unsafe impl<A: ArrayLayout> ArrayLayout for core::mem::ManuallyDrop<A> {
    type Item = core::mem::ManuallyDrop<A::Item>;
}
// SAFETY: Allowed by definition
unsafe impl<A: Array> Array for core::mem::ManuallyDrop<A> {
    type Length = A::Length;
}
// SAFETY: Allowed by definition
unsafe impl<A: Array> Array for core::mem::MaybeUninit<A> {
    type Item = core::mem::MaybeUninit<A::Item>;
    type Length = A::Length;
}
// SAFETY: repr(transparent)
unsafe impl<A: Array> Array for ArrApi<A> {
    type Item = A::Item;
    type Length = A::Length;
}
