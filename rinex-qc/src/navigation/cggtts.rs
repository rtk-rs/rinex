use log::{error, info};

use std::collections::HashMap;

use crate::{
    context::{meta::ObsMetaData, QcContext},
    navigation::{
        clock::ClockContext, environment::EnvironmentContext, eph::NullEphemeris,
        orbit::OrbitalContext, pvt::NavPvtSolver, signal::SignalSource, time::AbsoluteTimeContext,
    },
    QcError, QcRtkCggttsError,
};

use gnss_rtk::prelude::{
    Candidate, Carrier as RTKCarrier, Config as RTKConfig, Duration, Epoch,
    IonosphereBias as RTKIonosphereBias, Observation, Solver, UserParameters, SPEED_OF_LIGHT_M_S,
};

use cggtts::prelude::Track as CggttsTrack;

/// [Solver] deployed by [NavCggttsSolver]
type CggttsSolver<'a> = Solver<
    NullEphemeris,
    OrbitalContext<'a>,
    EnvironmentContext,
    ClockContext<'a>,
    AbsoluteTimeContext,
>;

/// [NavCggttsSolver] is an efficient structure to consume [QcContext]
/// and resolve all possible CGGTTS [Track]s from it.
pub struct NavCggttsSolver<'a> {
    pool: Vec<Candidate>,
    signal: SignalSource<'a>,
    solver: CggttsSolver<'a>,
    params: UserParameters,
    observations: HashMap<RTKCarrier, Observation>,
    // /// Track scheduling table
    // scheduler: CggttsScheduler,
    /// Epoch of next publication
    next_release: Epoch,
    /// Next track midpoint
    track_midpoint: Epoch,
}

impl<'a> Iterator for NavCggttsSolver<'a> {
    type Item = Result<CggttsTrack, QcRtkCggttsError>;

    fn next(&mut self) -> Option<Self::Item> {
        let (t, signals) = match self.signal.collect_epoch() {
            Some(collected) => collected,
            None => {
                info!("consumed all signals");
                return None;
            },
        };

        NavPvtSolver::candidates(&mut self.pool, &mut self.observations, t, signals);

        // attempt resolution
        match self.solver.ppp(t, self.params, &self.pool) {
            Ok(pvt_solution) => {
                let refsys = pvt_solution.clock_offset_s;

                for sv_pvt in pvt_solution.sv.iter() {
                    let (azimuth_deg, elevation_deg) = (sv_pvt.azimuth_deg, sv_pvt.elevation_deg);

                    let correction = sv_pvt.clock_correction.unwrap_or_default();

                    let refsv = refsys + correction.to_seconds();

                    // tropod always exists in CGGTTS
                    let _mdtr = sv_pvt.tropo_bias.unwrap_or_default() / SPEED_OF_LIGHT_M_S;

                    let _mdio = match sv_pvt.iono_bias {
                        Some(RTKIonosphereBias::Modeled(bias)) => Some(bias),
                        _ => None,
                    };

                    let _msio = match sv_pvt.iono_bias {
                        Some(RTKIonosphereBias::Measured(bias)) => Some(bias),
                        _ => None,
                    };

                    info!("{:?}({}): new solution: (azim={:.2}°, elev={:.2}°, refsv={:.3E}, refsys={:.3E})", t, sv_pvt.sv, azimuth_deg, elevation_deg, refsv, refsys);

                    // // form FitData
                    // let fitdata = FitData {
                    //     refsv,
                    //     refsys,
                    //     mdtr,
                    //     mdio,
                    //     msio,
                    //     azimuth_deg,
                    //     elevation_deg,
                    // };
                }
            },
            Err(e) => {
                error!("{}: rtk error: {}", t, e);
            },
        }

        Some(Err(QcRtkCggttsError::Dumy))
    }
}

impl QcContext {
    /// Create a new [NavCggttsSolver] ready to iterate this [QcContext]
    /// and resolve all possible CGGTTS solutions for specifically selected rover.
    /// ## Inputs
    /// - cfg: [RTKConfig] setup
    /// - meta: [ObsMetaData] rover selector
    /// - rx_position_ecef_km: ground position expressed in ECEF (km),
    /// the RINex position is used when not provided
    /// - tracking: [Duration] to be used by track scheduler
    pub fn nav_cggtts_solver<'a>(
        &'a self,
        cfg: RTKConfig,
        meta: &ObsMetaData,
        rx_position_ecef_km: Option<(f64, f64, f64)>,
        _tracking_duration: Duration,
    ) -> Result<NavCggttsSolver<'a>, QcError> {
        // Obtain ephemeris context
        let eph_ctx = self.ephemeris_context().ok_or(QcError::EphemerisSource)?;

        // Obtain signal source
        let signal = self
            .rover_signal_source(meta)
            .ok_or(QcError::SignalSource)?;

        let rinex = self.obs_dataset.get(meta).ok_or(QcError::RxPosition)?;

        // Reference position: prefer user settings over RINex position
        let rx_position_ecef_m =
            if let Some((x_ecef_km, y_ecef_km, z_ecef_km)) = rx_position_ecef_km {
                (x_ecef_km * 1.0E3, y_ecef_km * 1.0E3, z_ecef_km * 1.0E3)
            } else {
                // Using internal position (which then, needs to be defined)
                let (x_ecef_m, y_ecef_m, z_ecef_m) =
                    rinex.header.rx_position.ok_or(QcError::RxPosition)?;

                info!(
                    "using RINex ({}) reference position: {:?}",
                    meta,
                    (x_ecef_m, y_ecef_m, z_ecef_m)
                );

                (x_ecef_m, y_ecef_m, z_ecef_m)
            };

        // Deploy solver: share almanac & reference frame model
        let solver = self.deploy_solver(cfg, eph_ctx, Some(rx_position_ecef_m));

        // Initialize the track scheduler
        // let scheduler = CggttsScheduler::new(tracking_duration);
        // let next_release = scheduler.next_track_start(t0);
        // let track_midpoint =
        //    next_release - (3.0 * 60.0) * Unit::Second - (780.0 * Unit::Second) / 2.0;

        //info!("{}: {} until next track", t0, next_release - t0);

        Ok(NavCggttsSolver {
            solver,
            signal,
            // scheduler,
            params: UserParameters::default(),
            next_release: Default::default(),
            track_midpoint: Default::default(),
            pool: Vec::with_capacity(8),
            observations: HashMap::with_capacity(8),
        })
    }
}

#[cfg(test)]
mod test {

    use crate::{cfg::QcConfig, context::QcContext};

    use gnss_rtk::prelude::{Config as RTKConfig, Duration};

    #[test]
    #[cfg(feature = "flate2")]
    pub fn cggtts_solver() {
        let cfg = QcConfig::default();

        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_gzip_file(&format!(
            "{}/../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz",
            env!("CARGO_MANIFEST_DIR"),
        ))
        .unwrap();

        ctx.load_gzip_file(&format!(
            "{}/../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz",
            env!("CARGO_MANIFEST_DIR"),
        ))
        .unwrap();

        let rtk_cfg = RTKConfig::default();

        let meta = ctx
            .rover_observations_meta()
            .find(|meta| meta.meta.name == "ESBC00DNK")
            .expect("ESBC00DNK observations not loaded")
            .clone();

        let _ = ctx
            .nav_cggtts_solver(rtk_cfg, &meta, None, Duration::from_seconds(60.0))
            .unwrap();
    }
}
