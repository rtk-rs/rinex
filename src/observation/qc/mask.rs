use qc_traits::{QcMaskOperand, QcSubset};

use crate::{
    observation::Record,
    observation::SNR,
    prelude::{Constellation, Observable},
};

use std::str::FromStr;

pub fn mask_mut(rec: &mut Record, operand: QcMaskOperand, subset: &QcSubset) {
    match operand {
        QcMaskOperand::Equals => match subset {
            QcSubset::Datetime(epoch) => rec.retain(|k, _| k.epoch == *epoch),

            QcSubset::Constellations(constellations) => {
                let mut broad_sbas_filter = false;
                for c in constellations.iter() {
                    broad_sbas_filter |= *c == Constellation::SBAS;
                }

                rec.retain(|_, obs| {
                    obs.signals.retain(|sig| {
                        if broad_sbas_filter {
                            sig.sv.constellation.is_sbas()
                                || constellations.contains(&sig.sv.constellation)
                        } else {
                            constellations.contains(&sig.sv.constellation)
                        }
                    });
                    !obs.signals.is_empty()
                });
            },

            QcSubset::Satellites(satellites) => {
                rec.retain(|_, obs| {
                    obs.signals.retain(|sig| satellites.contains(&sig.sv));
                    !obs.signals.is_empty()
                });
            },

            QcSubset::SNR(snr) => {
                let filter = SNR::from(*snr);

                rec.retain(|_, obs| {
                    obs.signals.retain(|sig| {
                        if let Some(snr) = sig.snr {
                            snr == filter
                        } else {
                            false // no SNR: drop out
                        }
                    });
                    !obs.signals.is_empty()
                });
            },

            QcSubset::ComplexString(value) => {
                if let Ok(observable) = Observable::from_str(&value) {
                    rec.retain(|_, obs| {
                        obs.signals.retain(|sig| sig.observable == observable);
                        !obs.signals.is_empty()
                    });
                }
            },

            QcSubset::ComplexStringArray(values) => {
                let observables = values
                    .iter()
                    .filter_map(|f| {
                        if let Ok(ob) = Observable::from_str(f) {
                            Some(ob)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                if !observables.is_empty() {
                    rec.retain(|_, obs| {
                        obs.signals
                            .retain(|sig| observables.contains(&sig.observable));
                        !obs.signals.is_empty()
                    });
                }
            },
            _ => {},
        }, // MaskOperand::Equals

        QcMaskOperand::NotEquals => match subset {
            QcSubset::Datetime(epoch) => rec.retain(|k, _| k.epoch != *epoch),
            QcSubset::Constellations(constells) => {
                rec.retain(|_, obs| {
                    obs.signals
                        .retain(|sig| !constells.contains(&sig.sv.constellation));
                    !obs.signals.is_empty()
                });
            },
            QcSubset::Satellites(items) => {
                rec.retain(|_, obs| {
                    obs.signals.retain(|sig| !items.contains(&sig.sv));
                    !obs.signals.is_empty()
                });
            },
            QcSubset::ComplexString(value) => {
                if let Ok(observable) = Observable::from_str(&value) {
                    rec.retain(|_, obs| {
                        obs.signals.retain(|sig| sig.observable != observable);
                        !obs.signals.is_empty()
                    });
                }
            },

            QcSubset::ComplexStringArray(values) => {
                let observables = values
                    .iter()
                    .filter_map(|f| {
                        if let Ok(ob) = Observable::from_str(f) {
                            Some(ob)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                if !observables.is_empty() {
                    rec.retain(|_, obs| {
                        obs.signals
                            .retain(|sig| !observables.contains(&sig.observable));
                        !obs.signals.is_empty()
                    });
                }
            },
            _ => {},
        },
        QcMaskOperand::GreaterEquals => match subset {
            QcSubset::Datetime(epoch) => rec.retain(|k, _| k.epoch >= *epoch),
            QcSubset::Satellites(satellites) => {
                rec.retain(|_, obs| {
                    obs.signals.retain(|sig| {
                        let mut retained = true;
                        for satellite in satellites.iter() {
                            if satellite.constellation == sig.sv.constellation {
                                retained &= sig.sv.prn >= satellite.prn;
                            }
                        }
                        retained
                    });
                    !obs.signals.is_empty()
                });
            },
            QcSubset::SNR(snr) => {
                let filter = SNR::from(*snr);

                rec.retain(|_, obs| {
                    obs.signals.retain(|sig| {
                        if let Some(snr) = sig.snr {
                            snr >= filter
                        } else {
                            false // no SNR: drop out
                        }
                    });
                    !obs.signals.is_empty()
                });
            },
            _ => {},
        },
        QcMaskOperand::GreaterThan => match subset {
            QcSubset::Datetime(epoch) => rec.retain(|k, _| k.epoch > *epoch),
            QcSubset::Satellites(satellites) => {
                rec.retain(|_, obs| {
                    obs.signals.retain(|sig| {
                        let mut retained = true;
                        for satellite in satellites.iter() {
                            if satellite.constellation == sig.sv.constellation {
                                retained &= sig.sv.prn > satellite.prn;
                            }
                        }
                        retained
                    });
                    !obs.signals.is_empty()
                });
            },
            QcSubset::SNR(snr) => {
                let filter = SNR::from(*snr);

                rec.retain(|_, obs| {
                    obs.signals.retain(|sig| {
                        if let Some(snr) = sig.snr {
                            snr > filter
                        } else {
                            false // no SNR: drop out
                        }
                    });
                    !obs.signals.is_empty()
                });
            },
            _ => {},
        },
        QcMaskOperand::LowerEquals => match subset {
            QcSubset::Datetime(epoch) => rec.retain(|k, _| k.epoch <= *epoch),
            QcSubset::Satellites(satellites) => {
                rec.retain(|_, obs| {
                    obs.signals.retain(|sig| {
                        let mut retained = true;
                        for satellite in satellites.iter() {
                            if satellite.constellation == sig.sv.constellation {
                                retained &= sig.sv.prn <= satellite.prn;
                            }
                        }
                        retained
                    });
                    !obs.signals.is_empty()
                });
            },
            QcSubset::SNR(snr) => {
                let filter = SNR::from(*snr);

                rec.retain(|_, obs| {
                    obs.signals.retain(|sig| {
                        if let Some(snr) = sig.snr {
                            snr <= filter
                        } else {
                            false // no SNR: drop out
                        }
                    });
                    !obs.signals.is_empty()
                });
            },
            _ => {},
        },
        QcMaskOperand::LowerThan => match subset {
            QcSubset::Datetime(epoch) => rec.retain(|k, _| k.epoch < *epoch),
            QcSubset::Satellites(satellites) => {
                rec.retain(|_, obs| {
                    obs.signals.retain(|sig| {
                        let mut retained = true;
                        for satellite in satellites.iter() {
                            if satellite.constellation == sig.sv.constellation {
                                retained &= sig.sv.prn < satellite.prn;
                            }
                        }
                        retained
                    });
                    !obs.signals.is_empty()
                });
            },
            QcSubset::SNR(snr) => {
                let filter = SNR::from(*snr);

                rec.retain(|_, obs| {
                    obs.signals.retain(|sig| {
                        if let Some(snr) = sig.snr {
                            snr < filter
                        } else {
                            false // no SNR: drop out
                        }
                    });
                    !obs.signals.is_empty()
                });
            },
            _ => {},
        },
    }
}

#[cfg(feature = "qc")]
impl HeaderFields {
    /// Modifies in place Self, when applying preprocessing filter ops
    pub(crate) fn mask_mut(&mut self, f: &MaskFilter) {
        match f.operand {
            MaskOperand::Equals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    self.timeof_first_obs = Some(epoch.to_time_scale(ts));
                    self.timeof_last_obs = Some(epoch.to_time_scale(ts));
                },
                FilterItem::SvItem(svs) => {
                    let constells = svs
                        .iter()
                        .map(|sv| sv.constellation)
                        .unique()
                        .collect::<Vec<_>>();
                    self.codes.retain(|c, _| constells.contains(&c));
                    self.scaling.retain(|(c, _), _| constells.contains(&c));
                },
                FilterItem::ComplexItem(complex) => {
                    // try to interprate as [Observable]
                    let observables = complex
                        .iter()
                        .filter_map(|f| {
                            if let Ok(ob) = Observable::from_str(f) {
                                Some(ob)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    if observables.len() > 0 {
                        self.codes.retain(|_, obs| {
                            obs.retain(|ob| observables.contains(&ob));
                            !obs.is_empty()
                        });
                        self.scaling.retain(|(_, c), _| !observables.contains(c));
                    }
                },
                FilterItem::ConstellationItem(constells) => {
                    self.codes.retain(|c, _| constells.contains(&c));
                    self.scaling.retain(|(c, _), _| constells.contains(&c));
                },
                _ => {},
            },
            MaskOperand::NotEquals => match &f.item {
                FilterItem::SvItem(svs) => {
                    let constells = svs
                        .iter()
                        .map(|sv| sv.constellation)
                        .unique()
                        .collect::<Vec<_>>();
                    self.codes.retain(|c, _| !constells.contains(&c));
                    self.scaling.retain(|(c, _), _| !constells.contains(&c));
                },
                FilterItem::ConstellationItem(constells) => {
                    self.codes.retain(|c, _| !constells.contains(&c));
                    self.scaling.retain(|(c, _), _| !constells.contains(&c));
                },
                FilterItem::ComplexItem(complex) => {
                    // try to interprate as [Observable]
                    let observables = complex
                        .iter()
                        .filter_map(|f| {
                            if let Ok(ob) = Observable::from_str(f) {
                                Some(ob)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    if observables.len() > 0 {
                        self.codes.retain(|_, obs| {
                            obs.retain(|ob| observables.contains(&ob));
                            !obs.is_empty()
                        });
                        self.scaling.retain(|(_, c), _| !observables.contains(c));
                    }
                },
                _ => {},
            },
            MaskOperand::GreaterThan => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    if let Some(t) = self.timeof_first_obs {
                        if t < *epoch {
                            self.timeof_first_obs = Some(epoch.to_time_scale(ts));
                        }
                    } else {
                        self.timeof_first_obs = Some(epoch.to_time_scale(ts));
                    }
                },
                _ => {},
            },
            MaskOperand::GreaterEquals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    if let Some(t_first) = self.timeof_first_obs {
                        if t_first < *epoch {
                            self.timeof_first_obs = Some(epoch.to_time_scale(ts));
                        }
                    } else {
                        self.timeof_first_obs = Some(epoch.to_time_scale(ts));
                    }
                },
                _ => {},
            },
            MaskOperand::LowerThan => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    if let Some(t_last) = self.timeof_last_obs {
                        if t_last > *epoch {
                            self.timeof_last_obs = Some(epoch.to_time_scale(ts));
                        }
                    } else {
                        self.timeof_last_obs = Some(*epoch);
                    }
                },
                _ => {},
            },
            MaskOperand::LowerEquals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    if let Some(t_last) = self.timeof_last_obs {
                        if t_last > *epoch {
                            self.timeof_last_obs = Some(epoch.to_time_scale(ts));
                        }
                    } else {
                        self.timeof_last_obs = Some(epoch.to_time_scale(ts));
                    }
                },
                _ => {},
            },
        }
    }
}
