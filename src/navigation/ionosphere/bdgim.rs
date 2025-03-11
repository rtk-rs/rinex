use crate::{
    epoch::parse_in_timescale as parse_epoch_in_timescale,
    prelude::{Epoch, ParsingError, TimeScale},
};

use std::str::FromStr;

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
            f64::from_str(a0.trim()).map_err(|_| ParsingError::BdgimData)?,
            f64::from_str(a1.trim()).map_err(|_| ParsingError::BdgimData)?,
            f64::from_str(a2.trim()).map_err(|_| ParsingError::BdgimData)?,
            f64::from_str(a3.trim()).map_err(|_| ParsingError::BdgimData)?,
            f64::from_str(a4.trim()).map_err(|_| ParsingError::BdgimData)?,
            f64::from_str(a5.trim()).map_err(|_| ParsingError::BdgimData)?,
            f64::from_str(a6.trim()).map_err(|_| ParsingError::BdgimData)?,
            f64::from_str(a7.trim()).map_err(|_| ParsingError::BdgimData)?,
            f64::from_str(a8.trim()).map_err(|_| ParsingError::BdgimData)?,
        );

        Ok((epoch, Self { alpha }))
    }
    // /* converts self to meters of delay */
    // pub(crate) fn meters_delay(&self, freq: f64) -> f64 {
    //     0.0_f64
    // }
}
