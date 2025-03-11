//! RTCM to RINEX deserialization
use std::io::Read;

use rtcm_rs::next_msg_frame as next_rtcm_msg_frame;

/// RTCM2RNX can deserialize a RTCM stream to RINEX Tokens.
pub struct RTCM2RNX<R: Read> {
    /// internal buffer
    buf: Vec<u8>,
    /// True when EOS has been reached
    eos: bool,
    /// pointer
    ptr: usize,
    /// [Read]able interface
    reader: R,
}

impl<R: Read> Iterator for RTCM2RNX<R> {
    type Item = Option<()>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.eos {
            if self.ptr < self.buf.len() {
                // try filling with new bytes
                let size = self.reader.read(&mut self.buf).ok()?;
                if size == 0 {
                    self.eos = true;
                }
            }
        } else {
            if self.ptr == 0 {
                // done consuming the last bytes
                return None;
            }
        }

        match next_rtcm_msg_frame(&self.buf[self.ptr..]) {
            (_, Some(_)) => {},
            (_, None) => {},
        }

        None
    }
}

impl<R: Read> RTCM2RNX<R> {
    pub fn new(r: R) -> Self {
        Self {
            ptr: 0,
            reader: r,
            eos: false,
            buf: Vec::with_capacity(1024),
        }
    }
}
