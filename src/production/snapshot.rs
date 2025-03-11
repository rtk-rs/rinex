//! RINEX collection option
use crate::prelude::Duration;

/// [SnapshotMode] is used by RINEX collection methods,
/// like [BIN2RNX] for example, that needs to collect a RINEX
/// (usually of fixed duration)
/// from a stream that is possibly infinite.
#[derive(Debug, Copy, Clone)]
pub enum SnapshotMode {
    /// Dump as RINEX every day at midnight.
    /// This is the prefered [SnapshotMode] because
    /// standard RINEX span 24h.
    DailyMidnight,
    /// Dump as RINEX every day at midnight.
    /// Use this to produce 12h RINEX.
    DailyMidnightNoon,
    /// Publish every hour
    /// Use this to produce hourly RINEX.
    Hourly,
    /// Publish periodically.
    /// Use this to produce custom RINEX.
    Periodic(Duration),
}

impl SnapshotMode {
    /// Create a period [SnapshotMode]
    pub fn periodic(dt: Duration) -> Self {
        Self::Periodic(dt)
    }
}
