//! Test

use {const_exhaustive::Exhaustive, std::convert::Infallible};

#[test]
fn primitives() {
    assert_eq!(0, Infallible::ALL.len());
    assert_eq!([()], <() as Exhaustive>::ALL.as_slice());
    assert_eq!([false, true], bool::ALL.as_slice());
}

#[test]
fn unit_struct() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct Unit;

    assert_eq!([Unit], Unit::ALL.as_slice());
}

#[test]
fn tuple_struct() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct TupleUnit(());

    assert_eq!([TupleUnit(())], TupleUnit::ALL.as_slice());

    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct TupleUnits((), (), ());

    assert_eq!([TupleUnits((), (), ())], TupleUnits::ALL.as_slice());

    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct TupleBool(bool);

    assert_eq!(
        [TupleBool(false), TupleBool(true)],
        TupleBool::ALL.as_slice()
    );

    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct TupleBools(bool, bool, bool);

    assert_eq!(
        [
            TupleBools(false, false, false),
            TupleBools(true, false, false),
            TupleBools(false, true, false),
            TupleBools(true, true, false),
            //
            TupleBools(false, false, true),
            TupleBools(true, false, true),
            TupleBools(false, true, true),
            TupleBools(true, true, true),
        ],
        TupleBools::ALL.as_slice()
    );
}

#[test]
fn normal_struct() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct Unit {}

    assert_eq!([Unit {}], Unit::ALL.as_slice());

    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct OneField {
        a: bool,
    }

    assert_eq!(
        [OneField { a: false }, OneField { a: true }],
        OneField::ALL.as_slice()
    );

    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct ManyFields {
        a: (),
        b: bool,
        c: bool,
    }

    assert_eq!(
        [
            ManyFields {
                a: (),
                b: false,
                c: false
            },
            ManyFields {
                a: (),
                b: true,
                c: false,
            },
            ManyFields {
                a: (),
                b: false,
                c: true
            },
            ManyFields {
                a: (),
                b: true,
                c: true
            }
        ],
        ManyFields::ALL.as_slice()
    );
}

#[test]
fn uninhabited() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    enum Uninhabited {}

    assert_eq!(0, Uninhabited::ALL.len());
}

#[test]
fn unit_enum() {
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
            Tuples::C(true, false),
            Tuples::C(false, true),
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
                foo: true,
                bar: false,
            },
            Fielded::C {
                foo: false,
                bar: true,
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

#[test]
fn generic() {
    // #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    // struct Wrapper<T>(T);
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
