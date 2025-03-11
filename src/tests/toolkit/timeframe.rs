//! Timeframe for complex sampling description and testing
use std::str::FromStr;
use std::vec::IntoIter;

use hifitime::{Duration, Epoch, TimeSeries};

#[derive(Debug, Clone)]
pub enum TimeFrame {
    Erratic(IntoIter<Epoch>),
    EvenlySpaced(TimeSeries),
}

impl TimeFrame {
    pub fn evenly_spaced(ts: TimeSeries) -> Self {
        Self::EvenlySpaced(ts.into_iter())
    }

    pub fn from_erratic_csv(csv: &str) -> Self {
        let epochs = csv
            .split(',')
            .map(|c| Epoch::from_str(c).unwrap())
            .collect::<Vec<_>>();
        Self::Erratic(epochs.into_iter())
    }

    pub fn from_inclusive_csv(csv: &str) -> Self {
        let mut csv = csv.split(',');
        let t0 = csv.next().unwrap();
        let t1 = csv.next().unwrap();
        let dt = csv.next().unwrap();
        let t0 = Epoch::from_str(t0).unwrap();
        let t1 = Epoch::from_str(t1).unwrap();
        let dt = Duration::from_str(dt).unwrap();
        let ts = TimeSeries::inclusive(t0, t1, dt);
        Self::evenly_spaced(ts)
    }

    pub fn from_exclusive_csv(csv: &str) -> Self {
        let mut csv = csv.split(',');
        let t0 = csv.next().unwrap();
        let t1 = csv.next().unwrap();
        let dt = csv.next().unwrap();
        let t0 = Epoch::from_str(t0).unwrap();
        let t1 = Epoch::from_str(t1).unwrap();
        let dt = Duration::from_str(dt).unwrap();
        let ts = TimeSeries::exclusive(t0, t1, dt);
        Self::evenly_spaced(ts)
    }
}

impl Iterator for TimeFrame {
    type Item = Epoch;
    fn next(&mut self) -> Option<Epoch> {
        match self {
            Self::Erratic(i) => i.next(),
            Self::EvenlySpaced(i) => i.next(),
        }
    }
}
