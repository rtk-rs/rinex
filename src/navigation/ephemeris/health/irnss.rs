use num_derive::FromPrimitive;

/// IRNSS orbit health indication
#[derive(Default, Debug, Clone, FromPrimitive, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum IrnssHealth {
    Healthy = 0,
    #[default]
    Unknown = 1,
}

impl std::fmt::UpperExp for IrnssHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Healthy => 0.0_f64.fmt(f),
            Self::Unknown => 1.0_f64.fmt(f),
        }
    }
}
