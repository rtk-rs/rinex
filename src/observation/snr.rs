use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone)]
pub enum Error {
    InvalidSNRCode,
}

/// Signal to noise ratio description, generally closely tied
/// to raw GNSS signal observations.
#[derive(Default, PartialOrd, Ord, PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SNR {
    /// SNR ~= 0 dB/Hz
    DbHz0,
    /// SNR < 12 dB/Hz
    DbHz12,
    /// 12 dB/Hz <= SNR < 17 dB/Hz
    DbHz12_17,
    /// 18 dB/Hz <= SNR < 23 dB/Hz
    DbHz18_23,
    /// 24 dB/Hz <= SNR < 29 dB/Hz
    #[default]
    DbHz24_29,
    /// 30 dB/Hz <= SNR < 35 dB/Hz
    DbHz30_35,
    /// 36 dB/Hz <= SNR < 41 dB/Hz
    DbHz36_41,
    /// 42 dB/Hz <= SNR < 47 dB/Hz
    DbHz42_47,
    /// 48 dB/Hz <= SNR < 53 dB/Hz
    DbHz48_53,
    /// SNR >= 54 dB/Hz
    DbHz54,
}

impl std::fmt::LowerHex for SNR {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let descriptor = match self {
            Self::DbHz0 => "0",
            Self::DbHz12 => "1",
            Self::DbHz12_17 => "2",
            Self::DbHz18_23 => "3",
            Self::DbHz24_29 => "4",
            Self::DbHz30_35 => "5",
            Self::DbHz36_41 => "6",
            Self::DbHz42_47 => "7",
            Self::DbHz48_53 => "8",
            Self::DbHz54 => "9",
        };
        f.write_str(descriptor)
    }
}

impl std::fmt::LowerExp for SNR {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let descriptor = match self {
            Self::DbHz0 => "<< 12 dB",
            Self::DbHz12 => "< 12 dB",
            Self::DbHz12_17 => "[12, 17[ dB",
            Self::DbHz18_23 => "[18, 23[ dB",
            Self::DbHz24_29 => "[24, 29[ dB",
            Self::DbHz30_35 => "[30, 35[ dB",
            Self::DbHz36_41 => "[36, 41[ dB",
            Self::DbHz42_47 => "[42, 47[ dB",
            Self::DbHz48_53 => "[48, 53[ dB",
            Self::DbHz54 => "> 54 dB",
        };
        f.write_str(descriptor)
    }
}

impl FromStr for SNR {
    type Err = Error;
    fn from_str(code: &str) -> Result<Self, Self::Err> {
        match code.trim() {
            "0" => Ok(SNR::DbHz0),
            "1" => Ok(SNR::DbHz12),
            "2" => Ok(SNR::DbHz12_17),
            "3" => Ok(SNR::DbHz18_23),
            "4" => Ok(SNR::DbHz24_29),
            "5" => Ok(SNR::DbHz30_35),
            "6" => Ok(SNR::DbHz36_41),
            "7" => Ok(SNR::DbHz42_47),
            "8" => Ok(SNR::DbHz48_53),
            "9" => Ok(SNR::DbHz54),
            "bad" => Ok(SNR::DbHz18_23),
            "weak" => Ok(SNR::DbHz24_29),
            "strong" => Ok(SNR::DbHz30_35),
            "excellent" => Ok(SNR::DbHz48_53),
            _ => Err(Error::InvalidSNRCode),
        }
    }
}

impl From<f64> for SNR {
    fn from(f_db: f64) -> Self {
        if f_db < 12.0 {
            Self::DbHz12
        } else if f_db <= 17.0 {
            Self::DbHz12_17
        } else if f_db <= 23.0 {
            Self::DbHz18_23
        } else if f_db <= 29.0 {
            Self::DbHz24_29
        } else if f_db <= 35.0 {
            Self::DbHz30_35
        } else if f_db <= 41.0 {
            Self::DbHz36_41
        } else if f_db <= 47.0 {
            Self::DbHz42_47
        } else if f_db <= 53.0 {
            Self::DbHz48_53
        } else {
            Self::DbHz54
        }
    }
}

impl From<SNR> for f64 {
    fn from(val: SNR) -> Self {
        match val {
            SNR::DbHz0 => 0.0_f64,
            SNR::DbHz12 => 12.0_f64,
            SNR::DbHz12_17 => 17.0_f64,
            SNR::DbHz18_23 => 23.0_f64,
            SNR::DbHz24_29 => 29.0_f64,
            SNR::DbHz30_35 => 35.0_f64,
            SNR::DbHz36_41 => 41.0_f64,
            SNR::DbHz42_47 => 47.0_f64,
            SNR::DbHz48_53 => 53.0_f64,
            SNR::DbHz54 => 54.0_f64,
        }
    }
}

impl From<u8> for SNR {
    fn from(u: u8) -> Self {
        match u {
            1 => Self::DbHz12,
            2 => Self::DbHz12_17,
            3 => Self::DbHz18_23,
            4 => Self::DbHz24_29,
            5 => Self::DbHz30_35,
            6 => Self::DbHz36_41,
            7 => Self::DbHz42_47,
            8 => Self::DbHz48_53,
            9 => Self::DbHz54,
            _ => Self::DbHz0,
        }
    }
}

impl SNR {
    /// Returns true if self describes a bad signal level
    pub fn bad(self) -> bool {
        self <= SNR::DbHz18_23
    }
    /// Returns true if `self` describes a weak signal level
    pub fn weak(self) -> bool {
        self < SNR::DbHz30_35
    }
    /// Returns true if `self` describes a strong signal level, defined in standard specifications
    pub fn strong(self) -> bool {
        self >= SNR::DbHz30_35
    }
    /// Returns true if `self` is a very strong signal level
    pub fn excellent(self) -> bool {
        self > SNR::DbHz42_47
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn observation_snr() {
        let snr = SNR::from_str("0").unwrap();
        assert_eq!(snr, SNR::DbHz0);
        assert!(snr.bad());

        let snr = SNR::from_str("8").unwrap();
        assert_eq!("8", format!("{:x}", snr));

        let snr = SNR::from_str("9").unwrap();
        assert!(snr.excellent());

        let snr = SNR::from_str("10");
        assert!(snr.is_err());

        let snr: SNR = SNR::from(8);
        assert_eq!(snr, SNR::DbHz48_53);
        assert!(snr.excellent());
        assert_eq!(format!("{:x}", snr), "8");
        assert_eq!(format!("{:e}", snr), "[48, 53[ dB");

        let snr: SNR = SNR::from(31.3);
        assert_eq!(snr, SNR::DbHz30_35);
        assert!(snr.strong());

        let snr: SNR = SNR::from(3.0);
        assert_eq!(snr, SNR::DbHz12);
        assert!(snr.bad());

        assert_eq!(SNR::from_str("excellent"), Ok(SNR::DbHz48_53));
        assert_eq!(SNR::from_str("strong"), Ok(SNR::DbHz30_35));
        assert_eq!(SNR::from_str("weak"), Ok(SNR::DbHz24_29));
        assert_eq!(SNR::from_str("bad"), Ok(SNR::DbHz18_23));

        assert!(SNR::from_str("bad").unwrap().bad());
        assert!(SNR::from_str("weak").unwrap().weak());
        assert!(SNR::from_str("strong").unwrap().strong());
        assert!(SNR::from_str("excellent").unwrap().excellent());
    }
}
