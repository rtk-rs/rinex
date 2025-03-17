use itertools::Itertools;

use std::io::{BufWriter, Write};

use crate::{
    epoch::epoch_decompose as epoch_decomposition,
    error::FormattingError,
    navigation::{NavFrameType, NavKey, Record},
    prelude::{Constellation, Header},
};

/// Special BufWriter to rework the ASCII stream
/// and make it NAV compatible
pub struct NavBufWriter<W: Write> {
    writer: W,
    len: usize,
    input: String,
    output: String,
    copy: String,
    exponent_size: usize,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum State {
    #[default]
    Nominal,
    InsideExponent,
}

impl<W: Write> Write for NavBufWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // append to pending list
        let ascii = String::from_utf8_lossy(&buf);
        self.input.push_str(&ascii);
        self.len += ascii.len();

        // process
        self.run();

        // update & return

        Ok(size)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write> NavBufWriter<W> {
    fn run(&mut self) -> usize {
        let mut size = 0;
        let bytes = self.input.as_bytes();

        // process all bytes
        for i in 0..self.len {
            match self.state {
                State::InsideExponent => {
                    if bytes[i] == b' ' {
                        self.state = State::Nominal;
                    } else {
                    }
                },
                State::Nominal => {
                    if bytes[i] == b'e' || bytes[i] == b'E' {
                        self.state = State::InsideExponent;
                    }
                },
            }
            size += 1;
        }

        return;
    }

    // e-0 => e-00
    // e0 => e+00
    fn buffer_format(s: &mut String) {
        let len = s.len();

        let cloned = s.clone();
        let bytes = cloned.as_bytes();
        let str_bytes = cloned.as_str();

        let mut ptr = 0;
        loop {
            if bytes[ptr] == b'e' || bytes[ptr] == b'E' {
                if ptr < len - 1 {
                    if bytes[ptr + 1].is_ascii_digit() {
                        if ptr < len - 2 {
                            if bytes[ptr + 2].is_ascii_digit() {
                                s.insert(ptr + 1, '+');
                                //s.replace_range(ptr +1.., &str_bytes[i+1..]);
                            }
                        }
                    }
                }
            }

            ptr += 1;
            if ptr == len {
                break;
            }
        }
    }

    /// Create a new [NavBufWriter] dedicated to Navigation RINEX formatting
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            exponent_size: 0,
            state: State::default(),
            input: String::with_capacity(256),
            output: String::with_capacity(256),
            exponent_digits: String::with_capacity(8),
        }
    }
}

#[cfg(test)]
mod test {
    use super::NavBufWriter;
    use crate::tests::formatting::Utf8Buffer;

    #[test]
    fn test_nav_buffer() {
        let mut buf = NavBufWriter::new(Utf8Buffer::new(1024));

        for (test, result) in [
            (
                "ABCAZDAZDCADQSMLKMLKO°0AZD00AZDKKKAZDK",
                "ABCAZDAZDCADQSMLKMLKO°0AZD00AZDKKKAZDK",
            ),
            ("00110e10", "00110e+10"),
            ("00220e1", "00220e+01"),
        ] {
            let mut s = test.to_string();
            NavBufWriter::<Utf8Buffer>::buffer_format(&mut s);
            assert_eq!(s, result);
        }
    }
}
