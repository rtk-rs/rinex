use bitflags::bitflags;

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags! {
    /// GEO / SBAS SV health flags
    #[derive(Default, Debug, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GeoHealth: u8 {
        const RESERVED = 0x08;
    }
}
