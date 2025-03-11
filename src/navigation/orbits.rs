//! NAV Orbits description, spanning all revisions and constellations
use super::health;
use bitflags::bitflags;
use num::FromPrimitive;
use std::str::FromStr;

use crate::prelude::ParsingError;

include!(concat!(env!("OUT_DIR"), "/nav_orbits.rs"));

bitflags! {
    #[derive(Default, Debug, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GloStatus: u32 {
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

/// `OrbitItem` item is Navigation ephemeris entry.
/// It is a complex data wrapper, for high level
/// record description, across all revisions and constellations
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum OrbitItem {
    /// unsigned byte
    U8(u8),
    /// signed byte
    I8(i8),
    /// unsigned 32 bit value
    U32(u32),
    /// double precision data
    F64(f64),
    /// GPS/QZSS orbit/sv health indication
    Health(health::Health),
    /// GLO orbit/sv health indication
    GloHealth(health::GloHealth),
    /// GLO NAV4 Orbit7 status mask
    GloStatus(GloStatus),
    /// GEO/SBAS orbit/sv health indication
    GeoHealth(health::GeoHealth),
    /// GAL orbit/sv health indication
    GalHealth(health::GalHealth),
    /// IRNSS orbit/sv health indication
    IrnssHealth(health::IrnssHealth),
}

impl From<u32> for OrbitItem {
    fn from(value: u32) -> Self {
        Self::U32(value)
    }
}

impl From<f64> for OrbitItem {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

impl OrbitItem {
    /// Builds a `OrbitItem` from type descriptor and string content
    pub fn new(
        type_desc: &str,
        content: &str,
        constellation: Constellation,
    ) -> Result<OrbitItem, ParsingError> {
        match type_desc {
            "u8" => {
                // float->unsigned conversion
                let float = f64::from_str(&content.replace('D', "e"))
                    .map_err(|_| ParsingError::OrbitUnsignedData)?;
                Ok(OrbitItem::U8(float as u8))
            },
            "i8" => {
                // float->signed conversion
                let float = f64::from_str(&content.replace('D', "e"))
                    .map_err(|_| ParsingError::OrbitSignedData)?;
                Ok(OrbitItem::I8(float as i8))
            },
            "u32" => {
                // float->signed conversion
                let float = f64::from_str(&content.replace('D', "e"))
                    .map_err(|_| ParsingError::OrbitUnsignedData)?;
                Ok(OrbitItem::U32(float as u32))
            },
            "f64" => {
                let float = f64::from_str(&content.replace('D', "e"))
                    .map_err(|_| ParsingError::OrbitFloatData)?;
                Ok(OrbitItem::F64(float))
            },
            "gloStatus" => {
                // float->unsigned conversion
                let float = f64::from_str(&content.replace('D', "e"))
                    .map_err(|_| ParsingError::OrbitUnsignedData)?;
                let unsigned = float as u32;
                let status = GloStatus::from_bits(unsigned).unwrap_or(GloStatus::empty());
                Ok(OrbitItem::GloStatus(status))
            },
            "health" => {
                // float->unsigned conversion
                let float = f64::from_str(&content.replace('D', "e"))
                    .map_err(|_| ParsingError::OrbitUnsignedData)?;
                let unsigned = float as u32;
                match constellation {
                    Constellation::GPS | Constellation::QZSS => {
                        let flag: health::Health =
                            FromPrimitive::from_u32(unsigned).unwrap_or(health::Health::default());
                        Ok(OrbitItem::Health(flag))
                    },
                    Constellation::Glonass => {
                        let flag: health::GloHealth = FromPrimitive::from_u32(unsigned)
                            .unwrap_or(health::GloHealth::default());
                        Ok(OrbitItem::GloHealth(flag))
                    },
                    Constellation::Galileo => {
                        let flags = health::GalHealth::from_bits(unsigned as u8)
                            .unwrap_or(health::GalHealth::empty());
                        Ok(OrbitItem::GalHealth(flags))
                    },
                    Constellation::IRNSS => {
                        let flag: health::IrnssHealth = num::FromPrimitive::from_u32(unsigned)
                            .unwrap_or(health::IrnssHealth::default());
                        Ok(OrbitItem::IrnssHealth(flag))
                    },
                    c => {
                        if c.is_sbas() {
                            let flag: health::GeoHealth = num::FromPrimitive::from_u32(unsigned)
                                .unwrap_or(health::GeoHealth::default());
                            Ok(OrbitItem::GeoHealth(flag))
                        } else {
                            // Constellation::Mixed will not happen here,
                            // it's always defined in the database
                            unreachable!("unhandled case!");
                        }
                    },
                }
            }, // "health"
            _ => Err(ParsingError::NoNavigationDefinition),
        }
    }
    /// Formats self following RINEX standards,
    /// mainly used when producing a file
    pub fn to_string(&self) -> String {
        match self {
            OrbitItem::U8(n) => format!("{:14.11E}", *n as f64),
            OrbitItem::I8(n) => format!("{:14.11E}", *n as f64),
            OrbitItem::U32(n) => format!("{:14.11E}", *n as f64),
            OrbitItem::F64(f) => format!("{:14.11E}", f),
            OrbitItem::Health(h) => format!("{:14.11E}", h),
            OrbitItem::GloHealth(h) => format!("{:14.11E}", h),
            OrbitItem::GeoHealth(h) => format!("{:14.11E}", h),
            OrbitItem::IrnssHealth(h) => format!("{:14.11E}", h),
            OrbitItem::GalHealth(h) => format!("{:14.11E}", h.bits() as f64),
            OrbitItem::GloStatus(h) => format!("{:14.11E}", h.bits() as f64),
        }
    }
    /// Unwraps OrbitItem as f64
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            OrbitItem::F64(f) => Some(*f),
            OrbitItem::U8(val) => Some(*val as f64),
            OrbitItem::I8(val) => Some(*val as f64),
            OrbitItem::U32(val) => Some(*val as f64),
            _ => None,
        }
    }
    /// Unwraps self as u32 (if possible)
    pub fn as_u32(&self) -> Option<u32> {
        match self {
            OrbitItem::U32(v) => Some(*v),
            OrbitItem::U8(val) => Some(*val as u32),
            OrbitItem::I8(val) => Some(*val as u32),
            OrbitItem::F64(val) => Some(val.round() as u32),
            _ => None,
        }
    }
    /// Unwraps OrbitItem as u8
    pub fn as_u8(&self) -> Option<u8> {
        match self {
            OrbitItem::U8(u) => Some(*u),
            OrbitItem::U32(val) => Some(*val as u8),
            OrbitItem::I8(val) => Some(*val as u8),
            OrbitItem::F64(val) => Some(val.round() as u8),
            _ => None,
        }
    }
    /// Unwraps OrbitItem as i8
    pub fn as_i8(&self) -> Option<i8> {
        match self {
            OrbitItem::I8(i) => Some(*i),
            _ => None,
        }
    }
    /// Unwraps Self as GPS/QZSS orbit Health indication
    pub fn as_gps_health(&self) -> Option<health::Health> {
        match self {
            OrbitItem::Health(h) => Some(h.clone()),
            _ => None,
        }
    }
    /// Unwraps Self as GEO/SBAS orbit Health indication
    pub fn as_geo_health(&self) -> Option<health::GeoHealth> {
        match self {
            OrbitItem::GeoHealth(h) => Some(h.clone()),
            _ => None,
        }
    }
    /// Unwraps Self as GLO orbit Health indication
    pub fn as_glo_health(&self) -> Option<health::GloHealth> {
        match self {
            OrbitItem::GloHealth(h) => Some(h.clone()),
            _ => None,
        }
    }
    /// Unwraps Self as GAL orbit Health indication
    pub fn as_gal_health(&self) -> Option<health::GalHealth> {
        match self {
            OrbitItem::GalHealth(h) => Some(*h),
            _ => None,
        }
    }
    /// Unwraps Self as IRNSS orbit Health indication
    pub fn as_irnss_health(&self) -> Option<health::IrnssHealth> {
        match self {
            OrbitItem::IrnssHealth(h) => Some(h.clone()),
            _ => None,
        }
    }
}

/// Identifies closest (but older) revision contained in NAV database.
/// Closest content (in time) is used during record parsing to identify and sort data.
/// Returns None
/// - if no database entries were found for requested constellation.
///  - or only newer revision exist : we prefer matching on older revisions
pub(crate) fn closest_nav_standards(
    constellation: Constellation,
    revision: Version,
    msg: NavMessageType,
) -> Option<&'static NavHelper<'static>> {
    let database = &NAV_ORBITS;
    // start by trying to locate desired revision.
    // On each mismatch, we decrement and move on to next major/minor combination.
    let (mut major, mut minor): (u8, u8) = revision.into();
    loop {
        // filter on both:
        //  + Exact Constellation
        //  + Exact NavMessageType
        //  + Exact revision we're currently trying to locate
        //    algorithm: decreasing, starting from desired revision
        let items: Vec<_> = database
            .iter()
            .filter(|item| {
                item.constellation == constellation
                    && item.msg == msg
                    && item.version == Version::new(major, minor)
            })
            .collect();

        if items.is_empty() {
            if minor == 0 {
                // we're done with this major
                // -> downgrade to previous major
                //    we start @ minor = 10, which is
                //    larger than most revisions we know
                if major == 0 {
                    // we're done: browsed all possibilities
                    break;
                } else {
                    major -= 1;
                    minor = 10;
                }
            } else {
                minor -= 1;
            }
        } else {
            return Some(items[0]);
        }
    } // loop
    None
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::navigation::NavMessageType;

    #[test]
    fn orbit_database_sanity() {
        for frame in NAV_ORBITS.iter() {
            // Test data fields description
            let constellation = frame.constellation;
            for (key, value) in frame.items.iter() {
                let fake_content: Option<String> = match value {
                    &"f64" => Some(String::from("0.000")), // like we would parse it,
                    &"u32" => Some(String::from("0.000")),
                    &"u8" => Some(String::from("0.000")),
                    &"spare" => None, // such fields are actually dropped
                    _ => None,
                };
                if let Some(content) = fake_content {
                    // Item construction, based on this descriptor, must work.
                    // Like we use it when parsing..
                    let e = OrbitItem::new(value, &content, constellation);
                    assert!(
                        e.is_ok(),
                        "failed to build Orbit Item from (\"{}\", \"{}\", \"{}\")",
                        key,
                        value,
                        constellation,
                    );
                }
            }
        }
    }

    #[test]
    fn nav_standards_finder() {
        // Constellation::Mixed is not contained in db!
        assert_eq!(
            closest_nav_standards(
                Constellation::Mixed,
                Version::default(),
                NavMessageType::LNAV
            ),
            None,
            "Mixed GNSS constellation is or should not exist in the DB"
        );

        // Test existing (exact match) entries
        for (constellation, rev, msg) in [
            (Constellation::GPS, Version::new(1, 0), NavMessageType::LNAV),
            (Constellation::GPS, Version::new(2, 0), NavMessageType::LNAV),
            (Constellation::GPS, Version::new(4, 0), NavMessageType::LNAV),
            (Constellation::GPS, Version::new(4, 0), NavMessageType::CNAV),
            (Constellation::GPS, Version::new(4, 0), NavMessageType::CNV2),
            (
                Constellation::Glonass,
                Version::new(2, 0),
                NavMessageType::LNAV,
            ),
            (
                Constellation::Glonass,
                Version::new(3, 0),
                NavMessageType::LNAV,
            ),
            (
                Constellation::Galileo,
                Version::new(3, 0),
                NavMessageType::LNAV,
            ),
            (
                Constellation::Galileo,
                Version::new(4, 0),
                NavMessageType::INAV,
            ),
            (
                Constellation::Galileo,
                Version::new(4, 0),
                NavMessageType::FNAV,
            ),
            (
                Constellation::QZSS,
                Version::new(3, 0),
                NavMessageType::LNAV,
            ),
            (
                Constellation::QZSS,
                Version::new(4, 0),
                NavMessageType::LNAV,
            ),
            (
                Constellation::QZSS,
                Version::new(4, 0),
                NavMessageType::CNAV,
            ),
            (
                Constellation::QZSS,
                Version::new(4, 0),
                NavMessageType::CNV2,
            ),
            (
                Constellation::BeiDou,
                Version::new(3, 0),
                NavMessageType::LNAV,
            ),
            (
                Constellation::BeiDou,
                Version::new(4, 0),
                NavMessageType::D1,
            ),
            (
                Constellation::BeiDou,
                Version::new(4, 0),
                NavMessageType::D2,
            ),
            (
                Constellation::BeiDou,
                Version::new(4, 0),
                NavMessageType::CNV1,
            ),
            (
                Constellation::BeiDou,
                Version::new(4, 0),
                NavMessageType::CNV2,
            ),
            (
                Constellation::BeiDou,
                Version::new(4, 0),
                NavMessageType::CNV3,
            ),
            (
                Constellation::SBAS,
                Version::new(4, 0),
                NavMessageType::SBAS,
            ),
        ] {
            let found = closest_nav_standards(constellation, rev, msg);
            assert!(
                found.is_some(),
                "should have identified {}:V{} ({}) frame that actually exists in DB",
                constellation,
                rev,
                msg
            );

            let standards = found.unwrap();
            assert!(
                standards.constellation == constellation,
                "bad constellation identified \"{}\", expecting \"{}\"",
                constellation,
                standards.constellation
            );
            assert!(
                standards.version == rev,
                "should have matched {} V{} exactly, because it exists in DB",
                constellation,
                rev,
            );
        }

        // Test cases where the nearest revision is used, not that exact revision
        for (constellation, desired, expected, msg) in [
            (
                Constellation::GPS,
                Version::new(5, 0),
                Version::new(4, 0),
                NavMessageType::LNAV,
            ),
            (
                Constellation::GPS,
                Version::new(4, 1),
                Version::new(4, 0),
                NavMessageType::LNAV,
            ),
            (
                Constellation::Glonass,
                Version::new(3, 4),
                Version::new(3, 0),
                NavMessageType::LNAV,
            ),
        ] {
            let found = closest_nav_standards(constellation, desired, msg);
            assert!(
                found.is_some(),
                "should have converged for \"{}\" V\"{}\" (\"{}\") to nearest frame revision",
                constellation,
                desired,
                msg
            );

            let standards = found.unwrap();

            assert!(
                standards.constellation == constellation,
                "bad constellation identified \"{}\", expecting \"{}\"",
                constellation,
                standards.constellation
            );

            assert!(
                standards.version == expected,
                "closest_nav_standards() converged to wrong revision {}:{}({}) while \"{}\" was expected", 
                constellation,
                desired,
                msg,
                expected);
        }
    }

    #[test]
    fn test_db_item() {
        let e = OrbitItem::U8(10);
        assert_eq!(e.as_u8(), Some(10));

        let e = OrbitItem::F64(10.0);
        assert_eq!(e.as_u8(), Some(10));
        assert_eq!(e.as_u32(), Some(10));
        assert_eq!(e.as_f64(), Some(10.0));

        let e = OrbitItem::U32(1);
        assert_eq!(e.as_u32(), Some(1));
        assert_eq!(e.as_f64(), Some(1.0));
    }

    #[test]
    fn test_orbit_channel_5() {
        let _ = OrbitItem::new("i8", "5.000000000000D+00", Constellation::Glonass).unwrap();
    }
}
