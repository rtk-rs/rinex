use bitflags::bitflags;

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags! {
    /// IRNSS orbit health indication
    #[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct IrnssHealth : u8 {
        const UNKNOWN = 0x01;
    }
}
