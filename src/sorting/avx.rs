use std::arch::asm;
use std::arch::x86_64::*;
use std::ops::Shr;
use crate::extract::ExtractedData;

pub fn radix_sort_avx(arr: &mut [ExtractedData]) {
    const RADIX: usize = 256;
    let max_val = arr.iter().map(|x| x.id).max().unwrap_or_default();
    let mut exp = 1;

    while max_val / exp > 0 {
        unsafe { counting_sort_avx(arr, exp as usize) };
        exp *= RADIX as u64;
    }
}

#[target_feature(enable = "avx2")]
unsafe fn counting_sort_avx(arr: &mut [ExtractedData], exp: usize) {
    const RADIX: usize = 256;
    let n = arr.len();
    let mut output = vec![ExtractedData::default(); n];
    let mut count = [0usize; RADIX];

    // Compute shift amount based on exp (e.g., exp = 1, 256, 65536, etc.)
    // exp = 256^k, shift = 8*k to get the k-th byte
    let shift = exp.trailing_zeros() as i32 as *const i32; // 0 for exp=1, 8 for exp=256, 16 for exp=65536

    // Process 4 elements at a time with AVX2
    let mut i = 0;
    while i + 4 <= n {
        // Load 4 ids into a 256-bit vector
        let ids = _mm256_set_epi64x(
            arr[i + 3].id as i64,
            arr[i + 2].id as i64,
            arr[i + 1].id as i64,
            arr[i].id as i64,
        );

        // Shift right to extract the digit

        let shifted = _mm256_srli_epi64::<8>(ids);

        // Mask to get the least significant byte (digit)
        let digits = _mm256_and_si256(shifted, _mm256_set1_epi64x(0xFF));

        // Extract the 4 digits
        let digit0 = _mm256_extract_epi64::<0>(digits) as u8;
        let digit1 = _mm256_extract_epi64::<1>(digits) as u8;
        let digit2 = _mm256_extract_epi64::<2>(digits) as u8;
        let digit3 = _mm256_extract_epi64::<3>(digits) as u8;

        // Update count array
        count[digit0 as usize] += 1;
        count[digit1 as usize] += 1;
        count[digit2 as usize] += 1;
        count[digit3 as usize] += 1;

        i += 4;
    }

    // Handle remaining elements scalarly
    for j in i..n {
        let digit = ((arr[j].id.shr(shift as u64)) & 0xFF) as usize;
        count[digit] += 1;
    }

    // Compute prefix sum
    for j in 1..RADIX {
        count[j] += count[j - 1];
    }

    // Build output array
    for val in arr.iter().rev() {
        let digit = ((val.id.shr(shift as u64)) & 0xFF) as usize;
        count[digit] -= 1;
        output[count[digit]] = val.clone();
    }

    // Copy back to input array
    unsafe {
        super::cleanup(arr, output, n);
    }
}

// #[target_feature(enable = "avx2")]
// #[unsafe(no_mangle)]
// #[inline(never)]
// unsafe fn counting_sort_avx(arr: &mut [ExtractedData], exp: usize) {
//     unsafe {
//         const RADIX: usize = 256;
//         let p = std::mem::size_of::<ExtractedData>();
//         let n = arr.len();
//
//         // let c = std::mem::size_of::<usize>();
//         // n *= p;; // index out of bounds: the len (n) is 7106 but the index is 7160 (note: this is 54 bytes past the end of the array), we're adding 'padding' by lazyily using the size of ExtractedData
//
//         println!("Size of ExtractedData: {}", p);
//         println!("Size of arr: {}", n);
//         println!("LEN of arr: {}", arr.len());
//
//         let mut output = vec![ExtractedData::default(); n];
//         let mut count = [0usize; RADIX];
//         // println!("Output val: {:?}", output);
//
//         asm!(
//         // Load data into AVX registers
//         "vmovdqu ymm0, [rdi]",   // Load first vector
//         "vmovdqu ymm1, [rdi + 32]",  // Load second vector
//
//         // Shift right to extract digit (using immediate for shift amount)
//         "vpsrlq ymm2, ymm0, {shift}",
//         "vpsrlq ymm3, ymm1, {shift}",
//
//         // Mask to get least significant byte (using vector comparison)
//         "vpand ymm2, ymm2, ymm4",
//         "vpand ymm3, ymm3, ymm4",
//
//         // Store results back to memory
//         "vmovdqu [{output}], ymm2",
//         "vmovdqu [{output} + 32], ymm3",
//         shift = const 8,  // Example shift amount
//         output = in(reg) count.as_mut_ptr(),
//         in("rdi") arr.as_ptr(),
//         in("ymm4") _mm256_set1_epi64x(0xFF),  // Mask vector
//         out("ymm0") _,
//         out("ymm1") _,
//         out("ymm2") _,
//         out("ymm3") _,
//         options(nostack, preserves_flags)
//         );
//
//         // Remaining scalar processing
//         for j in arr.iter().take(n + 1).skip(32) {
//             let digit = ((j.id / exp as u64) % RADIX as u64) as usize;
//             // let digit = ((arr[j].id / exp as u64) % RADIX as u64) as usize;
//             count[digit] += 1;
//         }
//
//         // Prefix sum and output array generation
//         // (Similar to previous implementations)
//         // for j in 1..RADIX {
//         for j in 1..RADIX.min(count.len()) {
//             count[j] += count[j - 1];
//         }
//
//         for val in arr.iter().take(n + 1).skip(32).rev() {
//             let b = val.id / exp as u64;
//             let digit = (b % RADIX as u64) as usize;
//             println!("b: {}", b);
//             println!("digitttttt: {}", digit);
//             // let digit = ((val.id / exp as u64) % RADIX as u64) as usize;
//             println!("Digit: {}", digit);
//             count[digit] -= 1;
//             println!("Count[digit]: {}", count[digit]);
//             output[count[digit]] = val.clone();
//         }
//
//         // Cleanup
//         super::cleanup(arr, output, n);
//     }
// }
