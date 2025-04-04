use num_derive::FromPrimitive;

/// SBAS/GEO orbit health indication
#[derive(Default, Debug, Clone, FromPrimitive, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum GeoHealth {
    #[default]
    Unknown = 0,
    Reserved = 8,
}

impl std::fmt::UpperExp for GeoHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Unknown => 0.fmt(f),
            Self::Reserved => 8.fmt(f),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_geo() {
        assert_eq!(GeoHealth::default(), GeoHealth::Unknown);
        assert_eq!(format!("{:E}", GeoHealth::default()), "0E0");
    }
}
