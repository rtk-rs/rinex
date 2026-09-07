//! RINEX revision handling
#[cfg(test)]
mod test {
    use crate::prelude::*;
    use flate2::read::GzDecoder;
    use std::io::{BufReader, Cursor, Read};
    use std::path::Path;

    /// Reads a (possibly gzip compressed) test resource into memory
    fn read_resource(path: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../test_resources")
            .join(path);
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {}", path.display(), e));
        let mut content = String::new();
        if path.extension().map_or(false, |ext| ext == "gz") {
            GzDecoder::new(bytes.as_slice())
                .read_to_string(&mut content)
                .unwrap();
        } else {
            content = String::from_utf8(bytes).unwrap();
        }
        content
    }

    /// Rewrites the "RINEX VERSION / TYPE" header line with a new revision number
    fn rewrite_version(content: &str, version: &str) -> String {
        let mut rewritten = false;
        let content = content
            .lines()
            .map(|line| {
                if line.contains("RINEX VERSION / TYPE") {
                    rewritten = true;
                    format!("{:>9}{}", version, &line[9..])
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(rewritten, "no RINEX VERSION / TYPE line");
        content + "\n"
    }

    fn parse(content: &str) -> Result<Rinex, ParsingError> {
        Rinex::parse(&mut BufReader::new(Cursor::new(content)))
    }

    /// Parses `path` as published (4.00) and rewritten as `version`,
    /// then verifies that both parse to the same record.
    fn v4_minor_revision_test(path: &str, version: &str, minor: u8) {
        let content = read_resource(path);
        let model = parse(&content).unwrap_or_else(|e| panic!("{}: {:?}", path, e));
        assert_eq!(model.header.version, Version::new(4, 0));

        let rewritten = rewrite_version(&content, version);
        let dut = parse(&rewritten).unwrap_or_else(|e| panic!("{} as {}: {:?}", path, version, e));
        assert_eq!(dut.header.version, Version::new(4, minor));
        assert_eq!(dut.header.rinex_type, model.header.rinex_type);
        assert_eq!(
            dut.record, model.record,
            "{} parsed differently as {}",
            path, version
        );
    }

    #[test]
    fn observation_v4_minor_revisions() {
        let path = "OBS/V4/ACRG00GHA_R_20240010000_01H_30S_MO.rnx";
        v4_minor_revision_test(path, "4.01", 1);
        v4_minor_revision_test(path, "4.02", 2);
    }

    #[test]
    fn crinex_v4_minor_revisions() {
        let path = "CRNX/V4/ACRG00GHA_R_20240010000_01H_30S_MO.crx";
        v4_minor_revision_test(path, "4.01", 1);
        v4_minor_revision_test(path, "4.02", 2);
    }

    #[test]
    fn navigation_v4_minor_revisions() {
        let path = "NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz";
        v4_minor_revision_test(path, "4.01", 1);
        v4_minor_revision_test(path, "4.02", 2);
    }

    #[test]
    fn unsupported_major_revision() {
        let content = read_resource("OBS/V4/ACRG00GHA_R_20240010000_01H_30S_MO.rnx");
        let rewritten = rewrite_version(&content, "5.00");
        assert!(matches!(
            parse(&rewritten),
            Err(ParsingError::NonSupportedVersion)
        ));
    }
}
