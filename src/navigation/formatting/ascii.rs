use itertools::Itertools;

use std::io::{BufWriter, Write};

use crate::{
    epoch::epoch_decompose as epoch_decomposition,
    error::FormattingError,
    navigation::{NavFrameType, NavKey, Record},
    prelude::{Constellation, Header},
};

#[derive(Debug, Clone, Default, PartialEq)]
enum State {
    #[default]
    Nominal,
    Digit,
    Exponent,
    PlusDigit1,
    MinusDigit1,
    MinusDigit2,
}

/// Special BufWriter to rework the ASCII stream
/// and make it 100% NAV compatible
#[derive(Debug)]
pub struct AsciiString {
    inner: String,
}

impl std::fmt::Display for AsciiString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsciiString {
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    /// Create a new [AsciiString] dedicated to Navigation RINEX formatting
    pub fn from_str(s: &str) -> Self {
        let mut ptr = 0;
        let len = s.len();
        let mut copy = 0u8;
        let bytes = s.as_bytes();
        let mut inner = String::with_capacity(len);
        let mut state = State::default();

        loop {
            match state {
                State::Nominal => {
                    if bytes[ptr] == b'e' || bytes[ptr] == b'E' {
                        state = State::Exponent;
                    }
                    inner.push(bytes[ptr].into());
                },
                State::Digit => {
                    if bytes[ptr] == b'.' {
                    } else {
                    }
                    inner.push(copy.into());
                    inner.push(bytes[ptr].into());
                    state = State::Nominal;
                },
                State::Exponent => {
                    if bytes[ptr] == b'-' {
                        inner.push('-');
                        state = State::MinusDigit1;
                    } else {
                        inner.push('+');
                        copy = bytes[ptr];
                        state = State::PlusDigit1;

                        if ptr == len - 1 {
                            inner.push('0');
                            inner.push(bytes[ptr].into());
                        }
                    }
                },
                State::PlusDigit1 => {
                    if bytes[ptr] == b' ' || bytes[ptr] == b'\n' {
                        inner.push('0');
                        inner.push(copy.into());
                        inner.push(bytes[ptr].into());
                    } else {
                        inner.push(copy.into());
                        inner.push(bytes[ptr].into());
                    }
                    state = State::Nominal;
                },
                State::MinusDigit1 => {
                    copy = bytes[ptr];
                    state = State::MinusDigit2;
                },
                State::MinusDigit2 => {
                    if bytes[ptr] == b' ' {
                        inner.push('0');
                        inner.push(copy.into());
                    } else {
                        inner.push(copy.into());
                        inner.push(bytes[ptr].into());
                    }
                    state = State::Nominal;
                },
            }
            ptr += 1;
            if ptr == len {
                break;
            }
        }
        Self { inner }
    }
}

#[cfg(test)]
mod test {
    use super::AsciiString;

    #[test]
    fn test_nav_ascii() {
        for (test, result) in [
            ("ABCAZDAZDCADQSML", "ABCAZDAZDCADQSML"),
            ("10e10", "10e+10"),
            ("10e-10", "10e-10"),
            ("10010e10", "10010e+10"),
            ("10e10 10e9", "10e+10 10e+09"),
            ("10E10", "10E+10"),
            ("10E-10", "10E-10"),
            ("10010E10", "10010E+10"),
            ("10E10 10E9", "10E+10 10E+09"),
            ("01.01E3", "01.01E+03"),
        ] {
            let output = AsciiString::from_str(test);
            assert_eq!(output.inner, result, "failed for \"{}\"", test);
        }
    }
}
