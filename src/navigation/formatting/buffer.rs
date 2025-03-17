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
    buffer: String,
}

impl<W: Write> Write for NavBufWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // erase
        self.buffer = String::from_utf8_lossy(&buf).to_string();
        // replace
        Self::buffer_format(&mut self.buffer);
        // write
        let size = self.writer.write(&self.buffer.as_bytes())?;
        Ok(size)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write> NavBufWriter<W> {
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
                if bytes[ptr+1].is_ascii_digit() {
                    s.insert(ptr +1, '+');

                    if ptr  len -1 {
                        s.insert(ptr +2, '0');
                    } else if bytes[ptr +2] == b' ' {
                        s.insert(ptr +2, '0');

                    }
                    //s.replace_range(ptr +1.., &str_bytes[i+1..]);
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
            buffer: String::with_capacity(128),
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
            ("ABCAZDAZDCADQSMLKMLKO°0AZD00AZDKKKAZDK", "ABCAZDAZDCADQSMLKMLKO°0AZD00AZDKKKAZDK"),
            ("00110e10", "00110e+10"),
            ("00220e1", "00220e+01"),
        ] {
            let mut s = test.to_string();
            NavBufWriter::<Utf8Buffer>::buffer_format(&mut s);
            assert_eq!(s, result);
        }
    }
}
