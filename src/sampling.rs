use crate::prelude::{Duration, Epoch, Rinex, TimeSeries};

impl Rinex {
    /// Returns first [Epoch] encountered in time
    pub fn first_epoch(&self) -> Option<Epoch> {
        self.epoch_iter().next()
    }

    /// Returns last [Epoch] encountered in time
    pub fn last_epoch(&self) -> Option<Epoch> {
        self.epoch_iter().last()
    }

    /// Returns total [Duration] this [Rinex].
    pub fn duration(&self) -> Option<Duration> {
        let start = self.first_epoch()?;
        let end = self.last_epoch()?;
        Some(end - start)
    }

    /// Form a [`Timeseries`] iterator spanning [Self::duration]
    /// with [Self::dominant_sample_rate] spacing
    pub fn timeseries(&self) -> Option<TimeSeries> {
        let start = self.first_epoch()?;
        let end = self.last_epoch()?;
        let dt = self.dominant_sampling_interval()?;
        Some(TimeSeries::inclusive(start, end, dt))
    }

    /// Returns sample rate report by the GNSS receiver (if any).
    /// NB: this is not actual data set analysis.
    pub fn sampling_interval(&self) -> Option<Duration> {
        self.header.sampling_interval
    }

    /// Returns dominant sampling period, expressed as [Duration], by actual data analysis.
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// assert_eq!(
    ///     rnx.dominant_sampling_interval(),
    ///     Some(Duration::from_seconds(60.0)));
    /// ```
    pub fn dominant_sampling_interval(&self) -> Option<Duration> {
        self.sampling_histogram()
            .max_by(|(_, pop_i), (_, pop_j)| pop_i.cmp(pop_j))
            .map(|dominant| dominant.0)
    }

    /// Returns dominant sample rate (in Hertz) by actual data analysis.
    pub fn dominant_sampling_rate_hz(&self) -> Option<f64> {
        let interval = self.dominant_sampling_interval()?;
        Some(1.0 / interval.to_seconds())
    }

    /// Histogram analysis on Epoch interval. Although
    /// it is feasible on all types indexed by [Epoch],
    /// this operation only makes truly sense on Observation Data.
    /// ```
    /// use rinex::prelude::*;
    /// use itertools::Itertools;
    /// use std::collections::HashMap;
    /// let rinex = Rinex::from_file("test_resources/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    ///  assert!(
    ///     rinex.sampling_histogram().sorted().eq(vec![
    ///         (Duration::from_seconds(30.0), 1),
    ///     ]),
    ///     "sampling_histogram failed"
    /// );
    /// ```
    pub fn sampling_histogram(&self) -> Box<dyn Iterator<Item = (Duration, usize)> + '_> {
        // compute dt = |e_k+1 - e_k| : instantaneous epoch delta
        //              then compute an histogram on these intervals
        Box::new(
            self.epoch_iter()
                .zip(self.epoch_iter().skip(1))
                .map(|(ek, ekp1)| ekp1 - ek) // following step computes the histogram
                // and at the same time performs a .unique() like filter
                .fold(vec![], |mut list, dt| {
                    let mut found = false;
                    for (delta, pop) in list.iter_mut() {
                        if *delta == dt {
                            *pop += 1;
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        list.push((dt, 1));
                    }
                    list
                })
                .into_iter(),
        )
    }

    /// Returns True if Self has a steady sampling, ie., all epoch interval
    /// are evenly spaced
    pub fn steady_sampling(&self) -> bool {
        self.sampling_histogram().count() == 1
    }

    /// Returns an iterator over unexpected data gaps,
    /// in the form ([`Epoch`], [`Duration`]), where
    /// epoch is the starting datetime, and its related duration.
    /// ```
    /// use std::str::FromStr;
    /// use rinex::prelude::{Rinex, Epoch, Duration};
    /// let rinex = Rinex::from_file("test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    ///
    /// // when tolerance is set to None,
    /// // the reference sample rate is [Self::dominant_sample_rate].
    /// let mut tolerance : Option<Duration> = None;
    /// let gaps : Vec<_> = rinex.data_gaps(tolerance).collect();
    /// assert!(
    ///     rinex.data_gaps(None).eq(
    ///         vec![
    ///             (Epoch::from_str("2015-01-01T00:09:00 UTC").unwrap(), Duration::from_seconds(8.0 * 3600.0 + 51.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T09:04:00 UTC").unwrap(), Duration::from_seconds(10.0 * 3600.0 + 21.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T19:54:00 UTC").unwrap(), Duration::from_seconds(3.0 * 3600.0 + 1.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T23:02:00 UTC").unwrap(), Duration::from_seconds(7.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T23:21:00 UTC").unwrap(), Duration::from_seconds(31.0 * 60.0)),
    ///         ]),
    ///     "data_gaps(tol=None) failed"
    /// );
    ///
    /// // with a tolerance, we tolerate the given gap duration
    /// tolerance = Some(Duration::from_seconds(3600.0));
    /// let gaps : Vec<_> = rinex.data_gaps(tolerance).collect();
    /// assert!(
    ///     rinex.data_gaps(Some(Duration::from_seconds(3.0 * 3600.0))).eq(
    ///         vec![
    ///             (Epoch::from_str("2015-01-01T00:09:00 UTC").unwrap(), Duration::from_seconds(8.0 * 3600.0 + 51.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T09:04:00 UTC").unwrap(), Duration::from_seconds(10.0 * 3600.0 + 21.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T19:54:00 UTC").unwrap(), Duration::from_seconds(3.0 * 3600.0 + 1.0 * 60.0)),
    ///         ]),
    ///     "data_gaps(tol=3h) failed"
    /// );
    /// ```
    pub fn data_gaps(
        &self,
        tolerance: Option<Duration>,
    ) -> Box<dyn Iterator<Item = (Epoch, Duration)> + '_> {
        let sample_rate: Duration = match tolerance {
            Some(dt) => dt, // user defined
            None => {
                match self.dominant_sampling_interval() {
                    Some(dt) => dt,
                    None => {
                        match self.sampling_interval() {
                            Some(dt) => dt,
                            None => {
                                // not enough information
                                // this is probably not an Epoch iterated RINEX
                                return Box::new(Vec::<(Epoch, Duration)>::new().into_iter());
                            },
                        }
                    },
                }
            },
        };
        Box::new(
            self.epoch_iter()
                .zip(self.epoch_iter().skip(1))
                .filter_map(move |(ek, ekp1)| {
                    let dt = ekp1 - ek; // gap
                    if dt > sample_rate {
                        // too large
                        Some((ek, dt)) // retain starting datetime and gap duration
                    } else {
                        None
                    }
                }),
        )
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::Rinex;

    #[test]
    #[cfg(feature = "flate2")]
    fn glacier_20240506_dominant_sample_rate() {
        let rnx = Rinex::from_gzip_file(format!(
            "{}/test_resources/OBS/V3/240506_glacier_station.obs.gz",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();

        let sampling_rate_hz = rnx.dominant_sampling_rate_hz().unwrap();

        assert_eq!(sampling_rate_hz, 1.0);
    }
}
