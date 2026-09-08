use crate::{
    epoch::parse_in_timescale as parse_epoch_in_timescale,
    parse_f64,
    prelude::{Epoch, ParsingError, TimeScale},
};

/// BDGIM Model payload
#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct BdModel {
    /// Alpha coefficients in TEC unit
    pub alpha: (f64, f64, f64, f64, f64, f64, f64, f64, f64),
}

impl BdModel {
    /// Parses [BdModel] from Lines Iter
    pub(crate) fn parse(
        mut lines: std::str::Lines<'_>,
        ts: TimeScale,
    ) -> Result<(Epoch, Self), ParsingError> {
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(ParsingError::EmptyEpoch),
        };

        let (epoch, rem) = line.split_at(23);
        let (a0, rem) = rem.split_at(19);
        let (a1, a2) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(ParsingError::EmptyEpoch),
        };
        let (a3, rem) = line.split_at(23);
        let (a4, rem) = rem.split_at(19);
        let (a5, a6) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(ParsingError::EmptyEpoch),
        };
        let (a7, a8) = line.split_at(23);

        let epoch = parse_epoch_in_timescale(epoch.trim(), ts)?;

        let alpha = (
            parse_f64(a0.trim()).map_err(|_| ParsingError::BdgimData)?,
            parse_f64(a1.trim()).map_err(|_| ParsingError::BdgimData)?,
            parse_f64(a2.trim()).map_err(|_| ParsingError::BdgimData)?,
            parse_f64(a3.trim()).map_err(|_| ParsingError::BdgimData)?,
            parse_f64(a4.trim()).map_err(|_| ParsingError::BdgimData)?,
            parse_f64(a5.trim()).map_err(|_| ParsingError::BdgimData)?,
            parse_f64(a6.trim()).map_err(|_| ParsingError::BdgimData)?,
            parse_f64(a7.trim()).map_err(|_| ParsingError::BdgimData)?,
            parse_f64(a8.trim()).map_err(|_| ParsingError::BdgimData)?,
        );

        Ok((epoch, Self { alpha }))
    }
}

impl BdModel {
    /// Formats the RINEX 4 BDGIM ION record (Table A37).
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
            NavFormatter::new(self.alpha.0),
            NavFormatter::new(self.alpha.1),
            NavFormatter::new(self.alpha.2),
        )?;

        writeln!(
            w,
            "    {}{}{}{}",
            NavFormatter::new(self.alpha.3),
            NavFormatter::new(self.alpha.4),
            NavFormatter::new(self.alpha.5),
            NavFormatter::new(self.alpha.6),
        )?;

        writeln!(
            w,
            "    {}{}",
            NavFormatter::new(self.alpha.7),
            NavFormatter::new(self.alpha.8),
        )?;

        Ok(())
    }
}
