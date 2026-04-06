#![expect(missing_docs, reason = "test module")]

use {
    const_exhaustive::Exhaustive,
    core::{fmt::Debug, marker::PhantomData},
};

fn assert_all<T: Exhaustive + Debug + PartialEq>(values: impl IntoIterator<Item = T>) {
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
        EndUnit,
    }

    assert_eq!(
        [
            Dummy::Unit,
            Dummy::Tuple(false),
            Dummy::Tuple(true),
            Dummy::Named { foo: false },
            Dummy::Named { foo: true },
            Dummy::EndUnit,
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

#[test]
#[expect(clippy::items_after_statements, reason = "easier to read")]
fn generic() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct Wrapper1<T>(T);
    assert_eq!(Wrapper1::<()>::ALL.as_slice(), [Wrapper1(())]);
    assert_eq!(
        Wrapper1::<bool>::ALL.as_slice(),
        [Wrapper1(false), Wrapper1(true)],
    );

    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct Wrapper2<A, B>(A, B);
    assert_eq!(Wrapper2::<(), ()>::ALL.as_slice(), [Wrapper2((), ())]);
    assert_eq!(
        Wrapper2::<(), bool>::ALL.as_slice(),
        [Wrapper2((), false), Wrapper2((), true)],
    );
    assert_eq!(
        Wrapper2::<bool, bool>::ALL.as_slice(),
        [
            Wrapper2(false, false),
            Wrapper2(false, true),
            Wrapper2(true, false),
            Wrapper2(true, true)
        ],
    );

    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct WrapperNamed<A, B> {
        a: A,
        b: B,
    }
    assert_eq!(
        WrapperNamed::<(), ()>::ALL.as_slice(),
        [WrapperNamed { a: (), b: () }]
    );
    assert_eq!(
        WrapperNamed::<(), bool>::ALL.as_slice(),
        [
            WrapperNamed { a: (), b: false },
            WrapperNamed { a: (), b: true }
        ]
    );
    assert_eq!(
        WrapperNamed::<bool, bool>::ALL.as_slice(),
        [
            WrapperNamed { a: false, b: false },
            WrapperNamed { a: false, b: true },
            WrapperNamed { a: true, b: false },
            WrapperNamed { a: true, b: true }
        ],
    );

    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct WrapperOption<A, B> {
        a: A,
        b: Option<B>,
    }
    assert_eq!(
        WrapperOption::<(), ()>::ALL.as_slice(),
        [
            WrapperOption { a: (), b: None },
            WrapperOption { a: (), b: Some(()) }
        ]
    );
    assert_eq!(
        WrapperOption::<(), bool>::ALL.as_slice(),
        [
            WrapperOption { a: (), b: None },
            WrapperOption {
                a: (),
                b: Some(false)
            },
            WrapperOption {
                a: (),
                b: Some(true)
            },
        ]
    );
    assert_eq!(
        WrapperOption::<bool, bool>::ALL.as_slice(),
        [
            WrapperOption { a: false, b: None },
            WrapperOption {
                a: false,
                b: Some(false)
            },
            WrapperOption {
                a: false,
                b: Some(true)
            },
            WrapperOption { a: true, b: None },
            WrapperOption {
                a: true,
                b: Some(false)
            },
            WrapperOption {
                a: true,
                b: Some(true)
            }
        ],
    );

    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct WrapperPhantom<T> {
        _phantom: PhantomData<T>,
    }
    assert_eq!(
        WrapperPhantom::<()>::ALL.as_slice(),
        [WrapperPhantom {
            _phantom: PhantomData
        }],
    );
    assert_eq!(
        WrapperPhantom::<bool>::ALL.as_slice(),
        [WrapperPhantom {
            _phantom: PhantomData
        }],
    );

    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    enum WrapperEnum1<T> {
        T(T),
    }
    assert_eq!(WrapperEnum1::<()>::ALL.as_slice(), [WrapperEnum1::T(())]);
    assert_eq!(
        WrapperEnum1::<bool>::ALL.as_slice(),
        [WrapperEnum1::T(false), WrapperEnum1::T(true)]
    );

    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    enum WrapperEnum2<T> {
        A { a: T },
        B(T),
        C((T, T)),
        D,
    }
    assert_eq!(
        WrapperEnum2::<()>::ALL.as_slice(),
        [
            WrapperEnum2::A { a: () },
            WrapperEnum2::B(()),
            WrapperEnum2::C(((), ())),
            WrapperEnum2::D
        ]
    );
}
