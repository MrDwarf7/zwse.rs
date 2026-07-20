mod avx;
mod radix;
mod radix_simd;

pub use avx::*;
#[allow(unused_imports)]
pub use radix::*;

#[allow(unused_imports)]
// pub use radix_simd::*;
use crate::extract::ExtractedData;

unsafe fn cleanup(arr: &mut [ExtractedData], output: Vec<ExtractedData>, n: usize) {
    debug_assert_eq!(arr.len(), n);
    debug_assert_eq!(output.len(), n);
    println!("Cleaning up");
    // dbg!(n);
    // dbg!(arr.len());

    let output_ptr = output.as_ptr();
    let arr_ptr = arr.as_mut_ptr();

    let _ = output.into_boxed_slice();
    unsafe {
        std::ptr::copy_nonoverlapping(output_ptr, arr_ptr, n);
    }
}
