use {
    core::{
        cell::UnsafeCell,
        mem::{MaybeUninit, forget},
        ops::Add,
    },
    generic_array::{ArrayLength, GenericArray},
    typenum::Sum,
};

#[macro_export]
#[doc(hidden)]
macro_rules! map {
    ($src:expr, | $in:pat_param | -> $T:ty { $expr:expr } $(,)?) => {{
        use {
            ::core::{cell::UnsafeCell, mem::MaybeUninit},
            $crate::{array::assert_same_length, generic_array::const_transmute},
        };

        let dst = assert_same_length::<_, UnsafeCell<MaybeUninit<$T>>, _>(&$src, unsafe {
            MaybeUninit::uninit().assume_init()
        });

        let mut i = 0;
        while i < $src.as_slice().len() {
            let $in = $src.as_slice()[i];
            let output = $expr;
            unsafe {
                *dst.as_slice()[i].get() = MaybeUninit::new(output);
            }
            i += 1;
        }

        unsafe { const_transmute(dst) }
    }};
}

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
    let dst: GenericArray<UnsafeCell<MaybeUninit<T>>, Sum<NA, NB>> = unsafe {
        #[expect(clippy::uninit_assumed_init, reason = "same layout as an array")]
        MaybeUninit::uninit().assume_init()
    };

    let mut i_a = 0;
    while i_a < src_a.as_slice().len() {
        let value = src_a.as_slice()[i_a];
        unsafe {
            *dst.as_slice()[i_a].get() = MaybeUninit::new(value);
        }
        i_a += 1;
    }

    let mut i_b = 0;
    while i_b < src_b.as_slice().len() {
        let value = src_b.as_slice()[i_b];
        unsafe {
            *dst.as_slice()[i_a + i_b].get() = MaybeUninit::new(value);
        }
        i_b += 1;
    }

    unsafe { generic_array::const_transmute(dst) }
}
