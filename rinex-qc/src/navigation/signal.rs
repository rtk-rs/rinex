use crate::context::{
    meta::{MetaData, ObsMetaData},
    QcContext,
};

use rinex::prelude::{obs::SignalObservation, Epoch};

pub struct SignalSource<'a> {
    /// [Epoch] of the buffered signal (first signal of the next epoch)
    pub t: Option<Epoch>,
    /// Signals of the epoch being collected
    pub pending: Vec<SignalObservation>,
    /// First signal of the next epoch
    pub next: Option<SignalObservation>,
    pub iter: Box<dyn Iterator<Item = (Epoch, &'a SignalObservation)> + 'a>,
}

impl<'a> SignalSource<'a> {
    /// Collects all signals of the next [Epoch].
    /// Returns None once the source is exhausted.
    pub fn collect_epoch(&mut self) -> Option<(Epoch, &[SignalObservation])> {
        self.pending.clear();

        // the signal buffered by the previous call opens this epoch
        let mut epoch = self.t;

        if let Some(next) = self.next.take() {
            self.pending.push(next);
        }

        loop {
            match self.iter.next() {
                Some((t, signal)) => match epoch {
                    Some(current) if t > current => {
                        // new epoch: buffer this signal and exit
                        self.next = Some(signal.clone());
                        self.t = Some(t);
                        break;
                    },
                    _ => {
                        epoch = Some(t);
                        self.pending.push(signal.clone());
                    },
                },
                None => {
                    // source exhausted: publish the last epoch
                    self.t = None;
                    break;
                },
            }
        }

        let epoch = epoch?;

        if self.pending.is_empty() {
            None
        } else {
            Some((epoch, &self.pending))
        }
    }
}

impl QcContext {
    /// Obtain [SignalSource] from this [QcContext] for this particular [MetaData].
    pub fn rover_signal_source(&self, meta: &ObsMetaData) -> Option<SignalSource> {
        let rinex = self.obs_dataset.get(&meta)?;
        let iter = rinex.signal_observations_sampling_ok_iter();

        Some(SignalSource {
            iter,
            t: None,
            next: None,
            pending: Vec::with_capacity(128),
        })
    }

    /// Obtain [SignalSource] from this [QcContext] for this particular [MetaData].
    pub fn base_station_signal_source(&self, meta: &MetaData) -> Option<SignalSource> {
        let rinex = self.obs_dataset.get(&meta.to_base_obs_meta())?;
        let iter = rinex.signal_observations_sampling_ok_iter();

        Some(SignalSource {
            iter,
            t: None,
            next: None,
            pending: Vec::with_capacity(128),
        })
    }
}

#[cfg(test)]
mod test {
    use super::SignalSource;
    use rinex::prelude::{obs::SignalObservation, Epoch, Rinex};
    use std::collections::BTreeMap;

    /// Every epoch must be collected with its own signals and its own timestamp
    #[test]
    fn collect_epochs() {
        let path = format!(
            "{}/../test_resources/OBS/V3/DUTH0630.22O",
            env!("CARGO_MANIFEST_DIR")
        );

        let rinex = Rinex::from_file(&path).unwrap();

        let mut source = SignalSource {
            t: None,
            next: None,
            pending: Vec::new(),
            iter: rinex.signal_observations_sampling_ok_iter(),
        };

        // expected signals per (valid) epoch
        let mut expected = BTreeMap::<Epoch, Vec<&SignalObservation>>::new();

        for (k, sig) in rinex.signal_observations_iter() {
            if k.flag.is_ok() {
                expected.entry(k.epoch).or_default().push(sig);
            }
        }

        assert!(expected.len() > 1);

        for (epoch, signals) in expected.iter() {
            let (t, collected) = source.collect_epoch().expect("missing epoch");

            assert_eq!(t, *epoch);
            assert_eq!(collected.len(), signals.len(), "{}", t);
            assert!(collected.iter().all(|sig| signals.contains(&sig)), "{}", t);
        }

        assert!(source.collect_epoch().is_none());
    }
}
