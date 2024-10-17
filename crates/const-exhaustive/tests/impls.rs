#![expect(missing_docs, reason = "test module")]

use {
    const_exhaustive::Exhaustive,
    core::convert::Infallible,
    std::{
        any::Any,
        marker::{PhantomData, PhantomPinned},
    },
};

#[test]
fn uninhabited() {
    let infallibles: &[Infallible] = &[];
    assert_eq!(infallibles, Infallible::ALL.as_slice());
}

#[test]
fn unit() {
    assert_eq!([()], <()>::ALL.as_slice());
}

#[test]
fn phantom_pinned() {
    assert_eq!([PhantomPinned], PhantomPinned::ALL.as_slice());
}

#[test]
fn phantom_data() {
    struct WithLifetime<'a> {
        _value: &'a i32,
    }

    assert_eq!([PhantomData], PhantomData::<i32>::ALL.as_slice()); // sized
    assert_eq!([PhantomData], PhantomData::<dyn Any>::ALL.as_slice()); // unsized
    assert_eq!(
        [PhantomData],
        PhantomData::<WithLifetime<'static>>::ALL.as_slice()
    );
}

#[test]
fn bools() {
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
