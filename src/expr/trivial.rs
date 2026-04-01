use super::*;

/// Checks if `N` is zero.
pub type IsZero<N> = If<N, crate::lit!(0), crate::lit!(1)>;

/// Checks if `N` is nonzero.
pub type IsNonzero<N> = If<N, crate::lit!(1), crate::lit!(0)>;
