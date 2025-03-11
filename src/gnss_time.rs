//use thiserror::Error;
//use std::str::FromStr;

/*
/// System Time corrections decoding error
#[derive(Error, Debug)]
#[derive(PartialEq)]
pub enum Error {
    /// Failed to decode time systems
    #[error("failed to decode time systems")]
    DecodingError,
    #[error("faulty header TIME SYSTEM CORR field")]
    FaultyTimeSystemCorr,
    #[error("faulty header D-UTC descriptor")]
    FaultyDUtc,
    #[error("faulty header CORR TO SYSTEM TIME descriptor")]
    FaultyCorrToSystemTime,
    #[error("unknown constellation")]
    UnknownConstellation,
    #[error("unknown timescale")]
    UnknownTimeScale,
    #[error("failed to parse (a0, a1) coefficients")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse (wn, secs) counters")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("hifitime::_maybe_from_gregorian_")]
    HifitimeError(#[from] hifitime::Errors),
}

/// Decodes corretion from `lhs` system to `rhs` system
fn decode_system(content: &str) -> Result<(Constellation, TimeScale), Error> {
    if content.len() != 4 {
        return Err(Error::DecodingError);
    }
    let start_off = &content[..2];
    let gnss: Constellation = match start_off {
        "SB" => Constellation::Geo,
        "GP" => Constellation::GPS,
        "GL" => Constellation::Glonass,
        "GA" => Constellation::Galileo,
        "BD" => Constellation::BeiDou,
        "QZ" => Constellation::QZSS,
        "IR" => Constellation::IRNSS,
        _ => return Err(Error::UnknownConstellation),
    };
    let ending = &content[2..];
    let ts: TimeScale = match ending {
        "UT" => TimeScale::UTC,
        "GP" => TimeScale::GPST,
        "GA" => TimeScale::GST,
        "BD" => TimeScale::BDT,
        _ => return Err(Error::UnknownTimeScale),
    };
    Ok((gnss, ts))
}

/// Decodes V1 descriptor (very old)
pub fn decode_corr_to_system_time(content: &str) -> Result<TimeCorrection, Error> {
    let (yyyy, rem) = content.split_at(6);
    let (month, rem) = rem.split_at(6);
    let (day, rem) = rem.split_at(6);
    let yyyy = i32::from_str_radix(yyyy.trim(), 10)?;
    let month = u8::from_str_radix(month.trim(), 10)?;
    let day = u8::from_str_radix(day.trim(), 10)?;
    let corr = f64::from_str(rem.trim())?;
    Ok(TimeCorrection {
        epoch: Epoch::maybe_from_gregorian(yyyy, month, day, 0, 0, 0, 0, TimeScale::GPST)?,
        a0: 0.0,
        a1: 0.0,
    })
}

/// Decodes V3 descriptor
pub fn decode_time_system_corr(content: &str) -> Result<TimeCorrection, Error> {
    if content.len() < 50 {
        // can't parse basic structure
        return Err(Error::FaultyTimeSystemCorr);
    }
    // decode system (lhs, rhs)
    let (source, timescale) = decode_system(&content[..4])?;

    let a0 = &content[5..22];
    let a1 = &content[22..22+17];
    let t_ref = &content[22+17..22+17+6];
    let w_ref = &content[22+17+6..22+17+11];
    println!("A0 \"{}\"", a0); // DEBUG
    println!("A1 \"{}\"", a1); // DEBUG
    println!("t_ref \"{}\"", t_ref); // DEBUG
    println!("w_ref \"{}\"", w_ref); // DEBUG

    //DT = A0 + A0(t-tref) for fractional part
    //of time system difference
    //excluding leap seconds

    let a0 = f64::from_str(a0.replace("D","E").trim())?;
    let a1 = f64::from_str(a1.replace("D","E").trim())?;

    // t_ref: seconds into GPST,
    let t_ref = u32::from_str_radix(t_ref.trim(), 10)?;
    // w_ref: free running GPST week counter
    let w_ref = u16::from_str_radix(w_ref.trim(), 10)?;
    let mut duration = Duration::from_days((w_ref * 7) as f64);
    duration += Duration::from_seconds(t_ref as f64);

    Ok(TimeCorrection {
        //source,
        //epoch: Epoch::default(),
        //utc_provider: None,
        a0,
        a1,
        /*
         * apparently, it's always defined related to GPST
         */
        epoch: Epoch::from_gpst_duration(duration),
    })
}

/// Decodes from V2 descriptor
/// Decodes from V1 descriptor

pub struct StCorr {
    pub y: i8,
    pub m: i8,
    pub d: i8,
    pub corr: f64,
}

pub struct DUtc {
    /// correction term
    pub a0: f64,
    /// correction term ^2
    pub a1: f64,
    /// Reference time for polynomial (seconds into GPS week)
    pub t: i8,
    /// Reference week number (GPS week, continuous number)
    pub w: i8,
    /// system
    pub s: system Svnn or SBAS
    pub fn u: UTc provider
}

/// (Navigation frame) Time Correction parameters
pub struct TimeCorrection {
    /// Source: GNSS satellite broadcasting the system time difference,
    /// or SBAS vehicle broadcasting the MT12.
    // pub source: Constellation,
    /// correction parameter
    pub a0: f64,
    /// correction parameter
    pub a1: f64,
    /// Epoch in GPST
    pub epoch: Epoch,
    // /// UTC provider: in case this applies to UTC time scale
    // pub utc_provider: Option<UTCProvider>,
}

impl GnssTime {
    /// Corrects self to given reference using given correction parameters
    /// correction: correction to be applied
    /// reference: reference time (must match expected reference)
    /// TODO: refer to p39
    pub fn correct (&mut self, correction: &GnssTimeCorrection, reference: &GnssTime) -> Result<(), Error> {
        // check this is the expected reference time
        match correction.corr_type {
            TimeCorrectionType::GPUT => {
                // check time system matches the expected one
                match self.gnss {
                    constellation::Constellation::GPS => {},
                    _ => return Err(Error::CorrectionTimeSystemError),
                }
            },
            TimeCorrectionType::GAUT => {
                // check time system matches the expected one
                match self.gnss {
                    constellation::Constellation::Galileo => {},
                    _ => return Err(Error::CorrectionTimeSystemError),
                }
            },
            TimeCorrectionType::SBUT => {
                // check time system matches the expected one
                match self.gnss {
                    constellation::Constellation::Sbas => {},
                    _ => return Err(Error::CorrectionTimeSystemError),
                }
            },
            TimeCorrectionType::GLUT => {
                // check time system matches the expected one
                match self.gnss {
                    constellation::Constellation::Glonass => {},
                    _ => return Err(Error::CorrectionTimeSystemError),
                }
            },
            TimeCorrectionType::GPGA => {
                if reference.gnss != constellation::Constellation::Galileo {
                    return Err(Error::CorrectionTimeReferenceError)
                }
                // check time system matches the expected one
                match self.gnss {
                    constellation::Constellation::GPS => {},
                    _ => return Err(Error::CorrectionTimeSystemError),
                }
            },
            TimeCorrectionType::GLGP => {
                if reference.gnss != constellation::Constellation::GPS {
                    return Err(Error::CorrectionTimeReferenceError)
                }
                // check time system matches the expected one
                match self.gnss {
                    constellation::Constellation::Glonass => {},
                    _ => return Err(Error::CorrectionTimeSystemError),
                }
            },
            TimeCorrectionType::GZUT => {
                // check time system matches the expected one
                match self.gnss {
                    constellation::Constellation::QZSS => {},
                    _ => return Err(Error::CorrectionTimeSystemError),
                }
            },
        }
        Ok(())
    }
}

/// List of known correction types:
/// GPUT: GPS->UTC  (a0,a1)
/// GAUT: GAL->UTC  (a0,a1)
/// SBUT: SBAS->UTC (a0,a1)
/// GLUT: GLO->UTC  a0=τ(c) a1=0
/// GPGA: GPS->GAL  a0=a0g  a1=a1g
/// GLGP: GLO->GPS  a0=τ(gps) a1=zero
/// GZUT: QZS->UTC  a0,a1
pub enum TimeCorrectionType {
    GPUT,
    GAUT,
    SBUT,
    GLUT,
    GPGA,
    GLGP,
    GZUT,
}

impl std::str::FromStr for TimeCorrectionType {
    type Err = Error;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.eq("GPUT") {
            Ok(TimeCorrectionType::GPUT)
        } else if s.eq("GAUT") {
            Ok(TimeCorrectionType::GAUT)
        } else if s.eq("SBUT") {
            Ok(TimeCorrectionType::SBUT)
        } else if s.eq("GLUT") {
            Ok(TimeCorrectionType::GLUT)
        } else if s.eq("GPGA") {
            Ok(TimeCorrectionType::GPGA)
        } else if s.eq("GLGP") {
            Ok(TimeCorrectionType::GLGP)
        } else if s.eq("GZUT") {
            Ok(TimeCorrectionType::GZUT)
        } else {
            Err(Error::InvalidTimeSystem(s.to_string()))
        }
    }
}

/// Describes known UTC providers
/// (laboratories)
pub enum UtcProvider {
    Unknown,
    NIST,
    USNO,
    SU,
    BIPM,
    EuropeLab,
    CRL,
}

impl std::str::FromStr for UtcProvider {
    type Err = Error;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.eq("1") {
            Ok(UtcProvider::NIST)
        } else if s.eq("2") {
            Ok(UtcProvider::USNO)
        } else if s.eq("3") {
            Ok(UtcProvider::SU)
        } else if s.eq("4") {
            Ok(UtcProvider::BIPM)
        } else if s.eq("5") {
            Ok(UtcProvider::EuropeLab)
        } else if s.eq("6") {
            Ok(UtcProvider::CRL)
        } else {
            Ok(UtcProvider::Unknown)
        }
    }
}

/// Not documented ?
pub enum AugmentationSystem {
    EGNOS,
    WAAS,
    MSAS,
}

impl std::str::FromStr for AugmentationSystem {
    type Err = Error;
    /// Builds `AugmentationSystem` from string
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.eq("EGNOS") {
            Ok(AugmentationSystem::EGNOS)
        } else if s.eq("WAAS") {
            Ok(AugmentationSystem::WAAS)
        } else if s.eq("MSAS") {
            Ok(AugmentationSystem::MSAS)
        } else {
            Err(Error::UnknownAugmentationSystem(s.to_string()))
        }
    }
}

/// `GnssTimeCorrection` describes
/// GNSS Time System corrections.
/// `system` : XXYY: XX corrected to YY
/// (a0, a1): correction params ((s), (s.s⁻¹))
/// delta_t: correction param
/// week: week number counter
/// `augmentation system`: (EGNOS,WAAS,MSAS)
/// utc_provider: provider identifier
#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct GnssTimeCorrection {
    corr_type: TimeCorrectionType,
    params: (f64,f64),
    delta_t: u32,
    week: u32,
    augmentation: Option<AugmentationSystem>,
    utc_provider: Option<UtcProvider>,
}

impl std::str::FromStr for GnssTimeCorrection {
    type Err = Error;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        Ok(GnssTimeCorrection::default())
        /*
SBUT  0.1331791282D-06 0.107469589D-12 552960 1025 EGNOS  5 TIME SYSTEM CORR
        let systype = TimeCorrectionType::from_str()?;
        let params ...
        let delta_t..
        let week..
        if not null:
            AugmentationSystem::from_str()?
        if not null:
            UtcProvider::from_str()?
        */
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_decoding() {
        assert_eq!(decode_system("QZUT"), Ok((Constellation::QZSS, TimeScale::UTC)));
        assert_eq!(decode_system("GPUT"), Ok((Constellation::GPS, TimeScale::UTC)));
        assert_eq!(decode_system("BDUT"), Ok((Constellation::BeiDou, TimeScale::UTC)));
        assert_eq!(decode_system("GAUT"), Ok((Constellation::Galileo, TimeScale::UTC)));
        assert_eq!(decode_system("SBUT"), Ok((Constellation::Geo, TimeScale::UTC)));
        assert_eq!(decode_system("IRGP"), Ok((Constellation::IRNSS, TimeScale::GPST)));
        assert_eq!(decode_system("IRGA"), Ok((Constellation::IRNSS, TimeScale::GST)));
        assert_eq!(decode_system("IRBD"), Ok((Constellation::IRNSS, TimeScale::BDT)));
        assert!(decode_system("QZGL").is_err()); // Glonass timescale not known yet
    }
    #[test]
    fn test_time_system_corr_decoding() {
        let corr = decode_time_system_corr("GAUT  1.8626451492e-09-8.881784197e-16 432000 2138");
        assert!(corr.is_ok());
        let corr = corr.unwrap();
        assert_eq!(corr.a0, 1.8626451492e-09);
        assert_eq!(corr.a1, -8.881784197e-16);
        //assert_eq!(corr.t_ref, 432000);
        //assert_eq!(corr.w_ref, 2138);

        let corr = decode_time_system_corr("GPUT -3.7252902985e-09-1.065814104e-14  61440 2139");
        assert!(corr.is_ok());
        let corr = corr.unwrap();
        assert_eq!(corr.a0,  -3.7252902985e-9);
        assert_eq!(corr.a1,  -1.065814104e-14);
        //assert_eq!(corr.t_ref, 61440);
        //assert_eq!(corr.w_ref, 2139);

        let corr = decode_time_system_corr("GLGP -2.1420419216e-08 0.000000000e+00 518400 2138");
        assert!(corr.is_ok());
        let corr = corr.unwrap();
        assert_eq!(corr.a0, -2.1420419216e-08);
        assert_eq!(corr.a1, 0.0);
        //assert_eq!(corr.t_ref, 518400);
        //assert_eq!(corr.w_ref, 2138);

        let corr = decode_time_system_corr("GPUT  -.3725290298E-08 -.106581410E-13  61440 2139");
        assert!(corr.is_ok());
        let corr = corr.unwrap();
        assert_eq!(corr.a0, -0.3725290298E-08);
        assert_eq!(corr.a1, -0.106581410E-13);
        //assert_eq!(corr.t_ref, 61440);
        //assert_eq!(corr.w_ref, 2139);

        let corr = decode_time_system_corr("IRGP -4.9476511776e-10-2.664535259e-15 432288 2138");
        assert!(corr.is_ok());
        let corr = corr.unwrap();
        assert_eq!(corr.a0, -4.9476511776e-10);
        assert_eq!(corr.a1, -2.664535259e-15);
        //assert_eq!(corr.t_ref, 432288);
        //assert_eq!(corr.w_ref, 2138);

        let corr = decode_time_system_corr("GAGP  2.1536834538e-09-9.769962617e-15 432000 2138");
        assert!(corr.is_ok());
        let corr = corr.unwrap();
        assert_eq!(corr.a0, 2.1536834538e-09);
        assert_eq!(corr.a1, -9.769962617e-15);
        //assert_eq!(corr.t_ref, 432000 );
        //assert_eq!(corr.w_ref, 2138);

        let corr = decode_time_system_corr("QZUT   .5587935448E-08  .000000000E+00  94208 2139");
        assert!(corr.is_ok());
        let corr = corr.unwrap();
        assert_eq!(corr.a0, 0.5587935448E-08);
        assert_eq!(corr.a1, 0.0);
        //assert_eq!(corr.t_ref, 94208 );
        //assert_eq!(corr.w_ref, 2139);
    }
    #[test]
    fn test_corr_to_system_time() {
        let content = "2021     1     1   -1.862645149231D-09";
        let corr = decode_corr_to_system_time(content);
        assert!(corr.is_ok());
    }
}
*/
