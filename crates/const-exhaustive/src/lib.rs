#![cfg_attr(any(docsrs, docsrs_dep), feature(rustdoc_internals))]
#![doc = include_str!("../../../README.md")]
#![no_std]

mod array;

use {
    array::{concat, from_fn, map},
    const_default::ConstDefault,
    core::{
        cell::UnsafeCell,
        convert::Infallible,
        marker::{PhantomData, PhantomPinned},
        mem::MaybeUninit,
        ops::{Add, Mul},
    },
    generic_array::{ArrayLength, GenericArray},
    typenum::{Const, Pow, Sum, ToUInt, U, U0, U1, U2, Unsigned},
    variadics_please::all_tuples,
};
pub use {
    const_exhaustive_derive::Exhaustive,
    generic_array::{self, const_transmute},
    typenum,
};

/// All values of this type are known at compile time.
///
/// This trait should be derived instead of implemented manually - see
/// [`const_exhaustive_derive::Exhaustive`].
///
/// If a type implements this trait, it guarantees that there is a finite set
/// of possible values which may exist for this type, and that they can be
/// enumerated at compile time. Due to this, an [`Exhaustive`] type must be
/// [`Copy`], meaning that this type:
/// - cannot have [`Drop`] logic
/// - cannot store references or pointers
///   - this rules out many complex types, including any heap-allocating types,
///     such as strings
///
/// This trait is implemented for most types in `core` for which it makes sense.
/// However, it is not implemented for any numerical types. Although there are
/// practically a finite set of numbers for any given type (because they have to
/// fit in a finite number of bits, e.g. a [`u8`] must fit in 8 bits), there are
/// theoretically an infinite number of numbers, which goes against the spirit
/// of this trait.
///
/// However, you may still want to define an exhaustive integer, where values
/// may only be in a specific range e.g. `0..4`. In this case, you can either:
/// - define an enum with each value explicitly
/// - write a wrapper type which ensures that the value within it is always in
///   range, then `unsafe impl Exhaustive` on the wrapper
///
/// # Safety
///
/// All possible values of this type, as representable in memory, must be
/// present in [`Exhaustive::ALL`].
///
/// # Examples
///
/// ```
/// use const_exhaustive::Exhaustive;
///
/// // there is 1 value of `()`
/// assert_eq!([()], <()>::ALL.as_slice());
///
/// // there are 2 values of `bool`
/// assert_eq!([false, true], bool::ALL.as_slice());
///
/// // works on types with generics
/// assert_eq!(
///     [None, Some(false), Some(true)],
///     Option::<bool>::ALL.as_slice()
/// );
///
/// // write your own exhaustive types
/// #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
/// enum Direction {
///     North,
///     South,
///     East,
///     West,
/// }
///
/// assert_eq!(
///     [
///         Direction::North,
///         Direction::South,
///         Direction::East,
///         Direction::West,
///     ],
///     Direction::ALL.as_slice()
/// );
/// ```
///
/// Implementing [`Exhaustive`] manually:
///
/// ```
/// use const_exhaustive::{Exhaustive, generic_array::GenericArray, typenum};
///
/// // if you were implementing this for real,
/// // you should probably use `NonZero<u8>` or `nonmax::NonMaxU8` for niche optimization;
/// // but this is a simplified example
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// struct UintUpTo4(u8);
///
/// impl UintUpTo4 {
///     #[must_use]
///     pub const fn new(n: u8) -> Option<Self> {
///         if n < 4 { Some(Self(n)) } else { None }
///     }
///
///     #[must_use]
///     pub const fn get(self) -> u8 {
///         self.0
///     }
/// }
///
/// unsafe impl Exhaustive for UintUpTo4 {
///     type Num = typenum::U4;
///
///     const ALL: GenericArray<Self, Self::Num> =
///         GenericArray::from_array([Self(0), Self(1), Self(2), Self(3)]);
/// }
///
/// assert_eq!(
///     [
///         None,
///         Some(UintUpTo4(0)),
///         Some(UintUpTo4(1)),
///         Some(UintUpTo4(2)),
///         Some(UintUpTo4(3)),
///     ],
///     Option::<UintUpTo4>::ALL.as_slice()
/// );
/// ```
#[diagnostic::on_unimplemented(
    message = "all values of `{Self}` are not known statically",
    label = "not exhaustive",
    note = "consider annotating `{Self}` with `#[derive(Exhaustive)]`"
)]
pub unsafe trait Exhaustive: Sized + Copy {
    /// Number of values that may exist of this type.
    ///
    /// Use [`typenum::Unsigned`] to get an actual [`usize`] out of this
    /// type.
    ///
    /// # Examples
    ///
    /// ```
    /// use const_exhaustive::{Exhaustive, typenum::Unsigned};
    ///
    /// assert_eq!(1, <() as Exhaustive>::Num::USIZE);
    /// assert_eq!(2, <bool as Exhaustive>::Num::USIZE);
    /// ```
    type Num: ArrayLength<ArrayType<Self>: Copy>;

    /// All values of this type.
    ///
    /// # Order
    ///
    /// Values in this array are guaranteed to be in a stable order. The
    /// ordering of values is part of the public interface of this trait, so if
    /// an implementation changes the ordering between versions, this is
    /// considered a breaking change.
    ///
    /// Values should be ordered in a "binary counting" way, if it makes sense
    /// on this type. Some examples of this ordering are outlined below. All
    /// first-party implementations of [`Exhaustive`] follow this ordering.
    ///
    /// ## Primitives
    ///
    /// - [`Infallible`] has no values
    /// - [`()`]: `[ () ]`
    /// - [`bool`]: `[ false, true ]`
    ///
    /// ## Tuples
    ///
    /// ```
    /// # use const_exhaustive::Exhaustive;
    /// // in the same way that you count up in binary in this order:
    /// //   00, 01, 10, 11
    /// // we use a similar order for tuples
    ///
    /// assert_eq!(
    ///     [(false, false), (false, true), (true, false), (true, true)],
    ///     <(bool, bool)>::ALL.as_slice(),
    /// );
    /// ```
    ///
    /// ## Arrays
    ///
    /// ```
    /// # use const_exhaustive::Exhaustive;
    /// // `[T; N]` follows the same ordering as a tuple of `T`s with arity `N`:
    /// // - `[T; 2]` has the same ordering as `(T, T)`
    /// // - `[T; 3]` has the same ordering as `(T, T, T)`
    ///
    /// assert_eq!(
    ///     [[false, false], [false, true], [true, false], [true, true]],
    ///     <[bool; 2]>::ALL.as_slice(),
    /// );
    /// ```
    ///
    /// ## Derived on structs
    ///
    /// ```
    /// # use const_exhaustive::Exhaustive;
    /// // this has the exact same ordering as tuples
    /// #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    /// struct MyStruct(bool, bool);
    ///
    /// // this also has the same ordering:
    /// // struct MyStruct { a: bool, b: bool }
    ///
    /// assert_eq!(
    ///     [
    ///         MyStruct(false, false),
    ///         MyStruct(false, true),
    ///         MyStruct(true, false),
    ///         MyStruct(true, true),
    ///     ],
    ///     MyStruct::ALL.as_slice()
    /// );
    /// ```
    ///
    /// ## Derived on enums
    ///
    /// ```
    /// # use const_exhaustive::Exhaustive;
    /// #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    /// enum MyEnum {
    ///     A,
    ///     B(bool),
    ///     C,
    /// }
    ///
    /// assert_eq!(
    ///     [MyEnum::A, MyEnum::B(false), MyEnum::B(true), MyEnum::C],
    ///     MyEnum::ALL.as_slice()
    /// );
    /// ```
    const ALL: GenericArray<Self, Self::Num>;
}

unsafe impl Exhaustive for Infallible {
    type Num = U0;

    const ALL: GenericArray<Self, Self::Num> = GenericArray::from_array([]);
}

unsafe impl Exhaustive for () {
    type Num = U1;

    const ALL: GenericArray<Self, Self::Num> = GenericArray::from_array([()]);
}

unsafe impl Exhaustive for PhantomPinned {
    type Num = U1;

    const ALL: GenericArray<Self, Self::Num> = GenericArray::from_array([Self]);
}

unsafe impl<T: ?Sized> Exhaustive for PhantomData<T> {
    type Num = U1;

    const ALL: GenericArray<Self, Self::Num> = GenericArray::from_array([Self]);
}

unsafe impl Exhaustive for bool {
    type Num = U2;

    const ALL: GenericArray<Self, Self::Num> = GenericArray::from_array([false, true]);
}

unsafe impl<T: Exhaustive> Exhaustive for Option<T>
where
    U1: Add<T::Num, Output: ArrayLength<ArrayType<Self>: Copy>>,
{
    type Num = Sum<U1, T::Num>;

    const ALL: GenericArray<Self, Self::Num> =
        concat::<_, U1, T::Num>(GenericArray::from_array([None]), map!(T::ALL, |t| Some(t)));
}

unsafe impl<T: Exhaustive, E: Exhaustive> Exhaustive for Result<T, E>
where
    T::Num: Add<E::Num, Output: ArrayLength<ArrayType<Self>: Copy>>,
{
    type Num = Sum<T::Num, E::Num>;

    const ALL: GenericArray<Self, Self::Num> = concat::<_, T::Num, E::Num>(
        map!(T::ALL, |t| Ok::<T, E>(t)),
        map!(E::ALL, |t| Err::<T, E>(t)),
    );
}

unsafe impl<T: Exhaustive, const N: usize> Exhaustive for [T; N]
where
    Const<N>: ToUInt<Output: ArrayLength>,
    <T::Num as ArrayLength>::ArrayType<usize>: ConstDefault,
    T::Num: Pow<U<N>, Output: ArrayLength<ArrayType<Self>: Copy>>,
{
    type Num = <T::Num as Pow<U<N>>>::Output;

    const ALL: GenericArray<Self, Self::Num> = from_fn!(Self::Num, |i| {
        let perm: GenericArray<T, U<N>> = from_fn!(U<N>, |j| {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "we have no other way to cast in a const context"
            )]
            let index = (i / T::Num::USIZE.pow(N as u32 - j as u32 - 1)) % T::Num::USIZE;
            T::ALL.as_slice()[index]
        });
        perm
    });
}

// based on:
// https://discord.com/channels/273534239310479360/1120124565591425034/1288250308958486579
// https://discord.com/channels/273534239310479360/1120124565591425034/1288260177652617238
// https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=3932fdb89b5b8f4e757cb62b43023e01

type ProdAll<T> = <T as MulAll>::Output;

// must be `pub` since it is used in `Exhaustive::Num`
#[doc(hidden)]
pub trait MulAll {
    type Output: ArrayLength;
}

impl MulAll for () {
    type Output = U1;
}

impl<T: ArrayLength> MulAll for (T,) {
    type Output = T;
}

macro_rules! impl_variadic {
    ($(#[$meta:meta])* $(($T:ident, $t:ident)),*) => {
        $(#[$meta])*
        impl<$($T,)* Last> MulAll for ($($T,)* Last,)
        where
            ($($T,)*): MulAll,
            Last: Mul<<($($T,)*) as MulAll>::Output, Output: ArrayLength>,
        {
            type Output = <Last as Mul<<($($T,)*) as MulAll>::Output>>::Output;
        }

        $(#[$meta])*
        unsafe impl<$($T: Exhaustive,)*> Exhaustive for ($($T,)*)
        where
            ($($T::Num,)*): MulAll,
            <ProdAll<($($T::Num,)*)> as ArrayLength>::ArrayType<Self>: Copy,
        {
            type Num = ProdAll<($($T::Num,)*)>;

            const ALL: GenericArray<Self, Self::Num> = {
                let all: GenericArray<UnsafeCell<MaybeUninit<Self>>, Self::Num> =
                    unsafe { MaybeUninit::uninit().assume_init() };

                let mut i = 0;
                while i < <ProdAll<($($T::Num,)*)>>::USIZE {
                    let [$($t,)*] = split_index(i, [$($T::Num::USIZE,)*]);
                    let tuple = ($($T::ALL.as_slice()[$t],)*);
                    unsafe {
                        *all.as_slice()[i].get() = MaybeUninit::new(tuple);
                    }
                    i += 1;
                }

                unsafe { const_transmute(all) }
            };
        }
    };
}

all_tuples!(
    #[doc(fake_variadic)]
    impl_variadic,
    1,
    15,
    T,
    t
);

const fn split_index<const N: usize>(mut index: usize, lengths: [usize; N]) -> [usize; N] {
    let mut result = [0; N];
    let mut i = 0;
    while i < N {
        result[N - 1 - i] = index % lengths[N - 1 - i];
        index /= lengths[N - 1 - i];
        i += 1;
    }
    result
}
