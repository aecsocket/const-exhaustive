//! Test

use {
    const_exhaustive::Exhaustive,
    generic_array::GenericArray,
    typenum::{Prod, Sum, U3},
};

#[derive(Debug, Clone, Copy)]
enum TriState {
    A,
    B,
    C,
}

unsafe impl Exhaustive for TriState {
    type Num = U3;

    const ALL: GenericArray<Self, Self::Num> =
        GenericArray::from_array([Self::A, Self::B, Self::C]);
}

#[derive(Debug, Clone, Copy)]
struct StructNamedFields {
    a: bool,
    b: bool,
    c: TriState,
}

unsafe impl Exhaustive for StructNamedFields {
    type Num = Prod<
        Prod<<bool as Exhaustive>::Num, <bool as Exhaustive>::Num>,
        <TriState as Exhaustive>::Num,
    >;

    const ALL: GenericArray<Self, Self::Num> = {
        use {
            ::const_exhaustive::{
                __util::const_transmute, generic_array::GenericArray, typenum::Unsigned,
            },
            ::core::{cell::UnsafeCell, mem::MaybeUninit},
        };

        let all: GenericArray<UnsafeCell<MaybeUninit<Self>>, Self::Num> =
            unsafe { MaybeUninit::uninit().assume_init() };

        let mut i = 0;
        let mut i_a = 0;
        while i_a < <bool as Exhaustive>::Num::USIZE {
            let mut i_b = 0;
            while i_b < <bool as Exhaustive>::Num::USIZE {
                let mut i_c = 0;
                while i_c < <TriState as Exhaustive>::Num::USIZE {
                    let ptr = all.as_slice()[i].get();
                    unsafe {
                        *ptr = MaybeUninit::new(Self {
                            a: <bool as Exhaustive>::ALL.as_slice()[i_a],
                            b: <bool as Exhaustive>::ALL.as_slice()[i_b],
                            c: <TriState as Exhaustive>::ALL.as_slice()[i_c],
                        });
                    }
                    i += 1;

                    i_c += 1;
                }
                i_b += 1;
            }
            i_a += 1;
        }

        unsafe { const_transmute(all) }
    };
}

#[derive(Debug, Clone, Copy)]
struct StructUnnamedFields((), bool, TriState);

unsafe impl Exhaustive for StructUnnamedFields {
    type Num = Prod<
        Prod<<() as Exhaustive>::Num, <bool as Exhaustive>::Num>,
        <TriState as Exhaustive>::Num,
    >;

    const ALL: GenericArray<Self, Self::Num> = {
        use {
            ::const_exhaustive::{
                __util::const_transmute, generic_array::GenericArray, typenum::Unsigned,
            },
            ::core::{cell::UnsafeCell, mem::MaybeUninit},
        };

        let all: GenericArray<UnsafeCell<MaybeUninit<Self>>, Self::Num> =
            unsafe { MaybeUninit::uninit().assume_init() };

        let mut i = 0;
        let mut i_0 = 0;
        while i_0 < <() as Exhaustive>::Num::USIZE {
            let mut i_1 = 0;
            while i_1 < <bool as Exhaustive>::Num::USIZE {
                let mut i_2 = 0;
                while i_2 < <TriState as Exhaustive>::Num::USIZE {
                    let ptr = all.as_slice()[i].get();
                    unsafe {
                        *ptr = MaybeUninit::new(Self(
                            <() as Exhaustive>::ALL.as_slice()[i_0],
                            <bool as Exhaustive>::ALL.as_slice()[i_1],
                            <TriState as Exhaustive>::ALL.as_slice()[i_2],
                        ));
                    }
                    i += 1;

                    i_2 += 1;
                }
                i_1 += 1;
            }
            i_0 += 1;
        }

        unsafe { const_transmute(all) }
    };
}

#[derive(Debug, Clone, Copy)]
enum EnumFields {
    Foo(bool, bool),
    Bar(TriState),
    Baz { hi: bool, yo: bool },
}

unsafe impl Exhaustive for EnumFields {
    type Num = Sum<
        Sum<
            Prod<<bool as Exhaustive>::Num, <bool as Exhaustive>::Num>,
            <TriState as Exhaustive>::Num,
        >,
        Prod<<bool as Exhaustive>::Num, <bool as Exhaustive>::Num>,
    >;

    const ALL: GenericArray<Self, Self::Num> = {
        use {
            ::const_exhaustive::{
                __util::const_transmute, generic_array::GenericArray, typenum::Unsigned,
            },
            ::core::{cell::UnsafeCell, mem::MaybeUninit},
        };

        let all: GenericArray<UnsafeCell<MaybeUninit<Self>>, Self::Num> =
            unsafe { MaybeUninit::uninit().assume_init() };

        let mut i = 0;

        // Foo
        let mut i_0 = 0;
        while i_0 < <bool as Exhaustive>::Num::USIZE {
            let mut i_1 = 0;
            while i_1 < <bool as Exhaustive>::Num::USIZE {
                let ptr = all.as_slice()[i].get();
                unsafe {
                    *ptr = MaybeUninit::new(Self::Foo(
                        <bool as Exhaustive>::ALL.as_slice()[i_0],
                        <bool as Exhaustive>::ALL.as_slice()[i_1],
                    ));
                }
                i += 1;

                i_1 += 1;
            }
            i_0 += 1;
        }

        // Bar
        let mut i_0 = 0;
        while i_0 < <TriState as Exhaustive>::Num::USIZE {
            let ptr = all.as_slice()[i].get();
            unsafe {
                *ptr = MaybeUninit::new(Self::Bar(<TriState as Exhaustive>::ALL.as_slice()[i_0]));
            }
            i += 1;

            i_0 += 1;
        }

        // Baz
        let mut i_hi = 0;
        while i_hi < <bool as Exhaustive>::Num::USIZE {
            let mut i_yo = 0;
            while i_yo < <bool as Exhaustive>::Num::USIZE {
                let ptr = all.as_slice()[i].get();
                unsafe {
                    *ptr = MaybeUninit::new(Self::Baz {
                        hi: <bool as Exhaustive>::ALL.as_slice()[i_hi],
                        yo: <bool as Exhaustive>::ALL.as_slice()[i_yo],
                    });
                }
                i += 1;

                i_yo += 1;
            }
            i_hi += 1;
        }

        unsafe { const_transmute(all) }
    };
}

#[derive(Debug, Clone, Copy, Exhaustive)]
struct Testing {
    a: bool,
}

#[test]
fn foo() {
    dbg!(StructNamedFields::ALL);
    dbg!(StructUnnamedFields::ALL);
    dbg!(EnumFields::ALL);
    dbg!(Testing::ALL);
}
