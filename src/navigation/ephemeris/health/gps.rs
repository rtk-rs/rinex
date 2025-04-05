use bitflags::bitflags;

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags! {
    /// [Gpsl1l2l5Health] flag as per the LNAV historical frame.
    #[derive(Default, Debug, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct Gpsl1l2l5Health : u32 {
        const L1_HEALTHY = 0x00000001;
        const L2_HEALTHY = 0x00000002;
        const L5_HEALTHY = 0x00000004;
    }
}
    
bitflags! {
    /// [Gpsl1cHealth] L1 C/A sanity flag 
    #[derive(Default, Debug, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct Gpsl1cHealth : u32 {
        const UNHEALTHY = 0x00000001;
    }
}
