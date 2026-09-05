//! Lost of Lock Indication (LLI) for phase tracking
use bitflags::bitflags;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Debug, Copy, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct LliFlags: u8 {
        /// Current epoch is marked Ok or Unknown status
        const OK_OR_UNKNOWN = 0x00;
        /// Lock lost between previous observation and current observation,
        /// cycle slip is possible
        const LOCK_LOSS = 0x01;
        /// Half cycle slip marker
        const HALF_CYCLE_SLIP = 0x02;
        /// Observing under anti spoofing,
        /// might suffer from decreased SNR - decreased signal quality
        const UNDER_ANTI_SPOOFING = 0x04;
    }
}

impl LliFlags {
    /// Returns true if the phase tracking is considered sane:
    /// neither a lock loss nor a half cycle slip is reported.
    /// Tracking under anti spoofing is tolerated.
    pub fn is_ok(self) -> bool {
        !self.intersects(Self::LOCK_LOSS | Self::HALF_CYCLE_SLIP)
    }
}

#[cfg(test)]
mod test {
    use super::LliFlags;

    #[test]
    fn is_ok() {
        assert!(LliFlags::OK_OR_UNKNOWN.is_ok());
        assert!(LliFlags::UNDER_ANTI_SPOOFING.is_ok());
        assert!(!LliFlags::LOCK_LOSS.is_ok());
        assert!(!LliFlags::HALF_CYCLE_SLIP.is_ok());
        assert!(!(LliFlags::LOCK_LOSS | LliFlags::HALF_CYCLE_SLIP).is_ok());
        assert!(!(LliFlags::LOCK_LOSS | LliFlags::UNDER_ANTI_SPOOFING).is_ok());
    }
}
