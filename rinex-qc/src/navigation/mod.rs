// pub(crate) mod buffer;
pub(crate) mod eph;
// pub(crate) mod gpx;
// pub(crate) mod kml;
pub(crate) mod clock;
pub(crate) mod environment;
pub(crate) mod orbit;
pub(crate) mod pvt;
pub(crate) mod signal;
pub(crate) mod time;

#[cfg(feature = "cggtts")]
pub(crate) mod cggtts;

use gnss_rtk::prelude::Carrier as RTKCarrier;
use rinex::prelude::Carrier;

/// Converts [Carrier] to [RTKCarrier].
/// The solver identifies carriers by their frequency: signals
/// sharing a frequency (L1/E1/B1C, L5/E5a/B2a, ...) map to the same
/// [RTKCarrier], and signals the solver does not know are dropped.
pub(crate) fn carrier_to_rtk(carrier: &Carrier) -> Option<RTKCarrier> {
    RTKCarrier::from_frequency_mega_hz(carrier.frequency_mhz()).ok()
}

#[cfg(test)]
mod test {
    use super::carrier_to_rtk;
    use gnss_rtk::prelude::Carrier as RTKCarrier;
    use rinex::prelude::Carrier;

    #[test]
    fn carrier_mapping() {
        for (carrier, expected) in [
            (Carrier::L1, Some(RTKCarrier::L1)),
            (Carrier::E1, Some(RTKCarrier::L1)),
            (Carrier::B1C, Some(RTKCarrier::L1)),
            (Carrier::L2, Some(RTKCarrier::L2)),
            (Carrier::L5, Some(RTKCarrier::L5)),
            (Carrier::E5a, Some(RTKCarrier::L5)),
            (Carrier::B2A, Some(RTKCarrier::L5)),
            (Carrier::E5b, Some(RTKCarrier::E5b)),
            (Carrier::B2I, Some(RTKCarrier::E5b)),
            (Carrier::E5, Some(RTKCarrier::E5a5b)),
            (Carrier::B2, Some(RTKCarrier::E5a5b)),
            (Carrier::B1I, Some(RTKCarrier::B1)),
            (Carrier::B3, Some(RTKCarrier::B3)),
            (Carrier::E6, None),
            (Carrier::G1(None), None),
            (Carrier::G1(Some(3)), None),
            (Carrier::S, None),
        ] {
            assert_eq!(carrier_to_rtk(&carrier), expected, "{}", carrier);
        }
    }
}
