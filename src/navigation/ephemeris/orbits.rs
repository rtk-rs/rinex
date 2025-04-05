//! NAV Orbits description, spanning all revisions and constellations
use std::str::FromStr;

use crate::{
    navigation::{
        ephemeris::health::{
            gal::GalHealth,
            geo::GeoHealth,
            glonass::{GlonassHealth, GlonassStatus},
            gps::{Gpsl1cHealth, Gpsl1l2l5Health},
            irnss::IrnssHealth,
        },
        NavMessageType,
    },
    prelude::ParsingError,
};

#[cfg(feature = "serde")]
use serde::Serialize;

include!(concat!(env!("OUT_DIR"), "/nav_orbits.rs"));

/// [OrbitItem] item is Navigation ephemeris entry.
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
    /// GPS [Gpsl1cHealth] flag
    Gpsl1cHealth(Gpsl1cHealth),
    /// GPS or QZSS [Gpsl1l2l5Health] flag
    Gpsl1l2l5Health(Gpsl1l2l5Health),
    /// SV health
    GeoHealth(GeoHealth),
    /// SV health
    GalHealth(GalHealth),
    /// SV health
    IrnssHealth(IrnssHealth),
    /// SV health
    GlonassHealth(GlonassHealth),
    /// NAV4 Orbit7 status mask
    GlonassStatus(GlonassStatus),
}

impl std::fmt::Display for OrbitItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::U8(val) => write!(f, "{:02x}", val),
            Self::I8(val) => write!(f, "{:02x}", val),
            Self::U32(val) => write!(f, "{:08X}", val),
            Self::F64(val) => write!(f, "{}", val),
            Self::GeoHealth(val) => write!(f, "{:?}", val),
            Self::GalHealth(val) => write!(f, "{:?}", val),
            Self::Gpsl1cHealth(val) => write!(f, "{:?}", val),
            Self::Gpsl1l2l5Health(val) => write!(f, "{:?}", val),
            Self::IrnssHealth(val) => write!(f, "{:?}", val),
            Self::GlonassHealth(val) => write!(f, "{:?}", val),
            Self::GlonassStatus(val) => write!(f, "{:?}", val),
        }
    }
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
    /// Builds a [OrbitItem] from type descriptor and string content
    pub fn new(
        name_str: &str,
        type_str: &str,
        val_str: &str,
        msgtype: &NavMessageType,
        constellation: Constellation,
    ) -> Result<OrbitItem, ParsingError> {

        // make it "rust" compatible
        let float = f64::from_str(&val_str.replace('D', "e"))
            .map_err(|_| ParsingError::OrbitFloatData)?;


        // do not tolerate zero values for native types
        match type_str {
            "u8" | "i8" | "u32" | "f64" => {
                if float == 0.0 {
                    return Err(ParsingError::NavNullOrbit);
                }
            },
        }

        // handle native type right away & exit
        let native_type = match type_str {
            "u8" => {
                let unsigned = float.round() as u8;
                return Ok(OrbitItem::U8(unsigned));
            },

            "i8" => {
                let signed = float.round() as i8;
                return Ok(OrbitItem::U8(signed));
            },

            "u32" => {
                let unsigned = float.round() as u32;
                return Ok(OrbitItem::U8(unsigned));
            },

            "f64" => {
                return Ok(OrbitItem::F64(float));
            },
            _ => {},
        };

        // handle complex types
        match type_str {
            "flag" => {
                // bit flags interpretation
                let unsigned = float.round() as u32;
                
                match name_str {
                    "health" | "satH1"  => {
                        // complex health flag interpretation

                        // handle GEO case
                        if constellation.is_sbas() {
                            match msgtype {
                                NavMessageType::LNAV => {

                                },
                            }
                        }

                        // other cases
                        match (msgtype, constellation) {
                            (NavMessageType::LNAV | NavMessageType::CNAV, Constellation::GPS | Constellation::QZSS) => {
                                let flags = Gpsl1l2l5Health::from_bits(unsigned).unwrap_or_default();
                                Ok(OrbitItem::Gpsl1l2l5Health(flags))
                            },
                            (NavMessageType::LNAV | NavMessageType::CNV2, Constellation::GPS | Constellation::QZSS) => {
                                let flags = Gpsl1cHealth::from_bits(unsigned).unwrap_or_default();
                                Ok(OrbitItem::Gpsl1l2l5Health(flags))
                            },
                            _ => {
                                Err(ParsingError::NavHealthFlagDefinition)
                            }
                        }
                    },
                    _ => {
                        Err(ParsingError::NavFlagsDefinition)
                    },
                }
            },
            _ => {
                // unknown complex type
                Err(ParsingError::NavUnknownComplexType)
            }
        }

            "gloStatus" => {
                let unsigned = float.round() as u32;
                let status = GlonassStatus::from_bits(unsigned).unwrap_or(GlonassStatus::empty());
                Ok(OrbitItem::GlonassStatus(status))
            },


                

                match (msgtype, constellation) {
                    Constellation::Glonass => {
                        let flags = GlonassHealth::from_bits(unsigned).unwrap_or_default();
                        Ok(OrbitItem::GlonassHealth(flags))
                    },
                    Constellation::Galileo => {
                        let flags =
                            GalHealth::from_bits(unsigned).unwrap_or(GalHealth::empty());
                        Ok(OrbitItem::GalHealth(flags))
                    },
                    Constellation::IRNSS => {
                        let flags = IrnssHealth::from_bits(unsigned).unwrap_or_default();
                        Ok(OrbitItem::IrnssHealth(flags))
                    },
                    c => {
                        if c.is_sbas() {
                            let flags = GeoHealth::from_bits(unsigned).unwrap_or_default();
                            Ok(OrbitItem::GeoHealth(flags))
                        } else {
                            // We're left with [Constellation::Mixed] which is not defined in the database.
                            unreachable!("unhandled case!");
                        }
                    },
                }
            }, // "flag(s)"
            _ => Err(ParsingError::NoNavigationDefinition),
        }
    }

    /// True if this [OrbitItem] is a native type
    pub(crate) fn is_native_type(&self) -> bool {
        matches!(*self, 
            OrbitItem::F64(_) 
            | OrbitItem::U8(_)
            | OrbitItem::I8(_)
            | OrbitItem::U32(_)
        )
    }

    /// Unwraps [OrbitItem] as [f64] (always feasible)
    pub fn as_f64(&self) -> f64 {
        match self {
            OrbitItem::F64(f) => *f,
            OrbitItem::U8(val) => *val as f64,
            OrbitItem::I8(val) => *val as f64,
            OrbitItem::U32(val) => *val as f64,
            OrbitItem::GalHealth(flags) => flags.bits() as f64,
            OrbitItem::GpsHealth(flags) => flags.bits() as f64,
            OrbitItem::GeoHealth(flags) => flags.bits() as f64,
            OrbitItem::IrnssHealth(flags) => flags.bits() as f64,
            OrbitItem::GlonassHealth(flags) => flags.bits() as f64,
            OrbitItem::GlonassStatus(status) => status.bits() as f64,
        }
    }

    /// Unwraps [OrbitItem] as [u32] (always feasible)
    pub fn as_u32(&self) -> u32 {
        match self {
            OrbitItem::U32(v) => *v,
            OrbitItem::U8(val) => *val as u32,
            OrbitItem::I8(val) => *val as u32,
            OrbitItem::F64(val) => val.round() as u32,
            OrbitItem::GalHealth(health) => health.bits(),
            OrbitItem::GpsHealth(health) => health.bits(),
            OrbitItem::GeoHealth(health) => health.bits(),
            OrbitItem::IrnssHealth(health) => health.bits(),
            OrbitItem::GlonassHealth(health) => health.bits(),
            OrbitItem::GlonassStatus(status) => status.bits(),
        }
    }

    /// Unwraps [OrbitItem] as [u8] (always feasible)
    pub fn as_u8(&self) -> u8 {
        match self {
            OrbitItem::U32(v) => *v as u8,
            OrbitItem::U8(val) => *val,
            OrbitItem::I8(val) => *val as u8,
            OrbitItem::F64(val) => val.round() as u8,
            OrbitItem::GalHealth(health) => health.bits() as u8,
            OrbitItem::GpsHealth(health) => health.bits() as u8,
            OrbitItem::GeoHealth(health) => health.bits() as u8,
            OrbitItem::IrnssHealth(health) => health.bits() as u8,
            OrbitItem::GlonassHealth(health) => health.bits() as u8,
            OrbitItem::GlonassStatus(status) => status.bits()as u8,
        }
    }

    /// Unwraps [OrbitItem] as [i8] (always feasible)
    pub fn as_i8(&self) -> i8 {
        match self {
            OrbitItem::U32(v) => *v as i8,
            OrbitItem::I8(val) => *val,
            OrbitItem::U8(val) => *val as i8,
            OrbitItem::F64(val) => val.round() as i8,
            OrbitItem::GalHealth(health) => health.bits() as i8,
            OrbitItem::GpsHealth(health) => health.bits() as i8,
            OrbitItem::GeoHealth(health) => health.bits() as i8,
            OrbitItem::IrnssHealth(health) => health.bits() as i8,
            OrbitItem::GlonassHealth(health) => health.bits() as i8,
            OrbitItem::GlonassStatus(status) => status.bits() as i8,
        }
    }

    /// Unwraps Self as [GpsHealth] flag (if feasible)
    pub fn as_gps_health_flag(&self) -> Option<GpsHealth> {
        match self {
            OrbitItem::GpsHealth(h) => Some(h.clone()),
            _ => None,
        }
    }

    /// Unwraps Self as [GeoHealth] flag (if feasible)
    pub fn as_geo_health_flag(&self) -> Option<GeoHealth> {
        match self {
            OrbitItem::GeoHealth(h) => Some(h.clone()),
            _ => None,
        }
    }

    /// Unwraps Self as [GlonassHealth] flag (if feasible)
    pub fn as_glonass_health_flag(&self) -> Option<GlonassHealth> {
        match self {
            OrbitItem::GlonassHealth(h) => Some(h.clone()),
            _ => None,
        }
    }

    /// Unwraps Self as [GalHealth] flag (if feasible)
    pub fn as_galileo_health_flag(&self) -> Option<GalHealth> {
        match self {
            OrbitItem::GalHealth(h) => Some(*h),
            _ => None,
        }
    }

    /// Unwraps Self as [IrnssHealth] flag (if feasible)
    pub fn as_irnss_health_flag(&self) -> Option<IrnssHealth> {
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
        assert_eq!(e.as_u8(), 10);

        let e = OrbitItem::F64(10.0);
        assert_eq!(e.as_u8(), 10);
        assert_eq!(e.as_u32(), 10);
        assert_eq!(e.as_f64(), 10.0);

        let e = OrbitItem::U32(1);
        assert_eq!(e.as_u32(), 1);
        assert_eq!(e.as_f64(), 1.0);
    }

    #[test]
    fn test_orbit_channel_5() {
        let _ = OrbitItem::new("i8", "5.000000000000D+00", Constellation::Glonass).unwrap();
    }
}
