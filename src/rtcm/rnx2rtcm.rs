//! RINEX to BINEX serialization
use std::io::Read;

use crate::prelude::Rinex;

use rtcm_rs::msg::message::Message;

/// RNX2RTCM can serialize a RINEX to a stream of RTCM Messages.
pub struct RNX2RTCM<W: Write> {
    /// RTCM encoder
    encoder: Encoder<W>,
}

impl Iterator for RNX2RTCM {
    type Item = Option<Message>;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl RNX2BIN {
    /// Creates a new [RNX2BIN].
    pub fn new<W: Write>(w: W) -> Self {
        Self {
            encoder: Encoder::new(r),
        }
    }
}
