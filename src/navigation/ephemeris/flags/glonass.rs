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

/// [GlonassStatus] 9-bit binary status mask
#[derive(Default, Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct GlonassStatus(u32);

impl GlonassStatus {

    /// P1 bits 2-3 : update & valdity interval
    pub fn glonass_p2_update_validity_interval(&self) -> GlonassUpdateValidyInterval {
        GlonassUpdateValidityInterval::from(self.0)
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
#[derive(Default, Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
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

/// [GlonassTimeOffsetSource]
#[derive(Default, Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum GlonassTimeOffsetSource {
    GroundSource = 0x00,
    GroundClockOnBoardGps = 0x01,
    OnBoardClockGroundGps = 0x02,
    OnBoardCloakAndGps = 0x03,
}
