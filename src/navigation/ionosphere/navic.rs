//! NavIC L1NV ionosphere models (RINEX 4.02)
use crate::{
    epoch::parse_in_timescale as parse_epoch_in_timescale,
    navigation::ionosphere::parse_e19_fields,
    prelude::{Epoch, ParsingError, TimeScale},
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// NavIC Klobuchar model, from the L1NV ION KLOB message (RINEX 4.02 Table A38)
#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct NavicKbModel {
    /// Issue of data, Klobuchar
    pub iodk: f64,
    /// Alpha coefficients
    /// ((sec), (sec.semi-circle⁻¹), (sec.semi-circle⁻²), (sec.semi-circle⁻³))
    pub alpha: (f64, f64, f64, f64),
    /// Beta coefficients
    /// ((sec), (sec.semi-circle⁻¹), (sec.semi-circle⁻²), (sec.semi-circle⁻³))
    pub beta: (f64, f64, f64, f64),
    /// Region of validity: (min, max) longitude in degrees
    pub longitude_deg: (f64, f64),
    /// Region of validity: (min, max) latitude in degrees
    pub latitude_deg: (f64, f64),
}

impl NavicKbModel {
    /// Formats the RINEX 4.02 NavIC Klobuchar ION record (Table A38).
    pub(crate) fn format_v4<W: std::io::Write>(
        &self,
        w: &mut std::io::BufWriter<W>,
        epoch: Epoch,
    ) -> Result<(), crate::error::FormattingError> {
        use crate::navigation::formatting::{format_epoch_v4_fields, NavFormatter};
        use std::io::Write;

        writeln!(
            w,
            "    {}{}",
            format_epoch_v4_fields(epoch),
            NavFormatter::new(self.iodk)
        )?;

        writeln!(
            w,
            "    {}{}{}{}",
            NavFormatter::new(self.alpha.0),
            NavFormatter::new(self.alpha.1),
            NavFormatter::new(self.alpha.2),
            NavFormatter::new(self.alpha.3),
        )?;

        writeln!(
            w,
            "    {}{}{}{}",
            NavFormatter::new(self.beta.0),
            NavFormatter::new(self.beta.1),
            NavFormatter::new(self.beta.2),
            NavFormatter::new(self.beta.3),
        )?;

        writeln!(
            w,
            "    {}{}{}{}",
            NavFormatter::new(self.longitude_deg.0),
            NavFormatter::new(self.longitude_deg.1),
            NavFormatter::new(self.latitude_deg.0),
            NavFormatter::new(self.latitude_deg.1),
        )?;

        Ok(())
    }

    /// Parses [NavicKbModel] from lines Iter
    pub(crate) fn parse(
        mut lines: std::str::Lines<'_>,
        ts: TimeScale,
    ) -> Result<(Epoch, Self), ParsingError> {
        let line = lines.next().ok_or(ParsingError::EmptyEpoch)?;
        let (epoch, rem) = line.split_at(23);
        let epoch = parse_epoch_in_timescale(epoch.trim(), ts)?;
        let iodk = parse_e19_fields(rem, 0, 1).ok_or(ParsingError::NavicIonosphereData)?[0];

        let line = lines.next().ok_or(ParsingError::EmptyEpoch)?;
        let alpha = parse_e19_fields(line, 4, 4).ok_or(ParsingError::NavicIonosphereData)?;

        let line = lines.next().ok_or(ParsingError::EmptyEpoch)?;
        let beta = parse_e19_fields(line, 4, 4).ok_or(ParsingError::NavicIonosphereData)?;

        let line = lines.next().ok_or(ParsingError::EmptyEpoch)?;
        let region = parse_e19_fields(line, 4, 4).ok_or(ParsingError::NavicIonosphereData)?;

        Ok((
            epoch,
            Self {
                iodk,
                alpha: (alpha[0], alpha[1], alpha[2], alpha[3]),
                beta: (beta[0], beta[1], beta[2], beta[3]),
                longitude_deg: (region[0], region[1]),
                latitude_deg: (region[2], region[3]),
            },
        ))
    }
}

/// One region of the NavIC NeQuick-N model
#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct NavicNeqnRegion {
    /// a_i coefficients (sfu, (sfu.deg⁻¹), (sfu.deg⁻²))
    pub a: (f64, f64, f64),
    /// Ionosphere disturbance flag
    pub idf: f64,
    /// Region of validity: (min, max) longitude in degrees
    pub longitude_deg: (f64, f64),
    /// Region of validity: (min, max) modified dip latitude in degrees
    pub modip_deg: (f64, f64),
}

/// NavIC NeQuick-N model, from the L1NV ION NEQN message (RINEX 4.02 Table A39)
#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct NavicNeqnModel {
    /// Issue of data, NeQuick-N
    pub iodn: f64,
    /// Three regional coefficient sets
    pub regions: [NavicNeqnRegion; 3],
}

impl NavicNeqnModel {
    /// Formats the RINEX 4.02 NavIC NeQuick-N ION record (Table A39).
    pub(crate) fn format_v4<W: std::io::Write>(
        &self,
        w: &mut std::io::BufWriter<W>,
        epoch: Epoch,
    ) -> Result<(), crate::error::FormattingError> {
        use crate::navigation::formatting::{format_epoch_v4_fields, NavFormatter};
        use std::io::Write;

        writeln!(
            w,
            "    {}{}",
            format_epoch_v4_fields(epoch),
            NavFormatter::new(self.iodn)
        )?;

        for region in self.regions.iter() {
            writeln!(
                w,
                "    {}{}{}{}",
                NavFormatter::new(region.a.0),
                NavFormatter::new(region.a.1),
                NavFormatter::new(region.a.2),
                NavFormatter::new(region.idf),
            )?;

            writeln!(
                w,
                "    {}{}{}{}",
                NavFormatter::new(region.longitude_deg.0),
                NavFormatter::new(region.longitude_deg.1),
                NavFormatter::new(region.modip_deg.0),
                NavFormatter::new(region.modip_deg.1),
            )?;
        }

        Ok(())
    }

    /// Parses [NavicNeqnModel] from lines Iter
    pub(crate) fn parse(
        mut lines: std::str::Lines<'_>,
        ts: TimeScale,
    ) -> Result<(Epoch, Self), ParsingError> {
        let line = lines.next().ok_or(ParsingError::EmptyEpoch)?;
        let (epoch, rem) = line.split_at(23);
        let epoch = parse_epoch_in_timescale(epoch.trim(), ts)?;
        let iodn = parse_e19_fields(rem, 0, 1).ok_or(ParsingError::NavicIonosphereData)?[0];

        let mut regions = [NavicNeqnRegion::default(); 3];

        for region in regions.iter_mut() {
            let line = lines.next().ok_or(ParsingError::EmptyEpoch)?;
            let a = parse_e19_fields(line, 4, 4).ok_or(ParsingError::NavicIonosphereData)?;

            let line = lines.next().ok_or(ParsingError::EmptyEpoch)?;
            let bounds = parse_e19_fields(line, 4, 4).ok_or(ParsingError::NavicIonosphereData)?;

            *region = NavicNeqnRegion {
                a: (a[0], a[1], a[2]),
                idf: a[3],
                longitude_deg: (bounds[0], bounds[1]),
                modip_deg: (bounds[2], bounds[3]),
            };
        }

        Ok((epoch, Self { iodn, regions }))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn navic_klobuchar() {
        // RINEX 4.02 Table A41, without the record header
        let content = "    2023 06 24 00 07 30 1.000000000000e+00
     5.867332220078e-08 2.533197402954e-07-1.430511474609e-06-7.510185241699e-06
     1.495040000000e+05-5.406720000000e+05 2.883584000000e+06 8.323072000000e+06
     5.000000000000e+01 1.100000000000e+02 0.000000000000e+00 5.000000000000e+01";
        let (epoch, model) = NavicKbModel::parse(content.lines(), TimeScale::GPST).unwrap();
        assert_eq!(epoch, Epoch::from_str("2023-06-24T00:07:30 GPST").unwrap());
        assert_eq!(
            model,
            NavicKbModel {
                iodk: 1.0,
                alpha: (
                    5.867332220078e-08,
                    2.533197402954e-07,
                    -1.430511474609e-06,
                    -7.510185241699e-06
                ),
                beta: (
                    1.495040000000e+05,
                    -5.406720000000e+05,
                    2.883584000000e+06,
                    8.323072000000e+06
                ),
                longitude_deg: (50.0, 110.0),
                latitude_deg: (0.0, 50.0),
            }
        );
    }

    #[test]
    fn navic_nequick() {
        // RINEX 4.02 Table A41, without the record header
        let content = "    2023 06 24 00 17 24 0.000000000000e+00
     1.982500000000e+02 0.000000000000e+00 0.000000000000e+00 1.000000000000e+00
     3.000000000000e+01 1.300000000000e+02-3.000000000000e+01-1.000000000000e+01
     2.085000000000e+02-3.085937500000e-01-3.417968750000e-03 1.000000000000e+00
     3.000000000000e+01 1.300000000000e+02-5.000000000000e+00 3.000000000000e+01
     1.982500000000e+02 0.000000000000e+00 0.000000000000e+00 1.000000000000e+00
     3.000000000000e+01 1.300000000000e+02 3.500000000000e+01 5.000000000000e+01";
        let (epoch, model) = NavicNeqnModel::parse(content.lines(), TimeScale::GPST).unwrap();
        assert_eq!(epoch, Epoch::from_str("2023-06-24T00:17:24 GPST").unwrap());
        assert_eq!(model.iodn, 0.0);
        assert_eq!(
            model.regions[0],
            NavicNeqnRegion {
                a: (198.25, 0.0, 0.0),
                idf: 1.0,
                longitude_deg: (30.0, 130.0),
                modip_deg: (-30.0, -10.0),
            }
        );
        assert_eq!(
            model.regions[1],
            NavicNeqnRegion {
                a: (208.5, -3.085937500000e-01, -3.417968750000e-03),
                idf: 1.0,
                longitude_deg: (30.0, 130.0),
                modip_deg: (-5.0, 30.0),
            }
        );
        assert_eq!(
            model.regions[2],
            NavicNeqnRegion {
                a: (198.25, 0.0, 0.0),
                idf: 1.0,
                longitude_deg: (30.0, 130.0),
                modip_deg: (35.0, 50.0),
            }
        );
    }

    #[test]
    fn navic_nequick_truncated() {
        let content = "    2023 06 24 00 17 24 0.000000000000e+00
     1.982500000000e+02 0.000000000000e+00 0.000000000000e+00 1.000000000000e+00";
        assert!(NavicNeqnModel::parse(content.lines(), TimeScale::GPST).is_err());
    }
}
