use crate::{
    navigation::{Ephemeris, NavKey},
    prelude::{Constellation, Epoch, Rinex, SV},
};

use ublox::{CfgAntBuilder, MgaGloEph, MgaGpsEphRef, PacketRef, RxmSfrbx};

/// NAV Record Streamer
pub struct Streamer<'a> {
    ephemeris_iter: Box<dyn Iterator<Item = (&'a NavKey, &'a Ephemeris)> + 'a>,
}

fn forge_gps_ephemeris_mga<'a>(_toc: &Epoch, sv: SV, eph: &Ephemeris) -> Option<MgaGpsEphRef<'a>> {
    None
}

impl<'a> Streamer<'a> {
    pub fn new(rinex: &'a Rinex) -> Self {
        Self {
            ephemeris_iter: rinex.nav_ephemeris_frames_iter(),
        }
    }
}

impl<'a> Iterator for Streamer<'a> {
    type Item = PacketRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let (key, eph) = self.ephemeris_iter.next()?;

        match key.sv.constellation {
            Constellation::GPS => {
                let mga = forge_gps_ephemeris_mga(&key.epoch, key.sv, eph)?;
                Some(PacketRef::MgaGpsEph(mga))
            },
            _ => None,
        }
    }
}
