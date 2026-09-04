//! Y. Hatanaka lossless TextDiff algorithm

/// [TextDiff] is a structure that implements the Text diff. algorithm
/// designed by Y. Hatanaka, which is a lossless text compression algorithm.
/// [TextDiff] in its current form does not allow compressing & decompressing (back & forth)
/// at the same time: you need two dedicated objects.
/// This does not bother our application because it only operates in one way,
/// but it is one aspect to keep in mind.
#[derive(Debug)]
pub struct TextDiff {
    buffer: String,
    compressed: String,
}

impl Default for TextDiff {
    fn default() -> Self {
        Self::new("")
    }
}

impl TextDiff {
    /// Creates a new [TextDiff] structure to apply the
    /// differential algorithm developped by Y. Hatanaka.
    pub fn new(data: &str) -> Self {
        Self {
            buffer: data.to_string(),
            compressed: data.to_string(),
        }
    }

    /// Force kernel reset using new content
    pub fn force_init(&mut self, data: &str) {
        self.buffer = data.to_string();
        self.compressed = data.to_string();
    }

    /// Decompresses given data. Returns recovered value.
    pub fn decompress(&mut self, data: &str) -> &str {
        let size = data.len();
        let buf_len = self.buffer.len();

        if size > buf_len {
            // extend new bytes as is
            self.buffer.push_str(&data[buf_len..]);
        }

        // browse all bytes and save only new content
        let bytes = data.bytes();
        unsafe {
            let buf = self.buffer.as_bytes_mut();

            // browse all new bytes
            for (i, byte) in bytes.enumerate() {
                if let Some(buf) = buf.get_mut(i) {
                    if byte != b' ' {
                        if byte == b'&' {
                            *buf = b' ';
                        } else {
                            *buf = byte;
                        }
                    }
                }
            }
            &self.buffer
        }
    }

    /// Compress given data using the Textdiff algorithm.
    /// Returns compressed text.
    ///
    /// The internal buffer mirrors what a decompressor recovers: it only
    /// ever grows, characters past the end of a shorter input are kept.
    /// A character identical to the buffer is published as a blank, a
    /// blank replacing a character as '&', anything else as is.
    pub fn compress(&mut self, data: &str) -> &str {
        let data = data.as_bytes();
        let history_len = self.buffer.len();

        self.compressed.clear();

        for (i, byte) in data.iter().enumerate() {
            let previous = self.buffer.as_bytes().get(i).copied();
            let compressed = match previous {
                Some(previous) if previous == *byte => b' ',
                _ => {
                    if *byte == b' ' {
                        b'&'
                    } else {
                        *byte
                    }
                },
            };
            self.compressed.push(compressed as char);
        }

        // shorter input than history: recovered content is kept
        for _ in data.len()..history_len {
            self.compressed.push(' ');
        }

        // mirror the decompressor: overwrite known positions, extend past them
        let mut buffer = std::mem::take(&mut self.buffer).into_bytes();
        for (i, byte) in data.iter().enumerate() {
            if i < buffer.len() {
                buffer[i] = *byte;
            } else {
                buffer.push(*byte);
            }
        }
        self.buffer = String::from_utf8(buffer).expect("textdiff buffer is ascii");

        &self.compressed
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_decompression() {
        let mut diff = TextDiff::new("ABCDEFG 12 000 33 XXACQmpLf");

        let compressed: Vec<&str> = vec![
            "         3   1 44 xxACq   F",
            "        4 ",
            " 11 22   x   0 4  y     p  ",
            "              1     ",
            "                   z",
            " ",
            "                           &",
            "&                           ",
            " ",
        ];
        let expected: Vec<&str> = vec![
            "ABCDEFG 13 001 44 xxACqmpLF",
            "ABCDEFG 43 001 44 xxACqmpLF",
            "A11D22G 4x 000 44 yxACqmpLF",
            "A11D22G 4x 000144 yxACqmpLF",
            "A11D22G 4x 000144 yzACqmpLF",
            "A11D22G 4x 000144 yzACqmpLF",
            "A11D22G 4x 000144 yzACqmpLF ",
            " 11D22G 4x 000144 yzACqmpLF ",
            " 11D22G 4x 000144 yzACqmpLF ",
        ];

        for i in 0..compressed.len() {
            let decompressed = diff.decompress(compressed[i]);
            assert_eq!(
                decompressed,
                expected[i],
                "failed for {}th \"{}\"",
                i + 1,
                compressed[i]
            );
        }

        // test re-init
        diff.force_init(" 2200 123      G 07G08G09G   XX XX");

        let masks: Vec<&str> = vec![
            "        F       1  3",
            " x    1 f  f   p",
            " ",
            "  3       4       ",
        ];
        let expected: Vec<&str> = vec![
            " 2200 12F      G107308G09G   XX XX",
            " x200 12f  f   p107308G09G   XX XX",
            " x200 12f  f   p107308G09G   XX XX",
            " x300 12f 4f   p107308G09G   XX XX",
        ];
        for i in 0..masks.len() {
            let mask = masks[i];
            let result = diff.decompress(mask);
            assert_eq!(
                result,
                expected[i].to_string(),
                "failed for [{}]{}",
                i,
                mask
            );
        }
    }

    #[test]
    fn test_compression() {
        let mut diff = TextDiff::new("0");
        assert_eq!(diff.compress("0"), " ");
        assert_eq!(diff.compress("4"), "4");
        assert_eq!(diff.compress("4"), " ");
        assert_eq!(diff.compress("4 "), " &");
        assert_eq!(diff.compress("4  "), "  &");
        assert_eq!(diff.compress("0"), "0  ");
        assert_eq!(diff.compress("0"), "   ");
        assert_eq!(diff.compress("   "), "&  ");

        diff.force_init("Default 1234");
        assert_eq!(diff.compress("DEfault 1234"), " E          ");
        assert_eq!(diff.compress("DEfault 1234"), "            ");
        assert_eq!(diff.compress("             "), "&&&&&&& &&&&&");
    }
}
