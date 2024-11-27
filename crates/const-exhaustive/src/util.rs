use {
    core::{cell::UnsafeCell, mem::MaybeUninit},
    generic_array::{ArrayLength, GenericArray},
    typenum::{Sum, U3, U6},
};

macro_rules! arr_map {
    ($src:expr, | $in:pat_param | -> $T:ty { $expr:expr }) => {{
        const fn map<N: ArrayLength>(src: &GenericArray<$T, N>) -> GenericArray<$T, N> {
            let __dst: GenericArray<UnsafeCell<MaybeUninit<$T>>, N> =
                unsafe { MaybeUninit::uninit().assume_init() };

            let mut __i = 0;
            while __i < src.as_slice().len() {
                let $in = src.as_slice()[__i];
                let output: $T = $expr;
                unsafe {
                    *__dst.as_slice()[__i].get() = MaybeUninit::new(output);
                }
                __i += 1;
            }

            unsafe { generic_array::const_transmute(__dst) }
        }

        map(&$src)
    }};
}

// macro_rules! arr_append {
//     ($src_a:expr, $len_a:ty, $src_b:expr, $len_b:ty) => {{
//         let dst: GenericArray<UnsafeCell<MaybeUninit<_>>, Sum<$len_a,
// $len_b>> =             unsafe { MaybeUninit::uninit().assume_init() };

//         let mut i_a = 0;
//         while i_a < $src_a.as_slice().len() {
//             let value = $src_a.as_slice()[i_a];
//             unsafe {
//                 *dst.as_slice()[i_a].get() = MaybeUninit::new(value);
//             }
//             i_a += 1;
//         }

//         let mut i_b = 0;
//         while i_b < $src_b.as_slice().len() {
//             let value = $src_a.as_slice()[i_b];
//             unsafe {
//                 *dst.as_slice()[i_a + i_b].get() = MaybeUninit::new(value);
//             }
//             i_b += 1;
//         }

//         unsafe { generic_array::const_transmute(dst) }
//     }};
// }

#[test]
pub fn x() {
    const ARR: GenericArray<usize, U3> = {
        let arr = GenericArray::from_array([0, 1, 2]);
        arr_map!(arr, |x| -> usize { x * 2 })
    };

    // const ARR2: GenericArray<usize, U6> = {
    //     let a = GenericArray::from_array([0, 1, 2]);
    //     let b = GenericArray::from_array([3, 4, 5]);
    //     arr_append!(a, U3, b, U3)
    // };
}
