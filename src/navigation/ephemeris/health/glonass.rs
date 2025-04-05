use bitflags::bitflags;

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags! {
    /// [GlonassHealth] flag
    #[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GlonassHealth : u32 {
        const UNHEALTHY = 0x00000004;
    }
}

bitflags! {
    /// [GlonassStatus] flag
    #[derive(Default, Debug, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GlonassStatus: u32 {
        const GROUND_GPS_ONBOARD_OFFSET = 0x00000001;
        const ONBOARD_GPS_GROUND_OFFSET = 0x00000002;
        const ONBOARD_OFFSET = 0x00000003;
        const HALF_HOUR_VALIDITY = 0x00000004;
        const THREE_QUARTER_HOUR_VALIDITY = 0x00000006;
        const ONE_HOUR_VALIDITY = 0x00000007;
        const ODD_TIME_INTERVAL = 0x00000008;
        const SAT5_ALMANAC = 0x00000010;
        const DATA_UPDATED = 0x00000020;
        const MK = 0x00000040;
    }
}
