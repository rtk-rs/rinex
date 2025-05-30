// NAV V4 System Time Messages
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::prelude::{Epoch, TimeScale};

use hifitime::{Duration, Polynomial};

pub(crate) mod formatting;
pub(crate) mod parsing;

#[cfg(feature = "processing")]
use qc_traits::TimePolynomial;

/// System Time (offset) Message
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TimeOffset {
    /// Left hand side [TimeScale]
    pub lhs: TimeScale,
    /// Reference [TimeScale]
    pub rhs: TimeScale,
    /// Reference time expressed as week counter and nanoseconds of week.
    pub t_ref: (u32, u64),
    /// Possible UTC ID# in case this came from RINEXv4
    pub utc: Option<String>,
    /// Interpolation polynomial
    pub polynomial: (f64, f64, f64),
}

impl TimeOffset {
    /// Define a new [TimeOffset]
    pub fn from_epoch(
        t_ref: Epoch,
        lhs: TimeScale,
        rhs: TimeScale,
        polynomial: (f64, f64, f64),
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
        polynomial: (f64, f64, f64),
    ) -> Self {
        Self {
            lhs,
            rhs,
            utc: None,
            polynomial,
            t_ref: (t_week, t_nanos),
        }
    }

    fn to_hifitime_polynomial(&self) -> Polynomial {
        Polynomial {
            constant: Duration::from_seconds(self.polynomial.0),
            rate: Duration::from_seconds(self.polynomial.1),
            accel: Duration::from_seconds(self.polynomial.2),
        }
    }

    #[cfg(feature = "processing")]
    pub(crate) fn to_time_polynomial(&self) -> TimePolynomial {
        TimePolynomial::from_reference_time_of_week_nanos(
            self.t_ref.0,
            self.t_ref.1,
            self.lhs,
            self.rhs,
            self.to_hifitime_polynomial(),
        )
    }
}
