//! Navigation RINEX formatting round trips
use crate::{
    navigation::{NavFrameType, NavMessageType, OrbitItem, Record},
    prelude::{Constellation, Rinex, Version},
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

/// Parses the formatted text
fn reparse(formatted: &str) -> Rinex {
    Rinex::parse(&mut BufReader::new(Cursor::new(formatted)))
        .unwrap_or_else(|e| panic!("formatted file does not parse: {:?}", e))
}

/// Orbits of `eph` with the RINEX 3 BeiDou group delay names
/// renamed to the RINEX 4 names
fn v4_orbit_names(
    orbits: &std::collections::HashMap<String, OrbitItem>,
) -> BTreeMap<String, OrbitItem> {
    orbits
        .iter()
        .map(|(k, v)| {
            let k = match k.as_str() {
                "tgd1b1b3" => "tgdb1b3",
                "tgd2b2b3" => "tgdb2b3",
                k => k,
            };
            (k.to_string(), v.clone())
        })
        .collect()
}

#[test]
fn nav_v4_to_v3_conversion() {
    let mut model = parse(&read_resource(
        "NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz",
    ));
    model.header.version = Version::new(3, 5);

    let formatted = format(&model);
    let dut = reparse(&formatted);
    assert_eq!(dut.header.version, Version::new(3, 5));
    let dut = dut.record.as_nav().unwrap();

    // legacy ephemerides only: STO, ION and modern messages are skipped
    let legacy = model
        .record
        .as_nav()
        .unwrap()
        .iter()
        .filter(|(k, _)| {
            k.frmtype == NavFrameType::Ephemeris
                && matches!(
                    k.msgtype,
                    NavMessageType::LNAV
                        | NavMessageType::FDMA
                        | NavMessageType::INAV
                        | NavMessageType::FNAV
                        | NavMessageType::D1
                        | NavMessageType::D2
                        | NavMessageType::SBAS
                )
        })
        .collect::<Vec<_>>();
    assert!(legacy.len() > 300);

    // every legacy ephemeris is written: one record per satellite
    // line at column 0
    let nb_records = formatted
        .lines()
        .skip_while(|l| !l.contains("END OF HEADER"))
        .skip(1)
        .filter(|l| !l.starts_with(' '))
        .count();
    assert_eq!(nb_records, legacy.len());

    // RINEX 3 files every ephemeris as LNAV: a Galileo INAV and FNAV
    // pair of the same epoch shares a key once parsed back, and the
    // last written (INAV) is kept
    let mut expected = BTreeMap::new();
    for (k, frame) in legacy {
        let key = crate::navigation::NavKey {
            msgtype: NavMessageType::LNAV,
            ..*k
        };
        expected.insert(key, frame);
    }
    assert_eq!(dut.len(), expected.len());

    for (key, frame) in expected {
        let dut = dut
            .get(&key)
            .unwrap_or_else(|| panic!("{:?} missing", key))
            .as_ephemeris()
            .unwrap();
        let eph = frame.as_ephemeris().unwrap();
        let k = &key;
        assert_eq!(dut.clock_bias, eph.clock_bias);
        assert_eq!(dut.clock_drift, eph.clock_drift);
        assert_eq!(dut.clock_drift_rate, eph.clock_drift_rate);
        // the RINEX 3 layout is a subset of the RINEX 4 one, BeiDou
        // group delays under their RINEX 3 names
        assert!(!dut.orbits.is_empty());
        for (name, item) in v4_orbit_names(&dut.orbits).iter() {
            assert_eq!(Some(item), eph.orbits.get(name), "{:?} {}", k, name);
        }
    }
}

#[test]
fn nav_v3_to_v4_conversion() {
    let model = parse(&read_resource("NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx"));
    let model_rec = model.record.as_nav().unwrap();

    let mut v4 = model.clone();
    v4.header.version = Version::new(4, 0);
    let formatted = format(&v4);

    // RINEX 4 message types assigned from the constellation and content
    assert!(formatted.contains("> EPH R"));
    assert!(formatted.contains(" FDMA\n"));
    assert!(formatted.contains(" INAV\n") || formatted.contains(" FNAV\n"));
    assert!(formatted.contains("> EPH C05 D2\n"));
    assert!(formatted.contains("> EPH C21 D1\n"));
    assert!(!formatted.contains("> EPH R") || !formatted.contains("> EPH R05 LNAV"));

    let dut = reparse(&formatted);
    assert_eq!(dut.header.version, Version::new(4, 0));
    let dut_rec = dut.record.as_nav().unwrap();
    assert_eq!(dut_rec.len(), model_rec.len());

    for ((k, frame), (dk, dframe)) in model_rec.iter().zip(dut_rec.iter()) {
        assert_eq!((k.epoch, k.sv, k.frmtype), (dk.epoch, dk.sv, dk.frmtype));
        let expected = match k.sv.constellation {
            Constellation::Glonass => NavMessageType::FDMA,
            Constellation::BeiDou if k.sv.prn <= 5 || k.sv.prn >= 59 => NavMessageType::D2,
            Constellation::BeiDou => NavMessageType::D1,
            Constellation::Galileo => {
                let eph = frame.as_ephemeris().unwrap();
                if eph.get_orbit_f64("dataSrc").unwrap_or(0.0) as u32 & 0x02 != 0 {
                    NavMessageType::FNAV
                } else {
                    NavMessageType::INAV
                }
            },
            c if c.is_sbas() => NavMessageType::SBAS,
            _ => NavMessageType::LNAV,
        };
        assert_eq!(dk.msgtype, expected, "{:?}", k);
        let eph = frame.as_ephemeris().unwrap();
        let deph = dframe.as_ephemeris().unwrap();
        assert_eq!(deph.sv_clock(), eph.sv_clock(), "{:?}", k);
        // RINEX 4 layouts carry every RINEX 3 field, BeiDou group
        // delays under their RINEX 4 names
        let model_orbits = v4_orbit_names(&eph.orbits);
        for (name, item) in model_orbits.iter() {
            assert_eq!(deph.orbits.get(name), Some(item), "{:?} {}", k, name);
        }
    }

    // and back to RINEX 3: identical record
    let mut v3 = dut;
    v3.header.version = model.header.version;
    let back = reparse(&format(&v3));
    assert_records_equal(back.record.as_nav().unwrap(), model_rec);
}

#[test]
fn nav_v3_to_v2_conversion() {
    let model = parse(&read_resource("NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx"));
    let model_rec = model.record.as_nav().unwrap();

    // RINEX 2 GPS file from a mixed RINEX 3 record: GPS only
    let mut v2 = model.clone();
    v2.header.version = Version::new(2, 11);
    v2.header.constellation = Some(Constellation::GPS);
    let formatted = format(&v2);

    let dut = reparse(&formatted);
    assert_eq!(dut.header.version, Version::new(2, 11));
    let dut_rec = dut.record.as_nav().unwrap();

    let gps = model_rec
        .iter()
        .filter(|(k, _)| k.sv.constellation == Constellation::GPS)
        .collect::<Vec<_>>();
    assert!(!gps.is_empty());
    assert_eq!(dut_rec.len(), gps.len());
    for (k, frame) in gps {
        assert_eq!(dut_rec.get(k), Some(frame), "{:?}", k);
    }
}
