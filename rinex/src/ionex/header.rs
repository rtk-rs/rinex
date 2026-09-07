/// IONEX specific header fields

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    epoch::format_ionex_utc,
    fmt_rinex,
    ionex::{BiasSource, Grid, MappingFunction, RefSystem},
    linspace::Linspace,
    prelude::{Epoch, FormattingError},
};

use std::{
    collections::HashMap,
    io::{BufWriter, Write},
};

#[cfg(feature = "processing")]
use qc_traits::{FilterItem, MaskFilter, MaskOperand};

#[cfg(feature = "processing")]
use crate::prelude::TimeScale;

/// IONEX specific [HeaderFields]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Total number of TEC maps
    pub number_of_maps: usize,
    /// Epoch of first map
    pub epoch_of_first_map: Epoch,
    /// Epoch of last map
    pub epoch_of_last_map: Epoch,
    /// Reference system used for following TEC maps,
    /// cf. [system::RefSystem].
    pub reference: RefSystem,
    /// It is highly recommended to give a brief description
    /// of the technique, model.. description is not a
    /// general purpose comment
    pub description: Option<String>,
    /// Mapping function adopted for TEC determination,
    /// if None: No mapping function, e.g altimetry
    pub mapping: Option<MappingFunction>,
    /// Maps dimension, can either be a 2D (= fixed altitude mode), or 3D
    pub map_dimension: u8,
    /// Mean earth radius or bottom of height grid, in km.
    pub base_radius: f32,
    /// Reference rid definition.
    pub grid: Grid,
    /// Minimum elevation angle filter used. In degrees.
    pub elevation_cutoff: f32,
    /// Verbose description of observables used in determination.
    /// When no Observables were used, that means we're based off a theoretical model.
    pub observables: Option<String>,
    /// Number of stations that contributed to following data
    pub nb_stations: u32,
    /// Number of satellites that contributed to following data
    pub nb_satellites: u32,
    /// exponent: scaling to apply in current TEC blocs
    pub exponent: i8,
    /// Differential Code Biases (DBCs),
    /// per Vehicle #PRN, (Bias and RMS bias) values.
    pub dcbs: HashMap<BiasSource, (f64, f64)>,
}

impl Default for HeaderFields {
    fn default() -> Self {
        Self {
            // default exponent value
            // this is very important: it allows to support
            // the parsing of IONEX that omit the exponent
            exponent: -1,
            number_of_maps: 0,
            // 2D by default
            map_dimension: 2,
            mapping: None,
            observables: None,
            description: None,
            elevation_cutoff: 0.0,
            // Standard Earth radius [km]
            base_radius: 6371.0,
            grid: Grid::default(),
            nb_stations: 0,
            nb_satellites: 0,
            dcbs: HashMap::new(),
            reference: RefSystem::default(),
            epoch_of_last_map: Epoch::default(),
            epoch_of_first_map: Epoch::default(),
        }
    }
}

impl HeaderFields {
    /// Formats [HeaderFields] into [BufWriter], in the layout
    /// the header parser reads back.
    pub(crate) fn format<W: Write>(&self, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        if let Some(description) = &self.description {
            for line in Self::wrap_words(description, 60) {
                writeln!(w, "{}", fmt_rinex(&line, "DESCRIPTION"))?;
            }
        }

        writeln!(
            w,
            "{}",
            fmt_rinex(
                &format_ionex_utc(self.epoch_of_first_map),
                "EPOCH OF FIRST MAP"
            )
        )?;

        writeln!(
            w,
            "{}",
            fmt_rinex(
                &format_ionex_utc(self.epoch_of_last_map),
                "EPOCH OF LAST MAP"
            )
        )?;

        writeln!(
            w,
            "{}",
            fmt_rinex(&format!("{:6}", self.number_of_maps), "# OF MAPS IN FILE")
        )?;

        let mapping = match self.mapping {
            Some(MappingFunction::CosZ) => "COSZ",
            Some(MappingFunction::QFac) => "QFAC",
            None => "NONE",
        };

        writeln!(
            w,
            "{}",
            fmt_rinex(&format!("  {}", mapping), "MAPPING FUNCTION")
        )?;

        writeln!(
            w,
            "{}",
            fmt_rinex(
                &format!("{:8.1}", self.elevation_cutoff),
                "ELEVATION CUTOFF"
            )
        )?;

        if let Some(observables) = &self.observables {
            writeln!(w, "{}", fmt_rinex(observables, "OBSERVABLES USED"))?;
        }

        if self.nb_stations > 0 {
            writeln!(
                w,
                "{}",
                fmt_rinex(&format!("{:6}", self.nb_stations), "# OF STATIONS")
            )?;
        }

        if self.nb_satellites > 0 {
            writeln!(
                w,
                "{}",
                fmt_rinex(&format!("{:6}", self.nb_satellites), "# OF SATELLITES")
            )?;
        }

        writeln!(
            w,
            "{}",
            fmt_rinex(&format!("{:8.1}", self.base_radius), "BASE RADIUS")
        )?;

        writeln!(
            w,
            "{}",
            fmt_rinex(&format!("{:6}", self.map_dimension), "MAP DIMENSION")
        )?;

        for (linspace, marker) in [
            (&self.grid.height, "HGT1 / HGT2 / DHGT"),
            (&self.grid.latitude, "LAT1 / LAT2 / DLAT"),
            (&self.grid.longitude, "LON1 / LON2 / DLON"),
        ] {
            writeln!(
                w,
                "{}",
                fmt_rinex(
                    &format!(
                        "  {:6.1}{:6.1}{:6.1}",
                        linspace.start, linspace.end, linspace.spacing
                    ),
                    marker
                )
            )?;
        }

        writeln!(
            w,
            "{}",
            fmt_rinex(&format!("{:6}", self.exponent), "EXPONENT")
        )?;

        Ok(())
    }

    /// Splits text into lines of at most `width` characters,
    /// on blanks so the parser recovers the same words.
    fn wrap_words(text: &str, width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut line = String::new();

        for word in text.split(' ') {
            if !line.is_empty() && line.len() + 1 + word.len() > width {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }

        if !line.is_empty() {
            lines.push(line);
        }

        lines
    }

    /// Copies self and returns with updated number of maps
    pub fn with_number_of_maps(&self, num: usize) -> Self {
        let mut s = self.clone();
        s.number_of_maps = num;
        s
    }

    /// Copies self with given time of first map
    pub fn with_epoch_of_first_map(&self, t: Epoch) -> Self {
        let mut s = self.clone();
        s.epoch_of_first_map = t;
        s
    }

    /// Copies self with given time of last map
    pub fn with_epoch_of_last_map(&self, t: Epoch) -> Self {
        let mut s = self.clone();
        s.epoch_of_last_map = t;
        s
    }

    /// Copies and builds Self with given Reference System
    pub fn with_reference_system(&self, reference: RefSystem) -> Self {
        let mut s = self.clone();
        s.reference = reference;
        s
    }

    /// Copies and sets exponent / scaling to currently use
    pub fn with_exponent(&self, e: i8) -> Self {
        let mut s = self.clone();
        s.exponent = e;
        s
    }

    /// Copies and sets model description
    pub fn with_description(&self, desc: &str) -> Self {
        let mut s = self.clone();
        if let Some(ref mut d) = s.description {
            d.push(' ');
            d.push_str(desc)
        } else {
            s.description = Some(desc.to_string())
        }
        s
    }

    pub fn with_mapping_function(&self, mf: MappingFunction) -> Self {
        let mut s = self.clone();
        s.mapping = Some(mf);
        s
    }

    /// Copies & sets minimum elevation angle used.
    pub fn with_elevation_cutoff(&self, e: f32) -> Self {
        let mut s = self.clone();
        s.elevation_cutoff = e;
        s
    }

    pub fn with_observables(&self, o: &str) -> Self {
        let mut s = self.clone();
        if !o.is_empty() {
            s.observables = Some(o.to_string())
        }
        s
    }

    /// Returns true if this Ionosphere Maps describes
    /// a theoretical model, not measured data
    pub fn is_theoretical_model(&self) -> bool {
        self.observables.is_some()
    }

    /// Copies self and set number of stations
    pub fn with_nb_stations(&self, n: u32) -> Self {
        let mut s = self.clone();
        s.nb_stations = n;
        s
    }

    /// Copies self and set number of satellites
    pub fn with_nb_satellites(&self, n: u32) -> Self {
        let mut s = self.clone();
        s.nb_satellites = n;
        s
    }

    /// Copies & set Base Radius in km
    pub fn with_base_radius(&self, b: f32) -> Self {
        let mut s = self.clone();
        s.base_radius = b;
        s
    }

    pub fn with_map_dimension(&self, d: u8) -> Self {
        let mut s = self.clone();
        s.map_dimension = d;
        s
    }

    /// Adds latitude grid definition
    pub fn with_latitude_grid(&self, grid: Linspace) -> Self {
        let mut s = self.clone();
        s.grid.latitude = grid;
        s
    }

    /// Adds longitude grid definition
    pub fn with_longitude_grid(&self, grid: Linspace) -> Self {
        let mut s = self.clone();
        s.grid.longitude = grid;
        s
    }

    /// Adds altitude grid definition
    pub fn with_altitude_grid(&self, grid: Linspace) -> Self {
        let mut s = self.clone();
        s.grid.height = grid;
        s
    }

    /// Copies & sets Diffenretial Code Bias estimates
    /// for given vehicle
    pub fn with_dcb(&self, src: BiasSource, value: (f64, f64)) -> Self {
        let mut s = self.clone();
        s.dcbs.insert(src, value);
        s
    }
}

#[cfg(feature = "processing")]
impl HeaderFields {
    /// Modifies [HeaderFields] by applying [MaskFilter] with mutable access.
    pub(crate) fn mask_mut(&mut self, f: &MaskFilter) {
        match f.operand {
            MaskOperand::NotEquals => {},
            MaskOperand::Equals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    self.epoch_of_first_map = epoch.to_time_scale(TimeScale::UTC);
                    self.epoch_of_last_map = epoch.to_time_scale(TimeScale::UTC);
                },
                FilterItem::SvItem(svs) => {
                    self.nb_satellites = svs.len() as u32;
                },
                _ => {},
            },
            MaskOperand::GreaterThan | MaskOperand::GreaterEquals => match &f.item {
                FilterItem::EpochItem(t) => {
                    let t_utc = t.to_time_scale(TimeScale::UTC);
                    if self.epoch_of_first_map < t_utc {
                        self.epoch_of_first_map = t_utc;
                    }
                },
                _ => {},
            },
            MaskOperand::LowerThan | MaskOperand::LowerEquals => match &f.item {
                FilterItem::EpochItem(t) => {
                    let t_utc = t.to_time_scale(TimeScale::UTC);
                    if self.epoch_of_last_map > t_utc {
                        self.epoch_of_last_map = t_utc;
                    }
                },
                _ => {},
            },
        }
    }
}
