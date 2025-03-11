//! Earth Orientation NAV frames

use crate::{
    epoch::parse_in_timescale as parse_epoch_in_timescale,
    prelude::{Epoch, ParsingError, TimeScale},
};

use std::str::FromStr;

/// Earth Orientation Message
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct EarthOrientation {
    /// ((arc-sec), (arc-sec.day⁻¹), (arc-sec.day⁻²))
    pub x: (f64, f64, f64),
    /// ((arc-sec), (arc-sec.day⁻¹), (arc-sec.day⁻²))
    pub y: (f64, f64, f64),
    /// Message transmmission time in seconds of GNSS week
    pub t_tm: u32,
    /// Delta UT1 ((sec), (sec.day⁻¹), (sec.day⁻²))
    pub delta_ut1: (f64, f64, f64),
}

impl EarthOrientation {
    pub(crate) fn parse(
        line_1: &str,
        line_2: &str,
        line_3: &str,
        ts: TimeScale,
    ) -> Result<(Epoch, Self), ParsingError> {
        let (epoch, rem) = line_1.split_at(23);
        let (xp, rem) = rem.split_at(19);
        let (dxp, ddxp) = rem.split_at(19);

        let (_, rem) = line_2.split_at(23);
        let (yp, rem) = rem.split_at(19);
        let (dyp, ddyp) = rem.split_at(19);

        let (t_tm, rem) = line_3.split_at(23);
        let (dut, rem) = rem.split_at(19);
        let (ddut, dddut) = rem.split_at(19);

        let epoch = parse_epoch_in_timescale(epoch.trim(), ts)?;

        let x = (
            f64::from_str(xp.trim()).unwrap_or(0.0_f64),
            f64::from_str(dxp.trim()).unwrap_or(0.0_f64),
            f64::from_str(ddxp.trim()).unwrap_or(0.0_f64),
        );

        let y = (
            f64::from_str(yp.trim()).unwrap_or(0.0_f64),
            f64::from_str(dyp.trim()).unwrap_or(0.0_f64),
            f64::from_str(ddyp.trim()).unwrap_or(0.0_f64),
        );

        let t_tm = f64::from_str(t_tm.trim()).unwrap_or(0.0_f64);

        let delta_ut1 = (
            f64::from_str(dut.trim()).unwrap_or(0.0_f64),
            f64::from_str(ddut.trim()).unwrap_or(0.0_f64),
            f64::from_str(dddut.trim()).unwrap_or(0.0_f64),
        );

        Ok((
            epoch,
            Self {
                x,
                y,
                t_tm: t_tm as u32,
                delta_ut1,
            },
        ))
    }
}

#[cfg(test)]
mod test {
    use super::EarthOrientation;
    use crate::prelude::{Epoch, TimeScale};
    use std::str::FromStr;
    #[test]
    fn earth_orientations_parsing() {
        for (
            line_1,
            line_2,
            line_3,
            ts,
            test_epoch,
            x_0,
            x_1,
            x_2,
            y_0,
            y_1,
            y_2,
            t_tm,
            dut1,
            ddut1,
            dddut1,
        ) in [(
            "    2023 03 14 16 51 12-4.024982452393e-02 3.957748413086e-05 0.000000000000e+00",
            "                        3.562908172607e-01 2.602100372314e-03 0.000000000000e+00",
            "     4.392000000000e+03-1.940387487411e-02-1.411736011505e-04 0.000000000000e+00",
            TimeScale::UTC,
            "2023-03-14T16:51:12 UTC",
            -4.024982452393e-02,
            3.957748413086e-05,
            0.0,
            3.562908172607e-01,
            2.602100372314e-03,
            0.0,
            4392,
            -1.940387487411e-02,
            -1.411736011505e-04,
            0.000000000000e+00,
        )] {
            let (t, eop) = EarthOrientation::parse(line_1, line_2, line_3, ts).unwrap();

            let test_epoch = Epoch::from_str(test_epoch).unwrap();

            assert_eq!(t, test_epoch);

            assert_eq!(eop.x.0, x_0);
            assert_eq!(eop.x.1, x_1);
            assert_eq!(eop.x.2, x_2);

            assert_eq!(eop.y.0, y_0);
            assert_eq!(eop.y.1, y_1);
            assert_eq!(eop.y.2, y_2);

            assert_eq!(eop.t_tm, t_tm);

            assert_eq!(eop.delta_ut1.0, dut1);
            assert_eq!(eop.delta_ut1.1, ddut1);
            assert_eq!(eop.delta_ut1.2, dddut1);
        }
    }
}
