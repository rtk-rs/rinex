use crate::navigation::TimeOffset;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Navigation specific header fields
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// [TimeOffset] definitions
    pub time_offsets: Vec<TimeOffset>,
}

impl HeaderFields {
    pub fn with_time_offset(&self, offset: TimeOffset) -> Self {
        let mut s = self.clone();
        s.time_offsets.push(offset);
        s
    }

    /// Add a [TimeOffset] definition
    pub fn add_time_offset(&mut self, offset: TimeOffset) {
        self.time_offsets.push(offset);
    }
}
