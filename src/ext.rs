use super::utf8;
use super::ascii;

trait ValidateExt {
    fn valid_ascii(self) -> bool;
    fn valid_utf8(self) -> bool;
}

impl<'a> ValidateExt for &'a [u8] {
    fn valid_ascii(self) -> bool {
        ascii::validate(self)
    }

    fn valid_utf8(self) -> bool {
        utf8::validate(self)
    }
}

impl<'a> ValidateExt for &'a str {
    fn valid_ascii(self) -> bool {
        ascii::validate(self.as_bytes())
    }

    fn valid_utf8(self) -> bool {
        utf8::validate(self.as_bytes())
    }
}

impl ValidateExt for String {
    fn valid_ascii(self) -> bool {
        ascii::validate(self.as_bytes())
    }

    fn valid_utf8(self) -> bool {
        utf8::validate(self.as_bytes())
    }
}
