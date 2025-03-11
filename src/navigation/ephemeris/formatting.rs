//! Ephemeris message formatting
use crate::{
    navigation::{orbits::closest_nav_standards, Ephemeris, NavMessageType},
    prelude::{Constellation, SV},
    FormattingError, Version,
};

use std::io::{BufWriter, Write};

impl Ephemeris {
    /// Formats [Ephemeris] according to RINEX standards
    pub(crate) fn format<W: Write>(
        &self,
        w: &mut BufWriter<W>,
        sv: SV,
        version: Version,
        msgtype: NavMessageType,
    ) -> Result<(), FormattingError> {
        let sv_constellation = if sv.constellation.is_sbas() {
            Constellation::SBAS
        } else {
            sv.constellation
        };

        // retrieve standard specs
        let standard_specs = match closest_nav_standards(sv_constellation, version, msgtype) {
            Some(specs) => specs,
            None => {
                return Err(FormattingError::MissingNavigationStandards);
            },
        };

        // starts with (clock_bias, drift, rate)
        // epoch has already been buffered
        write!(
            w,
            "{:.17E} {:.17E} {:.17E}\n",
            self.clock_bias, self.clock_drift, self.clock_drift_rate
        )?;

        // following standard specs
        let data_fields = &standard_specs.items;
        for i in 0..data_fields.len() {
            write!(w, "0.000000000000D+00")?;
            if let Some(value) = self.get_orbit_f64(data_fields[i].0) {
                write!(w, "0.000000000000D+00")?;
            } else {
                // standardized missing field
                write!(w, "0.000000000000D+00")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::navigation::Ephemeris;

    use std::io::BufWriter;
    use std::str::FromStr;

    use crate::tests::formatting::Utf8Buffer;

    #[test]
    fn test_ephemeris_fmt() {
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
}
