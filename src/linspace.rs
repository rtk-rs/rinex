use std::ops::Rem;

use crate::prelude::ParsingError;

/// Linear space as used in IONEX or Antenna grid definitions.
/// Linear space starting from `start` ranging to `end` (included).
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Linspace {
    /// First value
    pub start: f64,
    /// Last value (included)
    pub end: f64,
    /// Spacing (increment)
    pub spacing: f64,
}

impl Linspace {
    /// Builds a new Linear space
    pub fn new(start: f64, end: f64, spacing: f64) -> Result<Self, ParsingError> {
        if start == end && spacing == 0.0 {
            Ok(Self {
                start,
                end,
                spacing,
            })
        } else {
            let r = end.rem(start);
            /*
             * End / Start must be multiple of one another
             */
            if r == 0.0 {
                if end.rem(spacing) == 0.0 {
                    Ok(Self {
                        start,
                        end,
                        spacing,
                    })
                } else {
                    Err(ParsingError::BadIonexGridSpecs)
                }
            } else {
                Err(ParsingError::BadIonexGridSpecs)
            }
        }
    }

    /// Returns grid length, in terms of data points
    pub fn length(&self) -> usize {
        (self.end / self.spacing).floor() as usize
    }

    /// Returns true if self is a single point space
    pub fn is_single_point(&self) -> bool {
        (self.end == self.start) && self.spacing == 0.0
    }

    /// Returns nearest lower bound from point p in the [Linspace]
    pub fn nearest_lower(&self, p: f64) -> Option<f64> {
        let mut start = self.start;
        while start < self.end {
            if start > p {
                return Some(start - self.spacing);
            }
            start += self.spacing;
        }

        None
    }

    /// Returns nearest lower bound from point p in the [Linspace]
    pub fn nearest_above(&self, p: f64) -> Option<f64> {
        let lower = self.nearest_lower(p)?;
        Some(lower + self.spacing)
    }
}

impl From<(f64, f64, f64)> for Linspace {
    fn from(tuple: (f64, f64, f64)) -> Self {
        Self {
            start: tuple.0,
            end: tuple.1,
            spacing: tuple.2,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Linspace;

    #[test]
    fn linspace() {
        let linspace = Linspace::new(1.0, 180.0, 1.0).unwrap();
        assert_eq!(linspace.length(), 180);
        assert!(!linspace.is_single_point());

        let linspace = Linspace::new(1.0, 180.0, 0.5).unwrap();
        assert_eq!(linspace.length(), 180 * 2);
        assert!(!linspace.is_single_point());

        let linspace = Linspace::new(350.0, 350.0, 0.0).unwrap();
        assert!(linspace.is_single_point());
    }

    #[test]
    fn latitude_linspace() {
        let linspace = Linspace::new(-87.5, 87.5, 2.5).unwrap();
        assert_eq!(linspace.nearest_lower(-85.0), Some(-85.0));
    }

    #[test]
    fn longitude_linspace() {
        let linspace = Linspace::new(-180.0, 180.0, 5.0).unwrap();
        assert_eq!(linspace.nearest_lower(-179.0), Some(-180.0));
    }
}
