use super::Error;

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum DataSource {
    /// Source of data is hardware (radio) receiver.
    /// It can also represent a sensor in case of meteo observations.
    Receiver,
    /// Other stream source, like RTCM
    Stream,
    /// Unknown data source
    #[default]
    Unknown,
}

impl std::str::FromStr for DataSource {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        if content.eq("R") || content.eq("RCVR") {
            Ok(Self::Receiver)
        } else if content.eq("S") || content.eq("STREAM") {
            Ok(Self::Stream)
        } else {
            Ok(Self::Unknown)
        }
    }
}

impl DataSource {
    pub(crate) fn to_char(&self) -> char {
        match self {
            Self::Receiver => 'R',
            Self::Stream => 'S',
            Self::Unknown => 'U',
        }
    }
}
