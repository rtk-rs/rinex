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

/// Known [BdsSatelliteType]s
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum BdsSatelliteType {
    /// BDS-GEO Satellite
    GEO = 1,
    /// BDS IGSO Satellite
    IGSO = 2,
    /// BDS MEO Satellite
    MEO = 3,
    /// Reserved value
    Reserved = 0,
}

impl From<u32> for BdsSatelliteType {
    fn from(val: u32) -> Self {
        match val {
            1 => BdsSatelliteType::GEO,
            2 => BdsSatelliteType::IGSO,
            3 => BdsSatelliteType::MEO,
            _ => BdsSatelliteType::Reserved,
        }
    }
}

impl Into<u32> for BdsSatelliteType {
    fn into(self) -> u32 {
        match self {
            Self::GEO => 1,
            Self::IGSO => 2,
            Self::MEO => 3,
            Self::Reserved => 0,
        }
    }
}

/// Modern [BdsHealth] flag
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum BdsHealth {
    /// Healthy Satellite
    Healthy = 0,
    /// Unhealthy or in-testing Satellite
    UnhealthyTesting = 1,
    /// Reserved value
    Reserved = 2,
}

impl From<u32> for BdsHealth {
    fn from(val: u32) -> Self {
        match val {
            0 => Self::Healthy,
            1 => Self::UnhealthyTesting,
            _ => Self::Reserved,
        }
    }
}

impl Into<u32> for BdsHealth {
    fn into(self) -> u32 {
        match self {
            Self::Healthy => 0,
            Self::UnhealthyTesting => 1,
            Self::Reserved => 2,
        }
    }
}

bitflags! {
    /// [BdsB1cIntegrity] flag
    #[derive(Debug, Default, Copy, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct BdsB1cIntegrity: u32 {
        const B1C_AIF_INTEGRITY = 0x00000001;
        const B1C_SIF_INTEGRITY = 0x00000002;
        const B1C_DIF_INTEGRITY = 0x00000004;
    }
}

bitflags! {
    /// [BdsB1cIntegrity] flag
    #[derive(Debug, Default, Copy, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct BdsB2aB1cIntegrity: u32 {
        const B1C_AIF_INTEGRITY = 0x00000001;
        const B1C_SIF_INTEGRITY = 0x00000002;
        const B1C_DIF_INTEGRITY = 0x00000004;
        const B2A_AIF_INTEGRITY = 0x00000008;
        const B2A_SIF_INTEGRITY = 0x00000010;
    }
}

bitflags! {
    /// [BdsB1cIntegrity] flag
    #[derive(Debug, Default, Copy, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct BdsB2bIntegrity: u32 {
        const B2B_AIF_INTEGRITY = 0x00000001;
        const B2B_SIF_INTEGRITY = 0x00000002;
        const B2B_DIF_INTEGRITY = 0x00000004;
    }
}
