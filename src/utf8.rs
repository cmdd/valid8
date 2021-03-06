use std::{
    ptr::copy_nonoverlapping, simd::{i8x32, m8x32, u8x32, FromBits},
};

const CONT_LENGTHS: [i8; 64] = [
    1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 3, 4, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 3, 4,
    1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 3, 4, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 3, 4,
];

#[derive(Clone, Copy)]
enum AlignCount {
    ThirtyOne,
    Thirty,
}

fn alignri(a: i8x32, b: i8x32, bytes: AlignCount) -> i8x32 {
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))]
    {
        #[cfg(target_arch = "x86")]
        use std::{arch::x86::_mm256_alignr_epi8, mem::transmute};
        #[cfg(target_arch = "x86_64")]
        use std::{arch::x86_64::_mm256_alignr_epi8, mem::transmute};

        return match bytes {
            AlignCount::ThirtyOne => unsafe {
                transmute(_mm256_alignr_epi8(transmute(a), transmute(b), 31))
            },
            AlignCount::Thirty => unsafe {
                transmute(_mm256_alignr_epi8(transmute(a), transmute(b), 30))
            },
        };
    }

    #[cfg(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            not(target_feature = "avx2"),
            target_feature = "ssse3"
        )
    )]
    {
        #[cfg(target_arch = "x86")]
        use std::{
            arch::x86::{__m128i, _mm_alignr_epi8},
        };
        #[cfg(target_arch = "x86_64")]
        use std::{
            arch::x86_64::{__m128i, _mm_alignr_epi8},
        };

        return unsafe {
            let mut a = a;
            
            let a1: *mut __m128i = (&mut a as *mut i8x32) as *mut __m128i;
            let a2 = a1.offset(1);

            let b1: *const __m128i = (&b as *const i8x32) as *const __m128i;
            let b2 = b1.offset(1);

            *a1 = match bytes {
                AlignCount::ThirtyOne => _mm_alignr_epi8(*a1, *b1, 31),
                AlignCount::Thirty => _mm_alignr_epi8(*a1, *b1, 30),
            };
            *a2 = match bytes {
                AlignCount::ThirtyOne => _mm_alignr_epi8(*a2, *b2, 31),
                AlignCount::Thirty => _mm_alignr_epi8(*a2, *b2, 30),
            };

            *(a1 as *mut i8x32)
        };
    }

    // This is incorrect
    #[cfg(
        not(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                any(target_feature = "avx2", target_feature = "ssse3")
            )
        )
    )]
    {
        let bytes = match bytes {
            AlignCount::ThirtyOne => 31,
            AlignCount::Thirty => 30,
        };
        let mut zero = [0; 96];
        let z_ptr = zero.as_mut_ptr();
        let sa: *const i8 = (&a as *const i8x32) as *const i8;
        let sb: *const i8 = (&b as *const i8x32) as *const i8;

        unsafe {
            copy_nonoverlapping(sa, z_ptr.offset(32), 32);
            copy_nonoverlapping(sb, z_ptr.offset(64), 32);
        }

        return i8x32::load_unaligned(&zero[64 - bytes..96 - bytes]);
    }
}

fn shuffle(a: i8x32, b: i8x32) -> i8x32 {
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))]
    {
        #[cfg(target_arch = "x86")]
        use std::{arch::x86::_mm256_shuffle_epi8, mem::transmute};
        #[cfg(target_arch = "x86_64")]
        use std::{arch::x86_64::_mm256_shuffle_epi8, mem::transmute};

        return unsafe { transmute(_mm256_shuffle_epi8(transmute(a), transmute(b))) };
    }

    #[cfg(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            not(target_feature = "avx2"),
            target_feature = "ssse3"
        )
    )]
    {
        #[cfg(target_arch = "x86")]
        use std::{
            arch::x86::{__m128i, _mm_shuffle_epi8},
        };
        #[cfg(target_arch = "x86_64")]
        use std::{
            arch::x86_64::{__m128i, _mm_shuffle_epi8},
        };

        return unsafe {
            let mut a = a;

            let a1: *mut __m128i = (&mut a as *mut i8x32) as *mut __m128i;
            let a2 = a1.offset(1);

            let b1: *const __m128i = (&b as *const i8x32) as *const __m128i;
            let b2 = b1.offset(1);

            *a1 = _mm_shuffle_epi8(*a1, *b1);
            *a2 = _mm_shuffle_epi8(*a2, *b2);

            *(a1 as *mut i8x32)
        };
    }

    #[cfg(
        not(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                any(target_feature = "avx2", target_feature = "ssse3")
            )
        )
    )]
    {
        return unsafe {
            let mut a = a;
            let a: *mut i8 = (&mut a as *mut i8x32) as *mut i8;
            let b: *const i8 = (&b as *const i8x32) as *const i8;
        
            for i in 0..16 {
                *a.offset(i) = if *b.offset(i) & -128 == 0 {
                    *a.offset((*b.offset(i) % 16) as isize)
                } else {
                    0
                };
                
                *a.offset(i + 16) = if *b.offset(i + 16) & -128 == 0 {
                    *a.offset((*b.offset(i + 16) % 16 + 16) as isize)
                } else {
                    0
                }
            }

            *(a as *mut i8x32)
        }
    }
}

pub fn validate(input: &[u8]) -> bool {
    // We keep a running mask of if we've seen an error yet.
    let mut err = m8x32::splat(false);
    let mut bytes = i8x32::splat(0);
    let mut high = i8x32::splat(0);
    let mut conts = i8x32::splat(0);

    let nib_mask = i8x32::splat(0x0F);
    let max = u8x32::splat(0xF4);
    let cont_lengths = i8x32::load_unaligned(&CONT_LENGTHS);
    let initial_mins = i8x32::new(
        -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -62, -128, -31,
        -15, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -62, -128,
        -31, -15,
    );
    let second_mins = i8x32::new(
        -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, 127, 127, -96,
        -112, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, 127, 127,
        -96, -112,
    );

    let check_bytes = |mut err, bytes, prev_bytes, prev_high, conts| {
        let off1 = alignri(bytes, prev_bytes, AlignCount::ThirtyOne);

        // My understanding of how bits are stored (at least on Intel x86) for SIMD purposes is
        // that the most significant bit is at the end of a byte, but that this is just an
        // implementation detail; all operations assume human-readable bit order (hence why the
        // slice notation is reversed in the Intel Intrinsics documentation).
        let high_nibbles = (bytes >> 4) & nib_mask;

        // no unicode byte is larger than 0xF4.
        err |= u8x32::from_bits(bytes).gt(max);

        let initial_lengths = shuffle(cont_lengths, high_nibbles);
        let conts = {
            let sum = initial_lengths
                + (alignri(initial_lengths, conts, AlignCount::ThirtyOne) - i8x32::splat(1));
            sum + (alignri(sum, conts, AlignCount::Thirty) - i8x32::splat(2))
        };
        err |= conts
            .gt(initial_lengths)
            .eq(initial_lengths.gt(i8x32::splat(0)));

        let mask_ed = off1.eq(i8x32::splat(-19));
        let mask_f4 = off1.eq(i8x32::splat(-12));
        let bad_ed = bytes.gt(i8x32::splat(-97)) & mask_ed;
        let bad_f4 = bytes.gt(i8x32::splat(-97)) & mask_f4;
        err |= bad_ed | bad_f4;

        let off1_high = alignri(high_nibbles, prev_high, AlignCount::ThirtyOne);
        let initial_under = shuffle(initial_mins, off1_high).gt(off1);
        let second_under = shuffle(second_mins, off1_high).gt(bytes);
        err |= initial_under & second_under;

        (err, high_nibbles, conts)
    };

    for i in 0..input.len() / 32 {
        let buf = &input[i * 32..=i * 32 + 31];
        let nb = i8x32::from_bits(u8x32::load_unaligned(buf));
        let res = check_bytes(err, nb, bytes, high, conts);
        bytes = nb;
        err = res.0;
        high = res.1;
        conts = res.2;
    }

    if input.len() % 32 != 0 {
        let mut remain = [0; 32];
        let rest = &input[32 * (input.len() / 32)..];
        unsafe {
            copy_nonoverlapping(
                rest.as_ptr(),
                remain.as_mut_ptr(),
                input.len() - 32 * (input.len() / 32),
            );
        }
        let nb = i8x32::from_bits(u8x32::load_unaligned(&remain));

        !check_bytes(err, nb, bytes, high, conts).0.or()
    } else {
        err |= conts.gt(i8x32::new(
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 1,
        ));

        !err.or()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignri_valid() {
        let a = i8x32::splat(1);
        let b = i8x32::splat(2);
        let valid_31 = i8x32::new(
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        );
        let valid_30 = i8x32::new(
            1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        );
        assert_eq!(valid_31, alignri(a, b, AlignCount::ThirtyOne));
        assert_eq!(valid_30, alignri(a, b, AlignCount::Thirty));
    }

    #[test]
    fn shuffle_valid() {
        let a = i8x32::splat(0).replace(0, 1).replace(16, 1);
        let b = i8x32::splat(0);
        assert_eq!(i8x32::splat(1), shuffle(a, b));
    }

    #[test]
    fn std_checks_utf8() {
        let s = b"affffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

        assert_eq!(true, validate(s));
    }

    #[test]
    fn std_checks_wrong_utf8() {
        let s = [0xc0 as u8, 0xae as u8];

        assert_eq!(false, validate(&s));
    }
}
