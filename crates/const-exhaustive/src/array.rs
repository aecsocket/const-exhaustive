use {
    core::{
        mem::{forget, MaybeUninit},
        ops::Add,
    },
    generic_array::{ArrayLength, GenericArray},
    typenum::Sum,
};

macro_rules! from_fn {
    ($len:ty, | $i:pat_param | $f:expr $(,)?) => {{
        use {::core::mem::MaybeUninit, $crate::generic_array::const_transmute};

        let mut dst: GenericArray<MaybeUninit<_>, $len> =
            unsafe { MaybeUninit::uninit().assume_init() };

        let mut i = 0;
        while i < dst.as_slice().len() {
            let $i = i;
            let value = $f;
            dst.as_mut_slice()[i] = MaybeUninit::new(value);
            i += 1;
        }

        unsafe { const_transmute(dst) }
    }};
}

macro_rules! map {
    ($src:expr, | $in:pat_param | $f:expr $(,)?) => {{
        use {
            ::core::mem::MaybeUninit,
            $crate::{array::assert_same_length, generic_array::const_transmute},
        };

        let mut dst = assert_same_length::<_, MaybeUninit<_>, _>(&$src, unsafe {
            MaybeUninit::uninit().assume_init()
        });

        let mut i = 0;
        while i < $src.as_slice().len() {
            let $in = $src.as_slice()[i];
            let output = $f;
            dst.as_mut_slice()[i] = MaybeUninit::new(output);
            i += 1;
        }

        unsafe { const_transmute(dst) }
    }};
}

pub(crate) use {from_fn, map};

pub const fn assert_same_length<A, B, N: ArrayLength>(
    _: &GenericArray<A, N>,
    b: GenericArray<B, N>,
) -> GenericArray<B, N> {
    b
}

#[must_use]
pub const fn concat<T, NA, NB>(
    src_a: GenericArray<T, NA>,
    src_b: GenericArray<T, NB>,
) -> GenericArray<T, Sum<NA, NB>>
where
    T: Copy,
    NA: ArrayLength + Add<NB, Output: ArrayLength>,
    NB: ArrayLength,
{
    let out = concat_copy(&src_a, &src_b);
    // ok because `T: Copy`
    forget((src_a, src_b));
    out
}

#[must_use]
pub const fn concat_copy<T, NA, NB>(
    src_a: &GenericArray<T, NA>,
    src_b: &GenericArray<T, NB>,
) -> GenericArray<T, Sum<NA, NB>>
where
    T: Copy,
    NA: ArrayLength + Add<NB, Output: ArrayLength>,
    NB: ArrayLength,
{
    let mut dst: GenericArray<MaybeUninit<T>, Sum<NA, NB>> = unsafe {
        #[expect(clippy::uninit_assumed_init, reason = "same layout as an array")]
        MaybeUninit::uninit().assume_init()
    };

    let mut i_a = 0;
    while i_a < src_a.as_slice().len() {
        let value = src_a.as_slice()[i_a];
        dst.as_mut_slice()[i_a] = MaybeUninit::new(value);
        i_a += 1;
    }

    let mut i_b = 0;
    while i_b < src_b.as_slice().len() {
        let value = src_b.as_slice()[i_b];
        dst.as_mut_slice()[i_a + i_b] = MaybeUninit::new(value);
        i_b += 1;
    }

    unsafe { generic_array::const_transmute(dst) }
}
