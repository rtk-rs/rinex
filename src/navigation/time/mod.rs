// NAV V4 System Time Messages
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::prelude::{Duration, Epoch, ParsingError, TimeScale};

use hifitime::Unit;

pub(crate) mod parsing;

/// System Time (offset) Message
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TimeOffset {
    /// Left hand side (compared) [TimeScale]
    pub lhs: TimeScale,
    /// Reference (right hand side) [TimeScale]
    pub rhs: TimeScale,
    /// Reference [Epoch]
    pub t_ref: Epoch,
    /// Possible UTC ID# in case this came from RINEXv4
    pub utc: Option<String>,
    /// Polynomials (s, s.s⁻¹, s.s⁻²)
    pub polynomials: (f64, f64, f64),
}

impl TimeOffset {
    /// Define a new [TimeOffset]
    pub fn new(lhs: TimeScale, rhs: TimeScale, t_ref: Epoch, polynomials: (f64, f64, f64)) -> Self {
        Self {
            lhs,
            rhs,
            t_ref,
            polynomials,
            utc: None,
        }
    }

    /// Calculate time offset as [Duration]
    pub fn time_offset(&self, t: Epoch) -> Duration {
        let dt = (t.to_time_scale(self.t_ref.time_scale) - self.t_ref).to_unit(Unit::Second);

        let (a0, a1, a2) = self.polynomials;

        Duration::from_seconds(a0 + a1 * dt + a2 * dt.powi(2))
    }
}
