use crate::ionex::Quantized;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuantizedCoordinates {
    /// Quantized latitude
    latitude: Quantized,
    /// Quantized longitude
    longitude: Quantized,
    /// Quantized altitude
    altitude: Quantized,
}

impl QuantizedCoordinates {
    /// Builds new [IonexMapCoordinates] from coordinates expressed in ddeg
    pub fn new(
        lat_ddeg: f64,
        lat_exponent: i8,
        long_ddeg: f64,
        long_exponent: i8,
        alt_km: f64,
        alt_exponent: i8,
    ) -> Self {
        Self {
            latitude: Quantized::new(lat_ddeg, lat_exponent),
            longitude: Quantized::new(long_ddeg, long_exponent),
            altitude: Quantized::new(alt_km, alt_exponent),
        }
    }

    /// Builds new [IonexMapCoordinates] from [Quantized] coordinates
    pub(crate) fn from_quantized(
        latitude: Quantized,
        longitude: Quantized,
        altitude: Quantized,
    ) -> Self {
        Self {
            latitude,
            longitude,
            altitude,
        }
    }

    /// Returns latitude in degrees
    pub fn latitude_ddeg(&self) -> f64 {
        self.latitude.real_value()
    }

    /// Returns longitude in degrees
    pub fn longitude_ddeg(&self) -> f64 {
        self.longitude.real_value()
    }
    /// Returns longitude in kilometers
    pub fn altitude_km(&self) -> f64 {
        self.altitude.real_value()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn quantized_coords() {
        let coords = QuantizedCoordinates::new(1.0, 1, 2.0, 1, 3.0, 1);
        assert_eq!(coords.latitude_ddeg(), 1.0);
        assert_eq!(coords.longitude_ddeg(), 2.0);
        assert_eq!(coords.altitude_km(), 3.0);

        let coords = QuantizedCoordinates::new(1.5, 1, 2.0, 1, 3.12, 2);
        assert_eq!(coords.latitude_ddeg(), 1.5);
        assert_eq!(coords.longitude_ddeg(), 2.0);
        assert_eq!(coords.altitude_km(), 3.12);
    }
}
