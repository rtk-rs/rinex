use bitflags::bitflags;

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags! {
    /// [GlonassHealth] flag
    #[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GlonassHealth : u32 {
        const UNHEALTHY = 0x000000001;
    }
}

bitflags! {
    /// Subsidary 3-bit [GlonassHealth2] flags
    #[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GlonassHealth2 : u32 {
        /// Attached Almanac is healthy.
        const HEALTHY_ALMANAC = 0x00000001;
        /// Almanac is being reported in this Ephemeris frame.
        const ALAMANAC_IS_REPORTED = 0x00000002;
        /// Should only be considered if GLO-M/K is
        /// [GlonassStatus::glonass_mk_type_flag] from the general status is true.
        const M_K_ONLY_L3_BIT = 0x00000004;
    }
}

/// [GlonassStatus] 9-bit binary status mask
#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct GlonassStatus(pub(crate) u32);

impl From<u32> for GlonassStatus {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

impl GlonassStatus {
    /// P1 bits 2-3 : update & valdity interval
    pub fn glonass_p2_update_validity_interval(&self) -> GlonassUpdateValidyInterval {
        GlonassUpdateValidyInterval::from(self.0)
    }

    /// P2 bit 4: odd time interval
    pub fn glonass_p2_odd_time_interval(&self) -> bool {
        self.0 & 0x00000010 > 0
    }

    /// P3 bit 5: 5 satellites in current almanac or only 4.
    pub fn glonass_p3_5_satellites_almanac(&self) -> bool {
        self.0 & 0x00000020 > 0
    }

    /// (Glonass M/K only) P4 data is updated or not
    pub fn glonass_p4_data_updated(&self) -> bool {
        self.0 & 0x00000020 > 0
    }

    /// (Glonass M/K only) P bit 0-1: time offset source
    pub fn glonass_mk_time_offset_source(&self) -> GlonassTimeOffsetSource {
        GlonassTimeOffsetSource::from(self.0)
    }

    /// True if GLO-M/K flag is asserted
    pub fn glonass_mk_type_flag(&self) -> bool {
        self.0 & 0x00000180 == 0x00000080
    }
}

/// [GlonassUpdateValidyInterval] 9-bit binary status mask
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum GlonassUpdateValidyInterval {
    /// Pending update (right now)
    PendingUpdate = 0x00,
    /// 30' validity
    HalfHourValidity = 0x01,
    /// 45' validity
    QuarterHourvalidity = 0x02,
    /// 60' validity
    HourValidity = 0x03,
}

impl From<u32> for GlonassUpdateValidyInterval {
    fn from(val: u32) -> Self {
        match val {
            1 => Self::HalfHourValidity,
            2 => Self::QuarterHourvalidity,
            3 => Self::HourValidity,
            _ => Self::PendingUpdate,
        }
    }
}

/// [GlonassTimeOffsetSource]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum GlonassTimeOffsetSource {
    GroundSource = 0x00,
    GroundClockOnBoardGps = 0x01,
    OnBoardClockGroundGps = 0x02,
    OnBoardClockAndGps = 0x03,
}

impl From<u32> for GlonassTimeOffsetSource {
    fn from(val: u32) -> Self {
        match val {
            1 => Self::GroundClockOnBoardGps,
            2 => Self::OnBoardClockGroundGps,
            3 => Self::OnBoardClockAndGps,
            _ => Self::GroundSource,
        }
    }
}
