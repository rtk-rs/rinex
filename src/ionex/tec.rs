use crate::ionex::Quantized;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Total Electron Content (TEC) estimate
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TEC {
    /// TEC quantized in TEcu
    tecu: Quantized,
    /// RMS (TEC)
    rms: Option<Quantized>,
    /// Altitude offset for complex 3D height map
    height: Option<Quantized>,
}

impl TEC {
    /// Builds new [TEC] from TEC estimate expressed in TECu (=10^16 m-2)
    pub fn from_tecu(tecu: f64) -> Self {
        let exponent = Quantized::find_exponent(tecu);
        Self {
            rms: None,
            height: None,
            tecu: Quantized::new(tecu, exponent),
        }
    }

    /// Builds new [TEC] from raw TEC estimate in m^-2
    pub fn from_tec_m2(tec: f64) -> Self {
        let tecu = tec / 10.0E16;
        let exponent = Quantized::find_exponent(tecu);
        Self {
            rms: None,
            height: None,
            tecu: Quantized::new(tecu, exponent),
        }
    }

    /// Builds new [TEC] estimate with associated RMS value
    pub fn with_rms(&self, rms: f64) -> Self {
        let mut s = self.clone();
        let exponent = Quantized::find_exponent(rms);
        s.rms = Some(Quantized::new(rms, exponent));
        s
    }

    /// Builds new [TEC] from TEC quantization in TECu
    pub(crate) fn from_quantized(tecu: i64, exponent: i8) -> Self {
        // IONEX stores quantized TEC as i=10*-k TECu
        Self {
            rms: None,
            height: None,
            tecu: Quantized {
                quantized: tecu,
                exponent: -exponent,
            },
        }
    }

    /// Updates RMS [TEC]
    pub(crate) fn set_quantized_rms(&mut self, rms: i64, exponent: i8) {
        self.rms = Some(Quantized {
            exponent: -exponent,
            quantized: rms,
        });
    }

    /// Returns Total Electron Content estimate, in TECu (=10^-16 m-2)
    pub fn tecu(&self) -> f64 {
        self.tecu.real_value()
    }

    /// Returns Total Electron Content estimate, in m-2
    pub fn tec(&self) -> f64 {
        self.tecu() * 10.0E16
    }

    /// Returns Root Mean Square (TEC) if it was provided.
    pub fn rms_tec(&self) -> Option<f64> {
        let rms = self.rms?;
        Some(rms.real_value())
    }

    // /// Returns possible altitude offset [km]
    // pub(crate) fn height_offset_km(&self) -> Option<f64> {
    //     let height = self.height?;
    //     Some(height.real_value())
    // }
}

#[cfg(feature = "qc")]
use qc_traits::{Merge, MergeError};

#[cfg(feature = "qc")]
impl Merge for TEC {
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError> {
        let mut s = self.clone();
        s.merge_mut(&rhs)?;
        Ok(s)
    }

    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError> {
        if self.rms.is_none() {
            self.rms = rhs.rms.clone();
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::TEC;

    #[test]
    fn quantized_tec() {
        let tec = TEC::from_quantized(30, -1);
        assert_eq!(tec.tecu(), 3.0);
        assert_eq!(tec.tec(), 3.0 * 10E16);

        let tec = TEC::from_quantized(30, -2);
        assert_eq!(tec.tecu(), 0.3);
        assert_eq!(tec.tec(), 0.3 * 10E16);

        let tec = TEC::from_tec_m2(1.0 * 10E16);
        assert_eq!(tec.tecu(), 1.0);
        assert_eq!(tec.tec(), 1.0 * 10E16);
        assert_eq!(tec, TEC::from_tecu(1.0));

        let tec = TEC::from_tec_m2(3.5 * 10E16);
        assert_eq!(tec.tecu(), 3.5);
        assert_eq!(tec.tec(), 3.5 * 10E16);
        assert_eq!(tec, TEC::from_tecu(3.5));

        let tec = TEC::from_tec_m2(190355078157525800.0);
        assert_eq!(tec.tec(), 1.903550781575258e17);
    }
}
