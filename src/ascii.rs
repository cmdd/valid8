use faster::*;
use std::{arch::x86_64::*, simd::*};

#[inline]
pub fn validate(input: &[u8]) -> bool {
    let len = input.len();
    let amax = u8x64::splat(0x80);
    let mut err = u8x64::splat(0);

    let mut i = 0;

    while i + 63 < len {
        err |= u8x64::load_unaligned(&input[i..=i + 63]);
        i += 64;
    }

    if len >= 64 {
        err |= u8x64::load_unaligned(&input[len - 64..len]);
        (err & amax).max_element() < 0x80
    } else {
        let err = (err & amax).max_element();

        let mut tail_has_char: u8 = 0;
        while i < len {
            tail_has_char |= input[i];
            i += 1;
        }

        (err | tail_has_char) & 0x80 == 0
    }
}

pub fn faster(input: &[u8]) -> bool {
    let amax = u8s(0x80);

    input
        .simd_iter(u8s(0))
        .simd_reduce(u8s(0), |acc, v| acc | v)
        .ge(amax)
        .scalar_reduce(true, |acc, v| acc && v == 0)
}

// TODO: Guard based on SIMD support
// see: https://doc.rust-lang.org/beta/std/arch/index.html
pub fn arch(input: &[u8]) -> bool {
    let len = input.len();
    let mut i = 0;
    unsafe {
        let mut err = _mm_setzero_si128();

        while i + 15 < len {
            let cb = _mm_loadu_si128(input.as_ptr().offset(i as isize) as *const __m128i);
            err = _mm_or_si128(err, cb);
            i += 16;
        }
        let err = _mm_movemask_epi8(err);
        let mut tail_err: u8 = 0;
        while i < len {
            tail_err |= input[i];
            i += 1;
        }
        let err = err | (tail_err & 0x80) as i32;
        return err == 0;
    }
}

pub fn super_arch(input: &[u8]) -> bool {
    let len = input.len();
    let mut i = 0;
    unsafe {
        let mut err = _mm256_setzero_si256();

        while i + 31 < len {
            let cb = _mm256_loadu_si256(input.as_ptr().offset(i as isize) as *const __m256i);
            err = _mm256_or_si256(err, cb);
            i += 32;
        }

        return if len >= 32 {
            let cb =
                _mm256_loadu_si256(input.as_ptr().offset((len - 32) as isize) as *const __m256i);
            err = _mm256_or_si256(err, cb);
            _mm256_movemask_epi8(err) == 0
        } else {
            let err = _mm256_movemask_epi8(err);
            let mut tail_err: u8 = 0;
            while i < len {
                tail_err |= input[i];
                i += 1;
            }
            let err = err | (tail_err & 0x80) as i32;
            err == 0
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #[test]
        fn std_checks_ascii(ref s in "\\PC*") {
            let s = s.as_bytes();
            prop_assert_eq!(s.is_ascii(), validate(s));
        }

        #[test]
        fn faster_checks_ascii(ref s in "\\PC*") {
            let s = s.as_bytes();
            prop_assert_eq!(s.is_ascii(), faster(s));
        }

        #[test]
        fn arch_checks_ascii(ref s in "\\PC*") {
            let s = s.as_bytes();
            prop_assert_eq!(s.is_ascii(), arch(s));
        }

        #[test]
        fn super_arch_checks_ascii(ref s in "\\PC*") {
            let s = s.as_bytes();
            prop_assert_eq!(s.is_ascii(), super_arch(s));
        }
    }
}
