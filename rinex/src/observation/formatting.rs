//! OBS RINEX formatting
use crate::{
    epoch::format as epoch_format,
    hatanaka::Compressor,
    observation::Record,
    observation::{ClockObservation, HeaderFields, ObsKey},
    prelude::{Constellation, Header, RinexType, SV},
    FormattingError,
};

use std::io::{BufWriter, Write};

use itertools::Itertools;

pub fn format<W: Write>(
    w: &mut BufWriter<W>,
    record: &Record,
    header: &Header,
) -> Result<(), FormattingError> {
    let major = header.version.major;

    let header = header
        .obs
        .as_ref()
        .ok_or(FormattingError::UndefinedObservables)?;

    if header.crinex.is_some() {
        format_compressed(w, record, header)
    } else {
        if major < 3 {
            format_v2(w, header, record)
        } else {
            format_v3(w, header, record)
        }
    }
}

/// Compressed format (non readable yet still ASCII)
/// following the Hatanaka Compression algorithm.
/// We use this in the RNX2CRX operation.
pub fn format_compressed<W: Write>(
    w: &mut BufWriter<W>,
    record: &Record,
    header: &HeaderFields,
) -> Result<(), FormattingError> {
    let mut compressor = Compressor::default();
    compressor.format(w, record, header)
}

/// Header observables are defined per constellation, and once for all
/// SBAS vehicles whatever their actual system.
fn sv_constellation(sv: &SV) -> Constellation {
    if sv.constellation.is_sbas() {
        Constellation::SBAS
    } else {
        sv.constellation
    }
}

fn format_epoch_v3<W: Write>(
    w: &mut BufWriter<W>,
    k: &ObsKey,
    sv_list: &[SV],
    clock: Option<ClockObservation>,
) -> Result<(), FormattingError> {
    let numsat = sv_list.len();

    if let Some(clock) = clock {
        writeln!(
            w,
            "> {}  {} {:2}  {:13.4}",
            epoch_format(k.epoch, RinexType::ObservationData, 3),
            k.flag,
            numsat,
            clock.offset_s,
        )?;
    } else {
        writeln!(
            w,
            "> {}  {} {:2}",
            epoch_format(k.epoch, RinexType::ObservationData, 3),
            k.flag,
            numsat,
        )?;
    }

    Ok(())
}

fn format_v3<W: Write>(
    w: &mut BufWriter<W>,
    header: &HeaderFields,
    record: &Record,
) -> Result<(), FormattingError> {
    let observables = &header.codes;

    for (k, v) in record.iter() {
        // retrieve unique sv list
        let sv_list = v
            .signals
            .iter()
            .map(|k| k.sv)
            .unique()
            .sorted()
            .collect::<Vec<_>>();

        // retrieve unique constellation list
        let constell_list = sv_list
            .iter()
            .map(|sv| sv.constellation)
            .unique()
            .sorted()
            .collect::<Vec<_>>();

        // encode new epoch
        format_epoch_v3(w, k, &sv_list, v.clock)?;

        // by sorted constellation then SV
        for constell in constell_list.iter() {
            // by sorted SV
            for sv in sv_list.iter().filter(|sv| sv.constellation == *constell) {
                write!(w, "{:x}", sv)?;

                // following header definition
                let observables = observables
                    .get(&sv_constellation(sv))
                    .ok_or(FormattingError::MissingObservableDefinition)?;

                for observable in observables.iter() {
                    if let Some(observation) = v
                        .signals
                        .iter()
                        .filter(|sig| sig.sv == *sv && sig.observable == *observable)
                        .reduce(|k, _| k)
                    {
                        write!(w, "{:14.3}", observation.value)?;

                        if let Some(lli) = &observation.lli {
                            write!(w, "{}", lli.bits())?;
                        } else {
                            write!(w, " ")?;
                        }

                        if let Some(snr) = &observation.snr {
                            write!(w, "{:x}", snr)?;
                        } else {
                            write!(w, " ")?;
                        }
                    } else {
                        // Blanking
                        write!(w, "{:14.3}", "")?;
                    }
                }
                write!(w, "{}", '\n')?;
            }
        }
    }
    Ok(())
}

fn format_epoch_v2<W: Write>(
    w: &mut BufWriter<W>,
    k: &ObsKey,
    sv_list: &[SV],
    clock: Option<ClockObservation>,
) -> Result<(), FormattingError> {
    const NUM_SV_PER_LINE: usize = 12;
    const NEW_LINE_PADDING: &str = "                                ";

    let numsat = sv_list.len();

    if let Some(clock) = clock {
        write!(
            w,
            " {}  {} {:2}",
            epoch_format(k.epoch, RinexType::ObservationData, 2),
            k.flag,
            numsat,
        )?;
    } else {
        write!(
            w,
            " {}  {} {:2}",
            epoch_format(k.epoch, RinexType::ObservationData, 2),
            k.flag,
            numsat,
        )?;
    }

    for (nth, sv) in sv_list.iter().enumerate() {
        if nth > 0 && (nth % NUM_SV_PER_LINE) == 0 {
            write!(w, "{}", NEW_LINE_PADDING)?;
        }
        write!(w, "{:x}", sv)?;
        if nth < numsat - 1 && nth % NUM_SV_PER_LINE == NUM_SV_PER_LINE - 1 {
            write!(w, "{}", '\n')?;
        }
        if nth == numsat - 1 {
            write!(w, "{}", '\n')?;
        }
    }

    Ok(())
}

fn format_v2<W: Write>(
    w: &mut BufWriter<W>,
    header: &HeaderFields,
    record: &Record,
) -> Result<(), FormattingError> {
    const BLANKING: &str = "        ";
    const OBSERVATIONS_PER_LINE: usize = 5;

    let observables = &header.codes;

    for (key, observations) in record.iter() {
        // Form unique SV list @t
        let sv_list = observations
            .signals
            .iter()
            .map(|sig| sig.sv)
            .unique()
            .sorted()
            .collect::<Vec<_>>();

        format_epoch_v2(w, key, &sv_list, observations.clock)?;

        for sv in sv_list.iter() {
            // following Header specs
            let observables = observables
                .get(&sv_constellation(sv))
                .ok_or(FormattingError::MissingObservableDefinition)?;

            let mut modulo = 0;

            for (nth, observable) in observables.iter().enumerate() {
                // retrieve observed signal (if any)
                if let Some(observation) = observations
                    .signals
                    .iter()
                    .filter(|sig| &sig.sv == sv && &sig.observable == observable)
                    .reduce(|k, _| k)
                {
                    write!(w, "{:14.3}", observation.value)?;

                    if let Some(lli) = observation.lli {
                        write!(w, "{:x}", lli)?;
                    } else {
                        write!(w, " ")?;
                    }

                    if let Some(snr) = observation.snr {
                        write!(w, "{:x}", snr)?;
                    } else {
                        write!(w, " ")?;
                    }
                } else {
                    // Blanking
                    write!(w, "{}", BLANKING)?;
                }

                if (nth % OBSERVATIONS_PER_LINE) == OBSERVATIONS_PER_LINE - 1 {
                    write!(w, "{}", '\n')?;
                }

                modulo = nth % OBSERVATIONS_PER_LINE;
            }

            if modulo != OBSERVATIONS_PER_LINE - 1 {
                write!(w, "{}", '\n')?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {

    use super::{format_epoch_v2, format_epoch_v3};

    use crate::prelude::{
        obs::{EpochFlag, ObsKey},
        Epoch, SV,
    };

    use std::io::BufWriter;
    use std::str::FromStr;

    use crate::tests::formatting::Utf8Buffer;

    #[test]
    fn test_fmt_epoch_v2() {
        let key = ObsKey {
            flag: EpochFlag::Ok,
            epoch: Epoch::from_str("2017-01-01T00:00:00 GPST").unwrap(),
        };

        let sv_list = [
            SV::from_str("G03").unwrap(),
            SV::from_str("G08").unwrap(),
            SV::from_str("G14").unwrap(),
            SV::from_str("G16").unwrap(),
            SV::from_str("G22").unwrap(),
            SV::from_str("G23").unwrap(),
            SV::from_str("G26").unwrap(),
            SV::from_str("G27").unwrap(),
            SV::from_str("G31").unwrap(),
            SV::from_str("G32").unwrap(),
        ];

        let mut buf = BufWriter::new(Utf8Buffer::new(1024));
        format_epoch_v2(&mut buf, &key, &sv_list, None).unwrap();

        let content = buf.into_inner().unwrap().to_ascii_utf8();
        assert_eq!(
            content,
            " 17  1  1  0  0  0.0000000  0 10G03G08G14G16G22G23G26G27G31G32\n",
        );

        let sv_list = [
            SV::from_str("G07").unwrap(),
            SV::from_str("G23").unwrap(),
            SV::from_str("G26").unwrap(),
            SV::from_str("G20").unwrap(),
            SV::from_str("G21").unwrap(),
            SV::from_str("G18").unwrap(),
            SV::from_str("R24").unwrap(),
            SV::from_str("R09").unwrap(),
            SV::from_str("G08").unwrap(),
            SV::from_str("G27").unwrap(),
            SV::from_str("G10").unwrap(),
            SV::from_str("G16").unwrap(),
            SV::from_str("R18").unwrap(),
            SV::from_str("G13").unwrap(),
            SV::from_str("R01").unwrap(),
            SV::from_str("R16").unwrap(),
            SV::from_str("R17").unwrap(),
            SV::from_str("G15").unwrap(),
            SV::from_str("R02").unwrap(),
            SV::from_str("R15").unwrap(),
        ];

        let mut buf = BufWriter::new(Utf8Buffer::new(1024));
        format_epoch_v2(&mut buf, &key, &sv_list, None).unwrap();

        let content = buf.into_inner().unwrap().to_ascii_utf8();
        assert_eq!(
            content,
            " 17  1  1  0  0  0.0000000  0 20G07G23G26G20G21G18R24R09G08G27G10G16\n                                R18G13R01R16R17G15R02R15\n",
        );

        let sv_list = [
            SV::from_str("G07").unwrap(),
            SV::from_str("G23").unwrap(),
            SV::from_str("G26").unwrap(),
            SV::from_str("G20").unwrap(),
            SV::from_str("G21").unwrap(),
            SV::from_str("G18").unwrap(),
            SV::from_str("R24").unwrap(),
            SV::from_str("R09").unwrap(),
            SV::from_str("G08").unwrap(),
            SV::from_str("G27").unwrap(),
            SV::from_str("G10").unwrap(),
            SV::from_str("G16").unwrap(),
            SV::from_str("R18").unwrap(),
        ];

        let mut buf = BufWriter::new(Utf8Buffer::new(1024));
        format_epoch_v2(&mut buf, &key, &sv_list, None).unwrap();

        let content = buf.into_inner().unwrap().to_ascii_utf8();
        assert_eq!(
            content,
            " 17  1  1  0  0  0.0000000  0 13G07G23G26G20G21G18R24R09G08G27G10G16\n                                R18\n",
        );

        let sv_list = [
            SV::from_str("G07").unwrap(),
            SV::from_str("G23").unwrap(),
            SV::from_str("G26").unwrap(),
            SV::from_str("G20").unwrap(),
            SV::from_str("G21").unwrap(),
            SV::from_str("G18").unwrap(),
            SV::from_str("R24").unwrap(),
            SV::from_str("R09").unwrap(),
            SV::from_str("G08").unwrap(),
            SV::from_str("G27").unwrap(),
            SV::from_str("G10").unwrap(),
            SV::from_str("G16").unwrap(),
            SV::from_str("R18").unwrap(),
            SV::from_str("G13").unwrap(),
            SV::from_str("R01").unwrap(),
            SV::from_str("R16").unwrap(),
            SV::from_str("R17").unwrap(),
            SV::from_str("G15").unwrap(),
            SV::from_str("R02").unwrap(),
            SV::from_str("R15").unwrap(),
            SV::from_str("C01").unwrap(),
            SV::from_str("C02").unwrap(),
            SV::from_str("C03").unwrap(),
            SV::from_str("C04").unwrap(),
        ];

        let mut buf = BufWriter::new(Utf8Buffer::new(1024));
        format_epoch_v2(&mut buf, &key, &sv_list, None).unwrap();

        let content = buf.into_inner().unwrap().to_ascii_utf8();
        assert_eq!(
            content,
            " 17  1  1  0  0  0.0000000  0 24G07G23G26G20G21G18R24R09G08G27G10G16\n                                R18G13R01R16R17G15R02R15C01C02C03C04\n",
        );

        let sv_list = [
            SV::from_str("G07").unwrap(),
            SV::from_str("G23").unwrap(),
            SV::from_str("G26").unwrap(),
            SV::from_str("G20").unwrap(),
            SV::from_str("G21").unwrap(),
            SV::from_str("G18").unwrap(),
            SV::from_str("R24").unwrap(),
            SV::from_str("R09").unwrap(),
            SV::from_str("G08").unwrap(),
            SV::from_str("G27").unwrap(),
            SV::from_str("G10").unwrap(),
            SV::from_str("G16").unwrap(),
            SV::from_str("R18").unwrap(),
            SV::from_str("G13").unwrap(),
            SV::from_str("R01").unwrap(),
            SV::from_str("R16").unwrap(),
            SV::from_str("R17").unwrap(),
            SV::from_str("G15").unwrap(),
            SV::from_str("R02").unwrap(),
            SV::from_str("R15").unwrap(),
            SV::from_str("C01").unwrap(),
            SV::from_str("C02").unwrap(),
            SV::from_str("C03").unwrap(),
            SV::from_str("C04").unwrap(),
            SV::from_str("C05").unwrap(),
        ];

        let mut buf = BufWriter::new(Utf8Buffer::new(1024));
        format_epoch_v2(&mut buf, &key, &sv_list, None).unwrap();

        let content = buf.into_inner().unwrap().to_ascii_utf8();
        assert_eq!(
            content,
            " 17  1  1  0  0  0.0000000  0 25G07G23G26G20G21G18R24R09G08G27G10G16\n                                R18G13R01R16R17G15R02R15C01C02C03C04\n                                C05\n",
        );
    }

    #[test]
    fn test_fmt_epoch_v3() {
        let key = ObsKey {
            flag: EpochFlag::Ok,
            epoch: Epoch::from_str("2021-01-01T00:00:00 GPST").unwrap(),
        };

        let sv_list = [
            SV::from_str("G01").unwrap(),
            SV::from_str("G02").unwrap(),
            SV::from_str("G03").unwrap(),
            SV::from_str("G04").unwrap(),
        ];

        let mut buf = BufWriter::new(Utf8Buffer::new(1024));
        format_epoch_v3(&mut buf, &key, &sv_list, None).unwrap();
        let content = buf.into_inner().unwrap().to_ascii_utf8();
        assert_eq!(content, "> 2021 01 01 00 00 00.0000000  0  4\n",);

        let sv_list = [
            SV::from_str("G01").unwrap(),
            SV::from_str("G02").unwrap(),
            SV::from_str("G03").unwrap(),
            SV::from_str("G04").unwrap(),
            SV::from_str("G05").unwrap(),
            SV::from_str("G06").unwrap(),
            SV::from_str("G07").unwrap(),
            SV::from_str("G08").unwrap(),
            SV::from_str("G09").unwrap(),
            SV::from_str("G10").unwrap(),
        ];

        let mut buf = BufWriter::new(Utf8Buffer::new(1024));
        format_epoch_v3(&mut buf, &key, &sv_list, None).unwrap();
        let content = buf.into_inner().unwrap().to_ascii_utf8();
        assert_eq!(content, "> 2021 01 01 00 00 00.0000000  0 10\n",);
    }
}
