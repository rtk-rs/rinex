use crate::{ionex::IonosphereParameters, prelude::Epoch};

/// IPPCoordinates; Ionosphere Pierce Point Coordinates,
/// describe the location in space-time a signal pierced
/// the Ionosphere layer.
pub struct IPPCoordinates {
    /// Instant the signal pierced Ionosphere, expressed as [Epoch]
    pub epoch: Epoch,
    /// Latitude (radians)
    pub latitude_rad: f64,
    /// Longitude (radians)
    pub longitude_rad: f64,
}

impl IPPCoordinates {
    /// Deduce [IonosphereParameters] from [IPPCoordinates]
    pub fn to_parameters_model(&self) -> IonosphereParameters {
        IonosphereParameters {
            amplitude_s: 0.0,
            period_s: 0.0,
            phase_rad: 0.0,
            slant_factor: 0.0,
        }
    }
}
