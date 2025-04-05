use bitflags::bitflags;

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags! {
    /// [GeoHealth] SV indication
    #[derive(Default, Debug, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GeoHealth : u32 {
        const GEO_HEALTH_MT17_BIT0 = 0x00000001;
        const GEO_HEALTH_MT17_BIT1 = 0x00000002;
        const GEO_HEALTH_MT17_BIT2 = 0x00000004;
        const GEO_HEALTH_MT17_BIT3 = 0x00000008;
        const GEO_HEALTH_MT17_UNAVAILABLE = 0x00000010;
        const GEO_URA_INDEX_IS_15 = 0x00000020;
    }
}
