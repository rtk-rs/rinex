use bitflags::bitflags;
use num_derive::FromPrimitive;

#[cfg(feature = "serde")]
use serde::Serialize;

/// GLO orbit health indication
#[derive(Default, Debug, Clone, FromPrimitive, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum GlonassHealth {
    Healthy = 0,
    #[default]
    Unhealthy = 4,
}

impl std::fmt::UpperExp for GlonassHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Healthy => 0_0_f64.fmt(f),
            Self::Unhealthy => 4.0_f64.fmt(f),
        }
    }
}

bitflags! {
    #[derive(Default, Debug, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GlonassStatus: u32 {
        const GROUND_GPS_ONBOARD_OFFSET = 0x01;
        const ONBOARD_GPS_GROUND_OFFSET = 0x02;
        const ONBOARD_OFFSET = 0x03;
        const HALF_HOUR_VALIDITY = 0x04;
        const THREE_QUARTER_HOUR_VALIDITY = 0x06;
        const ONE_HOUR_VALIDITY = 0x07;
        const ODD_TIME_INTERVAL = 0x08;
        const SAT5_ALMANAC = 0x10;
        const DATA_UPDATED = 0x20;
        const MK = 0x40;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_glo() {
        assert_eq!(GlonassHealth::default(), GlonassHealth::Unhealthy);
        assert_eq!(format!("{:E}", GlonassHealth::default()), "4E0");
    }
}
