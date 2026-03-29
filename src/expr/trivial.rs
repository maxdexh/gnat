use super::*;

/// Checks if `N` is zero.
pub type IsZero<N> = If<N, U0, U1>;

/// Checks if `N` is nonzero.
pub type IsNonzero<N> = If<N, U1, U0>;
