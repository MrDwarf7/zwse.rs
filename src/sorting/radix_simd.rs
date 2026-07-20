use std::ops::Div;
use std::simd::Simd;
use std::simd::prelude::SimdUint;

use crate::extract::ExtractedData;

pub fn radix_sort_simd(arr: &mut [ExtractedData]) {
    const RADIX: usize = 256;
    let max_val = arr.iter().max().cloned().unwrap_or_default().id;
    let mut exp = 1;
    while max_val / exp > 0 {
        counting_sort_simd(arr, exp as u32, RADIX);
        exp *= RADIX;
    }
}

pub fn counting_sort_simd(arr: &mut [ExtractedData], exp: u32, radix: usize) {
    let n = arr.len();
    let mut output = vec![ExtractedData::default(); n];
    let mut count = vec![0usize; radix];

    const LANES: usize = 8;
    let mut i = 0;
    while i + LANES <= n {
        let vals: Simd<_, LANES> =
            Simd::from_slice(&arr[i..i + LANES].iter().map(|x| x.id).collect::<Vec<_>>());
        let exp_simd = Simd::splat(exp);
        let radix_simd = Simd::splat(radix);

        // let digits = vals / exp_simd % radix_simd;
        let digits = (vals / exp_simd) % radix_simd;
        for digit in digits.as_array() {
            count[*digit as usize] += 1;
        }
        i += LANES;
    }

    #[allow(clippy::needless_range_loop)]
    for j in i..n {
        let digit = ((arr[j].id.div(exp as u64)) % radix as u64) as usize;
        count[digit] += 1;
    }

    for i in 1..radix {
        count[i] += count[i - 1];
    }

    for val in arr.iter().rev() {
        let digit = (val.id.div(exp) % radix as u64);
        output[count[digit] - 1] = val.to_owned();
        count[digit] -= 1;
    }
    unsafe {
        super::cleanup(arr, output, n);
    }

    // unsafe {
    //     let ptr = arr.as_mut_ptr();
    //     let output = output.into_boxed_slice();
    //     std::ptr::copy_nonoverlapping(output.as_ptr(), ptr, n);
    //     std::mem::forget(output);
    // }
}

pub trait Modulo {
    fn modulo(self, other: Self) -> Self;
}

impl<U> Modulo for U
where
    U: Copy + Div<Output = U>,
{
    fn modulo(self, other: U) -> U {
        self.div(other)
    }
}

// pub fn modulo_simd<T: Copy + Div<Output = T>>(a: T, b: T) -> T {
//     a.div(b)
// }
