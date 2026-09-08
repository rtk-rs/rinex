use crate::{
    observation::LliFlags,
    observation::SNR,
    prelude::{Observable, SV},
};

/// [SignalObservation] is the result of sampling one signal at
/// one point in time, by a GNSS receiver.
#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SignalObservation {
    /// [SV] is the signal source
    pub sv: SV,

    /// Actual measurement. Unit depends on [Observable].
    pub value: f64,

    /// [Observable]
    pub observable: Observable,

    /// Lock loss indicator (when present)
    pub lli: Option<LliFlags>,

    /// SNR estimate (when present)
    pub snr: Option<SNR>,
}

impl SignalObservation {
    /// Builds new signal observation
    pub fn new(sv: SV, observable: Observable, value: f64) -> Self {
        Self {
            sv,
            observable,
            value,
            lli: None,
            snr: None,
        }
    }

    /// Copy and define [SNR]
    pub fn with_snr(&self, snr: SNR) -> Self {
        let mut s = self.clone();
        s.snr = Some(snr);
        s
    }

    /// [Observation] is said OK when
    ///  - If LLI is present it must match [LliFlags::OK_OR_UNKNOWN]
    ///  - If SNR is present, it must be [SNR::strong]
    ///  - NB: when both are missing, we still return OK.
    /// This allows method that Iterate over OK Epoch Data to consider
    /// data when SNR or LLI are missing.
    pub fn is_ok(self) -> bool {
        let lli_ok = self.lli.unwrap_or(LliFlags::OK_OR_UNKNOWN) == LliFlags::OK_OR_UNKNOWN;
        let snr_ok = self.snr.map(|snr| snr.strong()).unwrap_or(true);
        lli_ok && snr_ok
    }

    /// [Observation::is_ok] with additional SNR criteria to match (>=).
    /// SNR must then be present otherwise this is not OK.
    pub fn is_ok_snr(&self, min_snr: SNR) -> bool {
        let lli_ok = self.lli.unwrap_or(LliFlags::OK_OR_UNKNOWN) == LliFlags::OK_OR_UNKNOWN;
        lli_ok && self.snr.map(|snr| snr >= min_snr).unwrap_or(false)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        observation::{LliFlags, SignalObservation, SNR},
        prelude::{Observable, SV},
    };
    use std::str::FromStr;

    #[test]
    fn signal_validity() {
        let g01 = SV::from_str("G01").unwrap();
        let l1c = Observable::from_str("L1C").unwrap();

        // no LLI, no SNR: OK, but not OK against a minimal SNR
        let signal = SignalObservation::new(g01, l1c, 1.0);
        assert!(signal.clone().is_ok());
        assert!(!signal.is_ok_snr(SNR::DbHz30_35));

        // strong SNR
        let strong = signal.with_snr(SNR::DbHz36_41);
        assert!(strong.clone().is_ok());
        assert!(strong.is_ok_snr(SNR::DbHz30_35));
        assert!(!strong.is_ok_snr(SNR::DbHz42_47));

        // weak SNR
        let weak = signal.with_snr(SNR::DbHz18_23);
        assert!(!weak.clone().is_ok());
        assert!(!weak.is_ok_snr(SNR::DbHz30_35));

        // sane LLI flags
        let mut flagged = strong.clone();
        flagged.lli = Some(LliFlags::OK_OR_UNKNOWN);
        assert!(flagged.clone().is_ok());
        assert!(flagged.is_ok_snr(SNR::DbHz30_35));

        // lock loss
        let mut lock_loss = strong.clone();
        lock_loss.lli = Some(LliFlags::LOCK_LOSS);
        assert!(!lock_loss.clone().is_ok());
        assert!(!lock_loss.is_ok_snr(SNR::DbHz30_35));
    }
}
