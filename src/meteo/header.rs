//! Meteo specific Header fields
use std::{
    io::{BufWriter, Write},
    str::FromStr,
};

use crate::prelude::FormattingError;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "processing")]
use qc_traits::{FilterItem, MaskFilter, MaskOperand};

use crate::{meteo::Sensor, prelude::Observable};

/// Meteo specific header fields
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Observation types contained in this file
    pub codes: Vec<Observable>,
    /// Sensors that produced the following observables
    pub sensors: Vec<Sensor>,
}

impl HeaderFields {
    /// Formats [HeaderFields] into [BufWriter].
    pub(crate) fn format<W: Write>(&self, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        const NUM_OBSERVABLES_PER_LINE: usize = 9;

        write!(w, "{:6}", self.codes.len())?;
        let mut modulo = 0;

        for (nth, observable) in self.codes.iter().enumerate() {
            if nth > 0 && (nth % NUM_OBSERVABLES_PER_LINE) == 0 {
                write!(w, "      ")?;
            }

            write!(w, "    {}", observable)?;

            if (nth % NUM_OBSERVABLES_PER_LINE) == NUM_OBSERVABLES_PER_LINE - 1 {
                write!(w, "# / TYPES OF OBSERV\n      ")?;
            }

            modulo = nth % NUM_OBSERVABLES_PER_LINE;
        }

        if modulo != 7 {
            writeln!(
                w,
                "{:>width$}",
                "# / TYPES OF OBSERV",
                width = 79 - 6 - (modulo + 1) * 6
            )?;
        }

        for sensor in self.sensors.iter() {
            writeln!(w, "{}", sensor)?;
        }

        Ok(())
    }
}

#[cfg(feature = "processing")]
impl HeaderFields {
    pub(crate) fn mask_mut(&mut self, f: &MaskFilter) {
        match f.operand {
            MaskOperand::Equals => match &f.item {
                FilterItem::ComplexItem(complex) => {
                    // try to interprate as [Observable]
                    let observables = complex
                        .iter()
                        .filter_map(|f| {
                            if let Ok(ob) = Observable::from_str(f) {
                                Some(ob)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    self.codes.retain(|c| observables.contains(&c));
                },
                _ => {},
            },
            MaskOperand::NotEquals => match &f.item {
                FilterItem::ComplexItem(complex) => {
                    // try to interprate as [Observable]
                    let observables = complex
                        .iter()
                        .filter_map(|f| {
                            if let Ok(ob) = Observable::from_str(f) {
                                Some(ob)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    self.codes.retain(|c| !observables.contains(&c));
                },
                _ => {},
            },
            _ => {},
        }
    }
}
