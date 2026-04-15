[![Crates.io](https://img.shields.io/crates/v/gnat.svg)](https://crates.io/crates/gnat)
[![Documentation](https://docs.rs/gnat/badge.svg)](https://docs.rs/gnat)
[![Rust](https://img.shields.io/badge/rust-1.90.0%2B-blue.svg?maxAge=3600)](https://github.com/rust-lang/rust)

<!-- cargo-reedme: start -->

<!-- cargo-reedme: info-start

    Do not edit this region by hand
    ===============================

    This region was generated from Rust documentation comments by `cargo-reedme` using this command:

        cargo +nightly reedme --all-features

    for more info: https://github.com/nik-rev/cargo-reedme

cargo-reedme: info-end -->

This crate provides type-level natural numbers, similar to [`typenum`](https://docs.rs/typenum/latest/typenum/).

A type-level number is a type that represents a number. The [`Nat`](https://docs.rs/gnat/latest/gnat/trait.Nat.html) trait functions as the
“meta-type” of type-level numbers, i.e. to accept a type-level number, use a generic
parameter `N: Nat`.

The use cases are the same as those of generic consts.

## Why this crate?
`gnat` differs from `typenum` in that [`Nat`](https://docs.rs/gnat/latest/gnat/trait.Nat.html) is not just a marker trait.
It is sufficient for generic operations, without any extra bounds.
This includes custom operations, see the [`expr`](https://docs.rs/gnat/latest/gnat/expr/) module docs.

### Motivating examples

#### Concatenating arrays at compile time
Using `generic_const_exprs` or `typenum`/`generic-array`:
```rust
#![feature(generic_const_exprs)]
const fn concat_arrays_gcex<T, const M: usize, const N: usize>(
    a: [T; M],
    b: [T; N],
) -> [T; M + N]
where
    [T; M + N]:, // Required well-formedness bound
{
    todo!() // Possible with unsafe code
}

use generic_array::{GenericArray, ArrayLength};
const fn concat_arrays_tnum<T, M: ArrayLength, N: ArrayLength>(
    a: GenericArray<T, M>,
    b: GenericArray<T, N>,
) -> GenericArray<T, typenum::op!(M + N)>
where // ArrayLength is not enough, we also need to add a bound for `+`
    M: std::ops::Add<N, Output: ArrayLength>,
{
    todo!() // Possible with unsafe code
}
```
Using this crate:
```rust
use gnat::{Nat, array::Arr};
const fn concat_arrays_gnat<T, M: Nat, N: Nat>(
    a: Arr<T, M>,
    b: Arr<T, N>,
) -> Arr<T, gnat::eval!(M + N)> { // No extra bounds!
    a.concat_arr(b).retype() // There is even a method for this :)
}
```
#### Const Recursion
Naively writing a function that recurses over the const parameter is impossible in
`generic_const_exprs` and `typenum`, since the recursive argument needs the same
bounds as the parameter:
```rust
#![feature(generic_const_exprs)]
fn recursive_gcex<const N: usize>() -> u32
where
    [(); N / 2]:, // The argument must be well-formed
    [(); (N / 2) / 2]:, // The argument's argument must be well-formed
    // ... need infinitely many bounds, even though N converges to 0
{
    if N == 0 {
        0
    } else {
        // The bounds above for N need to imply the same bounds for N / 2
        recursive_gcex::<{ N / 2 }>() + 1
    }
}

use {std::ops::Div, typenum::{P2, Unsigned}};
fn recursive_tnum<N>() -> u32
where
    N: Unsigned + Div<P2>,
    N::Output: Unsigned + Div<P2>,
    <N::Output as Div<P2>>::Output: Unsigned + Div<P2>,
    // ... again, we would need this to repeat infinitely often
{
    if N::USIZE == 0 { // (Pretend this correctly handles overflow)
        0
    } else {
        recursive_tnum::<typenum::op!(N / 2)>() + 1
    }
}
```
While this can be expressed using a helper trait like `trait RecDiv2: Unsigned { type Output: RecDiv2;  }`,
it is cumbersome and leaks into the bounds of every other calling function.

Using this crate, the naive implementation without bounds just works:
```rust
fn recursive_gnat<N: gnat::Nat>() -> u32 {
    if gnat::is_zero::<N>() {
        0
    } else {
        recursive_gnat::<gnat::eval!(N / 2)>() + 1
    }
}
assert_eq!(recursive_gnat::<gnat::lit!(10)>(), 4); // 10 5 2 1 0
```

<!-- cargo-reedme: end -->
