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
    Exponent,
    PlusDigit1,
    MinusDigit1,
    MinusDigit2,
}

/// Special BufWriter to rework the ASCII stream
/// and make it 100% NAV compatible
pub struct NavBufWriter<W: Write> {
    copy: u8,
    pub writer: W,
    state: State,
    input: String,
    output: String,
    to_write: usize,
    exponent_size: usize,
}

impl<W: Write> Write for NavBufWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // append to pending list
        let ascii = String::from_utf8_lossy(&buf);
        self.input = ascii.to_string();
        self.process();

        // update
        let written = self.writer.write(self.output.as_bytes())?;
        self.output = self.output[written..].to_string();
        self.to_write -= written;
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write> NavBufWriter<W> {
    pub fn into_inner(&self) -> &W {
        &self.writer
    }

    fn process(&mut self) {
        let mut ptr = 0;
        let len = self.input.len();
        let bytes = self.input.as_bytes();
        loop {
            match self.state {
                State::Nominal => {
                    if bytes[ptr] == b'e' || bytes[ptr] == b'E' {
                        self.state = State::Exponent;
                    }
                    self.output.push(bytes[ptr].into());
                    ptr += 1;
                },
                State::Exponent => {
                    if bytes[ptr] == b'-' {
                        self.output.push('-');
                        self.state = State::MinusDigit1;
                    } else {
                        self.output.push('+');
                        self.copy = bytes[ptr];
                        self.state = State::PlusDigit1;
                    }
                    ptr += 1;
                },
                State::PlusDigit1 => {
                    if bytes[ptr] == b' ' {
                        self.output.push('0');
                        self.output.push(self.copy.into());
                        self.output.push(bytes[ptr].into());
                    } else {
                        self.output.push(self.copy.into());
                        self.output.push(bytes[ptr].into());
                    }
                    self.state = State::Nominal;
                    ptr += 1;
                },
                State::MinusDigit1 => {
                    self.copy = bytes[ptr];
                    self.state = State::MinusDigit2;
                    ptr += 1;
                },
                State::MinusDigit2 => {
                    if bytes[ptr] == b' ' {
                        self.output.push('0');
                        self.output.push(self.copy.into());
                    } else {
                        self.output.push(self.copy.into());
                        self.output.push(bytes[ptr].into());
                    }
                    self.state = State::Nominal;
                    ptr += 1;
                },
            }
            if ptr == len {
                break;
            }
        }
        self.to_write = self.output.len();
    }

    /// Create a new [NavBufWriter] dedicated to Navigation RINEX formatting
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            copy: 0,
            to_write: 0,
            exponent_size: 0,
            state: State::default(),
            input: String::with_capacity(256),
            output: String::with_capacity(256),
        }
    }
}

#[cfg(test)]
mod test {
    use super::NavBufWriter;
    use crate::tests::formatting::Utf8Buffer;
    use std::io::{BufWriter, Write};

    #[test]
    fn test_nav_buffer() {
        let utf8_buf = Utf8Buffer::new(1024);
        let mut buf = NavBufWriter::new(utf8_buf);

        for (test, size, result) in [
            ("ABCAZDAZDCADQSML", 16, "ABCAZDAZDCADQSML"),
            ("10e10", 6, "10e+10"),
            ("10e-10", 6, "10e-10"),
            ("10010e10", 9, "10010e+10"),
            ("10e10 10e9", 9, "10e+10 10+e09"),
        ] {
            let mut s = test.to_string();

            let written = buf.write(s.as_bytes()).unwrap();

            assert_eq!(written, size);

            let utf8buf = buf.into_inner();
            let utf8 = utf8buf.to_ascii_utf8();
            assert_eq!(utf8, result);
        }
    }
}
