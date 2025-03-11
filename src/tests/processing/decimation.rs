// Decimation specific tests
#[cfg(test)]
mod decimation {
    use crate::prelude::*;
    use qc_traits::{Decimate, DecimationFilter};
    use std::path::Path;
    #[test]
    #[cfg(feature = "flate2")]
    fn obs_dt_decimation() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_resources")
            .join("CRNX")
            .join("V3")
            .join("ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz");

        let fullpath = path.to_string_lossy();
        let rinex = Rinex::from_gzip_file(fullpath.as_ref());
        assert!(rinex.is_ok(), "failed to parse \"{}\"", fullpath);

        let mut rinex = rinex.unwrap();
        let len = rinex.epoch_iter().count();

        let dt_60s = DecimationFilter::duration(Duration::from_seconds(60.0));
        let dt_120s = DecimationFilter::duration(Duration::from_seconds(120.0));

        rinex.decimate_mut(&dt_60s);
        let count = rinex.epoch_iter().count();
        assert_eq!(count, len / 2, "decimate(1'): error");

        rinex.decimate_mut(&dt_60s);
        let count = rinex.epoch_iter().count();
        assert_eq!(count, len / 2, "decimate(1'): error");

        rinex.decimate_mut(&dt_120s);
        let count = rinex.epoch_iter().count();
        assert_eq!(count, len / 4, "decimate(2'): error",);
    }
    #[test]
    #[cfg(feature = "flate2")]
    fn meteo_dt_decimation() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_resources")
            .join("MET")
            .join("V3")
            .join("POTS00DEU_R_20232540000_01D_05M_MM.rnx.gz");

        let fullpath = path.to_string_lossy();
        let rinex = Rinex::from_gzip_file(fullpath.as_ref());
        assert!(rinex.is_ok(), "failed to parse \"{}\"", fullpath);

        let mut rinex = rinex.unwrap();
        let len = rinex.epoch_iter().count();

        let dt_60s = DecimationFilter::duration(Duration::from_seconds(60.0));
        let dt_360s = DecimationFilter::duration(Duration::from_seconds(360.0));
        let dt_900s = DecimationFilter::duration(Duration::from_seconds(900.0));

        rinex.decimate_mut(&dt_60s);
        let count = rinex.epoch_iter().count();
        assert_eq!(count, len, "decimate(1'): error",);

        rinex.decimate_mut(&dt_360s);
        let count = rinex.epoch_iter().count();
        assert_eq!(count, len / 2, "decimate(6'): error",);

        rinex.decimate_mut(&dt_900s);
        let count = rinex.epoch_iter().count();
        assert_eq!(count, len / 4, "decimate(15'): error",);
    }
    #[test]
    #[cfg(feature = "flate2")]
    fn nav_dt_decimation() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_resources")
            .join("NAV")
            .join("V3")
            .join("ESBC00DNK_R_20201770000_01D_MN.rnx.gz");

        let fullpath = path.to_string_lossy();
        let rinex = Rinex::from_gzip_file(fullpath.as_ref());
        assert!(rinex.is_ok(), "failed to parse \"{}\"", fullpath);

        let dt_60s = DecimationFilter::duration(Duration::from_seconds(60.0));
        let dt_61s = DecimationFilter::duration(Duration::from_seconds(61.0));

        let mut rinex = rinex.unwrap();
        let _len = rinex.epoch_iter().count();

        rinex.decimate_mut(&dt_60s);
        let count = rinex.epoch_iter().count();
        assert_eq!(count, 1013, "decimate(1'): error",);

        rinex.decimate_mut(&dt_61s);
        let count = rinex.epoch_iter().count();
        assert_eq!(count, 1013, "decimate(1'+1s): error",);
    }
}
