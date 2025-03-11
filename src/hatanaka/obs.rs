//! Observation Compression kernel

use crate::hatanaka::{NumDiff, TextDiff};

pub struct ObsDiff<const M: usize> {
    pub data_diff: NumDiff<M>,
    pub snr_diff: TextDiff,
    pub lli_diff: TextDiff,
}

impl<const M: usize> ObsDiff<M> {
    pub fn new(obsdata: i64, obsdata_level: usize, snr: &str, lli: &str) -> Self {
        Self {
            snr_diff: TextDiff::new(snr),
            lli_diff: TextDiff::new(lli),
            data_diff: NumDiff::<M>::new(obsdata, obsdata_level),
        }
    }
    pub fn force_init(&mut self, obsdata: i64, obsdata_level: usize, snr: &str, lli: &str) {
        self.snr_diff.force_init(snr);
        self.lli_diff.force_init(lli);
        self.data_diff.force_init(obsdata, obsdata_level);
    }
}
