use gnss_rs::prelude::{COSPAR, SV};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct SvAntenna {
    /// Spacecraft to which this antenna is attached to
    pub sv: SV,
    /// [COSPAR] launch detail
    pub cospar: COSPAR,
    /// IGS antenna code
    pub igs_type: String,
}
