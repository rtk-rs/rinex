//! RINEX compression module (RNX2CRX operation)

use std::{collections::HashMap, fmt::Write as FmtWrite, io::Write};

use crate::{
    epoch::format as epoch_format,
    error::FormattingError,
    hatanaka::{NumDiff, TextDiff},
    observation::{HeaderFields, Record},
    prelude::{Constellation, RinexType, SV},
    BufWriter,
};

use itertools::Itertools;

/// [Compressor] limited to the compression orders the historical
/// CRX2RNX tool can decompress.
pub type Compressor = CompressorExpert<5>;

/// Compression order applied to every numerical field,
/// which is what the historical RNX2CRX tool uses.
const ORDER: usize = 3;

/// Compression state of one vehicle
struct SvState<const M: usize> {
    /// LLI/SNR flags text kernel
    flags: TextDiff,
    /// One kernel per observable, None while the observation is missing
    kernels: Vec<Option<NumDiff<M>>>,
}

/// [CompressorExpert] turns an Observation [Record] into CRINEX content.
/// M is the maximal compression order supported by the kernels.
pub struct CompressorExpert<const M: usize> {
    /// True until the first epoch has been published
    first_epoch: bool,
    /// Epoch description [TextDiff]
    epoch_diff: TextDiff,
    /// Receiver clock offset kernel, None while no offset is being reported
    clock_diff: Option<NumDiff<M>>,
    /// State of the vehicles present in the previous epoch
    sv_states: HashMap<SV, SvState<M>>,
    /// Epoch description being compressed
    epoch_buf: String,
    /// Observation line being compressed
    line_buf: String,
    /// LLI/SNR flags of the observation line being compressed
    flags_buf: String,
}

impl<const M: usize> Default for CompressorExpert<M> {
    fn default() -> Self {
        Self {
            first_epoch: true,
            epoch_diff: TextDiff::new(""),
            clock_diff: None,
            sv_states: HashMap::with_capacity(64),
            epoch_buf: String::with_capacity(256),
            line_buf: String::with_capacity(256),
            flags_buf: String::with_capacity(64),
        }
    }
}

impl<const M: usize> CompressorExpert<M> {
    /// Format [Record] using mutable [CompressorExpert].
    /// Compressed bytes are dumped in mutable [BufWriter].
    /// This permits the RNX2CRX compression ops.
    pub fn format<W: Write>(
        &mut self,
        w: &mut BufWriter<W>,
        record: &Record,
        header: &HeaderFields,
    ) -> Result<(), FormattingError> {
        // CRINEX 1 is used for RINEX 2, CRINEX 3 for RINEX 3 and above
        let v3 = header
            .crinex
            .as_ref()
            .map(|crinex| crinex.version.major > 1)
            .unwrap_or(true);

        // RINEX 3 formats the clock offset as F15.12, RINEX 2 as F12.9:
        // the CRINEX integer is the offset with the decimal point removed
        let clock_scaling = if v3 { 1.0E12 } else { 1.0E9 };

        for (k, v) in record.iter() {
            // form unique SV list
            let svnn = v
                .signals
                .iter()
                .map(|sig| sig.sv)
                .unique()
                .collect::<Vec<_>>();

            // 1. epoch description
            self.epoch_buf.clear();

            if v3 {
                write!(
                    self.epoch_buf,
                    "> {}  {} {:2}      ",
                    epoch_format(k.epoch, RinexType::ObservationData, 3),
                    k.flag,
                    svnn.len(),
                )?;
            } else {
                write!(
                    self.epoch_buf,
                    " {}  {} {:2}",
                    epoch_format(k.epoch, RinexType::ObservationData, 2),
                    k.flag,
                    svnn.len(),
                )?;
            }

            for sv in svnn.iter() {
                write!(self.epoch_buf, "{:x}", sv)?;
            }

            if self.first_epoch {
                // kernel initialization: the description is published as is,
                // introduced by '&' (CRINEX 1) or '>' (CRINEX 3)
                let marker = if v3 { '>' } else { '&' };
                writeln!(w, "{}{}", marker, self.epoch_buf[1..].trim_end())?;
                self.epoch_diff.force_init(&self.epoch_buf[1..]);
                self.first_epoch = false;
            } else {
                let compressed = self.epoch_diff.compress(&self.epoch_buf[1..]);
                writeln!(w, " {}", compressed.trim_end())?;
            }

            // 2. receiver clock offset
            match v.clock {
                Some(clock) => {
                    let value = (clock.offset_s * clock_scaling).round() as i64;
                    match &mut self.clock_diff {
                        Some(kernel) => {
                            writeln!(w, "{}", kernel.compress(value)?)?;
                        },
                        None => {
                            writeln!(w, "{}&{}", ORDER, value)?;
                            self.clock_diff = Some(NumDiff::<M>::new(value, ORDER));
                        },
                    }
                },
                None => {
                    // kernel reinitialized on next offset
                    writeln!(w)?;
                    self.clock_diff = None;
                },
            }

            // 3. observations, one line per SV
            for sv in svnn.iter() {
                let constellation = if sv.constellation.is_sbas() {
                    Constellation::SBAS
                } else {
                    sv.constellation
                };

                let observables = header
                    .codes
                    .get(&constellation)
                    .ok_or(FormattingError::MissingObservableDefinition)?;

                // vehicles that were not present in the previous epoch
                // have their kernels reinitialized
                let reinit = !self.sv_states.contains_key(sv);

                let state = self.sv_states.entry(*sv).or_insert_with(|| SvState {
                    flags: TextDiff::new(""),
                    kernels: Vec::with_capacity(observables.len()),
                });

                state.kernels.resize_with(observables.len(), || None);

                self.line_buf.clear();
                self.flags_buf.clear();

                for (nth, observable) in observables.iter().enumerate() {
                    if nth > 0 {
                        self.line_buf.push(' ');
                    }

                    let signal = v
                        .signals
                        .iter()
                        .find(|sig| sig.sv == *sv && &sig.observable == observable);

                    match signal {
                        Some(signal) => {
                            let value = (signal.value * 1000.0).round() as i64;

                            match &mut state.kernels[nth] {
                                Some(kernel) => {
                                    write!(self.line_buf, "{}", kernel.compress(value)?)?;
                                },
                                None => {
                                    write!(self.line_buf, "{}&{}", ORDER, value)?;
                                    state.kernels[nth] = Some(NumDiff::<M>::new(value, ORDER));
                                },
                            }

                            match signal.lli {
                                Some(lli) => write!(self.flags_buf, "{:x}", lli)?,
                                None => self.flags_buf.push(' '),
                            }

                            match signal.snr {
                                Some(snr) => write!(self.flags_buf, "{:x}", snr)?,
                                None => self.flags_buf.push(' '),
                            }
                        },
                        None => {
                            // missing observation: kernel reinitialized on next sample
                            state.kernels[nth] = None;
                            self.flags_buf.push_str("  ");
                        },
                    }
                }

                // LLI/SNR flags follow the last observation
                self.line_buf.push(' ');

                if reinit {
                    // published in full, blanks made explicit like RNX2CRX does
                    state.flags.force_init(&self.flags_buf);
                    for byte in self.flags_buf.bytes() {
                        self.line_buf
                            .push(if byte == b' ' { '&' } else { byte as char });
                    }
                } else {
                    let compressed = state.flags.compress(&self.flags_buf);
                    self.line_buf.push_str(compressed);
                }

                writeln!(w, "{}", self.line_buf.trim_end())?;
            }

            // vehicles missing from this epoch are reinitialized on their return
            self.sv_states.retain(|sv, _| svnn.contains(sv));
        }

        Ok(())
    }
}
