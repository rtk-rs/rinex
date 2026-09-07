use crate::navigation::eph::EphemerisContext;

use std::{cell::RefCell, rc::Rc};

use gnss_rtk::prelude::{Epoch, Frame, Orbit, OrbitSource, SV};

/// [OrbitalContext] implements the solver [OrbitSource] interface
/// from the ephemeris frames of the [EphemerisContext].
pub struct OrbitalContext<'a> {
    eph_ctx: Rc<RefCell<EphemerisContext<'a>>>,
}

impl<'a> OrbitalContext<'a> {
    pub fn new(eph_ctx: Rc<RefCell<EphemerisContext<'a>>>) -> Self {
        Self { eph_ctx }
    }
}

impl OrbitSource for OrbitalContext<'_> {
    fn state_at(&self, t: Epoch, sv: SV, fr: Frame) -> Option<Orbit> {
        let (toc, _, eph) = self.eph_ctx.borrow_mut().select(t, sv)?;
        let orbit = eph.kepler2position(sv, toc, t)?;

        // expressed in the frame the solver works with
        Some(Orbit::from_cartesian_pos_vel(
            orbit.to_cartesian_pos_vel(),
            t,
            fr,
        ))
    }
}
