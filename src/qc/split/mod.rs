use crate::prelude::{Duration, Epoch, Header, Record, Rinex};

use gnss_qc_traits::QcSplit;

mod clock;
mod doris;
mod header;
mod ionex;
mod meteo;
mod nav;
mod obs;
mod production;

use clock::{split_even_dt as clock_split_even_dt, split_mut as clock_split_mut};
use doris::{split_even_dt as doris_split_even_dt, split_mut as doris_split_mut};
use ionex::{split_even_dt as ionex_split_even_dt, split_mut as ionex_split_mut};
use meteo::{split_even_dt as meteo_split_even_dt, split_mut as meteo_split_mut};
use nav::{split_even_dt as nav_split_even_dt, split_mut as nav_split_mut};
use obs::{split_even_dt as obs_split_even_dt, split_mut as obs_split_mut};

impl QcSplit for Rinex {
    fn split_mut(&mut self, t: Epoch) -> Self {
        self.header.program = Some(format!(
            "rs-rust v{}",
            Header::format_pkg_version(env!("CARGO_PKG_VERSION"),)
        ));

        let record = if let Some(r) = self.record.as_mut_obs() {
            Record::ObsRecord(obs_split_mut(r, t))
        } else if let Some(r) = self.record.as_mut_nav() {
            Record::NavRecord(nav_split_mut(r, t))
        } else if let Some(r) = self.record.as_mut_clock() {
            Record::ClockRecord(clock_split_mut(r, t))
        } else if let Some(r) = self.record.as_mut_ionex() {
            Record::IonexRecord(ionex_split_mut(r, t))
        } else if let Some(r) = self.record.as_mut_doris() {
            Record::DorisRecord(doris_split_mut(r, t))
        } else if let Some(r) = self.record.as_mut_meteo() {
            Record::MeteoRecord(meteo_split_mut(r, t))
        } else {
            self.record.clone()
        };

        let header = self.header.split_mut(t);
        let production = self.production.split_mut(t);

        Self {
            record,
            header,
            production,
            comments: self.comments.clone(),
        }
    }

    fn split_even_dt(&self, dt: Duration) -> Vec<Self>
    where
        Self: Sized,
    {
        Default::default()
    }

    // fn split_even_dt(&self, dt: Duration) -> Vec<Self> {
    //     let records = if let Some(r) = self.record.as_obs() {
    //         obs_split_even_dt(r, dt)
    //             .into_iter()
    //             .map(|rec| Record::ObsRecord(rec))
    //             .collect::<Vec<_>>()
    //     } else if let Some(r) = self.record.as_clock() {
    //         clock_split_even_dt(r, dt)
    //             .into_iter()
    //             .map(|rec| Record::ClockRecord(rec))
    //             .collect::<Vec<_>>()
    //     } else if let Some(r) = self.record.as_meteo() {
    //         meteo_split_even_dt(r, dt)
    //             .into_iter()
    //             .map(|rec| Record::MeteoRecord(rec))
    //             .collect::<Vec<_>>()
    //     } else if let Some(r) = self.record.as_ionex() {
    //         ionex_split_even_dt(r, dt)
    //             .into_iter()
    //             .map(|rec| Record::IonexRecord(rec))
    //             .collect::<Vec<_>>()
    //     } else if let Some(r) = self.record.as_doris() {
    //         doris_split_even_dt(r, dt)
    //             .into_iter()
    //             .map(|rec| Record::DorisRecord(rec))
    //             .collect::<Vec<_>>()
    //     } else if let Some(r) = self.record.as_nav() {
    //         nav_split_even_dt(r, dt)
    //             .into_iter()
    //             .map(|rec| Record::NavRecord(rec))
    //             .collect::<Vec<_>>()
    //     } else {
    //         Vec::new()
    //     };

    //     // TODO:
    //     // comments (timewise) should be split
    //     // header section could be split as well:
    //     //  impl split_event_dt on Header directly

    //     records
    //         .iter()
    //         .map(|rec| Rinex {
    //             header: self.header.clone(),
    //             comments: self.comments.clone(),
    //             production: self.production.clone(),
    //             record: rec.clone(),
    //         })
    //         .collect()
    // }
}
