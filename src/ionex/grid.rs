use crate::linspace::Linspace;

#[cfg(feature = "serde")]
use serde::Serialize;

/// Reference Grid,
/// defined in terms of Latitude, Longitude and Altitude.
/// If 2D-TEC maps, static altitude is defined, ie.:
/// start = end altitude and spacing = 0.
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Grid {
    /// Latitude
    pub latitude: Linspace,
    /// Longitude
    pub longitude: Linspace,
    /// Altitude
    pub height: Linspace,
}

impl Grid {
    /// Returns true if self is defined for 3D TEC map
    pub fn is_3d_grid(&self) -> bool {
        !self.is_2d_grid()
    }
    /// Returns true if self is defined to 2D TEC maps,
    /// ie.: static altitude ref point with no altitude space
    /// definition.
    pub fn is_2d_grid(&self) -> bool {
        self.height.is_single_point()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_grid() {
        let default = Linspace::default();
        assert_eq!(
            default,
            Linspace {
                start: 0.0,
                end: 0.0,
                spacing: 0.0,
            }
        );
        let grid = Linspace::new(1.0, 10.0, 1.0).unwrap();
        assert_eq!(grid.length(), 10);
        assert!(!grid.is_single_point());
    }
}
