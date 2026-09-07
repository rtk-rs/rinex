use log::{error, info};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    context::{meta::ObsMetaData, QcContext},
    navigation::{
        carrier_to_rtk,
        clock::ClockContext,
        environment::EnvironmentContext,
        eph::{EphemerisContext, NullEphemeris},
        orbit::OrbitalContext,
        signal::SignalSource,
        time::AbsoluteTimeContext,
    },
    QcError,
};

use itertools::Itertools;

use gnss_rtk::prelude::{
    Candidate, Carrier as RTKCarrier, Config as RTKConfig, Error as RTKError, Observation, Orbit,
    PVTSolution, Solver, UserParameters,
};

use rinex::prelude::{obs::SignalObservation, Epoch, Observable};

/// [Solver] deployed by [NavPvtSolver]
type PvtSolver<'a> = Solver<
    NullEphemeris,
    OrbitalContext<'a>,
    EnvironmentContext,
    ClockContext<'a>,
    AbsoluteTimeContext,
>;

/// [NavPvtSolver] is an efficient structure to consume [QcContext]
/// and resolve all possible [PVTSolution]s from it.
pub struct NavPvtSolver<'a> {
    pool: Vec<Candidate>,
    signal: SignalSource<'a>,
    solver: PvtSolver<'a>,
    params: UserParameters,
    observations: HashMap<RTKCarrier, Observation>,
}

impl<'a> NavPvtSolver<'a> {
    /// Copies and returns [NavPvtSolver] with desired [UserParameters]
    pub fn with_user_parameters(mut self, params: UserParameters) -> Self {
        self.params = params;
        self
    }

    /// Gathers the [Observation]s of `sv` at this epoch.
    /// Signals the solver does not know are dropped.
    fn gather_observations(
        observations: &mut HashMap<RTKCarrier, Observation>,
        signals: &[SignalObservation],
        sv: rinex::prelude::SV,
    ) -> Vec<Observation> {
        observations.clear();

        for signal in signals.iter().filter(|sig| sig.sv == sv) {
            let carrier = match signal.observable.carrier(sv.constellation) {
                Ok(carrier) => carrier,
                Err(_) => continue,
            };

            let carrier = match carrier_to_rtk(&carrier) {
                Some(carrier) => carrier,
                None => continue,
            };

            let observation = observations.entry(carrier).or_insert_with(|| {
                let mut observation = Observation::default();
                observation.carrier = carrier;
                observation
            });

            match signal.observable {
                Observable::PhaseRange(_) => {
                    observation.phase_range_m = Some(signal.value);
                },
                Observable::Doppler(_) => {
                    observation.doppler = Some(signal.value);
                },
                Observable::PseudoRange(_) => {
                    observation.pseudo_range_m = Some(signal.value);
                },
                Observable::SSI(_) => {
                    observation.snr_dbhz = Some(signal.value);
                },
                _ => {},
            }
        }

        observations
            .values()
            .filter(|obs| obs.pseudo_range_m.is_some() || obs.phase_range_m.is_some())
            .copied()
            .collect()
    }

    /// Forms the [Candidate]s of this epoch
    pub(crate) fn candidates(
        pool: &mut Vec<Candidate>,
        observations: &mut HashMap<RTKCarrier, Observation>,
        t: Epoch,
        signals: &[SignalObservation],
    ) {
        pool.clear();

        let sv_list = signals
            .iter()
            .map(|sig| sig.sv)
            .unique()
            .collect::<Vec<_>>();

        for sv in sv_list {
            let observations = Self::gather_observations(observations, signals, sv);

            if !observations.is_empty() {
                pool.push(Candidate::new(sv, t, observations));
            }
        }
    }
}

impl<'a> Iterator for NavPvtSolver<'a> {
    type Item = Result<PVTSolution, RTKError>;

    fn next(&mut self) -> Option<Self::Item> {
        let (t, signals) = match self.signal.collect_epoch() {
            Some(collected) => collected,
            None => {
                info!("consumed all signals");
                return None;
            },
        };

        Self::candidates(&mut self.pool, &mut self.observations, t, signals);

        // attempt resolution
        match self.solver.ppp(t, self.params, &self.pool) {
            Ok(pvt) => Some(Ok(pvt)),
            Err(e) => {
                error!("{}: rtk error: {}", t, e);
                Some(Err(e))
            },
        }
    }
}

impl QcContext {
    /// Create a new [NavPvtSolver] ready to iterate this [QcContext]
    /// and resolve all possible navigation solutions for specifically selected rover.
    /// ## Inputs
    /// - cfg: [RTKConfig] setup
    /// - meta: [ObsMetaData] rover selector
    /// - initial: possible a priori rover position, expressed as [Orbit]
    pub fn nav_pvt_solver<'a>(
        &'a self,
        cfg: RTKConfig,
        meta: &ObsMetaData,
        initial: Option<Orbit>,
    ) -> Result<NavPvtSolver<'a>, QcError> {
        // Obtain ephemeris context
        let eph_ctx = self.ephemeris_context().ok_or(QcError::EphemerisSource)?;

        // Obtain signal source
        let signal = self
            .rover_signal_source(meta)
            .ok_or(QcError::SignalSource)?;

        let initial_ecef_m = initial.map(|orbit| {
            let state_km = orbit.to_cartesian_pos_vel();
            (
                state_km[0] * 1.0E3,
                state_km[1] * 1.0E3,
                state_km[2] * 1.0E3,
            )
        });

        // Deploy solver: share almanac & reference frame model
        let solver = self.deploy_solver(cfg, eph_ctx, initial_ecef_m);

        Ok(NavPvtSolver {
            solver,
            signal,
            params: UserParameters::default(),
            pool: Vec::with_capacity(8),
            observations: HashMap::with_capacity(8),
        })
    }

    /// Deploys a [Solver] on top of this [QcContext],
    /// with orbital states and clock corrections obtained from the [EphemerisContext].
    pub(crate) fn deploy_solver<'a>(
        &'a self,
        cfg: RTKConfig,
        eph_ctx: EphemerisContext<'a>,
        initial_ecef_m: Option<(f64, f64, f64)>,
    ) -> PvtSolver<'a> {
        let eph_ctx = Rc::new(RefCell::new(eph_ctx));

        let orbits = Rc::new(OrbitalContext::new(eph_ctx.clone()));
        let clocks = Rc::new(ClockContext::new(eph_ctx));
        let environment = Rc::new(EnvironmentContext::new(self));

        Solver::new(
            self.almanac.clone(),
            self.earth_cef,
            cfg,
            Rc::new(NullEphemeris {}),
            orbits,
            clocks,
            environment,
            AbsoluteTimeContext {},
            initial_ecef_m,
        )
    }
}

#[cfg(test)]
mod test {

    use crate::{
        cfg::QcConfig,
        context::{meta::ObsMetaData, QcContext},
    };

    use gnss_rtk::prelude::{
        ClockProfile, Config as RTKConfig, PVTSolutionType, UserParameters, UserProfile,
    };

    /// ESBC00DNK reference position (APPROX POSITION XYZ), in meters ECEF
    const ESBC00DNK_ECEF_M: (f64, f64, f64) = (3582105.2910, 532589.7313, 5232754.8054);

    /// Tolerated error on the first solutions, in meters
    const MAX_ERROR_M: f64 = 100.0;

    fn esbc00dnk_context() -> QcContext {
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

        ctx
    }

    fn esbc00dnk_meta(ctx: &QcContext) -> ObsMetaData {
        ctx.rover_observations_meta()
            .find(|meta| meta.meta.name == "ESBC00DNK")
            .expect("ESBC00DNK observations not loaded")
            .clone()
    }

    #[test]
    #[cfg(feature = "flate2")]
    pub fn pvt_solver() {
        let ctx = esbc00dnk_context();
        let rtk_cfg = RTKConfig::default();

        let _ = ctx
            .nav_pvt_solver(rtk_cfg, &esbc00dnk_meta(&ctx), None)
            .unwrap();
    }

    /// Resolves the first epochs of the ESBC00DNK station (broadcast ephemeris only)
    /// and verifies the solutions are close to the reference position.
    #[test]
    #[cfg(feature = "flate2")]
    pub fn pvt_solutions() {
        let ctx = esbc00dnk_context();
        let rtk_cfg = RTKConfig::default();

        let solver = ctx
            .nav_pvt_solver(rtk_cfg, &esbc00dnk_meta(&ctx), None)
            .unwrap()
            .with_user_parameters(UserParameters::new(
                UserProfile::Static,
                ClockProfile::Quartz,
            ));

        let mut solutions = 0;

        for solution in solver.take(10).flatten() {
            assert_eq!(solution.solution_type, PVTSolutionType::PPP);

            let (x_m, y_m, z_m) = solution.pos_m;

            let error_m = ((x_m - ESBC00DNK_ECEF_M.0).powi(2)
                + (y_m - ESBC00DNK_ECEF_M.1).powi(2)
                + (z_m - ESBC00DNK_ECEF_M.2).powi(2))
            .sqrt();

            assert!(
                error_m < MAX_ERROR_M,
                "{}: solution {:?} is {:.1} m away from the reference position",
                solution.epoch,
                solution.pos_m,
                error_m
            );

            solutions += 1;
        }

        assert!(solutions > 0, "no solution resolved");
    }
}
