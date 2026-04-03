use std::cmp::Ordering;

use crate::{Nat, array::CopyArr};

type PopDigit<N> = crate::Eval<crate::expr::_Shr<N, crate::consts::PtrBits>>;

#[crate::utils::apply(crate::expr::nat_expr)]
pub type _DigitLenRec<N: crate::NatExpr> = _DigitLen<PopDigit<N>>;
#[crate::utils::apply(crate::expr::nat_expr)]
pub type _DigitLen<N: crate::NatExpr> = crate::expr::If<
    N,
    crate::expr::_Inc<_DigitLenRec<N>>, //
    crate::lit!(0),
>;
type DigitLen<N> = crate::Eval<_DigitLen<N>>;
type DigitArr<N> = CopyArr<usize, DigitLen<N>>;

/// A number type that stores any number up to and including some [`Nat`].
pub(crate) struct Fin<N: Nat> {
    /// # Safety
    /// Represents a number in base `usize::MAX + 1`.
    /// Little-endian, i.e. least significant digit at index 0.
    ///
    /// Must be less than or equal to N
    unsafe_digits: DigitArr<N>,
}
#[inline]
const fn digit_max(a: usize, b: usize) -> usize {
    if a < b { b } else { a }
}

impl<N: Nat> Fin<N> {
    /// SAFETY: Must be <= N
    const unsafe fn from_digits_unchecked(digits: DigitArr<N>) -> Self {
        Self {
            unsafe_digits: digits,
        }
    }
    /// SAFETY: self must remain <= N
    const unsafe fn as_num_mut(&mut self) -> &mut NumSlice {
        NumSlice::from_digits_mut(self.unsafe_digits.as_mut_slice())
    }

    /// Converts to a [`Num`] slice.
    pub const fn as_num(&self) -> &NumSlice {
        NumSlice::from_digits(self.unsafe_digits.as_slice())
    }
    #[expect(dead_code)]
    pub const fn saturating_sub_assign(&mut self, rhs: &NumSlice) {
        // SAFETY: The value only decreases, so we stay <= N
        unsafe { self.as_num_mut() }.saturating_sub_assign(rhs);
    }
    #[expect(dead_code)]
    pub const fn cmp(&self, rhs: &NumSlice) -> Ordering {
        self.as_num().cmp(rhs)
    }
    pub(crate) const fn saturating_dec(&mut self) -> bool {
        // SAFETY: value only decreases
        unsafe { self.as_num_mut() }.saturating_dec()
    }
    /// # Safety
    /// `self < N`
    pub const unsafe fn inc_unchecked(&mut self) {
        // SAFETY: self < N, so incrementing is ok
        let mut digits = self.unsafe_digits.as_mut_slice();
        while let [lsd, rest @ ..] = digits {
            digits = rest;
            let ovfl;
            (*lsd, ovfl) = lsd.overflowing_add(1);
            if !ovfl {
                return;
            }
        }
    }
    pub const fn is_zero(&self) -> bool {
        self.as_num().is_zero()
    }

    pub const ZERO: Self = {
        // SAFETY: 0 <= N for any N: Nat
        unsafe { Self::from_digits_unchecked(CopyArr::of(0)) }
    };

    pub const MAX: Self = {
        const fn doit<N: Nat>() -> DigitArr<N> {
            const {
                if crate::is_zero::<N>() {
                    // 0 == 0
                    CopyArr::of(0)
                } else {
                    // (N / Base) * Base + (N % Base) == N
                    doit::<PopDigit<N>>()
                        .concat_arr([crate::to_usize_overflowing::<N>().0])
                        .try_retype()
                        .unwrap()
                }
            }
        }
        // SAFETY: The constructed number equals N, by construction
        unsafe { Self::from_digits_unchecked(doit::<N>()) }
    };

    pub const fn to_usize(&self) -> Option<usize> {
        match &self.as_num().digits {
            [] => Some(0),
            [lsd, rest @ ..] if NumSlice::from_digits(rest).is_zero() => Some(*lsd),
            _ => None,
        }
    }
}

#[repr(transparent)]
pub(crate) struct NumSlice {
    digits: [usize],
}
impl NumSlice {
    const fn from_digits(digits: &[usize]) -> &Self {
        // SAFETY: https://doc.rust-lang.org/reference/expressions/operator-expr.html#r-expr.as.pointer.unsized
        unsafe { &*(core::ptr::from_ref(digits) as *const NumSlice) }
    }
    const fn from_digits_mut(digits: &mut [usize]) -> &mut Self {
        // SAFETY: https://doc.rust-lang.org/reference/expressions/operator-expr.html#r-expr.as.pointer.unsized
        unsafe { &mut *(core::ptr::from_mut(digits) as *mut NumSlice) }
    }
    const fn get_digit(&self, idx: usize) -> usize {
        if idx < self.digits.len() {
            self.digits[idx]
        } else {
            0
        }
    }
    pub const fn cmp(&self, rhs: &Self) -> Ordering {
        let mut i = digit_max(self.digits.len(), rhs.digits.len());
        while i > 0 {
            i -= 1;
            let l = self.get_digit(i);
            let r = rhs.get_digit(i);
            if l < r {
                return Ordering::Less;
            }
            if l > r {
                return Ordering::Greater;
            }
        }
        Ordering::Equal
    }
    pub const fn is_zero(&self) -> bool {
        let mut digits = &self.digits;
        while let &[ref rest @ .., last] = digits {
            digits = rest;
            if last != 0 {
                return false;
            }
        }
        true
    }

    /// Result is unspecified if lhs < rhs
    const fn sub_assign_unchecked(&mut self, rhs: &Self) {
        let mut carry = false;
        let mut i = 0;
        while i < self.digits.len() {
            let (d, c1) = self.digits[i].overflowing_sub(rhs.get_digit(i));
            let (r, c2) = d.overflowing_sub(carry as _);
            self.digits[i] = r;
            carry = c1 || c2;
            i += 1;
        }
        debug_assert!(!carry);
    }

    pub const fn saturating_sub_assign(&mut self, rhs: &Self) {
        match self.cmp(rhs) {
            Ordering::Greater => self.sub_assign_unchecked(rhs),
            Ordering::Less | Ordering::Equal => {
                let mut i = 0;
                while i < self.digits.len() {
                    self.digits[i] = 0;
                    i += 1;
                }
            }
        }
    }

    const fn saturating_dec(&mut self) -> bool {
        if self.is_zero() {
            return false;
        }
        // SAFETY: self > 0, so decrementing is ok
        let mut digits = &mut self.digits;
        while let [lsd, rest @ ..] = digits {
            digits = rest;
            let ovfl;
            (*lsd, ovfl) = lsd.overflowing_sub(1);
            if !ovfl {
                break;
            }
        }
        true
    }
}
