use super::*;
//use navigation::*;
use thiserror::Error;
//use crate::observation::record::ObservationData;

#[derive(Debug, Error)]
pub enum Error {
    #[error("`base` data must be Observation RINEX for this operation")]
    NotObservationBase,
    #[error("`base` data must be Navigation RINEX for this operation")]
    NotNavigationBase,
    #[error("`rover` must be Observation RINEX for this operation")]
    NotObservationRover,
    #[error("`rover` must be Navigation RINEX for this operation")]
    NotNavigationRover,
    #[error("failed to parse RINEX data")]
    RinexError(#[from] super::Error),
}

/// Advanced RINEX processing algorithms require
/// combining two RINEX files together.
/// This structure helps forming such a context.
/// In the following, `base` is the reference RINEX,
/// and `rover` is the data to compare to "base".
/// We use this structure for operations / analysis that involve two file.
/// When substracting data (A-B), B is always the "base" or "reference station",
/// and A is the "rover" or "raw data".
#[derive(Debug, Clone, PartialEq)]
pub struct DiffContext {
    /// "base": reference RINEX
    pub base: Rinex,
    /// "rover": other RINEX
    pub rover: Rinex,
}

impl DiffContext {
    /// Builds a new DiffContext from two mutable RINEX.
    /// Sample rate is reworked to perfectly match:
    /// non shared epochs are dropped, to facilitate further processing.
    /// For Observation context: we only retain Phase and Pseudo Range observations,
    /// as we don't know of Differential analysis involving other observations.
    /// For Navigation context: we only retain Ephemeris frames,
    /// as we don't know of Differential analysis involving other frames.
    pub fn new(base: &Rinex, rover: &Rinex) -> Self {
        let base = base.clone();
        let rover = rover.clone();
        /*
            base.decim_match_mut(&rover);
            rover.decim_match_mut(&base);
            // match /adjust sample rates
            // For Navigation RINEX
            //  retain ephemeris frames only
            base.retain_navigation_ephemeris_mut();
            rover.retain_navigation_ephemeris_mut();
            // For Observation RINEX
            //  retain Phase and Pseudo Range observations only
            //    and only shared vehicles
            if let Some(record) = base.record.as_mut_obs() {
                record.retain(|e, (_, vehicles)| {
                    vehicles.retain(|sv, observations| {
                        if let Some(obs_rov) = rover.record.as_obs() {
                            let (_, rov_vehicles) = obs_rov.get(e)
                                .unwrap();
                            let mut shared = false;
                            for (rov_sv, _) in rov_vehicles {
                                shared |= rov_sv == sv;
                            }
                            if shared {
                                observations.retain(|code, _| {
                                    is_pseudo_range_obs_code!(code) || is_phase_carrier_obs_code!(code)
                                });
                                observations.len() > 0
                            } else {
                                false
                            }
                        } else if let Some(nav_rov) = rover.record.as_nav() {
                            let mut shared = false;
                            let rov_classes = nav_rov.get(e)
                                .unwrap();
                            for (rov_class, rov_frames) in rov_classes {
                                for rov_frame in rov_frames {
                                    let (_, rov_sv, _) = rov_frame.as_eph()
                                        .unwrap();
                                    shared |= rov_sv == sv;
                                }
                            }
                            if shared {
                                observations.retain(|code, _| {
                                    is_pseudo_range_obs_code!(code) || is_phase_carrier_obs_code!(code)
                                });
                                observations.len() > 0
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    });
                    vehicles.len() > 0
                });
            }
            // For Navigation (ephemeris..) RINEX
            //  retain only shared vehicles
            else if let Some(record) = base.record.as_mut_nav() {
                record.retain(|e, classes| {
                    classes.retain(|class, frames| {
                        frames.retain(|fr| {
                            let mut shared = false;
                            let (_, sv, _) = fr.as_eph()
                                .unwrap();
                            if let Some(obs_rec) = rover.record.as_obs() {
                                let (_, obs_vehicles) = obs_rec.get(e)
                                    .unwrap();
                                for (obs_sv, _) in obs_vehicles {
                                    shared |= obs_sv == sv;
                                }
                            } else if let Some(nav_rec) = rover.record.as_nav() {
                                let classes = nav_rec.get(e)
                                    .unwrap();
                                for (class, frames) in classes {
                                    for fr in frames {
                                        let (_, nav_sv, _) = fr.as_eph()
                                            .unwrap();
                                        shared |= nav_sv == sv;
                                    }
                                }
                            }
                            shared
                        });
                        frames.len() > 0
                    });
                    classes.len() > 0
                });
            }
            /*
             * at this point "base" is ready for processing,
             * let's strip "rover" to identical vehicles to
             * to speed up and eventually facilitate further operations
             */
            if let Some(record) = rover.record.as_mut_obs() {
                record.retain(|e, (_, vehicles)| {
                    vehicles.retain(|sv, _| {
                        let mut shared = false;
                        if let Some(obs_base) = base.record.as_obs() {
                            let (_, base_vehicles) = obs_base.get(e)
                                .unwrap();
                            for (base_sv, _) in base_vehicles {
                                shared |= base_sv == sv;
                            }
                        } else if let Some(nav_base) = base.record.as_nav() {
                            let base_classes = nav_base.get(e)
                                .unwrap();
                            for (class, frames) in base_classes {
                                for fr in frames {
                                    let (_, nav_sv, _) = fr.as_eph()
                                        .unwrap();
                                    shared |= nav_sv == sv;
                                }
                            }
                        }
                        false
                    });
                    vehicles.len() > 0
                });
            } else if let Some(record) = rover.record.as_mut_nav() {
                record.retain(|e, classes| {
                    classes.retain(|class, frames| {
                        frames.retain(|fr| {
                            let mut shared = false;
                            let (_, sv, _) = fr.as_eph()
                                .unwrap();
                            if let Some(obs_base) = base.record.as_obs() {
                                let (_, base_vehicles) = obs_base.get(e)
                                    .unwrap();
                                for (base_sv, _) in base_vehicles {
                                    shared |= sv == base_sv;
                                }
                            } else if let Some(nav_base) = base.record.as_nav() {
                                let nav_classes = nav_base.get(e)
                                    .unwrap();
                                for (class, frames) in nav_classes {
                                    for fr in frames {
                                        let (_, base_sv, _) = fr.as_eph()
                                            .unwrap();
                                        shared |= sv == base_sv;
                                    }
                                }
                            }
                            shared
                        });
                        frames.len() > 0
                    });
                    classes.len() > 0
                });
            }
        */
        Self { base, rover }
    }

    /// Builds Self from two local files
    pub fn from_files(fp: &str, rover_fp: &str) -> Result<Self, Error> {
        let rnx = Rinex::from_file(fp)?;
        let rover = Rinex::from_file(rover_fp)?;
        Ok(Self::new(&rnx, &rover))
    }
}
