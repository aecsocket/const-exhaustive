#![expect(missing_docs, reason = "test module")]

#[test]
#[expect(
    dead_code,
    reason = "if we're getting dead code warnings, we've succeeded"
)]
const fn hygiene() {
    // try and cause as many ident conflicts as possible

    #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
    struct Exhaustive;
    #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
    struct GenericArray;
    #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
    struct Unsigned;
    #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
    struct Add;
    #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
    struct Mul;
    #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
    struct UnsafeCell;
    #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
    struct MaybeUninit;

    #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
    struct Struct {
        a: Exhaustive,
        b: GenericArray,
        c: Unsigned,
        d: Add,
        e: Mul,
        f: UnsafeCell,
        g: MaybeUninit,
    }

    #[derive(Debug, Clone, Copy, const_exhaustive::Exhaustive)]
    enum Enum {
        A(
            Exhaustive,
            GenericArray,
            Unsigned,
            Add,
            Mul,
            UnsafeCell,
            MaybeUninit,
        ),
        B {
            a: Exhaustive,
            b: GenericArray,
            c: Unsigned,
            d: Add,
            e: Mul,
            f: UnsafeCell,
            g: MaybeUninit,
        },
    }
}
