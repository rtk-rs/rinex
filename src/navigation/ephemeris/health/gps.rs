use bitflags::bitflags;

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags! {
    /// GNSS / GPS orbit health indication
    #[derive(Default, Debug, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GpsHealth: u32 {
        const L1_HEALTHY = 0x01;
        const L2_HEALTHY = 0x02;
        const L5_HEALTHY = 0x04;
    }
}
