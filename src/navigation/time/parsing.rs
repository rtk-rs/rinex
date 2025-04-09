// NAV V4 System Time Messages
use std::str::FromStr;

use crate::{
    epoch::parse_in_timescale as parse_epoch_in_timescale,
    error::ParsingError,
    navigation::time::TimeOffset,
    prelude::{Epoch, TimeScale},
};

impl TimeOffset {
    /// Parse left hand side and right hand side [TimeScale]s
    fn parse_lhs_rhs_timescales(content: &str) -> Result<(TimeScale, TimeScale), ParsingError> {
        match content {
            // gps
            "GPGA" => Ok((TimeScale::GPST, TimeScale::GST)),
            "GPUT" => Ok((TimeScale::GPST, TimeScale::UTC)),
            // qzss
            "QZGP" => Ok((TimeScale::QZSST, TimeScale::GPST)),
            "QZUT" => Ok((TimeScale::QZSST, TimeScale::UTC)),
            // gal
            "GAGP" => Ok((TimeScale::GST, TimeScale::GPST)),
            "GAUT" => Ok((TimeScale::GST, TimeScale::UTC)),
            // beidou
            "BDUT" => Ok((TimeScale::BDT, TimeScale::UTC)),
            "BDGA" => Ok((TimeScale::BDT, TimeScale::GST)),
            "BDGP" => Ok((TimeScale::BDT, TimeScale::GPST)),
            // "sbas"
            "SBUT" => Ok((TimeScale::GPST, TimeScale::UTC)),
            _ => Err(ParsingError::NavInvalidTimescale),
        }
    }

    /// Parse [TimeOffset] from RINEXv2 standard
    pub fn parse_v2_delta_utc(line: &str) -> Result<Self, ParsingError> {
        let (a0, rem) = line.split_at(22);
        let (a1, rem) = rem.split_at(19);
        let (seconds, rem) = rem.split_at(9);
        let (week, _) = rem.split_at(9);

        let week = week
            .trim()
            .parse::<u32>()
            .map_err(|_| ParsingError::NavEpochWeekCounter)?;

        let seconds = seconds
            .trim()
            .parse::<u64>()
            .map_err(|_| ParsingError::NavEpochWeekCounter)?;

        let t_ref = Epoch::from_time_of_week(week, seconds, TimeScale::GPST);

        let a0 = a0
            .trim()
            .replace('D', "e")
            .parse::<f64>()
            .map_err(|_| ParsingError::NavTimeOffsetParinsg)?;

        let a1 = a1
            .trim()
            .replace('D', "e")
            .parse::<f64>()
            .map_err(|_| ParsingError::NavTimeOffsetParinsg)?;

        Ok(Self::new(
            TimeScale::GPST,
            TimeScale::UTC,
            t_ref,
            (a0, a1, 0.0),
        ))
    }

    /// Parse [TimeOffset] from RINEXv2 standard
    pub fn parse_v2_corr_to_system_time(line: &str) -> Result<Self, ParsingError> {
        let (year, rem) = line.split_at(6);
        let (month, rem) = rem.split_at(6);
        let (day, rem) = rem.split_at(6);
        let (tau, _) = rem.split_at(22);

        let year = year
            .trim()
            .parse::<i32>()
            .map_err(|_| ParsingError::NavTimeOffsetParinsg)?;

        let month = month
            .trim()
            .parse::<u8>()
            .map_err(|_| ParsingError::NavTimeOffsetParinsg)?;

        let day = day
            .trim()
            .parse::<u8>()
            .map_err(|_| ParsingError::NavTimeOffsetParinsg)?;

        let t_ref = Epoch::from_gregorian_utc_at_midnight(year, month, day);

        let a0 = tau
            .trim()
            .replace('D', "e")
            .parse::<f64>()
            .map_err(|_| ParsingError::NavTimeOffsetParinsg)?;

        Ok(Self::new(
            TimeScale::GPST, //TODO GlonassT
            TimeScale::UTC,
            t_ref,
            (a0, 0.0, 0.0),
        ))
    }

    /// Parse [TimeOffset] from RINEXv3 standard
    pub fn parse_v3(line: &str) -> Result<Self, ParsingError> {
        let (timescales, rem) = line.split_at(4);
        let (a0, rem) = rem.split_at(18);
        let (a1, rem) = rem.split_at(16);
        let (seconds, rem) = rem.split_at(7);
        let (week, _) = rem.split_at(5);

        let (lhs, rhs) = Self::parse_lhs_rhs_timescales(timescales)?;

        let week = week
            .trim()
            .parse::<u32>()
            .map_err(|_| ParsingError::NavEpochWeekCounter)?;

        let seconds = seconds
            .trim()
            .parse::<u64>()
            .map_err(|_| ParsingError::NavEpochWeekCounter)?;

        let t_ref = Epoch::from_time_of_week(week, seconds, lhs);

        let a0 = a0
            .trim()
            .replace('D', "e")
            .parse::<f64>()
            .map_err(|_| ParsingError::NavTimeOffsetParinsg)?;

        let a1 = a1
            .trim()
            .replace('D', "e")
            .parse::<f64>()
            .map_err(|_| ParsingError::NavTimeOffsetParinsg)?;

        Ok(Self::new(lhs, rhs, t_ref, (a0, a1, 0.0)))
    }

    /// Parse [TimeOffset] from RINEXv4 standard
    pub fn parse_v4(line_1: &str, line_2: &str) -> Result<Self, ParsingError> {
        let (epoch, rem) = line_1.split_at(23);
        let (timescales, rem) = rem.split_at(5);
        let (lhs, rhs) = Self::parse_lhs_rhs_timescales(timescales)?;

        let utc = rem.trim().to_string();
        let t_ref = parse_epoch_in_timescale(epoch.trim(), lhs)?;

        let (a0, rem) = line_2.split_at(23);
        let (a1, rem) = rem.split_at(19);
        let (a2, time) = rem.split_at(19);

        let t_tm = f64::from_str(time.trim()).map_err(|_| ParsingError::NavTimeOffsetParinsg)?;

        let polynomials = (
            a0.parse::<f64>()
                .map_err(|_| ParsingError::NavTimeOffsetParinsg)?,
            a1.parse::<f64>()
                .map_err(|_| ParsingError::NavTimeOffsetParinsg)?,
            a2.parse::<f64>()
                .map_err(|_| ParsingError::NavTimeOffsetParinsg)?,
        );

        Ok(Self::new(lhs, rhs, t_ref, polynomials))
    }
}

#[cfg(test)]
mod test {
    use super::TimeOffset;
    use crate::prelude::{Epoch, TimeScale};
    use std::str::FromStr;

    #[test]
    fn parsing_delta_utc_v2() {
        for (content, a0, a1, sec, week) in [(
            "     .133179128170D-06  .107469588780D-12   552960     1025 ",
            0.133179128170E-06,
            0.107469588780E-12,
            552960,
            1025,
        )] {
            let t_ref = Epoch::from_time_of_week(week, sec, TimeScale::GPST);
            let expected = TimeOffset::new(TimeScale::GPST, TimeScale::UTC, t_ref, (a0, a1, 0.0));

            let parsed = TimeOffset::parse_v2_delta_utc(content).unwrap();
            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn parsing_correction_to_system_time_v2() {
        for (content, y, m, d, a0) in [(
            "  2021     1     1   -1.862645149231D-09                    ",
            2021,
            1,
            1,
            -1.862645149231E-09,
        )] {
            let t_ref = Epoch::from_gregorian_utc_at_midnight(y, m, d);
            let expected = TimeOffset::new(TimeScale::GPST, TimeScale::UTC, t_ref, (a0, 0.0, 0.0));

            let parsed = TimeOffset::parse_v2_corr_to_system_time(content).unwrap();
            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn parsing_v3() {
        // GLUT -1.8626451492e-09 0.000000000e+00      0    0          TIME SYSTEM CORR
        // GLGP -2.1420419216e-08 0.000000000e+00 518400 2138          TIME SYSTEM CORR
        // IRUT -9.6333678812e-09 1.776356839e-15 345888 1114          TIME SYSTEM CORR
        // IRGP -4.9476511776e-10-2.664535259e-15 432288 2138          TIME SYSTEM CORR

        let content = "XXXX  1.7840648070e-08 5.773159728e-14 432288 2138          ";

        assert!(TimeOffset::parse_v3(content).is_err());

        for (content, a0, a1, week, sec, lhs, rhs) in [
            (
                "GAUT  1.8626451492e-09-8.881784197e-16 432000 2138          ",
                1.8626451492e-09,
                -8.881784197e-16,
                2138,
                432000,
                TimeScale::GST,
                TimeScale::UTC,
            ),
            (
                "GPUT -3.7252902985e-09-1.065814104e-14  61440 2139          ",
                -3.7252902985e-09,
                -1.065814104e-14,
                2139,
                61440,
                TimeScale::GPST,
                TimeScale::UTC,
            ),
            (
                "GAGP  2.1536834538e-09-9.769962617e-15 432000 2138          ",
                2.1536834538e-09,
                -9.769962617e-15,
                2138,
                432000,
                TimeScale::GST,
                TimeScale::GPST,
            ),
            (
                "BDUT  0.0000000000e+00-4.085620730e-14     14  782          ",
                0.0,
                -4.085620730e-14,
                782,
                14,
                TimeScale::BDT,
                TimeScale::UTC,
            ),
            (
                "QZUT  5.5879354477e-09 0.000000000e+00  94208 2139          ",
                5.5879354477e-09,
                0.000000000e+00,
                2139,
                94208,
                TimeScale::QZSST,
                TimeScale::UTC,
            ),
        ] {
            let t_ref = Epoch::from_time_of_week(week, sec, lhs);
            let expected = TimeOffset::new(lhs, rhs, t_ref, (a0, a1, 0.0));

            let parsed = TimeOffset::parse_v3(content).unwrap();

            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn parsing_v4() {
        for (line_1, line_2, t_ref, system, utc, t_tm, a_0, a_1, a_2) in [
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
            let t_ref = Epoch::from_str(t_ref).unwrap();

            let time_offset = TimeOffset::parse_v4(line_1, line_2).unwrap();

            assert_eq!(t_ref, time_offset.t_ref);
            // assert_eq!(sto.system, system);
            // assert_eq!(sto.utc, utc);
            // assert_eq!(sto.t_tm, t_tm);

            assert_eq!(time_offset.polynomials, (a_0, a_1, a_2));
        }
    }
}
