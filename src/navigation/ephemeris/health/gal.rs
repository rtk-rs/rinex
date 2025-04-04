use bitflags::bitflags;

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags! {
    /// GAL orbit health indication
    #[derive(Debug, Default, Copy, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GalHealth: u8 {
        const E1B_DVS = 0x01;
        const E1B_HS0 = 0x02;
        const E1B_HS1 = 0x04;
        const E5A_DVS = 0x08;
        const E5A_HS0 = 0x10;
        const E5A_HS1 = 0x20;
        const E5B_HS0 = 0x40;
        const E5B_HS1 = 0x80;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_gal() {
        assert_eq!(GalHealth::default(), GalHealth::empty());
    }
}
