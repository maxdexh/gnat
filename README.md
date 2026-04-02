[![Crates.io](https://img.shields.io/crates/v/generic-upper-bound.svg)](https://crates.io/crates/generic-upper-bound)
[![Documentation](https://docs.rs/generic-upper-bound/badge.svg)](https://docs.rs/generic-upper-bound)
[![Rust](https://img.shields.io/badge/rust-1.78.0%2B-blue.svg?maxAge=3600)](https://github.com/rust-lang/generic-upper-bound)

<!-- cargo-reedme: start -->

<!-- cargo-reedme: info-start

    Do not edit this region by hand
    ===============================

    This region was generated from Rust documentation comments by `cargo-reedme` using this command:

        cargo +nightly reedme --all-features

    for more info: https://github.com/nik-rev/cargo-reedme

cargo-reedme: info-end -->

This crate provides type-level natural numbers, similar to [`typenum`](https://docs.rs/typenum/latest/typenum/).

A type-level number is a type that represents a number. The [`Nat`](https://docs.rs/gnat/latest/gnat/trait.Nat.html) trait takes the role of the
“type-level number type”, i.e. one accepts a type-level number using a generic parameter with
bound [`Nat`](https://docs.rs/gnat/latest/gnat/trait.Nat.html).

The use cases are the same as those of generic consts.

`gnat` differs from `typenum` in that its [`Nat`](https://docs.rs/gnat/latest/gnat/trait.Nat.html) trait is not a marker trait, but defines
enough (internal) structure to be able to define and use operations on it, generically.

This crate is to the unstable `generic_const_expr` feature what `typenum` is to the already
stable `min_const_generics` feature. For example, consider the case of concatenating arrays
at compile time
```rust
// Ideal function, requires #![feature(generic_const_expr)]
const fn concat_arrays_gce<T, const M: usize, const N: usize>(
    a: [T; M],
    b: [T; N],
) -> [T; M + N] {
    todo!()
}
// typenum + generic-array implementation
use generic_array::{GenericArray, ArrayLength};
const fn concat_arrays_gar<T, M: ArrayLength, N: ArrayLength>(
    a: GenericArray<T, M>,
    b: GenericArray<T, N>,
) -> GenericArray<T, typenum::op!(M + N)>
where // ArrayLength is not enough, we also need to add a bound for `+`
    M: std::ops::Add<N, Output: ArrayLength>,
{
    todo!()
}
```
```rust
// gnat implementation
use gnat::{Nat, array::Arr};
const fn concat_arrays_nat<T, M: Nat, N: Nat>(
    a: Arr<T, M>,
    b: Arr<T, N>,
) -> Arr<T, gnat::eval!(M + N)> {
    a.concat_arr(b).retype()
}
```

It is also possible to implement custom operations without any extra bounds needed to use them.
See the [`mod@expr`](https://docs.rs/gnat/latest/gnat/expr/) module.

<!-- cargo-reedme: end -->
