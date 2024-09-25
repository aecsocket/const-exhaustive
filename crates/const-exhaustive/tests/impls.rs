//!

use {const_exhaustive::Exhaustive, core::convert::Infallible};

#[test]
fn primitives() {
    assert_eq!(0, Infallible::ALL.len());
    assert_eq!([()], <()>::ALL.as_slice());
    assert_eq!([false, true], bool::ALL.as_slice());
}

#[test]
fn unit_struct() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct Unit;

    assert_eq!([Unit], Unit::ALL.as_slice());
}

#[test]
fn tuples() {
    assert_eq!(
        [(false, false), (false, true), (true, false), (true, true)],
        <(bool, bool)>::ALL.as_slice()
    );

    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    enum Foo {
        A,
        B,
        C,
    }

    assert_eq!(
        [
            (false, Foo::A),
            (false, Foo::B),
            (false, Foo::C),
            (true, Foo::A),
            (true, Foo::B),
            (true, Foo::C),
        ],
        <(bool, Foo)>::ALL.as_slice()
    );
}

/*
#[test]
fn generic() {
    #[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
    struct Wrapper<T>(T);
}
 */
