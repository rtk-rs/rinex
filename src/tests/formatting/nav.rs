//! Navigation RINEX formatting, compared to the original files.
use crate::{navigation::NavFrameType, prelude::Rinex};

use std::{collections::HashMap, fs::read_to_string, fs::remove_file};

/// Splits a RINEX 4 navigation record section into blocks, one per
/// "> XXX" record, keyed by the record header, the epoch and, for STO
/// records, the time system pair. Exponent letters are uppercased and
/// trailing blanks trimmed on every line.
fn v4_record_blocks(content: &str) -> HashMap<String, Vec<String>> {
    let mut blocks = HashMap::<String, Vec<String>>::new();
    let mut header = Option::<String>::None;
    let mut current = Option::<String>::None;

    for line in content
        .lines()
        .skip_while(|line| !line.contains("END OF HEADER"))
        .skip(1)
    {
        let line = line.trim_end().replace('e', "E");
        if line.starts_with("> ") {
            header = Some(line.clone());
            current = None;
            continue;
        }
        if let Some(header) = header.take() {
            let key = if header.starts_with("> STO") {
                format!("{}\n{}", header, &line[..28.min(line.len())])
            } else {
                format!("{}\n{}", header, &line[..23.min(line.len())])
            };
            blocks.insert(key.clone(), Vec::new());
            current = Some(key);
        }
        if let Some(key) = &current {
            if let Some(block) = blocks.get_mut(key) {
                block.push(line);
            }
        }
    }

    blocks
}

/// Every STO, EOP and ION record written from `name` must be equal to
/// the original text, and every such record of the parsed file must be
/// written. Records sharing epoch, vehicle and message type overwrite
/// each other in the record, so the original file may hold more.
fn v4_records_write_back(name: &str) {
    let path = format!("data/NAV/V4/{}", name);
    let original = if name.ends_with(".gz") {
        Rinex::from_gzip_file(&path).unwrap()
    } else {
        Rinex::from_file(&path).unwrap()
    };

    let tmp = format!("test-{}.rnx", name);
    original.to_file(&tmp).unwrap();
    let written = read_to_string(&tmp).unwrap();
    let _ = remove_file(&tmp);

    let original_text = if name.ends_with(".gz") {
        let mut reader = flate2::read::GzDecoder::new(std::fs::File::open(&path).unwrap());
        let mut content = String::new();
        std::io::Read::read_to_string(&mut reader, &mut content).unwrap();
        content
    } else {
        read_to_string(&path).unwrap()
    };

    let model = v4_record_blocks(&original_text);
    let dut = v4_record_blocks(&written);

    let mut compared = 0;
    for (key, written) in dut.iter() {
        if key.starts_with("> EPH") {
            continue;
        }
        let block = model
            .get(key)
            .unwrap_or_else(|| panic!("written record not in the original file:\n{}", key));

        let (block, written) = if key.starts_with("> STO") {
            // the message transmission time is not stored: the first
            // field of the second line is not compared
            (
                vec![block[0].clone(), block[1][23..].to_string()],
                vec![written[0].clone(), written[1][23..].to_string()],
            )
        } else if key.starts_with("> ION") && block.len() == 3 && block[2].len() == 23 {
            // Klobuchar: a blank region code is read as worldwide (0)
            // and written as such
            let mut block = block.clone();
            block[2].push_str(" 0.000000000000E+00");
            (block, written.clone())
        } else {
            (block.clone(), written.clone())
        };

        assert_eq!(written, block, "record differs:\n{}", key);
        compared += 1;
    }

    let expected = original
        .record
        .as_nav()
        .unwrap()
        .iter()
        .filter(|(k, _)| k.frmtype != NavFrameType::Ephemeris)
        .count();

    assert_eq!(
        compared, expected,
        "not every STO, EOP or ION record written"
    );
    assert!(compared > 0, "no STO, EOP or ION record in {}", name);
}

#[test]
fn nav_v4_kms300dnk_records_write_back() {
    v4_records_write_back("KMS300DNK_R_20221591000_01H_MN.rnx.gz");
}

#[test]
fn nav_v4_brd400dlr_records_write_back() {
    v4_records_write_back("BRD400DLR_S_20230710000_01D_MN.rnx.gz");
}

/// RINEX 4.02 specification examples: NavIC and GLONASS CDMA ION
/// records, STO records with the new time system pairs and blank PRNs.
#[test]
fn nav_v4_02_examples_records_write_back() {
    v4_records_write_back("rinex402_examples_MN.rnx");
}
