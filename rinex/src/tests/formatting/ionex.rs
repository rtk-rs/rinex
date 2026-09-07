use crate::{prelude::Rinex, tests::formatting::Utf8Buffer};

use std::io::{BufReader, BufWriter};

/// Formats an IONEX file and checks that the maps are numbered from 1
/// in chronological order, each announced by its epoch, and that the
/// output parses back to the same record.
#[cfg(feature = "flate2")]
fn ionex_round_trip(file: &str, first_epoch: &str, last_epoch: &str, has_rms: bool) {
    let path = format!(
        "{}/../test_resources/IONEX/V1/{}",
        env!("CARGO_MANIFEST_DIR"),
        file
    );

    let model = Rinex::from_gzip_file(&path).unwrap();
    let nb_maps = model.header.ionex.as_ref().unwrap().number_of_maps;

    let mut buf = BufWriter::new(Utf8Buffer::new(1 << 20));
    model.format(&mut buf).unwrap();
    let content = buf.into_inner().unwrap().to_ascii_utf8();

    let lines = content.lines().collect::<Vec<_>>();

    for kind in ["TEC", "RMS"] {
        let starts = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.ends_with(&format!("START OF {} MAP", kind)))
            .collect::<Vec<_>>();

        if kind == "RMS" && !has_rms {
            assert!(starts.is_empty(), "{}: unexpected RMS maps", file);
            continue;
        }

        assert_eq!(
            starts.len(),
            nb_maps,
            "{}: wrong number of {} maps",
            file,
            kind
        );

        for (nth, (index, line)) in starts.iter().enumerate() {
            let number = line[..60].trim().parse::<usize>().unwrap();
            assert_eq!(number, nth + 1, "{}: {} map numbering", file, kind);

            let epoch = &lines[index + 1];
            assert!(
                epoch.ends_with("EPOCH OF CURRENT MAP"),
                "{}: {}",
                file,
                epoch
            );

            if nth == 0 {
                assert_eq!(epoch[..60].trim_end(), first_epoch);
            } else if nth == nb_maps - 1 {
                assert_eq!(epoch[..60].trim_end(), last_epoch);
            }

            let end = lines
                .iter()
                .skip(index + 1)
                .find(|line| line.ends_with(&format!("END OF {} MAP", kind)))
                .unwrap();
            assert_eq!(end[..60].trim().parse::<usize>().unwrap(), nth + 1);
        }
    }

    assert!(lines.last().unwrap().ends_with("END OF FILE"));

    let parsed = Rinex::parse(&mut BufReader::new(content.as_bytes())).unwrap();

    let (model_rec, parsed_rec) = (
        model.record.as_ionex().unwrap(),
        parsed.record.as_ionex().unwrap(),
    );

    assert_eq!(parsed_rec.len(), model_rec.len(), "{}: point count", file);
    assert_eq!(parsed_rec, model_rec, "{}: record differs", file);
}

#[test]
#[cfg(feature = "flate2")]
fn ionex_tec_maps_round_trip() {
    ionex_round_trip(
        "CKMG0020.22I.gz",
        "  2022     1     2     0     0     0",
        "  2022     1     3     0     0     0",
        false,
    );
}

#[test]
#[cfg(feature = "flate2")]
fn ionex_tec_rms_maps_round_trip() {
    ionex_round_trip(
        "jplg0010.17i.gz",
        "  2017     1     1     0     0     0",
        "  2017     1     2     0     0     0",
        true,
    );
}
