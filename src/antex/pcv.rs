//! Antenna Phase Center Variations
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown pcv code \"{0}\"")]
    UnknownPcv(String),
}

/// Antenna Phase Center Variation types
#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Pcv {
    /// PCV is absolute
    #[default]
    Absolute,
    /// PCV is relative to attached reference
    Relative(String),
}

impl std::str::FromStr for Pcv {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        if content.eq("A") {
            Ok(Self::Absolute)
        } else if content.eq("R") {
            Ok(Self::Relative("AOAD/M_T".to_string()))
        } else {
            Err(Error::UnknownPcv(content.to_string()))
        }
    }
}

impl Pcv {
    pub fn is_relative(&self) -> bool {
        matches!(self, Self::Relative(_))
    }
    pub fn is_absolute(&self) -> bool {
        !self.is_relative()
    }
    pub fn with_relative_type(&self, t: &str) -> Self {
        let mut s = self.clone();
        if s.is_relative() {
            s = Self::Relative(t.to_string())
        }
        s
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_pcv() {
        assert_eq!(Pcv::default(), Pcv::Absolute);
        assert!(Pcv::Absolute.is_absolute());
        assert!(!Pcv::Relative(String::from("AOAD/M_T")).is_absolute());

        let pcv = Pcv::from_str("A");
        assert!(pcv.is_ok());
        let pcv = pcv.unwrap();
        assert_eq!(pcv, Pcv::Absolute);

        let pcv = Pcv::from_str("R");
        assert!(pcv.is_ok());
        let pcv = pcv.unwrap();
        assert_eq!(pcv, Pcv::Relative(String::from("AOAD/M_T")));
    }
}
