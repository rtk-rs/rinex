// NAV V4 System Time Messages
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::prelude::{Epoch, TimeScale};

use hifitime::Polynomial;

pub(crate) mod formatting;
pub(crate) mod parsing;

/// System Time (offset) Message
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct TimeOffset {
    /// Left hand side [TimeScale]
    pub lhs: TimeScale,
    /// Reference [TimeScale]
    pub rhs: TimeScale,
    /// Reference time expressed as week counter and nanoseconds of week.
    pub t_ref: (u32, u64),
    /// Possible UTC ID# in case this came from RINEXv4
    pub utc: Option<String>,
    /// [Polynomial]
    pub polynomial: Polynomial,
}

impl TimeOffset {
    /// Define a new [TimeOffset]
    pub fn from_epoch(
        t_ref: Epoch,
        lhs: TimeScale,
        rhs: TimeScale,
        polynomial: Polynomial,
    ) -> Self {
        let t_ref = t_ref.to_time_scale(lhs).to_time_of_week();
        Self {
            lhs,
            rhs,
            t_ref,
            utc: None,
            polynomial,
        }
    }

    /// Define a new [TimeOffset]
    pub fn from_time_of_week(
        t_week: u32,
        t_nanos: u64,
        lhs: TimeScale,
        rhs: TimeScale,
        polynomial: Polynomial,
    ) -> Self {
        Self {
            lhs,
            rhs,
            polynomial,
            utc: None,
            t_ref: (t_week, t_nanos),
        }
    }

    /// Tranposes proposed [Epoch] into desired [TimeScale] if that is feasible using this [TimeOffset] structure.
    pub fn epoch_time_correction(&self, t: Epoch, target: TimeScale) -> Option<Epoch> {
        if self.lhs == t.time_scale && self.rhs == target {
            // forward
            let ref_epoch = Epoch::from_time_of_week(self.t_ref.0, self.t_ref.1, t.time_scale);

            match t.precise_timescale_conversion(true, ref_epoch, self.polynomial, target) {
                Ok(epoch) => Some(epoch),
                Err(_) => None, // should not happen at this point
            }
        } else if self.lhs == target && self.rhs == t.time_scale {
            // backwards
            let ref_epoch = Epoch::from_time_of_week(self.t_ref.0, self.t_ref.1, t.time_scale);

            match t.precise_timescale_conversion(false, ref_epoch, self.polynomial, target) {
                Ok(epoch) => Some(epoch),
                Err(_) => None, // should not happen at this point
            }
        } else {
            // invalid
            None
        }
    }
}
