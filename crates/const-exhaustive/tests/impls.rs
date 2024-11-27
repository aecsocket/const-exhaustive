#![expect(missing_docs, reason = "test module")]

use {
    const_exhaustive::Exhaustive,
    core::{
        any::Any,
        convert::Infallible,
        marker::{PhantomData, PhantomPinned},
    },
};

#[track_caller]
fn assert_all<T: Exhaustive + core::fmt::Debug + PartialEq>(values: impl IntoIterator<Item = T>) {
    let values = values.into_iter().collect::<Vec<_>>();
    assert_eq!(values.as_slice(), T::ALL.as_slice());
}

#[test]
fn uninhabited() {
    assert_all::<Infallible>([]);
}

#[test]
fn unit() {
    assert_all([()]);
}

#[test]
fn phantom_pinned() {
    assert_all([PhantomPinned]);
}

#[test]
fn phantom_data() {
    fn with_non_static_lifetime<'a>(_: &'a str) {
        assert_all([PhantomData::<&'a str>]);
    }

    assert_all([PhantomData::<u8>]); // sized
    assert_all([PhantomData::<dyn Any>]); // unsized

    let x = String::new();
    with_non_static_lifetime(&x);
}

#[test]
fn bools() {
    assert_all([false, true]);
}

#[test]
fn options() {
    assert_all([None::<Infallible>]);
    assert_all([None, Some(())]);
    assert_all([None, Some(false), Some(true)]);
}

#[test]
fn results() {
    assert_all::<Result<Infallible, Infallible>>([]);
    assert_all([Ok::<_, Infallible>(())]);
    assert_all([Ok::<_, Infallible>(false), Ok(true)]);
    assert_all([Err::<Infallible, _>(())]);
    assert_all([Err::<Infallible, _>(false), Err(true)]);
    assert_all([Ok(()), Err(())]);
    assert_all([Ok(false), Ok(true), Err(())]);
    assert_all([Ok(()), Err(false), Err(true)]);
    assert_all([Ok(false), Ok(true), Err(false), Err(true)]);
}

#[test]
fn arrays() {
    // assert_all::<[Infallible; 0]>([]);
    // assert_all::<[Infallible; 1]>([]);
    // assert_all::<[Infallible; 2]>([]);
    // assert_all::<[(); 0]>([[]]);

    // TODO
    // assert_all([[false, false], [false, true], [true, false], [true, true]]);

    // TODO
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
