use crate::{Nat, array::CopyArr};

pub type PopDigit<N> = crate::Eval<crate::expr::_Shr<N, crate::consts::PtrBits>>;

#[crate::utils::apply(crate::expr::nat_expr)]
pub type _DigitLenRec<N: crate::NatExpr> = _DigitLen<PopDigit<N>>;
#[crate::utils::apply(crate::expr::nat_expr)]
pub type _DigitLen<N: crate::NatExpr> = crate::expr::If<
    N,
    crate::expr::_Inc<_DigitLenRec<N>>, //
    crate::lit!(0),
>;
pub type DigitLen<N> = crate::Eval<_DigitLen<N>>;

/// A number type that stores any number up to and including some [`Nat`].
pub(crate) struct Fin<N: Nat> {
    /// # Safety
    /// Represents a number in base `usize::MAX + 1`.
    /// Little-endian, i.e. least significant digit at index 0.
    ///
    /// Must be less than or equal to N
    digits: CopyArr<usize, DigitLen<N>>,
}
const fn all_zeros(mut digits: &[usize]) -> bool {
    while let &[ref rest @ .., last] = digits {
        digits = rest;
        if last != 0 {
            return false;
        }
    }
    true
}
impl<N: Nat> Fin<N> {
    pub const fn dec(&mut self) -> bool {
        if self.is_zero() {
            return false;
        }
        // SAFETY: self > 0, so decrementing is ok
        let mut digits = self.digits.as_mut_slice();
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
    /// # Safety
    /// self < N
    pub const unsafe fn inc_unchecked(&mut self) {
        // SAFETY: self < N, so incrementing is ok
        let mut digits = self.digits.as_mut_slice();
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
        all_zeros(self.digits.as_slice())
    }
    pub const fn zero() -> Self {
        Self {
            digits: CopyArr::of(0),
        }
    }
    pub const fn max() -> Self {
        const {
            // SAFETY: This construction ensures self == N
            Self {
                digits: if crate::is_zero::<N>() {
                    CopyArr::of(0)
                } else {
                    Fin::<PopDigit<N>>::max()
                        .digits
                        .concat_arr([crate::to_usize_overflowing::<N>().0])
                        .try_retype()
                        .unwrap()
                },
            }
        }
    }
    pub const fn to_usize(&self) -> Option<usize> {
        match self.digits.as_slice() {
            [] => Some(0),
            [lsd, rest @ ..] => match all_zeros(rest) {
                true => Some(*lsd),
                false => None,
            },
        }
    }
}
