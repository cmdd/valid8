use std::{cmp::min, ptr::copy_nonoverlapping, simd::*};

const CONT_LENGTHS: [i8; 64] = [
    1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 3, 4, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 3, 4,
    1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 3, 4, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 3, 4,
];

// TODO: Performance here sucks.
fn alignri(a: i8x64, b: i8x64, bytes: u8) -> i8x64 {
    let bytes = min(bytes, 128) as usize;
    let mut zero = [0; 192];
    let z_ptr = zero.as_mut_ptr();
    let mut sa: [i8; 64] = [0; 64];
    let mut sb: [i8; 64] = [0; 64];

    i8x64::store_unaligned(a, &mut sa);
    i8x64::store_unaligned(b, &mut sb);

    unsafe {
        copy_nonoverlapping(sa.as_ptr(), z_ptr.offset(64), 64);
        copy_nonoverlapping(sb.as_ptr(), z_ptr.offset(128), 64);
    }

    i8x64::load_unaligned(&zero[128 - bytes..192 - bytes])
}

// TODO: Use more efficient pshuf instruction.
fn shuffle(a: i8x64, b: i8x64) -> i8x64 {
    fn shuffle_generic(a: i8x64, b: i8x64) -> i8x64 {
        let mut res = i8x64::splat(0);
        for i in 0..16 {
            if b.extract(i) & -128 == 0 {
                res = res.replace(i, a.extract((b.extract(i) % 16) as usize));
            }
            if b.extract(i + 16) & -128 == 0 {
                res = res.replace(i + 16, a.extract((b.extract(i + 16) % 16 + 16) as usize));
            }
            if b.extract(i + 32) & -128 == 0 {
                res = res.replace(i + 32, a.extract((b.extract(i + 32) % 16 + 32) as usize));
            }
            if b.extract(i + 48) & -128 == 0 {
                res = res.replace(i + 48, a.extract((b.extract(i + 48) % 16 + 48) as usize));
            }
        }

        res
    }

    shuffle_generic(a, b)
}

pub fn validate(input: &[u8]) -> bool {
    // We keep a running mask of if we've seen an error yet.
    let mut err = m1x64::splat(false);
    let mut bytes = i8x64::splat(0);
    let mut high = i8x64::splat(0);
    let mut conts = i8x64::splat(0);

    let nib_mask = i8x64::splat(0x0F);
    let max = u8x64::splat(0xF4);
    let cont_lengths = i8x64::load_unaligned(&CONT_LENGTHS);
    let initial_mins = i8x64::new(
        -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -62, -128, -31, -15, -128,
        -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -62, -128, -31, -15, -128, -128,
        -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -62, -128, -31, -15, -128, -128, -128,
        -128, -128, -128, -128, -128, -128, -128, -128, -128, -62, -128, -31, -15,
    );
    let second_mins = i8x64::new(
        -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, 127, 127, -96, -112, -128, -128,
        -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, 127, 127, -96, -112, -128, -128, -128, -128,
        -128, -128, -128, -128, -128, -128, -128, -128, 127, 127, -96, -112, -128, -128, -128, -128, -128, -128,
        -128, -128, -128, -128, -128, -128, 127, 127, -96, -112,
    );

    let check_bytes = |mut err, bytes, prev_bytes, prev_high, conts| {
        let off1 = alignri(bytes, prev_bytes, 63);

        // My understanding of how bits are stored (at least on Intel x86) for SIMD purposes is
        // that the most significant bit is at the end of a byte, but that this is just an
        // implementation detail; all operations assume human-readable bit order (hence why the
        // slice notation is reversed in the Intel Intrinsics documentation).
        let high_nibbles = (bytes >> 4) & nib_mask;

        // no unicode byte is larger than 0xF4.
        err |= u8x64::from_bits(bytes).gt(max);

        let initial_lengths = shuffle(cont_lengths, high_nibbles);
        let conts = {
            let sum = initial_lengths + (alignri(initial_lengths, conts, 63) - i8x64::splat(1));
            sum + (alignri(sum, conts, 62) - i8x64::splat(2))
        };
        err |= conts
            .gt(initial_lengths)
            .eq(initial_lengths.gt(i8x64::splat(0)));

        let mask_ed = off1.eq(i8x64::splat(-19));
        let mask_f4 = off1.eq(i8x64::splat(-12));
        let bad_ed = bytes.gt(i8x64::splat(-97)) & mask_ed;
        let bad_f4 = bytes.gt(i8x64::splat(-97)) & mask_f4;
        err |= bad_ed | bad_f4;

        let off1_high = alignri(high_nibbles, prev_high, 63);
        let initial_under = shuffle(initial_mins, off1_high).gt(off1);
        let second_under = shuffle(second_mins, off1_high).gt(bytes);
        err |= initial_under & second_under;

        (err, high_nibbles, conts)
    };

    for i in 0..input.len() / 64 {
        let buf = &input[i * 64..=i * 64 + 63];
        let nb = i8x64::from_bits(u8x64::load_unaligned(buf));
        let res = check_bytes(err, nb, bytes, high, conts);
        bytes = nb;
        err = res.0;
        high = res.1;
        conts = res.2;
    }

    // TODO: What to do with the rest of the bytes
    return if input.len() % 64 != 0 {
        let mut remain = [0; 64];
        let rest = &input[64 * (input.len() / 64)..];
        unsafe {
            copy_nonoverlapping(
                rest.as_ptr(),
                remain.as_mut_ptr(),
                input.len() - 64 * (input.len() / 64),
            );
        }
        let nb = i8x64::from_bits(u8x64::load_unaligned(&remain));

        !check_bytes(err, nb, bytes, high, conts).0.or()
    } else {
        err |= conts.gt(i8x64::new(
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 1,
        ));

        !err.or()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignri_zero_wrap() {
        let a = i8x64::splat(1);
        let b = i8x64::splat(2);
        assert_eq!(i8x64::splat(0), alignri(a, b, 128));
        assert_eq!(i8x64::splat(0), alignri(a, b, 255));
    }

    #[test]
    fn alignri_valid() {
        let a = i8x64::splat(1);
        let b = i8x64::splat(2);
        let valid = i8x64::new(
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2,
        );
        assert_eq!(valid, alignri(a, b, 48));
    }

    #[test]
    fn shuffle_valid() {
        let a = i8x64::splat(0)
            .replace(0, 1)
            .replace(16, 1)
            .replace(32, 1)
            .replace(48, 1);
        let b = i8x64::splat(0);
        assert_eq!(i8x64::splat(1), shuffle(a, b));
    }

    #[test]
    fn std_checks_utf8() {
        let s = b"affffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

        assert_eq!(true, validate(s));
    }
}
