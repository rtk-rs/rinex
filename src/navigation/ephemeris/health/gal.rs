use bitflags::bitflags;

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags! {
    /// GAL orbit health indication
    #[derive(Debug, Default, Copy, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GalHealth : u32 {
        const E1B_DVS = 0x00000001;
        const E1B_HS0 = 0x00000002;
        const E1B_HS1 = 0x00000004;
        const E5A_DVS = 0x00000008;
        const E5A_HS0 = 0x00000010;
        const E5A_HS1 = 0x00000020;
        const E5B_HS0 = 0x00000040;
        const E5B_HS1 = 0x00000080;
    }
}