//! Antex (ATX) - special RINEX, for antenna caracteristics
pub mod antenna;
pub mod frequency;
pub mod pcv;
pub mod record;

pub use pcv::Pcv;
// pub use frequency::{Frequency, Pattern};

pub use antenna::{
    Antenna, AntennaMatcher, AntennaSpecific, Calibration, CalibrationMethod, RxAntenna, SvAntenna,
};

pub use record::{FrequencyDependentData, Record};

use crate::prelude::FormattingError;

use std::io::{BufWriter, Write};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Type of Phase Center Variation in use
    pub pcv_type: pcv::Pcv,
    /// Optionnal reference antenna Serial Number
    /// used to produce this calibration file
    pub reference_ant_sn: Option<String>,
}

impl HeaderFields {
    /// Formats [HeaderFields] into [BufWriter].
    pub(crate) fn format<W: Write>(&self, _w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        Ok(())
    }

    /// Set Phase Center Variations type
    pub fn with_pcv_type(&self, pcv: Pcv) -> Self {
        let mut s = self.clone();
        s.pcv_type = pcv;
        s
    }

    /// Sets Reference Antenna serial number
    pub fn with_reference_antenna_sn(&self, sn: &str) -> Self {
        let mut s = self.clone();
        s.reference_ant_sn = Some(sn.to_string());
        s
    }
}
