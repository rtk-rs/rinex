use crate::{
    doris::format as format_doris_observations,
    ionex::format as format_ionex_maps,
    meteo::format as format_meteo_observations,
    navigation::format as format_navigation,
    observation::format as format_observations,
    prelude::{FormattingError, Header},
    record::Record,
};

use std::io::{BufWriter, Write};

impl Record {
    pub fn format<W: Write>(
        &self,
        w: &mut BufWriter<W>,
        header: &Header,
    ) -> Result<(), FormattingError> {
        if let Some(rec) = self.as_obs() {
            format_observations(w, rec, header)
        } else if let Some(rec) = self.as_meteo() {
            format_meteo_observations(w, rec, header)
        } else if let Some(rec) = self.as_doris() {
            format_doris_observations(w, rec, header)
        } else if let Some(rec) = self.as_nav() {
            format_navigation(w, rec, header)
        } else if let Some(rec) = self.as_ionex() {
            format_ionex_maps(w, rec, header)
        } else {
            Ok(())
        }
    }
}
