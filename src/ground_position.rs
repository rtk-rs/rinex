use dms_coordinates::DMS;
use map_3d::{deg2rad, ecef2geodetic, geodetic2ecef, rad2deg, Ellipsoid};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "qc")]
use maud::{html, Markup, Render};

#[derive(Default, Copy, Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GroundPosition(f64, f64, f64);

impl From<(f64, f64, f64)> for GroundPosition {
    fn from(xyz: (f64, f64, f64)) -> Self {
        Self(xyz.0, xyz.1, xyz.2)
    }
}

impl From<GroundPosition> for (f64, f64, f64) {
    fn from(val: GroundPosition) -> Self {
        (val.0, val.1, val.2)
    }
}

impl GroundPosition {
    /// Builds Self from ECEF WGS84 coordinates
    pub fn from_ecef_wgs84(pos: (f64, f64, f64)) -> Self {
        Self(pos.0, pos.1, pos.2)
    }
    /// Builds Self from Geodetic coordinates in ddeg
    pub fn from_geodetic(pos: (f64, f64, f64)) -> Self {
        let (x, y, z) = pos;
        let (x, y, z) = geodetic2ecef(deg2rad(x), deg2rad(y), deg2rad(z), Ellipsoid::WGS84);
        Self(x, y, z)
    }
    /// Converts Self to ECEF WGS84
    pub fn to_ecef_wgs84(&self) -> (f64, f64, f64) {
        (self.0, self.1, self.2)
    }
    /// Converts Self to geodetic coordinates in ddeg
    pub fn to_geodetic(&self) -> (f64, f64, f64) {
        let (x, y, z) = (self.0, self.1, self.2);
        let (lat, lon, alt) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
        (rad2deg(lat), rad2deg(lon), alt)
    }
    /// Returns position altitude
    pub fn altitude(&self) -> f64 {
        self.to_geodetic().2
    }
}

impl std::fmt::Display for GroundPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "WGS84 ({}m {}m {}m)", self.0, self.1, self.2)
    }
}

/*
 * RINEX compatible formatting
 */
impl std::fmt::UpperHex for GroundPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:14.4}{:14.4}{:14.4}", self.0, self.1, self.2)
    }
}

#[cfg(feature = "qc")]
impl Render for GroundPosition {
    fn render(&self) -> Markup {
        let ecef = (self.0, self.1, self.2);
        let geo = self.to_geodetic();
        html! {
            table {
                tr {
                    th {
                        "ECEF (WGS84)"
                    }
                }
                tr {
                    th {
                        "X"
                    }
                    td {
                        (format!("{:.3} m", ecef.0))
                    }
                    th {
                        "Y"
                    }
                    td {
                        (format!("{:.3} m", ecef.1))
                    }
                    th {
                       "Z"
                    }
                    td {
                        (format!("{:.3} m", ecef.2))
                    }
                }
                tr {
                    th {
                        "GEO"
                    }
                }
                tr {
                    th {
                        "Latitude"
                    }
                    td {
                        (format!("{:.6}°", geo.0))
                    }
                    th {
                        "Longitude"
                    }
                    td {
                        (format!("{:.6}°", geo.1))
                    }
                    th {
                        "Altitude"
                    }
                    td {
                        (format!("{:.3} m", geo.2))
                    }
                }
                tr {
                    th {
                        "DMS"
                    }
                    td {
                        (DMS::from_ddeg_latitude(geo.0).to_string())
                    }
                    th {
                        "DMS"
                    }
                    td {
                        (DMS::from_ddeg_longitude(geo.1).to_string())
                    }
                }
            }
        }
    }
}
