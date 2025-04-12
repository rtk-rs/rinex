// NAV V4 System Time Messages
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::prelude::{Epoch, TimeScale};

use hifitime::Unit;

pub(crate) mod formatting;
pub(crate) mod parsing;

/// System Time (offset) Message
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TimeOffset {
    /// Left hand side (compared) [TimeScale]
    pub lhs: TimeScale,
    /// Reference (right hand side) [TimeScale]
    pub rhs: TimeScale,
    /// Reference time expressed as week counter and nanoseconds of week.
    pub t_ref: (u32, u64),
    /// Possible UTC ID# in case this came from RINEXv4
    pub utc: Option<String>,
    /// Polynomials (s, s.s⁻¹, s.s⁻²)
    pub polynomials: (f64, f64, f64),
}

impl TimeOffset {
    /// Define a new [TimeOffset]
    pub fn from_epoch(
        t_ref: Epoch,
        lhs: TimeScale,
        rhs: TimeScale,
        polynomials: (f64, f64, f64),
    ) -> Self {
        let t_ref = t_ref.to_time_scale(lhs).to_time_of_week();
        Self {
            lhs,
            rhs,
            t_ref,
            polynomials,
            utc: None,
        }
    }

    /// Define a new [TimeOffset]
    pub fn from_time_of_week(
        t_week: u32,
        t_nanos: u64,
        lhs: TimeScale,
        rhs: TimeScale,
        polynomials: (f64, f64, f64),
    ) -> Self {
        Self {
            lhs,
            rhs,
            polynomials,
            utc: None,
            t_ref: (t_week, t_nanos),
        }
    }

    /// Returns the total number of nanoseconds to apply to convert this [Epoch] to targetted [TimeScale].
    /// [Epoch] must fall in the current week that this [TimeOffset] is expressed in.
    /// This means a weekly update (which is not enough for precise applications) is a minimum.
    pub fn time_correction_nanos(&self, t: Epoch, target: TimeScale) -> Option<f64> {
        if t.time_scale == target {
            // no correction required..
            return Some(0.0);
        }

        let (t_week, t_nanos) = t.to_time_of_week();

        if t_week != self.t_ref.0 {
            // week mismatch
            return None;
        }

        let (a0, a1, a2) = self.polynomials;
        let dt_s = (t_nanos as f64 - self.t_ref.1 as f64) * 1.0E-9;
        let dt_s = a0 + a1 * dt_s + a2 * dt_s.powi(2);
        Some(dt_s * 1.0E9)
    }

    /// Convert provided [Epoch] to desired [TimeScale] using this [TimeOffset] which needs
    /// to be available for the same current week. Note that a weekly update is not enough for
    /// precise applications. This method is limited by Hifitime to a 1 nanosecond precision.
    pub fn epoch_time_correction(&self, t: Epoch, target: TimeScale) -> Option<Epoch> {
        let correction_nanos = self.time_correction_nanos(t, target)?;

        if t.time_scale == self.lhs && target == self.rhs {
            Some(t + correction_nanos * Unit::Nanosecond)
        } else if t.time_scale == self.rhs && target == self.lhs {
            Some(t - correction_nanos * Unit::Nanosecond)
        } else {
            None
        }
    }
}
