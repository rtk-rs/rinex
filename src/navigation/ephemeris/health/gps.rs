use num_derive::FromPrimitive;

/// GNSS / GPS orbit health indication
#[derive(Default, Debug, Clone, FromPrimitive, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum GpsHealth {
    #[default]
    Unhealthy = 0,
    L1Healthy = 1,
    L2Healthy = 2,
    L1L2Healthy = 3,
    L5Healthy = 4,
    L1L5Healthy = 5,
    L2L5Healthy = 6,
    L1L2L5Healthy = 7,
}

impl std::fmt::UpperExp for GpsHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Unhealthy => 0.0_f64.fmt(f),
            Self::L1Healthy => 1.0_f64.fmt(f),
            Self::L2Healthy => 2.0_f64.fmt(f),
            Self::L1L2Healthy => 3.0_f64.fmt(f),
            Self::L5Healthy => 4.0_f64.fmt(f),
            Self::L1L5Healthy => 5.0_f64.fmt(f),
            Self::L2L5Healthy => 6.0_f64.fmt(f),
            Self::L1L2L5Healthy => 7.0_f64.fmt(f),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gps() {
        assert_eq!(GpsHealth::default(), GpsHealth::Unhealthy);
        assert_eq!(format!("{:E}", GpsHealth::default()), "0E0");
    }
}
