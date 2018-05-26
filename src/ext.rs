use super::ascii;
use super::utf8;

trait SimdValidateExt {
    fn is_ascii_simd(&self) -> bool;
    fn is_utf8_simd(&self) -> bool;
}

impl SimdValidateExt for [u8] {
    fn is_ascii_simd(&self) -> bool {
        ascii::validate(self)
    }

    fn is_utf8_simd(&self) -> bool {
        utf8::validate(self)
    }
}

impl SimdValidateExt for str {
    fn is_ascii_simd(&self) -> bool {
        ascii::validate(self.as_bytes())
    }

    fn is_utf8_simd(&self) -> bool {
        utf8::validate(self.as_bytes())
    }
}

