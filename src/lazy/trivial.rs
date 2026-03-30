use super::*;

/// Checks if `N` is zero.
pub type IsZero<N> = If<N, N0, N1>;

/// Checks if `N` is nonzero.
pub type IsNonzero<N> = If<N, N1, N0>;
