use crate::prelude::*;
use std::path::Path;

// Test our standardized name generator does follow the specs
#[test]
fn short_filename_conventions() {
    for testfile in [
        "OBS/V2/AJAC3550.21O",
        "OBS/V2/rovn0010.21o",
        "OBS/V3/LARM0010.22O",
        "OBS/V3/pdel0010.21o",
        "CRNX/V1/delf0010.21d",
        "CRNX/V1/zegv0010.21d",
        "CRNX/V3/DUTH0630.22D",
        "CRNX/V3/VLNS0010.22D",
        "MET/V2/abvi0010.15m",
        "MET/V2/clar0020.00m",
    ] {
        let fp = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_resources")
            .join(testfile);

        let rinex = Rinex::from_file(fp.to_string_lossy().as_ref()).unwrap();

        // these are all standard names: we only support uppercase formatting
        let filename = fp
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
            .to_uppercase();

        let output = rinex.standard_filename(true, None, None); // force short
        assert_eq!(output, filename, "bad short filename generated");
    }
}

// Test our standardized name generator does follow the specs
#[test]
#[cfg(feature = "flate2")]
fn long_filename_conventions() {
    for (testfile, is_gzip, expected, custom_suffix) in [
        (
            "OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx",
            false,
            "ACOR00ESP_R_20213550000_01D_30S_MO.rnx",
            None,
        ),
        (
            "OBS/V3/ALAC00ESP_R_20220090000_01D_30S_MO.rnx",
            false,
            "ALAC00ESP_R_20220090000_01D_13M_MO.rnx",
            None,
        ),
        (
            "CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz",
            true,
            "ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz",
            Some(".gz"),
        ),
    ] {
        let fp = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_resources")
            .join(testfile);

        let rinex = if is_gzip {
            Rinex::from_gzip_file(&fp).unwrap()
        } else {
            Rinex::from_file(&fp).unwrap()
        };

        let output = rinex.standard_filename(false, custom_suffix, None);
        assert_eq!(output, expected, "bad filename generated");
    }
}
