#[cfg(test)]
mod sampling {
    use crate::prelude::*;
    use crate::preprocessing::*;
    use itertools::Itertools;
    use std::path::Path;
    use std::str::FromStr;
    #[test]
    fn obs_sampling() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_resources")
            .join("OBS")
            .join("V2")
            .join("AJAC3550.21O");

        let fullpath = path.to_string_lossy();
        let rinex = Rinex::from_file(fullpath.as_ref());
        assert!(rinex.is_ok(), "failed to parse \"{}\"", fullpath);
        let rinex = rinex.unwrap();

        let expected = vec![(Duration::from_seconds(30.0), 1_usize)];

        let histogram: Vec<_> = rinex.sampling_histogram().sorted().collect();

        assert!(
            histogram == expected,
            "sampling_histogram failed.\nExpecting {:?}\nGot {:?}",
            expected.clone(),
            histogram.clone()
        );

        let initial_len = rinex.epoch().count();
        let decimated = rinex.decimate_by_interval(Duration::from_seconds(10.0));
        assert!(
            initial_len == decimated.epoch().count(),
            "decim with too small time interval failed"
        );
        let decimated = decimated.decimate_by_interval(Duration::from_hours(1.0));
        assert!(
            decimated.epoch().count() == 1,
            "failed to decimate to 1 hour epoch interval"
        );

        let decimated = rinex.decimate_by_ratio(2);
        assert_eq!(decimated.epoch().count(), 1, "decim by 2 failed");
    }
    #[test]
    fn meteo_sampling() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/test_resources/MET/V2/abvi0010.15m";
        let rinex = Rinex::from_file(&path).unwrap();
        assert_eq!(
            rinex.dominant_sample_rate(),
            Some(Duration::from_seconds(60.0)),
        );
        assert!(
            rinex.data_gaps(None).eq(vec![
                (
                    Epoch::from_str("2015-01-01T00:09:00 UTC").unwrap(),
                    Duration::from_parts(0, 31860000000000),
                ),
                (
                    Epoch::from_str("2015-01-01T09:04:00 UTC").unwrap(),
                    Duration::from_parts(0, 37260000000000),
                ),
                (
                    Epoch::from_str("2015-01-01T19:54:00 UTC").unwrap(),
                    Duration::from_parts(0, 10860000000000),
                ),
                (
                    Epoch::from_str("2015-01-01T23:02:00 UTC").unwrap(),
                    Duration::from_parts(0, 420000000000),
                ),
                (
                    Epoch::from_str("2015-01-01T23:21:00 UTC").unwrap(),
                    Duration::from_parts(0, 1860000000000),
                ),
            ]),
            "data_gaps(tol=None) failed",
        );
        assert!(
            rinex
                .data_gaps(Some(Duration::from_seconds(10.0 * 60.0)))
                .eq(vec![
                    (
                        Epoch::from_str("2015-01-01T00:09:00 UTC").unwrap(),
                        Duration::from_parts(0, 31860000000000),
                    ),
                    (
                        Epoch::from_str("2015-01-01T09:04:00 UTC").unwrap(),
                        Duration::from_parts(0, 37260000000000),
                    ),
                    (
                        Epoch::from_str("2015-01-01T19:54:00 UTC").unwrap(),
                        Duration::from_parts(0, 10860000000000),
                    ),
                    (
                        Epoch::from_str("2015-01-01T23:21:00 UTC").unwrap(),
                        Duration::from_parts(0, 1860000000000),
                    ),
                ]),
            "data_gaps(tol=10') failed",
        );
        assert!(
            rinex
                .data_gaps(Some(Duration::from_seconds(3.0 * 3600.0)))
                .eq(vec![
                    (
                        Epoch::from_str("2015-01-01T00:09:00 UTC").unwrap(),
                        Duration::from_parts(0, 31860000000000),
                    ),
                    (
                        Epoch::from_str("2015-01-01T09:04:00 UTC").unwrap(),
                        Duration::from_parts(0, 37260000000000),
                    ),
                    (
                        Epoch::from_str("2015-01-01T19:54:00 UTC").unwrap(),
                        Duration::from_parts(0, 10860000000000),
                    ),
                ]),
            "data_gaps(tol=3h) failed",
        );
    }
    #[test]
    fn steady_sampling() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/test_resources/CLK/V3/GRG0MGXFIN_20201770000_01D_30S_CLK.CLK.gz";
        let rinex = Rinex::from_file(&path).unwrap();
        assert!(rinex.steady_sampling());

        let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/test_resources/MET/V2/abvi0010.15m";
        let rinex = Rinex::from_file(&path).unwrap();
        assert!(!rinex.steady_sampling());
    }
}
