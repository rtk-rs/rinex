use bitflags::bitflags;

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags! {
    /// Glonass SV health flags
    #[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GlonassHealth :u32 {
        const UNHEALTHY = 0x04;
    }
}

bitflags! {
    /// Glonass SV status flags
    #[derive(Default, Debug, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GlonassStatus: u32 {
        const GROUND_GPS_ONBOARD_OFFSET = 0x01;
        const ONBOARD_GPS_GROUND_OFFSET = 0x02;
        const ONBOARD_OFFSET = 0x03;
        const HALF_HOUR_VALIDITY = 0x04;
        const THREE_QUARTER_HOUR_VALIDITY = 0x06;
        const ONE_HOUR_VALIDITY = 0x07;
        const ODD_TIME_INTERVAL = 0x08;
        const SAT5_ALMANAC = 0x10;
        const DATA_UPDATED = 0x20;
        const MK = 0x40;
    }
}
