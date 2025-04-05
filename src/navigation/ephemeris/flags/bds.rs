use bitflags::bitflags;

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags! {
    /// [BdsSatH1] navigation flag
    #[derive(Debug, Default, Copy, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct BdsSatH1: u32 {
        const UNHEALTHY = 0x00000001;
    }
}
