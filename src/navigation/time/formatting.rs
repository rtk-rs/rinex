use crate::{
    epoch::epoch_decompose,
    error::FormattingError,
    navigation::{formatting::NavFormatter, time::TimeOffset},
    prelude::TimeScale,
};

use std::io::{BufWriter, Write};

impl TimeOffset {
    /// Parse left hand side and right hand side [TimeScale]s
    fn to_lhs_rhs_timescales(&self) -> &str {
        match (self.lhs, self.rhs) {
            (TimeScale::GPST, TimeScale::UTC) => "GPUT",
            (TimeScale::GPST, TimeScale::GST) => "GPGA",
            (TimeScale::GPST, TimeScale::BDT) => "GPBD",
            (TimeScale::QZSST, TimeScale::UTC) => "QZUT",
            (TimeScale::QZSST, TimeScale::GST) => "QZGA",
            (TimeScale::QZSST, TimeScale::BDT) => "QZBD",
            (TimeScale::GST, TimeScale::UTC) => "GAUT",
            (TimeScale::GST, TimeScale::GPST) => "GAGP",
            (TimeScale::GST, TimeScale::BDT) => "GABD",
            (TimeScale::BDT, TimeScale::UTC) => "BDUT",
            (TimeScale::BDT, TimeScale::GST) => "BDGA",
            (TimeScale::BDT, TimeScale::GPST) => "BDGP",
            _ => "GPUT",
        }
    }

    /// Format [TimeOffset] according to RINEXv2 standard
    pub(crate) fn format_v2_delta_utc<W: Write>(
        &self,
        w: &mut BufWriter<W>,
    ) -> Result<(), FormattingError> {
        let (week, secs) = self.t_ref.to_time_of_week();

        write!(
            w,
            "   {}{} {:8} {:8} DELTA-UTC: A0,A1,T,W",
            NavFormatter::new(self.polynomials.0),
            NavFormatter::new(self.polynomials.1),
            secs,
            week,
        )?;

        Ok(())
    }

    /// Format [TimeOffset] according to RINEXv2 format
    pub(crate) fn format_v2_corr_to_system_time<W: Write>(
        &self,
        w: &mut BufWriter<W>,
    ) -> Result<(), FormattingError> {
        let (y, m, d, _, _, _, _) = epoch_decompose(self.t_ref);
        write!(
            w,
            "{:6}{:6}{:6}   {}",
            y,
            m,
            d,
            NavFormatter::new(self.polynomials.0)
        )?;
        Ok(())
    }

    /// Format [TimeOffset] according to RINEXv3 standard
    pub(crate) fn format_v3<W: Write>(&self, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        write!(w, "{} ", self.to_lhs_rhs_timescales())?;

        // TODO: convert to NavFormatter with programmable precision
        if self.polynomials.0 == 0.0 {
            write!(w, " 0.0000000000e+00",)?;
        } else if self.polynomials.0.is_sign_negative() {
            write!(w, "{:14.10E} ", self.polynomials.0,)?;
        } else {
            write!(w, " {:14.10E} ", self.polynomials.0,)?;
        }

        if self.polynomials.1 == 0.0 {
            write!(w, " 0.000000000e+00 ",)?;
        } else if self.polynomials.1.is_sign_negative() {
            write!(w, "{:14.9E} ", self.polynomials.1,)?;
        } else {
            write!(w, " {:14.9E} ", self.polynomials.1,)?;
        }

        let (week, secs) = self.t_ref.to_time_of_week();

        write!(w, "{:6}{:5}", secs, week,)?;

        Ok(())
    }

    pub(crate) fn format_v4<W: Write>(&self, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        let (y, m, d, hh, mm, ss, _) = epoch_decompose(self.t_ref);

        write!(
            w,
            "    {:04} {:02} {:02} {:02} {:02} {:02} {}\n",
            y,
            m,
            d,
            hh,
            mm,
            ss,
            self.to_lhs_rhs_timescales(),
        )?;

        write!(
            w,
            "    {}{}{}{}",
            NavFormatter::new(self.polynomials.0),
            NavFormatter::new(self.polynomials.1),
            NavFormatter::new(self.polynomials.2),
            NavFormatter::new(0.0),
        )?;

        // write!(
        //     w,
        //     "   {:14.13E}{:14.13E}{:14.13E}{:14.13E}",
        //     self.polynomials.0, self.polynomials.1, self.polynomials.2, 0.0
        // )?;

        Ok(())
    }
}
