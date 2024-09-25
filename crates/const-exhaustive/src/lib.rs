#![doc = include_str!("../../../README.md")]
#![no_std]

pub use {const_exhaustive_derive::Exhaustive, generic_array, typenum};
use {
    core::{convert::Infallible, ops::Mul},
    generic_array::{ArrayLength, GenericArray},
    typenum::{Prod, U0, U1, U2},
};

#[diagnostic::on_unimplemented(
    message = "`{Self}` is not `Exhaustive`",
    label = "not exhaustive",
    note = "consider annotating `{Self}` with `#[derive(Exhaustive)]`"
)]
pub unsafe trait Exhaustive: Sized + Copy + 'static {
    /// Number of values that may exist of this type.
    type Num: ArrayLength<ArrayType<Self>: Copy>;

    /// All values of this type.
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

unsafe impl Exhaustive for bool {
    type Num = U2;

    const ALL: GenericArray<Self, Self::Num> = GenericArray::from_array([false, true]);
}

// mod impls {
//     use {
//         super::*,
//         core::{cell::UnsafeCell, mem::MaybeUninit},
//         typenum::Unsigned,
//     };

//     impl<A: Exhaustive, B: Exhaustive> Exhaustive for (A, B)
//     where
//         A::Num: Mul<B::Num, Output: ArrayLength>,
//         <Prod<A::Num, B::Num> as ArrayLength>::ArrayType<Self>: Copy,
//     {
//         type Num = Prod<A::Num, B::Num>;

//         const ALL: GenericArray<Self, Self::Num> = {
//             use ::core::{cell::UnsafeCell, mem::MaybeUninit};

//             let array: GenericArray<UnsafeCell<MaybeUninit<Self>>, Self::Num>
// =                 unsafe { MaybeUninit::uninit().assume_init() };

//             let mut i0 = 0;
//             while i0 < A::Num::USIZE {
//                 let mut i1 = 0;
//                 while i1 < B::Num::USIZE {
//                     let i = (i0 * B::Num::USIZE) + i1;
//                     let ptr = array.as_slice()[i].get();
//                     unsafe {
//                         *ptr = MaybeUninit::new((A::ALL.as_slice()[i0],
// B::ALL.as_slice()[i1]));                     }
//                     i1 += 1;
//                 }
//                 i0 += 1;
//             }
//             unsafe { crate::__util::const_transmute(array) }
//         };
//     }
// }

#[doc(hidden)]
pub mod __util {
    use core::mem::{self, ManuallyDrop};

    pub const unsafe fn const_transmute<A, B>(a: A) -> B {
        #[repr(C)]
        union Union<A, B> {
            a: ManuallyDrop<A>,
            b: ManuallyDrop<B>,
        }

        if mem::size_of::<A>() != mem::size_of::<B>() {
            panic!("size mismatch for `const_transmute`");
        }

        let a = ManuallyDrop::new(a);
        ManuallyDrop::into_inner(Union { a }.b)
    }
}
