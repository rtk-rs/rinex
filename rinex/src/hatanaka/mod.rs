//! RINEX compression / decompression module
use thiserror::Error;

mod compressor;
mod crinex;
mod numdiff;
mod obs;
mod textdiff;

pub use crinex::CRINEX;
pub use numdiff::NumDiff;
pub use obs::ObsDiff;
pub use textdiff::TextDiff;

pub mod decompressor;
pub use decompressor::Decompressor;

use std::io::{Error as IoError, ErrorKind};

/// Hatanaka dedicated Errors
#[derive(Debug)]
pub enum Error {
    /// Header lines should only containt valid UTF-8 data.
    /// One reason being we need to analyse its content
    /// to adapt the compression/decompression scheme.
    // #[error("header contains invalid UTF8")]
    HeaderUtf8Data,
    /// Forwarded Epoch description does not look good: Invalid RINEX!
    EpochFormat,
    /// Recovered Epoch descriptor (in the decompression scheme)
    /// does not look good. Most likely due to invalid content
    /// generated by the decompressor: should never happen
    RecoveredEpochFormat,
    /// Recovered or forwarded SV description is incorrect (bad data)
    SVFormat,
    /// [SV] identification error: bad relationship between
    /// either:
    ///   - recovered Epoch description (in decompression scheme)
    ///   and parsing process
    ///   - invalid data being forwared and/or incompatibility
    ///   with previously formwared Header
    SVParsing,
    /// It is mandatory that the CRINEX version be correctly defined
    /// so we can parse it and adapt the (de-)compression scheme
    VersionParsing,
}

impl Error {
    /// Converts [Error] to custom [IoError]
    fn to_stdio(&self) -> IoError {
        let descriptor = match self {
            Self::HeaderUtf8Data => "bad utf-8 in header",
            Self::EpochFormat => "invalid epoch description",
            Self::RecoveredEpochFormat => "invalid recovered epoch",
            Self::SVFormat => "invalid sv formatting",
            Self::SVParsing => "sv parsing error",
            Self::VersionParsing => "crinex version parsing error",
        };
        IoError::new(ErrorKind::Other, descriptor)
    }
}
