//! RINEX production postponing
use crate::prelude::Epoch;

/// [Postponing] offers several options to postpone the BINEX message collection.
/// It allows to accurately control when the stream listener picks up the
/// BINEX content that should be collected.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Postponing {
    /// RINEX collection starts on first valid BINEX byte
    None,
    /// RINEX collection will start once system time reaches
    /// this value. Note that system time access is OS dependent.
    SystemTime(Epoch),
    /// RINEX collection starts after `size` BINEX bytes have been collected
    Size(usize),
    /// RINEX collection starts after discarding `size` valid BINEX messages
    Messages(usize),
}

impl Default for Postponing {
    fn default() -> Self {
        Self::None
    }
}
