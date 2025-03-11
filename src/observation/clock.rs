use crate::prelude::Epoch;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// [ClockObservation] represents the state of a clock with respect to a [TimeScale].
#[derive(Default, Copy, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ClockObservation {
    /// Offset to system time, in seconds.
    pub offset_s: f64,
    /// Possible clock drift estimate in s.s⁻¹
    pub drift_s_s: Option<f64>,
    /// Past [Epoch] of observation
    past_epoch: Option<Epoch>,
}

impl ClockObservation {
    /// Update fields with new offset [s].
    pub fn set_offset_s(&mut self, timeof_obs: Epoch, offset_s: f64) {
        if let Some(past) = self.past_epoch {
            let dt_s = (timeof_obs - past).to_seconds();
            self.drift_s_s = Some((offset_s - self.offset_s) / dt_s);
        }

        self.past_epoch = Some(timeof_obs);
        self.offset_s = offset_s;
    }

    /// Copies and define a new [ClockObservation] with given offset [s]
    pub fn with_offset_s(&self, timeof_obs: Epoch, offset_s: f64) -> Self {
        let mut s = self.clone();
        s.set_offset_s(timeof_obs, offset_s);
        s
    }
}
