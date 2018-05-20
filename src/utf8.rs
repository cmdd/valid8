use faster::*;
use std::arch::x86_64::*;
use std::simd::*;

fn validate(input: &[u8]) -> bool {
    let err = u8x64::splat(0);
    let max = u8x64::splat(0xF4);

    err |= input.gt(max);

    unimplemented!()
}

fn faster(input: &[u8]) -> bool {
    let max = u8s(0xF4);

}

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #[test]
        fn std_checks_unicode(ref s in "\\PC*") {
            let s = s.as_bytes();
            prop_assert_eq!(s.is_ascii(), validate(s));
        }

        #[test]
        fn faster_checks_unicode(ref s in "\\PC*") {
            let s = s.as_bytes();
            prop_assert_eq!(s.is_ascii(), faster(s));
        }
    }
}
