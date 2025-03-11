use crate::{
    ionex::TEC,
    observation::SignalObservation,
    prelude::{Epoch, Observable, Rinex, SV},
};

use std::collections::HashMap;

/// The [TEC] estimate is indexed by [TECKey] when
/// calculated from Observation RINEX.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TECKey {
    /// [SV] is the signal source
    pub sv: SV,
    /// [Epoch] of estimation
    pub epoch: Epoch,
    /// RHS signal
    pub rhs: Observable,
    /// Reference (pivot) signal
    pub reference: Observable,
}

impl Rinex {
    /// Calculates Total Electron Contect (TEC) for each SV signal observed by this
    /// Observation RINEX. The TEC is evaluated using a dual frequency model:
    /// this limits the method to dual frequency Observation RINEX.
    /// Returns [TEC] sorted per [TECKey]
    pub fn observation_dual_phase_ionosphere_tec(&self) -> HashMap<TECKey, TEC> {
        let mut t = Epoch::default();
        let mut ret = HashMap::new();
        let mut phases = Vec::<SignalObservation>::new();

        let dt = self.dominant_sampling_interval();
        if dt.is_none() {
            return ret;
        }

        let dt = dt.unwrap();

        for (k, ph) in self.phase_range_sampling_ok_iter() {
            if k >= t + dt {
                for ph_i in phases.iter() {
                    for ph_j in phases.iter() {
                        if let Some(tec) = ph_i.tec_estimate(&ph_j) {
                            ret.insert(
                                TECKey {
                                    epoch: t,
                                    sv: ph_i.sv,
                                    rhs: ph_j.observable.clone(),
                                    reference: ph_i.observable.clone(),
                                },
                                tec,
                            );
                        }
                    }
                }
                phases.clear();
            }

            t = k;
            phases.push(ph.clone());
        }

        ret
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::{Carrier, Epoch, Observable, Rinex, SV};
    use hifitime::Unit;
    use std::str::FromStr;

    #[test]
    fn dual_phase_range_tec() {
        let gamma = 1.0 / 40.308;

        let path = format!(
            "{}/test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx",
            env!("CARGO_MANIFEST_DIR"),
        );

        let rinex = Rinex::from_file(&path).unwrap();

        let t0 = Epoch::from_str("2021-12-21T00:00:00 GPST").unwrap();
        let t1 = t0 + 30.0 * Unit::Second;
        let t2 = t1 + 30.0 * Unit::Second;

        let g01 = SV::from_str("G01").unwrap();
        let g07 = SV::from_str("G07").unwrap();
        let e11 = SV::from_str("E11").unwrap();

        let f_l1 = Carrier::L1.frequency().powi(2);
        let f_l2 = Carrier::L2.frequency().powi(2);

        let l1c = Observable::from_str("L1C").unwrap();
        let l2s = Observable::from_str("L2S").unwrap();
        let l2w = Observable::from_str("L2W").unwrap();
        let l5q = Observable::from_str("L5Q").unwrap();
        let l6c = Observable::from_str("L6C").unwrap();
        let l7q = Observable::from_str("L7Q").unwrap();
        let l8q = Observable::from_str("L8Q").unwrap();

        let tec = rinex.observation_dual_phase_ionosphere_tec();

        let mut tests_passed = 0;

        for (k, tec) in tec {
            if k.epoch == t0 {
                if k.sv == g01 {
                    if (&k.reference, &k.rhs) == (&l1c, &l2s) {
                        let l1c = 129274705.784;
                        let l2s = 100733552.500;
                        let err = (tec.tecu()
                            - gamma / 10.0E16 * f_l1 * f_l2 / (f_l1 - f_l2) * (l1c - l2s))
                            .abs();
                        assert!(err < 1.0, "{}: error too large", err);
                        tests_passed += 1;
                    } else if (&k.reference, &k.rhs) == (&l1c, &l2w) {
                        let l1c = 129274705.784;
                        let l2w = 100733552.498;
                        let err = (tec.tecu()
                            - gamma / 10.0E16 * f_l1 * f_l2 / (f_l1 - f_l2) * (l1c - l2w))
                            .abs();
                        assert!(err < 1.0, "{}: error too large", err);
                        tests_passed += 1;
                    } else if (&k.reference, &k.rhs) == (&l1c, &l5q) {
                    } else {
                        panic!("found invalid observable");
                    }
                } else if k.sv == g07 {
                    if (&k.reference, &k.rhs) == (&l1c, &l2s) {
                        let l1c = 125167854.812;
                        let l2s = 97533394.668;
                        let err = (tec.tecu()
                            - gamma / 10.0E16 * f_l1 * f_l2 / (f_l1 - f_l2) * (l1c - l2s))
                            .abs();
                        assert!(err < 1.0, "{}: error too large", err);
                        tests_passed += 1;
                    } else if (&k.reference, &k.rhs) == (&l1c, &l2w) {
                        let l1c = 125167854.812;
                        let l2w = 97533382.650;
                        let err = (tec.tecu()
                            - gamma / 10.0E16 * f_l1 * f_l2 / (f_l1 - f_l2) * (l1c - l2w))
                            .abs();
                        assert!(err < 1.0, "{}: error too large", err);
                        tests_passed += 1;
                    } else if (&k.reference, &k.rhs) == (&l1c, &l5q) {
                    } else {
                        panic!("found invalid observable");
                    }
                } else if k.sv == e11 {
                    if (&k.reference, &k.rhs) == (&l1c, &l5q) {
                    } else if (&k.reference, &k.rhs) == (&l1c, &l6c) {
                    } else if (&k.reference, &k.rhs) == (&l1c, &l7q) {
                    } else if (&k.reference, &k.rhs) == (&l1c, &l8q) {
                    } else {
                        panic!("found invalid observable ({},{})", k.reference, k.rhs);
                    }
                }
            } else if k.epoch == t1 {
                if k.sv == g01 {
                    if (&k.reference, &k.rhs) == (&l1c, &l2s) {
                        let l1c = 129166303.651;
                        let l2s = 100649083.264;
                        let err = (tec.tecu()
                            - gamma / 10.0E16 * f_l1 * f_l2 / (f_l1 - f_l2) * (l1c - l2s))
                            .abs();
                        assert!(err < 1.0, "{}: error too large", err);
                        tests_passed += 1;
                    } else if (&k.reference, &k.rhs) == (&l1c, &l2w) {
                        let l1c = 129166303.651;
                        let l2w = 100649083.262;
                        let err = (tec.tecu()
                            - gamma / 10.0E16 * f_l1 * f_l2 / (f_l1 - f_l2) * (l1c - l2w))
                            .abs();
                        assert!(err < 1.0, "{}: error too large", err);
                        tests_passed += 1;
                    } else if (&k.reference, &k.rhs) == (&l1c, &l5q) {
                    } else {
                        panic!("found invalid observable");
                    }
                } else if k.sv == g07 {
                    if (&k.reference, &k.rhs) == (&l1c, &l2s) {
                        let l1c = 125169581.982;
                        let l2s = 97534740.496;
                        let err = (tec.tecu()
                            - gamma / 10.0E16 * f_l1 * f_l2 / (f_l1 - f_l2) * (l1c - l2s))
                            .abs();
                        assert!(err < 1.0, "{}: error too large", err);
                        tests_passed += 1;
                    } else if (&k.reference, &k.rhs) == (&l1c, &l2w) {
                        let l1c = 125169581.982;
                        let l2w = 97534728.482;
                        let err = (tec.tecu()
                            - gamma / 10.0E16 * f_l1 * f_l2 / (f_l1 - f_l2) * (l1c - l2w))
                            .abs();
                        assert!(err < 1.0, "{}: error too large", err);
                        tests_passed += 1;
                    } else if (&k.reference, &k.rhs) == (&l1c, &l5q) {
                    } else {
                        panic!("found invalid observable");
                    }
                } else if k.sv == e11 {
                    if (&k.reference, &k.rhs) == (&l1c, &l5q) {
                    } else if (&k.reference, &k.rhs) == (&l1c, &l6c) {
                    } else if (&k.reference, &k.rhs) == (&l1c, &l7q) {
                    } else if (&k.reference, &k.rhs) == (&l1c, &l8q) {
                    } else {
                        panic!("found invalid observable ({},{})", k.reference, k.rhs);
                    }
                }
            } else if k.epoch == t2 {
                if k.sv == g01 {
                    if (&k.reference, &k.rhs) == (&l1c, &l2s) {
                        let l1c = 129058023.661;
                        let l2s = 100564709.314;
                        let err = (tec.tecu()
                            - gamma / 10.0E16 * f_l1 * f_l2 / (f_l1 - f_l2) * (l1c - l2s))
                            .abs();
                        assert!(err < 1.0, "{}: error too large", err);
                        tests_passed += 1;
                    } else if (&k.reference, &k.rhs) == (&l1c, &l2w) {
                        let l1c = 129058023.661;
                        let l2w = 100564709.312;
                        let err = (tec.tecu()
                            - gamma / 10.0E16 * f_l1 * f_l2 / (f_l1 - f_l2) * (l1c - l2w))
                            .abs();
                        assert!(err < 1.0, "{}: error too large", err);
                        tests_passed += 1;
                    } else if (&k.reference, &k.rhs) == (&l1c, &l5q) {
                    } else {
                        panic!("found invalid observable");
                    }
                } else if k.sv == g07 {
                    if (&k.reference, &k.rhs) == (&l1c, &l2s) {
                        let l1c = 125171828.155;
                        let l2s = 97536490.732;
                        let err = (tec.tecu()
                            - gamma / 10.0E16 * f_l1 * f_l2 / (f_l1 - f_l2) * (l1c - l2s))
                            .abs();
                        assert!(err < 1.0, "{}: error too large", err);
                        tests_passed += 1;
                    } else if (&k.reference, &k.rhs) == (&l1c, &l2w) {
                        let l1c = 125171828.155;
                        let l2w = 97536478.727;
                        let err = (tec.tecu()
                            - gamma / 10.0E16 * f_l1 * f_l2 / (f_l1 - f_l2) * (l1c - l2w))
                            .abs();
                        assert!(err < 1.0, "{}: error too large", err);
                        tests_passed += 1;
                    } else if (&k.reference, &k.rhs) == (&l1c, &l5q) {
                    } else {
                        panic!("found invalid observable");
                    }
                } else if k.sv == e11 {
                    if (&k.reference, &k.rhs) == (&l1c, &l5q) {
                    } else if (&k.reference, &k.rhs) == (&l1c, &l6c) {
                    } else if (&k.reference, &k.rhs) == (&l1c, &l7q) {
                    } else if (&k.reference, &k.rhs) == (&l1c, &l8q) {
                    } else {
                        panic!("found invalid observable ({},{})", k.reference, k.rhs);
                    }
                }
            }
        }
        assert_eq!(tests_passed, 12);
    }
}
