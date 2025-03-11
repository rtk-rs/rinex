//! integrated tests
pub mod toolkit;

mod antex;
mod compression;
mod crinex;
mod filename;
pub mod formatting;
mod parsing;

#[cfg(feature = "flate2")]
mod production;

#[cfg(feature = "qc")]
mod merge;

#[cfg(feature = "clock")]
mod clock;

#[cfg(feature = "processing")]
mod processing;

#[cfg(feature = "doris")]
mod doris;

#[cfg(feature = "ionex")]
mod ionex;

#[cfg(feature = "meteo")]
mod meteo;

#[cfg(feature = "nav")]
mod nav;

mod obs;
