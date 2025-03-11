// NAV V4 System Time Messages
use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    epoch::parse_in_timescale as parse_epoch_in_timescale,
    prelude::{Epoch, ParsingError, TimeScale},
};

/// System Time (offset) Message
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SystemTime {
    /// Time System
    pub system: String,
    /// UTC ID
    pub utc: String,
    /// Message transmmission time in seconds of GNSS week
    pub t_tm: u32,
    /// (offset, drift, drift-rate) as (s, s.s⁻¹, s.s⁻²)
    pub a: (f64, f64, f64),
}

impl SystemTime {
    pub fn parse(line_1: &str, line_2: &str, ts: TimeScale) -> Result<(Epoch, Self), ParsingError> {
        let (epoch, rem) = line_1.split_at(23);
        let (system, rem) = rem.split_at(5);
        let utc = rem.trim().to_string();
        let epoch = parse_epoch_in_timescale(epoch.trim(), ts)?;

        let (a0, rem) = line_2.split_at(23);
        let (a1, rem) = rem.split_at(19);
        let (a2, time) = rem.split_at(19);
        let t_tm = f64::from_str(time.trim()).map_err(|_| ParsingError::SystemTimeData)?;

        Ok((
            epoch,
            Self {
                utc,
                system: system.trim().to_string(),
                t_tm: t_tm as u32,
                a: (
                    f64::from_str(a0.trim()).map_err(|_| ParsingError::SystemTimeData)?,
                    f64::from_str(a1.trim()).map_err(|_| ParsingError::SystemTimeData)?,
                    f64::from_str(a2.trim()).map_err(|_| ParsingError::SystemTimeData)?,
                ),
            },
        ))
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
