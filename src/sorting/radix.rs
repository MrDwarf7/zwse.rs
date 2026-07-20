use crate::extract::ExtractedData;

#[allow(dead_code)]
pub fn radix_sort(arr: &mut [ExtractedData]) {
    const RADIX: usize = 256;

    let max_val = arr.iter().map(|x| x.id).max().unwrap();
    let mut exp = 1;
    while max_val / exp > 0 {
        counting_sort(arr, exp as u32, RADIX);
        exp *= RADIX as u64;
    }
}

pub fn counting_sort(arr: &mut [ExtractedData], exp: u32, radix: usize) {
    let n = arr.len();
    let mut output = vec![ExtractedData::default(); n];
    let mut count = vec![0; radix];

    for val in arr.iter() {
        let digit = ((val.id as u32 / exp) % radix as u32) as usize;
        count[digit] += 1;
    }

    for i in 1..radix {
        count[i] += count[i - 1];
    }

    for val in arr.iter().rev() {
        let digit = ((val.id as u32 / exp) % radix as u32) as usize;
        output[count[digit] - 1] = val.clone();
        count[digit] -= 1;
    }

    unsafe {
        super::cleanup(arr, output, n);
    }

    // unsafe {
    //     let ptr = arr.as_mut_ptr();
    //     let output = output.into_boxed_slice();
    //
    //     std::ptr::copy_nonoverlapping(output.as_ptr(), ptr, n);
    //     std::mem::forget(output);
    // }
}
