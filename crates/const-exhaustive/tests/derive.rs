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
fn enum_unit_single() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    enum Dummy {
        A,
    }

    assert_eq!([Dummy::A], Dummy::ALL.as_slice());
}

#[test]
fn enum_unit_many() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    enum Dummy {
        A,
        B,
        C,
    }

    assert_eq!([Dummy::A, Dummy::B, Dummy::C], Dummy::ALL.as_slice());
}

#[test]
fn enum_tuple_variants() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    enum Dummy {
        A(),
        B(()),
        C(bool, bool),
    }

    assert_eq!(
        [
            Dummy::A(),
            Dummy::B(()),
            Dummy::C(false, false),
            Dummy::C(false, true),
            Dummy::C(true, false),
            Dummy::C(true, true),
        ],
        Dummy::ALL.as_slice()
    );
}

#[test]
fn enum_named_field_variants() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    enum Dummy {
        A {},
        B { foo: () },
        C { foo: bool, bar: bool },
    }

    assert_eq!(
        [
            Dummy::A {},
            Dummy::B { foo: () },
            Dummy::C {
                foo: false,
                bar: false,
            },
            Dummy::C {
                foo: false,
                bar: true,
            },
            Dummy::C {
                foo: true,
                bar: false,
            },
            Dummy::C {
                foo: true,
                bar: true,
            },
        ],
        Dummy::ALL.as_slice()
    );
}

#[test]
fn enum_variant_mix() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    enum Dummy {
        Unit,
        Tuple(bool),
        Named { foo: bool },
    }

    assert_eq!(
        [
            Dummy::Unit,
            Dummy::Tuple(false),
            Dummy::Tuple(true),
            Dummy::Named { foo: false },
            Dummy::Named { foo: true },
        ],
        Dummy::ALL.as_slice()
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
