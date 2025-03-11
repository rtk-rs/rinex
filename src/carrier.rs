//! Carrier channels and associated methods
use crate::prelude::{Constellation, Error, Observable, ParsingError, SV};

lazy_static! {
    pub(crate) static ref KNOWN_CODES: Vec<&'static str> = vec![
        "1A", "1B", "1C", "1D", "1E", "1L", "1M", "1P", "1S", "1W", "1X", "1Z", "2C", "2D", "2L",
        "2M", "2P", "2S", "2W", "3I", "3X", "3Q", "4A", "4B", "4X", "5A", "5B", "5C", "5D", "5I",
        "5P", "5Q", "5X", "6A", "6B", "6C", "6D", "6E", "6I", "6P", "6Q", "6X", "6Z", "7D", "7I",
        "7P", "7Q", "7X", "7Z", "8D", "8P", "8I", "8Q", "8X", "9A", "9B", "9C", "9X",
    ];
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Carrier {
    /// L1 (GPS, SBAS, QZSS)
    #[default]
    L1,
    /// L2 (GPS, QZSS)
    L2,
    /// L5 (GPS, SBAS), QZSS
    L5,
    /// L6 (LEX) QZSS
    L6,
    /// Glonass channel 1 with possible offset
    G1(Option<i8>),
    /// Glonass G1a
    G1a,
    /// Glonass channel 2 with possible offset
    G2(Option<i8>),
    /// Glonass G2a
    G2a,
    /// Glonass channel 3
    G3,
    /// E1 (Galileo)
    E1,
    /// E5 (Galileo)
    E5,
    /// E5a (Galileo)
    E5a,
    /// E5b (Galileo)
    E5b,
    /// E6 (Galileo military)
    E6,
    /// B1 (BDS)
    B1I,
    /// B1A (BDS)
    B1A,
    /// B1C (BDS)
    B1C,
    /// B2 (BDS)
    B2,
    /// B2i: BeiDou 2i
    B2I,
    /// B2a: BeiDou 2A
    B2A,
    /// B2b: BeiDou 2b
    B2B,
    /// B3
    B3,
    /// B3A
    B3A,
    /// IRNSS S
    S,
    /// DORIS S1 Frequency
    S1,
    /// DORIS U2 Frequency
    U2,
}

impl std::fmt::Display for Carrier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::L1 => write!(f, "L1"),
            Self::L2 => write!(f, "L2"),
            Self::L5 => write!(f, "L5"),
            Self::G1(_) | Self::G1a => write!(f, "L1"),
            Self::G2(_) | Self::G2a => write!(f, "L2"),
            Self::L6 => write!(f, "L6"),
            Self::G3 => write!(f, "L3"),
            Self::E1 => write!(f, "E1"),
            Self::E5 | Self::E5a | Self::E5b => write!(f, "E5"),
            Self::E6 => write!(f, "E6"),
            Self::S => write!(f, "S"),
            // B1
            Self::B1I => write!(f, "B1I"),
            Self::B1C => write!(f, "B1C"),
            Self::B1A => write!(f, "B1A"),
            // B2
            Self::B2 => write!(f, "B2"),
            Self::B2A => write!(f, "B2A"),
            Self::B2B => write!(f, "B2B"),
            Self::B2I => write!(f, "B2I"),
            // B3
            Self::B3 => write!(f, "B3"),
            Self::B3A => write!(f, "B3A"),
            // DORIS
            Self::S1 => write!(f, "S1"),
            Self::U2 => write!(f, "U2"),
        }
    }
}

impl std::str::FromStr for Carrier {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let content = s.to_uppercase();
        let content = content.trim();
        if content.eq("L1") {
            Ok(Self::L1)
        } else if content.eq("L2") {
            Ok(Self::L2)
        } else if content.eq("L5") {
            Ok(Self::L5)
        } else if content.eq("L6") {
            Ok(Self::L6)
        } else if content.eq("E1") {
            Ok(Self::E1)
        } else if content.eq("E5") {
            Ok(Self::E5)
        } else if content.eq("E6") {
            Ok(Self::E6)
        /*
         * Glonass
         */
        } else if content.eq("G1") {
            Ok(Self::G1(None))
        } else if content.eq("G1A") {
            Ok(Self::G1a)
        } else if content.eq("G2") {
            Ok(Self::G2(None))
        } else if content.eq("G2A") {
            Ok(Self::G2a)
        /*
         * BeiDou
         */
        } else if content.eq("B1I") {
            Ok(Self::B1I)
        } else if content.eq("B1A") {
            Ok(Self::B1A)
        } else if content.eq("B1C") {
            Ok(Self::B1C)
        } else if content.eq("B2") {
            Ok(Self::B2)
        } else if content.eq("B2I") {
            Ok(Self::B2I)
        } else if content.eq("B2B") {
            Ok(Self::B2B)
        } else if content.eq("B3") {
            Ok(Self::B3)
        } else if content.eq("B3A") {
            Ok(Self::B3A)
        /*
         * DORIS
         */
        } else if content.eq("S1") {
            Ok(Self::S1)
        } else if content.eq("U2") {
            Ok(Self::U2)
        } else {
            Err(ParsingError::ObservableParsing)
        }
    }
}

impl Carrier {
    /// Returns frequency associated to this channel in MHz
    pub fn frequency(&self) -> f64 {
        self.frequency_mhz() * 1.0E6
    }
    pub fn frequency_mhz(&self) -> f64 {
        match self {
            Self::L1 | Self::E1 | Self::B1A | Self::B1C => 1575.42_f64,
            Self::L2 => 1227.60_f64,
            Self::L6 | Self::E6 => 1278.750_f64,
            Self::L5 | Self::E5a | Self::B2A => 1176.45_f64,
            Self::E5 | Self::B2 => 1191.795_f64,
            Self::E5b | Self::B2I | Self::B2B => 1207.140_f64,
            /*
             * IRNSS
             */
            Self::S => 2492.028_f64,
            /*
             * Glonass
             */
            Self::G1a => 1600.995_f64,
            Self::G1(None) => 1602.0_f64,
            Self::G1(Some(c)) => 1602.0_f64 + (*c as f64 * 9.0 / 16.0),
            Self::G2a => 1248.06_f64,
            Self::G2(None) => 1246.06_f64,
            Self::G2(Some(c)) => 1246.06_f64 + (*c as f64 * 7.0 / 16.0),
            Self::G3 => 1202.025_f64,
            /*
             * BeiDou
             */
            Self::B1I => 1561.098_f64,
            Self::B3 | Self::B3A => 1268.520_f64,
            /*
             * DORIS
             */
            Self::S1 => 2036.25,
            Self::U2 => 401.25,
        }
    }

    /// True if this [Carrier] matches a L1 pivot carrier
    /// used in carrier recombinations
    #[cfg(feature = "obs")]
    pub(crate) fn is_l1_pivot(&self) -> bool {
        matches!(self, Self::L1 | Self::E1 | Self::B1A | Self::B1C)
    }

    /// Returns carrier wavelength
    pub fn wavelength(&self) -> f64 {
        299_792_458.0_f64 / self.frequency()
    }

    /// Returns channel bandwidth in MHz.
    pub fn bandwidth_mhz(&self) -> f64 {
        match self {
            Self::L1 | Self::G1(_) | Self::G1a | Self::E1 => 15.345_f64,
            Self::L2 | Self::G2(_) | Self::G2a => 11.0_f64,
            Self::L5 | Self::E5 | Self::E5a | Self::E5b => 12.5_f64,
            Self::G3 => todo!("G3 bandwidth is not known to this day"),
            Self::E6 => todo!("E6 bandwidth is not known to this day"),
            Self::L6 => todo!("L6 bandwidth is not known to this day"),
            Self::S => todo!("S bandwidth is not known to this day"),
            Self::B1I | Self::B1A | Self::B1C => todo!("B1X bandwidth is not known to this day"),
            Self::B2 | Self::B2A | Self::B2I | Self::B2B => {
                todo!("B2X bandwidth is not known to this day")
            },
            Self::B3 | Self::B3A => todo!("B3X bandwidth is not known to this day"),
            Self::S1 => panic!("DORIS signal bandwidth"),
            Self::U2 => panic!("DORIS signal bandwidth"),
        }
    }
    ///// Returns the code length (signal period) expressed in seconds,
    ///// for a signal carrier of given constellation. This is mostly used
    ///// in fractional pseudo range determination.
    //pub fn code_length(&self, constellation: Constellation) -> f64 {
    //    match constellation {
    //        Constellation::GPS => {
    //            match self {
    //                Self::L1 => 20.0E-3,
    //                Self::L2 => 1.0_f64, //TODO
    //                Self::L5 => 1.0E-3,
    //                _ => 1.0_f64, // does not apply
    //            },
    //        },
    //        Constellation::QZSS => {
    //            match self {
    //                Self::L1 => 1.0E-3,
    //                Self::L2 => 20.0E-3,
    //                Self::L5 => 1.0E-3, //TODO
    //                Self::L6 => 10.0E-3,
    //                _ => 1.0_f64, // does not apply
    //            },
    //        },
    //        Constellation::GEO | Constellation::SBAS(_) => {
    //            match self {
    //                Self::L1 => 20.0E-3,
    //                Self::L5 => 1.0E-3,
    //                _ => 1.0_f64, // does not apply
    //        },
    //        Constellation::BeiDou => {
    //            match self {
    //                Self::B1A => 1.0_f64, //TODO
    //                Self::B1I => 1.0E-3,
    //                Self::B1C => 1.0_f64, //TODO
    //                Self::B2 => 1.0_f64, //TODO
    //                Self::B2A => 10.0E-3,
    //                Self::B2I => 1.0_f64, //TODO
    //                Self::B2B => 1.0_f64, //TODO
    //                Self::B3 => 1.0_f64, //TODO
    //                Self::B3A => 1.0_f64, //TODO
    //                _ => 1.0_f64, // does not apply
    //            }
    //        },
    //        Constellation::Galileo => {
    //            match self {
    //                Self::E1 => 4.0E-3,
    //                Self::E5 => 10.0E-3,
    //                Self::E5a => 10.0E-3,
    //                Self::E5b => 10.0E-3,
    //                Self::E6 => 50.0E-3,
    //                _ => 1.0_f64, // does not apply
    //            }
    //        },
    //        Constellation::Glonass => {
    //            match self {
    //                Self::G1 | Self::G1(_) => 1.0_f64, //TODO
    //                Self::G2 | Self::G2(_) => 1.0_f64, //TODO
    //                Self::G3 => 1.0_f64, //TODO
    //                _ => 1.0_f64, // does not apply
    //            }
    //        },
    //        Constellation::IRNSS => {
    //            match self {
    //                Self::S => 1.0_f64, // TODO
    //                _ => 1.0_f64, // does not apply
    //            }
    //        },
    //    }
    //}
    /// Converts to exact Glonass carrier
    pub fn with_glonass_offset(&self, offset: i8) -> Self {
        match self {
            Self::L1 => Self::G1(Some(offset)),
            Self::L2 => Self::G2(Some(offset)),
            other => *other,
        }
    }
    pub(crate) fn gpsl1_codes() -> [&'static str; 40] {
        [
            "C1", "L1", "D1", "S1", "P1", "C1C", "L1C", "D1C", "S1C", "C1S", "L1S", "D1S", "S1S",
            "C1L", "L1L", "D1L", "S1L", "C1X", "L1X", "D1X", "S1X", "C1P", "L1P", "D1P", "S1P",
            "C1W", "L1W", "D1W", "S1W", "C1Y", "L1Y", "D1Y", "S1Y", "C1M", "L1M", "D1M", "S1M",
            "L1N", "D1N", "S1N",
        ]
    }
    pub(crate) fn gpsl2_codes() -> [&'static str; 44] {
        [
            "C2", "L2", "D2", "S2", "P2", "C2C", "L2C", "D2C", "S2C", "C2D", "L2D", "D2D", "S2D",
            "C2S", "L2S", "D2S", "S2S", "C2L", "L2L", "D2L", "S2L", "C2X", "L2X", "D2X", "S2X",
            "C2P", "L2P", "D2P", "S2P", "C2W", "L2W", "D2W", "S2W", "C2Y", "L2Y", "D2Y", "S2Y",
            "C2M", "L2M", "D2M", "S2M", "L2N", "D2N", "S2N",
        ]
    }
    pub(crate) fn gpsl5_codes() -> [&'static str; 16] {
        [
            "C5", "L5", "D5", "S5", "C5I", "L5I", "D5I", "S5I", "C5Q", "L5Q", "D5Q", "S5Q", "C5X",
            "L5X", "D5X", "S5X",
        ]
    }
    fn from_gps_observable(obs: &Observable) -> Result<Self, Error> {
        match obs {
            Observable::PhaseRange(code)
            | Observable::Doppler(code)
            | Observable::SSI(code)
            | Observable::PseudoRange(code) => {
                let code = code.as_str();
                if Self::gpsl1_codes().contains(&code) {
                    Ok(Self::L1)
                } else if Self::gpsl2_codes().contains(&code) {
                    Ok(Self::L2)
                } else if Self::gpsl5_codes().contains(&code) {
                    Ok(Self::L5)
                } else {
                    Err(Error::UnknownGPSObservable)
                }
            },
            _ => Err(Error::UnknownGPSObservable),
        }
    }
    pub(crate) fn g1_codes() -> [&'static str; 12] {
        [
            "C1", "L1", "D1", "S1", "C1C", "L1C", "D1C", "S1C", "C1P", "L1P", "D1P", "S1P",
        ]
    }
    pub(crate) fn g1a_codes() -> [&'static str; 12] {
        [
            "C4A", "L4A", "D4A", "S4A", "C4B", "L4B", "D4B", "S4B", "C4X", "L4X", "D4X", "S4X",
        ]
    }
    pub(crate) fn g2_codes() -> [&'static str; 12] {
        [
            "C2", "L2", "D2", "S2", "C2C", "L2C", "D2C", "S2C", "C2P", "L2P", "D2P", "S2P",
        ]
    }
    pub(crate) fn g2a_codes() -> [&'static str; 12] {
        [
            "C6A", "L6A", "D6A", "S6A", "C6B", "L6B", "D6B", "S6B", "C6X", "L6X", "D6X", "S6X",
        ]
    }
    pub(crate) fn g3_codes() -> [&'static str; 16] {
        [
            "C3", "L3", "D3", "S3", "C3I", "L3I", "D3I", "S3I", "C3Q", "L3Q", "D3Q", "S3Q", "C3X",
            "L3X", "D3X", "S3X",
        ]
    }
    fn from_glo_observable(obs: &Observable) -> Result<Self, Error> {
        match obs {
            Observable::PhaseRange(code)
            | Observable::Doppler(code)
            | Observable::SSI(code)
            | Observable::PseudoRange(code) => {
                let code = code.as_str();
                if Self::g1_codes().contains(&code) {
                    Ok(Self::G1(None))
                } else if Self::g1a_codes().contains(&code) {
                    Ok(Self::G1a)
                } else if Self::g2_codes().contains(&code) {
                    Ok(Self::G2(None))
                } else if Self::g2a_codes().contains(&code) {
                    Ok(Self::G2a)
                } else if Self::g3_codes().contains(&code) {
                    Ok(Self::G3)
                } else {
                    Err(Error::UnknownGlonassObservable)
                }
            },
            _ => Err(Error::UnknownGlonassObservable),
        }
    }
    pub(crate) fn e1_codes() -> [&'static str; 24] {
        [
            "C1", "L1", "D1", "S1", "C1A", "L1A", "D1A", "S1A", "C1B", "L1B", "D1B", "S1B", "C1C",
            "L1C", "D1C", "S1C", "C1X", "L1X", "D1X", "S1X", "C1Z", "L1Z", "D1Z", "S1Z",
        ]
    }
    pub(crate) fn e5_codes() -> [&'static str; 16] {
        [
            "C5", "L5", "D5", "S5", "C8I", "L8I", "D8I", "S8I", "C8Q", "L8Q", "D8Q", "S8Q", "C8X",
            "L8X", "D8X", "S8X",
        ]
    }
    pub(crate) fn e5a_codes() -> [&'static str; 12] {
        [
            "C5I", "L5I", "D5I", "S5I", "C5Q", "L5Q", "D5Q", "S5Q", "C5X", "L5X", "D5X", "S5X",
        ]
    }
    pub(crate) fn e5b_codes() -> [&'static str; 12] {
        [
            "C7I", "L7I", "D7I", "S7I", "C7Q", "L7Q", "D7Q", "S7Q", "C7X", "L7X", "D7X", "S7X",
        ]
    }
    pub(crate) fn e6_codes() -> [&'static str; 24] {
        [
            "C6", "L6", "D6", "S6", "C6A", "L6A", "D6A", "S6A", "C6B", "L6B", "D6B", "S6B", "C6C",
            "L6C", "D6C", "S6C", "C6X", "L6X", "D6X", "S6X", "C6Z", "L6Z", "D6Z", "S6Z",
        ]
    }
    fn from_gal_observable(obs: &Observable) -> Result<Self, Error> {
        match obs {
            Observable::PhaseRange(code)
            | Observable::Doppler(code)
            | Observable::SSI(code)
            | Observable::PseudoRange(code) => {
                let code = code.as_str();
                if Self::e1_codes().contains(&code) {
                    Ok(Self::E1)
                } else if Self::e5_codes().contains(&code) {
                    Ok(Self::E5)
                } else if Self::e6_codes().contains(&code) {
                    Ok(Self::E6)
                } else if Self::e5a_codes().contains(&code) {
                    Ok(Self::E5a)
                } else if Self::e5b_codes().contains(&code) {
                    Ok(Self::E5b)
                } else {
                    Err(Error::UnknownGalieoObservable)
                }
            },
            _ => Err(Error::UnknownGalieoObservable),
        }
    }
    pub(crate) fn geol1_codes() -> [&'static str; 8] {
        ["C1", "L1", "D1", "S1", "C1C", "L1C", "D1C", "S1C"]
    }
    pub(crate) fn geol5_codes() -> [&'static str; 16] {
        [
            "C5", "L5", "D5", "S5", "C5I", "L5I", "D5I", "S5I", "C5Q", "L5Q", "D5Q", "S5Q", "C5X",
            "L5X", "D5X", "S5X",
        ]
    }
    fn from_geo_observable(obs: &Observable) -> Result<Self, Error> {
        match obs {
            Observable::PhaseRange(code)
            | Observable::Doppler(code)
            | Observable::SSI(code)
            | Observable::PseudoRange(code) => {
                let code = code.as_str();
                if Self::geol1_codes().contains(&code) {
                    Ok(Self::L1)
                } else if Self::geol5_codes().contains(&code) {
                    Ok(Self::L5)
                } else {
                    Err(Error::UnknownSBASObservable)
                }
            },
            _ => Err(Error::UnknownSBASObservable),
        }
    }
    pub(crate) fn qzl1_codes() -> [&'static str; 32] {
        [
            "C1", "L1", "D1", "S1", "C1B", "L1B", "D1B", "S1B", "C1C", "L1C", "D1C", "S1C", "C1E",
            "L1E", "D1E", "S1E", "C1S", "L1S", "D1S", "S1S", "C1L", "L1L", "D1L", "S1L", "C1X",
            "L1X", "D1X", "S1X", "C1Z", "L1Z", "D1Z", "S1Z",
        ]
    }
    pub(crate) fn qzl2_codes() -> [&'static str; 16] {
        [
            "C2", "L2", "D2", "S2", "C2S", "L2S", "D2S", "S2S", "C2L", "L2L", "D2L", "S2L", "C2X",
            "L2X", "D2X", "S2X",
        ]
    }
    pub(crate) fn qzl5_codes() -> [&'static str; 28] {
        [
            "C5", "L5", "D5", "S5", "C5D", "L5D", "D5D", "S5D", "C5I", "L5I", "D5I", "S5I", "C5P",
            "L5P", "D5P", "S5P", "C5Q", "L5Q", "D5Q", "S5Q", "C5X", "L5X", "D5X", "S5X", "C5Z",
            "L5Z", "D5Z", "S5Z",
        ]
    }
    pub(crate) fn qzl6_codes() -> [&'static str; 24] {
        [
            "C6", "L6", "D6", "S6", "C6S", "L6S", "D6S", "S6S", "C6L", "L6L", "D6L", "S6L", "C6X",
            "L6X", "D6X", "S6X", "C6E", "L6E", "D6E", "S6E", "C6S", "L6S", "D6S", "S6Z",
        ]
    }
    fn from_qzss_observable(obs: &Observable) -> Result<Self, Error> {
        match obs {
            Observable::PhaseRange(code)
            | Observable::Doppler(code)
            | Observable::SSI(code)
            | Observable::PseudoRange(code) => {
                let code = code.as_str();
                if Self::qzl1_codes().contains(&code) {
                    Ok(Self::L1)
                } else if Self::qzl2_codes().contains(&code) {
                    Ok(Self::L2)
                } else if Self::qzl5_codes().contains(&code) {
                    Ok(Self::L5)
                } else if Self::qzl6_codes().contains(&code) {
                    Ok(Self::L6)
                } else {
                    Err(Error::UnknownQZSSObservable)
                }
            },
            _ => Err(Error::UnknownQZSSObservable),
        }
    }
    pub(crate) fn b1a_codes() -> [&'static str; 12] {
        [
            "C1S", "L1S", "D1S", "S1S", "C1L", "L1L", "D1L", "S1L", "C1Z", "L1Z", "D1Z", "S1Z",
        ]
    }
    pub(crate) fn b1c_codes() -> [&'static str; 12] {
        [
            "C1D", "L1D", "D1D", "S1D", "C1P", "L1P", "D1P", "S1P", "C1X", "L1X", "D1X", "S1X",
        ]
    }
    pub(crate) fn b1i_codes() -> [&'static str; 16] {
        [
            "C1", "L1", "D1", "S1", "C2I", "L2I", "D2I", "S2I", "C2Q", "L2Q", "D2Q", "S2Q", "C2X",
            "L2X", "D2X", "S2X",
        ]
    }
    pub(crate) fn b2a_codes() -> [&'static str; 12] {
        [
            "C5D", "L5D", "D5D", "S5D", "C5P", "L5P", "D5P", "S5P", "C5X", "L5X", "D5X", "S5X",
        ]
    }
    pub(crate) fn b2i_codes() -> [&'static str; 16] {
        [
            "C2", "L2", "D2", "S2", "C7I", "L7I", "D7I", "S7I", "C7Q", "L7Q", "D7Q", "S7Q", "C7X",
            "L7X", "D7X", "S7X",
        ]
    }
    pub(crate) fn b2b_codes() -> [&'static str; 12] {
        [
            "C7D", "L7D", "D7D", "S7D", "C7P", "L7P", "D7P", "S7P", "C7Z", "L7Z", "D7Z", "S7Z",
        ]
    }
    pub(crate) fn b2_codes() -> [&'static str; 12] {
        [
            "C8D", "L8D", "D8D", "S8D", "C8P", "L8P", "D8P", "S8P", "C8X", "L8X", "D8X", "S8X",
        ]
    }
    pub(crate) fn b3_codes() -> [&'static str; 12] {
        [
            "C6I", "L6I", "D6I", "S6I", "C6Q", "L6Q", "D6Q", "S6Q", "C6X", "L6X", "D6X", "S6X",
        ]
    }
    pub(crate) fn b3a_codes() -> [&'static str; 12] {
        [
            "C6D", "L6D", "D6D", "S6D", "C6P", "L6P", "D6P", "S6P", "C6Z", "L6Z", "D6Z", "S6Z",
        ]
    }
    fn from_bds_observable(obs: &Observable) -> Result<Self, Error> {
        match obs {
            Observable::PhaseRange(code)
            | Observable::Doppler(code)
            | Observable::SSI(code)
            | Observable::PseudoRange(code) => {
                let code = code.as_str();
                if Self::b1i_codes().contains(&code) {
                    Ok(Self::B1I)
                } else if Self::b1c_codes().contains(&code) {
                    Ok(Self::B1C)
                } else if Self::b1a_codes().contains(&code) {
                    Ok(Self::B1A)
                } else if Self::b2i_codes().contains(&code) {
                    Ok(Self::B2I)
                } else if Self::b2_codes().contains(&code) {
                    Ok(Self::B2)
                } else if Self::b2a_codes().contains(&code) {
                    Ok(Self::B2A)
                } else if Self::b2b_codes().contains(&code) {
                    Ok(Self::B2B)
                } else if Self::b3_codes().contains(&code) {
                    Ok(Self::B3)
                } else if Self::b3a_codes().contains(&code) {
                    Ok(Self::B3A)
                } else {
                    Err(Error::UnknownBeiDouObservable)
                }
            },
            _ => Err(Error::UnknownBeiDouObservable),
        }
    }
    pub(crate) fn irnl5_codes() -> [&'static str; 20] {
        [
            "C5", "L5", "D5", "S5", "C5A", "L5A", "D5A", "S5A", "C5B", "L5B", "D5B", "S5B", "C5C",
            "L5C", "D5C", "S5C", "C5X", "L5X", "D5X", "S5X",
        ]
    }
    pub(crate) fn irn_s_codes() -> [&'static str; 16] {
        [
            "C9A", "L9A", "D9A", "S9A", "C9B", "L9B", "D9B", "S9B", "C9C", "L9C", "D9C", "S9C",
            "C9X", "L9X", "D9X", "S9X",
        ]
    }
    fn from_irnss_observable(obs: &Observable) -> Result<Self, Error> {
        match obs {
            Observable::PhaseRange(code)
            | Observable::Doppler(code)
            | Observable::SSI(code)
            | Observable::PseudoRange(code) => {
                let code = code.as_str();
                if Self::irnl5_codes().contains(&code) {
                    Ok(Self::L5)
                } else if Self::irn_s_codes().contains(&code) {
                    Ok(Self::S)
                } else {
                    Err(Error::UnknownIRNSSObservable)
                }
            },
            _ => Err(Error::UnknownIRNSSObservable),
        }
    }
    /// Identifies Frequency channel, from given observable, related
    /// to given Constellation
    pub fn from_observable(
        constellation: Constellation,
        observable: &Observable,
    ) -> Result<Self, Error> {
        match constellation {
            Constellation::GPS => Self::from_gps_observable(observable),
            Constellation::BeiDou => Self::from_bds_observable(observable),
            Constellation::Glonass => Self::from_glo_observable(observable),
            Constellation::Galileo => Self::from_gal_observable(observable),
            Constellation::QZSS => Self::from_qzss_observable(observable),
            Constellation::IRNSS => Self::from_irnss_observable(observable),
            c => {
                if c.is_sbas() {
                    Self::from_geo_observable(observable)
                } else {
                    panic!("no known observable for constellation={}", constellation);
                }
            },
        }
    }

    /*
     * Build a frequency from standard SV description.
     * This is used in ATX records to identify the antenna frequency
     */
    pub(crate) fn from_sv(sv: SV) -> Result<Self, ParsingError> {
        match sv.constellation {
            Constellation::GPS => match sv.prn {
                1 => Ok(Self::L1),
                2 => Ok(Self::L2),
                5 => Ok(Self::L5),
                _ => Ok(Self::L1),
            },
            Constellation::Glonass => match sv.prn {
                1 => Ok(Self::G1(None)),
                2 => Ok(Self::G2(None)),
                3 => Ok(Self::G3),
                _ => Ok(Self::G1(None)),
            },
            Constellation::Galileo => match sv.prn {
                1 => Ok(Self::E1),
                5 => Ok(Self::E5a),
                6 => Ok(Self::E6),
                7 => Ok(Self::E5b),
                8 => Ok(Self::E5),
                _ => Ok(Self::E1),
            },
            Constellation::SBAS => match sv.prn {
                1 => Ok(Self::L1),
                5 => Ok(Self::L5),
                _ => Ok(Self::L1),
            },
            Constellation::BeiDou => match sv.prn {
                1 => Ok(Self::B1C),
                2 => Ok(Self::B1I),
                5 => Ok(Self::B2A),
                6 => Ok(Self::B3),
                7 => Ok(Self::B2B),
                8 => Ok(Self::B2),
                _ => Ok(Self::E1),
            },
            Constellation::QZSS => match sv.prn {
                1 => Ok(Self::L1),
                2 => Ok(Self::L2),
                5 => Ok(Self::L5),
                6 => Ok(Self::L6),
                _ => Ok(Self::L1),
            },
            Constellation::IRNSS => match sv.prn {
                1 => Ok(Self::L1),
                5 => Ok(Self::L5),
                9 => Ok(Self::S),
                _ => Ok(Self::L1),
            },
            _ => Err(ParsingError::AntexFrequency),
        }
    }

    /// Builds [Carrier] from DORIS [Observable]
    pub fn from_doris_observable(obs: &Observable) -> Result<Self, Error> {
        let obs = obs.to_string();
        if obs.contains("1") {
            Ok(Self::S1)
        } else if obs.contains("2") {
            Ok(Self::U2)
        } else {
            Err(Error::UnknownDORISObservable)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_carrier() {
        assert!(Carrier::from_str("L1").is_ok());
        assert!(Carrier::from_str("C1").is_err());
        assert!(Carrier::from_str("L5").is_ok());

        let l1 = Carrier::from_str("L1").unwrap();
        assert_eq!(l1.frequency_mhz(), 1575.42_f64);
        assert_eq!(l1.wavelength(), 299792458.0 / 1_575_420_000.0_f64);

        for constell in [
            Constellation::GPS,
            Constellation::SBAS,
            Constellation::Glonass,
            Constellation::Galileo,
            Constellation::BeiDou,
            Constellation::IRNSS,
            Constellation::QZSS,
        ] {
            /*
             * GPS
             */
            if constell == Constellation::GPS {
                let codes = vec!["L1", "L1C", "D1C", "L1N", "S1Y", "D1W"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::L1
                    );
                }
                let codes = vec!["L2", "L2C", "D2C", "L2N", "S2Y", "D2W"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::L2
                    );
                }
                let codes = vec!["C5", "L5I", "D5Q", "S5X", "C5X", "S5I"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::L5
                    );
                }
            /*
             * SBAS
             */
            } else if constell == Constellation::SBAS {
                let codes = vec!["C1", "L1C", "D1", "S1", "S1C", "D1C"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::L1
                    );
                }
                let codes = vec!["C5", "L5I", "D5I", "S5", "S5Q", "D5X", "S5X", "L5Q"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::L5
                    );
                }
            /*
             * Glonass
             */
            } else if constell == Constellation::Glonass {
                let codes = vec!["L1", "L1C", "D1P", "S1P", "S1", "C1P"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::G1(None),
                    );
                }
                let codes = vec!["L4A", "S4X", "D4B", "C4X"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::G1a
                    );
                }
                let codes = vec!["L2", "C2", "L2P", "S2C", "S2P", "D2"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::G2(None),
                    );
                }
                let codes = vec!["L6A", "D6A", "S6X", "L6X", "S6B"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::G2a
                    );
                }
                let codes = vec!["C3", "D3I", "S3Q", "L3X", "D3X", "C3Q"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::G3
                    );
                }
            /*
             * BeiDou
             */
            } else if constell == Constellation::BeiDou {
                let codes = vec!["L1", "L2I", "D2X", "D2Q", "S1", "S2I"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::B1I
                    );
                }
                let codes = vec!["C1D", "L1D", "S1X", "S1P"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::B1C
                    );
                }
                let codes = vec!["L1S", "D1S", "S1Z", "L1Z", "C1L", "C1Z"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::B1A
                    );
                }
                let codes = vec!["C5D", "S5D", "S5X", "S5P", "D5P", "C5X"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::B2A
                    );
                }
                let codes = vec!["C2", "L2", "C7I", "L7X", "S7X", "S7I"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::B2I
                    );
                }
                let codes = vec!["C7D", "L7D", "L7P", "C7Z", "S7Z", "L7Z"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::B2B
                    );
                }
                let codes = vec!["C8D", "L8D", "L8P", "C8X", "S8X", "L8X"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::B2
                    );
                }
                let codes = vec!["C6I", "L6I", "L6X", "C6X", "S6I", "S6Q", "D6Q"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::B3
                    );
                }
                let codes = vec!["C6D", "L6Z", "S6Z", "L6Z", "C6Z", "S6P"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::B3A
                    );
                }
            /*
             * Galileo
             */
            } else if constell == Constellation::Galileo {
                let codes = vec!["C1", "L1", "S1B", "L1A", "D1Z", "S1Z"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::E1
                    );
                }
                let codes = vec!["C5I", "L5X", "D5X", "S5Q", "C5X", "D5I"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::E5a
                    );
                }
                let codes = vec!["C7I", "L7X", "D7X", "S7Q", "C7X", "D7I"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::E5b
                    );
                }
                let codes = vec!["C5", "L8I", "C8I", "C8X", "L8X", "S8X"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::E5
                    );
                }
                let codes = vec!["C6", "L6", "L6A", "C6C", "S6Z", "D6X"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::E6
                    );
                }
            /*
             * IRNSS
             */
            } else if constell == Constellation::IRNSS {
                let codes = vec![
                    "C5", "L5", "L5A", "C5A", "S5B", "D5B", "C5C", "L5C", "D5X", "L5X",
                ];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::L5
                    );
                }
                let codes = vec!["C9A", "L9B", "L9X", "C9X", "S9B", "D9B", "C9B"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::S
                    );
                }
            /*
             * QZSS
             */
            } else if constell == Constellation::QZSS {
                let codes = vec!["C1", "L1", "L1B", "C1E", "S1Z", "S1L", "L1E", "S1S"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::L1
                    );
                }
                let codes = vec!["C2", "L2", "L2S", "C2S", "S2L", "S2X", "L2S", "S2X"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::L2
                    );
                }
                let codes = vec!["C5", "L5", "L5D", "C5I", "S5I", "S5X", "L5Z", "D5P"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::L5
                    );
                }
                let codes = vec!["C6", "L6", "L6S", "C6L", "S6S", "S6L", "L6X", "D6E"];
                for code in codes {
                    let obs = Observable::from_str(code).unwrap();
                    assert_eq!(
                        Carrier::from_observable(constell, &obs).unwrap(),
                        Carrier::L6
                    );
                }
            }
        }
    }
}
