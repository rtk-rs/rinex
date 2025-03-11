pub mod header;
pub mod obs;

use std::collections::HashMap;
use std::{io::Write, str::from_utf8};

#[derive(Debug)]
pub struct Utf8Buffer {
    pub inner: Vec<u8>,
}

impl Write for Utf8Buffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for b in buf {
            self.inner.push(*b);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.clear();
        Ok(())
    }
}

impl Utf8Buffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    pub fn to_ascii_utf8(&self) -> String {
        from_utf8(&self.inner).unwrap().to_string()
    }
}

pub fn generic_formatted_lines_test(utf8_content: &str, test_values: HashMap<usize, &str>) {
    let mut nb_tests = 0usize;
    let total_tests = test_values.len();
    for (nth, line) in utf8_content.lines().enumerate() {
        if let Some(content) = test_values.get(&nth) {
            assert_eq!(line, *content);
            nb_tests += 1;
        }
    }
    assert_eq!(nb_tests, total_tests);
}
