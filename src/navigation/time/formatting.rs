
use crate::navigation::time::TimeOffset;

impl TimeOffset {
	
    /// Format [TimeOffset] according to RINEXv2 standard
	pub(crate) fn format_v2<W: Write>(w: &mut BufWriter<W>) -> Result<(), FormattingError> {

		let (week, secs) = self.to_time_of_week(self.t_ref);

		write!(f,
			"   {}{} {:8d} {:5d} DELTA-UTC: A0,A1,T,W",
			self.a0, self.a1, week, secs)?;
	
		Ok(())
	}
	
	/// Format [TimeOffset] according to RINEXv3 standard
	pub(crate) fn format_v3<W: Write>(w: &mut BufWriter<W>) -> Result<(), FormattingError> {

		let (week, secs) = self.to_time_of_week(self.t_ref);

		let (gp, gu) = match (self.lhs, self.rhs) {
			(TimeScale::GPST, TimeScale::UTC) => "GPGU",
			(TimeScale::GST, TimeScale::GPST) => "GAGP",
			(TimeScale::GPST, TimeScale::GST) => "GPGA",
			(TimeScale::
			_ => {
				return Err(FormattingError::NonSupportedTimescale);
			},
		};

		write!(f,
			"   {}{} {:8d} {:5d} DELTA-UTC: A0,A1,T,W",
			self.a0, self.a1, week, secs)?;
	
		Ok(())
	}

    pub(crate) fn format_v4<W: Write>(w: &mut BufWriter<W>) -> Result<(), FormattingError> {

    }
}

#[cfg(test)]
mod test {
    use super::SystemTime;
    use crate::prelude::{Epoch, TimeScale};
    use std::str::FromStr;

    #[test]
    fn system_time_parsing() {
        for (line_1, line_2, test_epoch, system, utc, t_tm, a_0, a_1, a_2) in [
            (
                "    2022 06 08 00 00 00 GAUT                                  UTCGAL",
                "     2.952070000000E+05-1.862645149231E-09 8.881784197001E-16 0.000000000000E+00",
                "2022-06-08T00:00:00 GST",
                "GAUT",
                "UTCGAL",
                0,
                2.952070000000E+05,
                -1.862645149231E-09,
                8.881784197001E-16,
            ),
            (
                "    2022 06 10 19 56 48 GPUT                                  UTC(USNO)",
                "     2.952840000000E+05 9.313225746155E-10 2.664535259100E-15 0.000000000000E+00",
                "2022-06-10T19:56:48 GPST",
                "GPUT",
                "UTC(USNO)",
                0,
                2.952840000000E+05,
                9.313225746155E-10,
                2.664535259100E-15,
            ),
        ] {
            let test_epoch = Epoch::from_str(test_epoch).unwrap();

            let (epoch, sto) = SystemTime::parse(line_1, line_2, TimeScale::GST).unwrap();

            assert_eq!(epoch, test_epoch);
            assert_eq!(sto.system, system);
            assert_eq!(sto.utc, utc);
            assert_eq!(sto.t_tm, t_tm);
            assert_eq!(sto.a.0, a_0);
            assert_eq!(sto.a.1, a_1);
            assert_eq!(sto.a.2, a_2);
        }
    }
}
