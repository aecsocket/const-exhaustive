#![doc = include_str!("../../../README.md")]
#![no_std]

pub use {const_exhaustive_derive::Exhaustive, generic_array, typenum};
use {
    core::{
        cell::UnsafeCell,
        convert::Infallible,
        marker::{PhantomData, PhantomPinned},
        mem::{self, ManuallyDrop, MaybeUninit},
        ops::Mul,
    },
    generic_array::{ArrayLength, GenericArray},
    typenum::{U0, U1, U2, Unsigned},
};

/// All values of this type are known at compile time.
///
/// This trait should be derived instead of implemented manually - see
/// [`const_exhaustive_derive::Exhaustive`].
///
/// If a type implements this trait, it guarantees that there is a finite set
/// of possible values which may exist for this type, and that they can be
/// enumerated at compile time.
///
/// By default, this is implemented for:
/// - [`Infallible`] with 0 values
/// - [`()`][unit], [`PhantomPinned`], [`PhantomData`] with 1 value
/// - [`bool`] with 2 values
///
/// This trait is not implemented for any numerical types. Although there are
/// practically a finite set of numbers for any given type (because they have to
/// fit in a finite number of bits, e.g. a [`u8`] must fit in 8 bits), there are
/// theoretically an infinite number of numbers, which goes against the
/// intention of this trait.
///
/// However, you may still want to define an exhaustive integer, where values
/// may only be in a specific range e.g. from 0 to 5. In this case, you can
/// either:
/// - define an enum with each value explicitly
/// - write a wrapper type which ensures that the value within it is always in
///   range, then implement [`Exhaustive`] on the wrapper
///
/// This trait is not possible to implement on more complex types such as
/// strings or collections, since they are inherently non-exhaustive.
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
/// # Safety
///
/// All possible values of this type must be present in [`Exhaustive::ALL`].
///
/// You should prefer deriving this trait instead of implementing it manually
/// where possible.
///
/// # Limitations
///
/// These are technically possible, but have not been implemented yet:
/// - `impl<T: Exhaustive, const N: usize> Exhaustive for [T; N]`
///   - complicated logic for generating each permutation
/// - deriving on a type with generics
///   - requires extra `where` bounds which are hard to create
///   - you can still technically do this, but requires more explicit `where`
///     bounds
///
/// PRs welcome!
#[diagnostic::on_unimplemented(
    message = "not all values of `{Self}` are known statically",
    label = "not exhaustive",
    note = "consider annotating `{Self}` with `#[derive(Exhaustive)]`"
)]
pub unsafe trait Exhaustive: Sized + Copy + 'static {
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
    /// Values in this array are guaranteed to be in a specific order, similar
    /// in concept to binary counting. Some examples:
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

unsafe impl<T: ?Sized + 'static> Exhaustive for PhantomData<T> {
    type Num = U1;

    const ALL: GenericArray<Self, Self::Num> = GenericArray::from_array([Self]);
}

unsafe impl Exhaustive for bool {
    type Num = U2;

    const ALL: GenericArray<Self, Self::Num> = GenericArray::from_array([false, true]);
}

// based on:
// https://discord.com/channels/273534239310479360/1120124565591425034/1288250308958486579
// https://discord.com/channels/273534239310479360/1120124565591425034/1288260177652617238
// https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=3932fdb89b5b8f4e757cb62b43023e01

// must be `pub` since it is used by the derive macro
// we can't just use `core::mem::transmute` because of <https://github.com/rust-lang/rust/issues/61956>
#[doc(hidden)]
pub const unsafe fn const_transmute<A, B>(a: A) -> B {
    #[repr(C)]
    union Union<A, B> {
        a: ManuallyDrop<A>,
        b: ManuallyDrop<B>,
    }

    assert!(
        mem::size_of::<A>() == mem::size_of::<B>(),
        "size mismatch for `const_transmute`"
    );

    let a = ManuallyDrop::new(a);
    ManuallyDrop::into_inner(Union { a }.b)
}

/*
// TODO
unsafe impl<T: Exhaustive, const N: usize> Exhaustive for [T; N]
where
    Const<N>: ToUInt,
    T::Num: Pow<U<N>, Output: ArrayLength>,
    <<T::Num as Pow<U<N>>>::Output as ArrayLength>::ArrayType<Self>: Copy,
{
    type Num = <T::Num as Pow<U<N>>>::Output;

    const ALL: GenericArray<Self, Self::Num> = {
        let all: GenericArray<UnsafeCell<MaybeUninit<Self>>, Self::Num> =
            unsafe { MaybeUninit::uninit().assume_init() };

        todo!();

        unsafe { const_transmute(all) }
    };
}
*/

type ProdAll<T> = <T as MulAll>::Output;

// must be `pub` since it is used in `Exhaustive::Num`
#[doc(hidden)]
pub trait MulAll {
    type Output: ArrayLength;
}

impl MulAll for () {
    type Output = typenum::U1;
}

impl<T: ArrayLength> MulAll for (T,) {
    type Output = T;
}

macro_rules! impl_for_tuples {
    ($($($t:ident)*,)*) => { $(
        impl<$($t,)* Last> MulAll for ($($t,)* Last,)
        where
            ($($t,)*): MulAll,
            Last: Mul<<($($t,)*) as MulAll>::Output>,
            <Last as Mul<<($($t,)*) as MulAll>::Output>>::Output: ArrayLength,
        {
            type Output = <Last as Mul<<($($t,)*) as MulAll>::Output>>::Output;
        }

        unsafe impl<$($t: Exhaustive,)*> Exhaustive for ($($t,)*)
        where
            ($($t::Num,)*): MulAll,
            <ProdAll<($($t::Num,)*)> as ArrayLength>::ArrayType<Self>: Copy,
        {
            type Num = ProdAll<($($t::Num,)*)>;

            const ALL: GenericArray<Self, Self::Num> = {
                let all: GenericArray<UnsafeCell<MaybeUninit<Self>>, Self::Num> =
                    unsafe { MaybeUninit::uninit().assume_init() };

                let mut i = 0;
                while i < <ProdAll<($($t::Num,)*)>>::USIZE {
                    #[expect(nonstandard_style, reason = "uppercase variable name")]
                    let [$($t,)*] = split_index(i, [$($t::Num::USIZE,)*]);
                    let tuple = ($($t::ALL.as_slice()[$t],)*);
                    unsafe { *all.as_slice()[i].get() = MaybeUninit::new(tuple) }
                    i += 1;
                }

                unsafe { const_transmute(all) }
            };
        }
    )* }
}

impl_for_tuples! {
                   A,
                  A B,
                 A B C,
                A B C D,

               A B C D E,
              A B C D E F,
             A B C D E F G,
            A B C D E F G H,

           A B C D E F G H I,
          A B C D E F G H I J,
         A B C D E F G H I J K,
        A B C D E F G H I J K L,

       A B C D E F G H I J K L M,
      A B C D E F G H I J K L M N,
     A B C D E F G H I J K L M N O,
    A B C D E F G H I J K L M N O P,
}

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
