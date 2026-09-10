//! GLONASS CDMA ionosphere model (RINEX 4.02)
use crate::{
    epoch::parse_in_timescale as parse_epoch_in_timescale,
    navigation::ionosphere::parse_e19_fields,
    prelude::{Epoch, ParsingError, TimeScale},
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// GLONASS CDMA ionosphere model, from the LXOC ION message (RINEX 4.02 Table A40)
#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct GloCdmaModel {
    /// c_A coefficient
    pub c_a: f64,
    /// c_F10.7 coefficient
    pub c_f107: f64,
    /// c_Ap coefficient
    pub c_ap: f64,
}

impl GloCdmaModel {
    /// Formats the RINEX 4.02 GLONASS CDMA ION record (Table A40).
    pub(crate) fn format_v4<W: std::io::Write>(
        &self,
        w: &mut std::io::BufWriter<W>,
        epoch: Epoch,
    ) -> Result<(), crate::error::FormattingError> {
        use crate::navigation::formatting::{format_epoch_v4_fields, NavFormatter};
        use std::io::Write;

        writeln!(
            w,
            "    {}{}{}{}",
            format_epoch_v4_fields(epoch),
            NavFormatter::new(self.c_a),
            NavFormatter::new(self.c_f107),
            NavFormatter::new(self.c_ap),
        )?;

        Ok(())
    }

    /// Parses [GloCdmaModel] from lines Iter
    pub(crate) fn parse(
        mut lines: std::str::Lines<'_>,
        ts: TimeScale,
    ) -> Result<(Epoch, Self), ParsingError> {
        let line = lines.next().ok_or(ParsingError::EmptyEpoch)?;
        let (epoch, rem) = line.split_at(23);
        let epoch = parse_epoch_in_timescale(epoch.trim(), ts)?;
        let c = parse_e19_fields(rem, 0, 3).ok_or(ParsingError::GlonassIonosphereData)?;
        Ok((
            epoch,
            Self {
                c_a: c[0],
                c_f107: c[1],
                c_ap: c[2],
            },
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn glonass_cdma() {
        // RINEX 4.02 Table A41, without the record header
        let content =
            "    2024 02 03 00 01 09 1.000000000000e+00 1.410000000000e+02 5.000000000000e+00";
        let (epoch, model) = GloCdmaModel::parse(content.lines(), TimeScale::UTC).unwrap();
        assert_eq!(epoch, Epoch::from_str("2024-02-03T00:01:09 UTC").unwrap());
        assert_eq!(
            model,
            GloCdmaModel {
                c_a: 1.0,
                c_f107: 141.0,
                c_ap: 5.0,
            }
        );
    }
}
