use crate::{
    navigation::{NavFrameType, NavMessageType},
    prelude::{Constellation, Epoch, Rinex, TimeScale, SV},
    tests::init_logger,
    tests::toolkit::{generic_navigation_test, TimeFrame},
};

use hifitime::Unit;

use std::{path::PathBuf, str::FromStr};

#[test]
fn v3_kepler() {
    init_logger();

    let g10 = SV::from_str("G10").unwrap();

    let t1_gpst = Epoch::from_str("2020-06-25T02:15:00 GPST").unwrap();
    let t2_gpst = Epoch::from_str("2020-06-25T02:30:00 GPST").unwrap();

    let dut = Rinex::from_gzip_file("data/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz").unwrap();

    for (t_gpst, x_km, y_km, z_km) in [
        (
            "2020-06-25T02:00:00 GPST",
            -12792.677331,
            -12271.088242,
            19940.585214,
        ),
        (
            "2020-06-25T02:15:00 GPST",
            -10518.543139,
            -12708.987728,
            20952.929790,
        ),
        (
            "2020-06-25T02:30:00 GPST",
            -8177.521591,
            -13288.569687,
            21609.078377,
        ),
    ] {
        let t_gpst = Epoch::from_str(t_gpst).unwrap();

        let (_, _, eph) = dut.nav_ephemeris_selection(g10, t_gpst).unwrap();

        let orbit = eph.kepler2position(g10, t_gpst).unwrap();

        let pos_vel = orbit.to_cartesian_pos_vel();

        let (x_err, y_err, z_err) = (
            (pos_vel[0] - x_km).abs(),
            (pos_vel[1] - y_km).abs(),
            (pos_vel[2] - z_km).abs(),
        );

        assert!(
            x_err < 1.0E-3,
            "failed for {} G10(x) err={} km",
            t_gpst,
            x_err
        );
        assert!(
            y_err < 1.0E-3,
            "failed for {} G10(y) err={} km",
            t_gpst,
            y_err
        );
        assert!(
            z_err < 1.0E-3,
            "failed for {} G10(z) err={} km",
            t_gpst,
            z_err
        );
    }
}
