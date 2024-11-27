#![expect(missing_docs, reason = "test module")]

use const_exhaustive::Exhaustive;

fn assert_all<T: Exhaustive + core::fmt::Debug + PartialEq>(values: impl IntoIterator<Item = T>) {
    let values = values.into_iter().collect::<Vec<_>>();
    assert_eq!(values.as_slice(), T::ALL.as_slice());
}

#[test]
fn unit_struct() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct Dummy;

    assert_all([Dummy]);
}

#[test]
fn tuple_struct_unit_single() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct Dummy(());

    assert_all([Dummy(())]);
}

#[test]
fn tuple_struct_unit_many() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct Dummy((), (), ());

    assert_all([Dummy((), (), ())]);
}

#[test]
fn tuple_struct_bool_single() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct Dummy(bool);

    assert_all([Dummy(false), Dummy(true)]);
}

#[test]
fn tuple_struct_bool_many() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct Dummy(bool, bool, bool);

    assert_all([
        Dummy(false, false, false),
        Dummy(false, false, true),
        Dummy(false, true, false),
        Dummy(false, true, true),
        Dummy(true, false, false),
        Dummy(true, false, true),
        Dummy(true, true, false),
        Dummy(true, true, true),
    ]);
}

#[test]
fn named_field_struct_empty() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct Dummy {}

    assert_all([Dummy {}]);
}

#[test]
fn named_field_struct_small() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct Dummy {
        a: bool,
    }

    assert_all([Dummy { a: false }, Dummy { a: true }]);
}

#[test]
fn named_field_struct_large() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct Dummy {
        a: (),
        b: bool,
        c: bool,
    }

    assert_all([
        Dummy {
            a: (),
            b: false,
            c: false,
        },
        Dummy {
            a: (),
            b: false,
            c: true,
        },
        Dummy {
            a: (),
            b: true,
            c: false,
        },
        Dummy {
            a: (),
            b: true,
            c: true,
        },
    ]);
}

#[test]
fn enum_uninhabited() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    enum Dummy {}

    assert_all::<Dummy>([]);
}

#[test]
fn enum_unit() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    enum Unit {
        A,
    }

    assert_eq!([Unit::A], Unit::ALL.as_slice());

    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    enum Units {
        A,
        B,
        C,
    }

    assert_eq!([Units::A, Units::B, Units::C], Units::ALL.as_slice());
}

#[test]
fn tuple_variants() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    enum Tuples {
        A(),
        B(()),
        C(bool, bool),
    }

    assert_eq!(
        [
            Tuples::A(),
            Tuples::B(()),
            Tuples::C(false, false),
            Tuples::C(false, true),
            Tuples::C(true, false),
            Tuples::C(true, true),
        ],
        Tuples::ALL.as_slice()
    );
}

#[test]
fn fielded_variants() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    enum Fielded {
        A {},
        B { foo: () },
        C { foo: bool, bar: bool },
    }

    assert_eq!(
        [
            Fielded::A {},
            Fielded::B { foo: () },
            Fielded::C {
                foo: false,
                bar: false,
            },
            Fielded::C {
                foo: false,
                bar: true,
            },
            Fielded::C {
                foo: true,
                bar: false,
            },
            Fielded::C {
                foo: true,
                bar: true,
            },
        ],
        Fielded::ALL.as_slice()
    );
}

#[test]
fn variant_mix() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    enum Mix {
        Unit,
        Tuple(bool),
        Named { foo: bool },
    }

    assert_eq!(
        [
            Mix::Unit,
            Mix::Tuple(false),
            Mix::Tuple(true),
            Mix::Named { foo: false },
            Mix::Named { foo: true },
        ],
        Mix::ALL.as_slice()
    );
}

#[test]
fn compound() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct BoolWrapper(bool);

    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    enum Compound {
        A(bool),
        B(BoolWrapper),
    }

    assert_eq!(
        [
            Compound::A(false),
            Compound::A(true),
            Compound::B(BoolWrapper(false)),
            Compound::B(BoolWrapper(true)),
        ],
        Compound::ALL.as_slice()
    );
}

mod hygiene {
    #[test]
    #[expect(
        dead_code,
        reason = "if we're getting dead code warnings, we've succeeded"
    )]
    fn hygiene() {
        // try and cause as many ident conflicts as possible

        #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
        struct Exhaustive;
        #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
        struct GenericArray;
        #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
        struct Unsigned;
        #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
        struct Sum;
        #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
        struct Prod;
        #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
        struct UnsafeCell;
        #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
        struct MaybeUninit;

        #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
        struct Struct {
            a: Exhaustive,
            b: GenericArray,
            c: Unsigned,
            d: Sum,
            e: Prod,
            f: UnsafeCell,
            g: MaybeUninit,
        }

        #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
        enum Enum {
            A(
                Exhaustive,
                GenericArray,
                Unsigned,
                Sum,
                Prod,
                UnsafeCell,
                MaybeUninit,
            ),
            B {
                a: Exhaustive,
                b: GenericArray,
                c: Unsigned,
                d: Sum,
                e: Prod,
                f: UnsafeCell,
                g: MaybeUninit,
            },
        }
    }
}
