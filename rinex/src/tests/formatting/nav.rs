//! Navigation RINEX formatting round trips
use crate::{
    navigation::Record,
    prelude::{Rinex, Version},
    tests::formatting::Utf8Buffer,
};

use flate2::read::GzDecoder;
use std::collections::BTreeMap;
use std::io::{BufReader, BufWriter, Cursor, Read};
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

fn parse(content: &str) -> Rinex {
    Rinex::parse(&mut BufReader::new(Cursor::new(content))).unwrap()
}

fn format(rinex: &Rinex) -> String {
    let mut buf = BufWriter::new(Utf8Buffer::new(1 << 20));
    rinex.format(&mut buf).unwrap();
    buf.into_inner().unwrap().to_ascii_utf8()
}

/// Compares two navigation records
fn assert_records_equal(dut: &Record, model: &Record) {
    assert_eq!(
        dut.keys().collect::<Vec<_>>(),
        model.keys().collect::<Vec<_>>(),
        "keys differ"
    );
    for (k, model_frame) in model.iter() {
        assert_eq!(&dut[k], model_frame, "{:?}", k);
    }
}

/// Parses `path`, formats it back in its own revision and verifies
/// that the formatted text parses to the same header revision,
/// constellation and record.
fn round_trip(path: &str) -> (Rinex, String) {
    let model = parse(&read_resource(path));
    let formatted = format(&model);
    let dut = Rinex::parse(&mut BufReader::new(Cursor::new(formatted.as_str())))
        .unwrap_or_else(|e| panic!("{}: formatted file does not parse: {:?}", path, e));
    assert_eq!(dut.header.version, model.header.version, "{}", path);
    assert_eq!(
        dut.header.constellation, model.header.constellation,
        "{}",
        path
    );
    assert_records_equal(dut.record.as_nav().unwrap(), model.record.as_nav().unwrap());
    (model, formatted)
}

/// Maps every RINEX 4 record of a text file, keyed by its header
/// line and first line (satellite and epoch), to its remaining lines,
/// trailing blanks trimmed.
fn records_of(content: &str) -> BTreeMap<String, Vec<String>> {
    let mut records = BTreeMap::new();
    let mut header: Option<String> = None;
    let mut current: Option<String> = None;
    let mut after_header = false;
    for line in content.lines() {
        if !after_header {
            after_header = line.contains("END OF HEADER");
            continue;
        }
        let line = line.trim_end().to_string();
        if line.starts_with('>') {
            header = Some(line);
            current = None;
        } else if let Some(record_header) = header.take() {
            let key = format!("{}\n{}", record_header, line);
            records.insert(key.clone(), vec![]);
            current = Some(key);
        } else if let Some(key) = &current {
            records.get_mut(key).unwrap().push(line);
        }
    }
    records
}

/// Asserts that `lines` appear consecutively in `formatted`
/// (records are written in chronological order, not file order)
fn assert_block(formatted: &str, lines: &[&str]) {
    let formatted = formatted.lines().map(|l| l.trim_end()).collect::<Vec<_>>();
    assert!(
        formatted.windows(lines.len()).any(|w| w == lines),
        "block not found:\n{}",
        lines.join("\n")
    );
}

#[test]
fn nav_v2_gps_round_trip() {
    let (_, formatted) = round_trip("NAV/V2/cbw10010.21n.gz");
    // first record written like the original: two digit PRN and year,
    // D exponent, three blank prefix
    let expected = [
        " 1 21  1  1  2  0  0.0 7.874774746600D-04-5.911715561520D-12 0.000000000000D+00",
        "    5.200000000000D+01-7.362500000000D+01 4.318037039040D-09 2.893520298160D-02",
        "   -3.784894943240D-06 1.022444642150D-02 1.076608896260D-06 5.153693731310D+03",
        "    4.392000000000D+05-2.048909664150D-08-8.087355908090D-01 1.639127731320D-07",
        "    9.827409334590D-01 3.673750000000D+02 8.219747770630D-01-8.439637433360D-09",
        "   -3.007268045700D-10 1.000000000000D+00 2.138000000000D+03 0.000000000000D+00",
        "    0.000000000000D+00 0.000000000000D+00 5.122274160390D-09 5.200000000000D+01",
        "    4.329780000000D+05",
    ];
    assert_block(&formatted, &expected);
}

#[test]
fn nav_v2_glonass_round_trip() {
    let (_, formatted) = round_trip("NAV/V2/amel0010.21g");
    let expected = [
        " 1 20 12 31 23 45  0.0 7.282570004460D-05 0.000000000000D+00 7.380000000000D+04",
        "   -1.488799804690D+03-2.196182250980D+00 3.725290298460D-09 0.000000000000D+00",
        "    1.292880712890D+04-2.049269676210D+00 0.000000000000D+00 1.000000000000D+00",
        "    2.193169775390D+04 1.059645652770D+00-9.313225746150D-10 0.000000000000D+00",
    ];
    assert_block(&formatted, &expected);
    round_trip("NAV/V2/dlf10010.21g");
    round_trip("NAV/V2/ijmu3650.21n.gz");
}

#[test]
fn nav_v3_round_trip() {
    for path in [
        "NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx",
        "NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx",
        "NAV/V3/BRDC00GOP_R_20210010000_01D_MN.rnx.gz",
        "NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz",
    ] {
        round_trip(path);
    }
}

#[test]
fn nav_v4_round_trip() {
    let (_, formatted) = round_trip("NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz");
    // STO record layout: time offset codes, blank SBAS identifier, UTC identifier
    assert!(formatted.contains(
        "> STO G26 LNAV
    2022 06 10 19 56 48 GPUT                                  UTC(USNO)         
     2.952840000000E+05 9.313225746155E-10 2.664535259100E-15 0.000000000000E+00
"
    ));
    round_trip("NAV/V4/BRD400DLR_S_20230710000_01D_MN.rnx.gz");
}

#[test]
fn nav_v4_02_spec_examples_round_trip() {
    // every record of the RINEX 4.02 examples file, compared to the
    // specification text: same layout, uppercase exponent, padded lines
    let content = read_resource("NAV/V4/rinex402_examples_MN.rnx");
    let (model, formatted) = round_trip("NAV/V4/rinex402_examples_MN.rnx");
    assert_eq!(model.header.version, Version::new(4, 2));

    let expected = records_of(&content.to_uppercase());
    let formatted = records_of(&formatted.to_uppercase());
    assert_eq!(formatted.len(), 30);
    for (header, lines) in formatted.iter() {
        let model = expected
            .get(header)
            .unwrap_or_else(|| panic!("unexpected record {}", header));
        assert_eq!(lines, model, "{}", header);
    }
}
