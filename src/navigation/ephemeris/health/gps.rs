use bitflags::bitflags;

#[cfg(feature = "serde")]
use serde::Serialize;

/// [Gpsl1l2l5Health] flag as per the LNAV historical frame.
#[derive(Default, Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Gpsl1l2l5Health(u32);

impl From<u32> for Gpsl1l2l5Health {
    fn from(value: u32) -> Self {
        Self.0 = value;
    }
}

impl Gpsl1l2l5Health {
    /// Satellite totally healthy
    pub fn healthy(&self) -> bool {
        self.0 == 0
    }

    /// L1 signal problem
    pub fn l1_unhealthy(&self) -> bool {
        self.0 & 0x00000001 > 0
    }

    /// L2 signal problem
    pub fn l2_unhealthy(&self) -> bool {
        self.0 & 0x00000002 > 0
    }

    /// L5 signal problem
    pub fn l5_unhealthy(&self) -> bool {
        self.0 & 0x00000004 > 0
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
