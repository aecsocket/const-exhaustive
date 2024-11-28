# `const-exhaustive`

[![crates.io](https://img.shields.io/crates/v/const-exhaustive.svg)](https://crates.io/crates/const-exhaustive)
[![docs.rs](https://img.shields.io/docsrs/const-exhaustive)](https://docs.rs/const-exhaustive)

Enumerate all values of a type at compile time.

## Features

- **All values of `T: Exhaustive` are stored in a [`GenericArray`]** - allowing you to access all
  values at compile time, and in a const context.

- **Composable with `core` types** - supports `[T; N]`, tuples up to arity 16, `Option<T>`, and
  other types in `core`.

- **`#[derive(Exhaustive)]`** - to implement it on your own types.

- **`#![no_std]` and no `alloc`** - you can use it anywhere.

[`GenericArray`]: https://docs.rs/generic-array/

## Examples

```rust
use const_exhaustive::Exhaustive;

// there is 1 value of `()`
assert_eq!([()], <()>::ALL.as_slice());

// there are 2 values of `bool`
assert_eq!([false, true], bool::ALL.as_slice());

// works on types with generics
assert_eq!(
    [None, Some(false), Some(true)],
    Option::<bool>::ALL.as_slice()
);

// write your own exhaustive types
#[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
enum Direction {
    North,
    South,
    East,
    West,
}

assert_eq!(
    [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
    ],
    Direction::ALL.as_slice()
);

// works on arbitrarily complex types
#[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
enum Complex {
    Direction(Direction),
    More {
        foo: Option<bool>,
        bar: (Result<Direction, [bool; 4]>),
    },
}
```

## Testing

Run unit and doc tests:

```bash
cargo test
```

Run miri tests:

```bash
cargo +nightly miri test
```

Test generating docs:

```bash
RUSTDOCFLAGS="--cfg docsrs_dep" cargo +nightly doc --workspace --all-features
```
