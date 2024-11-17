//! CRINEX decompression module
use crate::{
    hatanaka::{Error, NumDiff, TextDiff},
    prelude::{Constellation, Observable, SV},
};

use std::{
    collections::HashMap,
    io::{Read, Result as IoResult},
    str::{from_utf8, FromStr},
};

use num_integer::div_ceil;

#[cfg(feature = "log")]
use log::debug;

#[cfg(docsrs)]
use crate::hatanaka::Compressor;

/// [Decompressor] is a structure to decompress CRINEX (compressed compacted RINEX)
/// into readable RINEX. It is scaled to operate according to the historical CRX2RNX tool,
/// which seems to limit itself to M=3 in the compression algorithm.
/// If you want complete control over the decompression algorithm, prefer [DecompressorExpert].
///
/// [Decompressor] implements the CRINEX decompression algorithm, following
/// the specifications written by Y. Hatanaka. Like RINEX, CRINEX (compact) RINEX
/// is a line based format (\n termination), this structures works on a line basis.
///
/// Although [Decompressor] is flexible, it currently does not tolerate critical
/// format issues, specifically:
///  - numsat incorrectly encoded in Epoch description
///  - missing or bad observable specifications
///  - missing or bad constellation specifications
///
/// In this example, we deploy the [Decompressor] over a local file, as an example
/// yet typical usage scenario. We use our [RinexReader] to provide the line Iterator.
/// ```
/// use std::fs::File;
/// use rinex::hatanaka::Decompressor;
///
/// // Working from local files is the typical application,
/// // but [Decompressor] may deploy over any [Read]able interface
/// let mut fd = File::open("../test_resources/CRNX/V1/AJAC3550.21D")
///     .unwrap();
///
/// // This file was compressed using the historical tool, M=5 limit is OK.
/// let decomp = Decompressor::new(fd);
///
/// // Dump this as a new (readable) RINEX
/// let mut total = 0;
/// let mut buf = Vec::<u8>::with_capacity(1024);
/// while let Some(size) = decomp.read(&mut buf) {
///     if size == 0 {
///         break; // EOS reached
///     }
///     total += size;
/// }
///
/// assert_eq!(total, 36); // total bytewise
/// ```
pub type Decompressor = DecompressorExpert<5>;

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum State {
    #[default]
    /// Recovering Epoch descriptor
    Epoch,
    /// Recovering Clock offset
    Clock,
    /// V1 Observations recovering
    ObservationV1,
    /// V3 Observations recovering
    ObservationV3,
}

impl State {
    /// Minimal size of a valid [Epoch] description in V1 revision    
    /// - Timestamp: Year uses 2 digits
    /// - Flag
    /// - Numsat
    const MIN_V1_EPOCH_DESCRIPTION_SIZE: usize = 26 + 3 + 3;

    /// Minimal size of a valid [Epoch] description in V3 revision  
    /// - >
    /// - Timestamp: Year uses 4 digits
    /// - Flag
    /// - Numsat
    const MIN_V3_EPOCH_DESCRIPTION_SIZE: usize = "                   1".len();

    /// Calculates number of bytes this state will forward to user
    fn size_to_produce(&self, v3: bool, numsat: usize, numobs: usize) -> usize {
        match self {
            // Epoch is recovered once Clock is recovered.
            // Because standard format says the clock data should be appended to epoch description
            // (in an inconvenient way, in V1 revision).
            Self::Clock => {
                if v3 {
                    Self::MIN_V3_EPOCH_DESCRIPTION_SIZE
                } else {
                    let mut size = Self::MIN_V1_EPOCH_DESCRIPTION_SIZE;
                    let num_extra = div_ceil(numsat, 12) - 1;
                    size += num_extra * 17; // padding
                    size += numsat * 3; // formatted
                    size
                }
            },
            Self::ObservationV1 => {
                let mut size = 1;
                size += numobs - 1; // separator
                size += 15 * numobs; // formatted
                let num_extra = div_ceil(numobs, 5) - 1;
                size += num_extra * 15; // padding
                size
            },
            Self::ObservationV3 => {
                let size = 3; // SVNN
                size + numobs * 16
            },
            // Other states do not generate any data
            // we need to consume lines to progress to states that actually produce something
            _ => 0,
        }
    }
}

/// [DecompressorExpert] gives you full control over the maximal compression ratio.
/// When decoding, we adapt to the compression ratio applied when the stream was encoded.
/// RNX2CRX is historically limited to M<=3 while 5 is said to be the optimal.
/// With [DecompressorExpert] you can support any value.
/// Keep in mind that CRINEX is not a lossless compression for signal observations.
/// The higher the compression order, the larger the error over the signal observations.
pub struct DecompressorExpert<const M: usize> {
    /// Whether this is a V3 parser or not
    v3: bool,
    /// Internal [State]. Use this to determine
    /// whether we're still inside the Header section (algorithm is not active),
    /// or inside file body (algorithm is active).
    state: State,
    /// For internal logic: remains true until
    /// first [Epoch] description was decoded.
    first_epoch: bool,
    /// pointers
    numsat: usize, // total
    sv_ptr: usize, // inside epoch
    sv: SV,
    /// pointers
    numobs: usize, // total
    obs_ptr: usize, // inside epoch
    /// [TextDiff] that works on entire Epoch line
    epoch_diff: TextDiff,
    /// Epoch descriptor, for single allocation
    epoch_descriptor: String,
    epoch_desc_len: usize, // for internal logic
    /// [TextDiff] for observation flags
    flags_diff: HashMap<SV, TextDiff>,
    /// Clock offset differentiator
    clock_diff: NumDiff<M>,
    /// Observation differentiators
    obs_diff: HashMap<(SV, usize), NumDiff<M>>,
    /// [Observable]s specs for each [Constellation]
    gnss_observables: HashMap<Constellation, Vec<Observable>>,
}

impl<const M: usize> DecompressorExpert<M> {
    /// Minimal timestamp length in V1 revision
    const V1_TIMESTAMP_SIZE: usize = 24;
    const V1_NUMSAT_OFFSET: usize = Self::V1_TIMESTAMP_SIZE + 4;
    const V1_SV_OFFSET: usize = Self::V1_NUMSAT_OFFSET + 3;

    /// Minimal timestamp length in V3 revision
    const V3_TIMESTAMP_SIZE: usize = 26;
    const V3_NUMSAT_OFFSET: usize = Self::V3_TIMESTAMP_SIZE + 1 + 4;
    const V3_SV_OFFSET: usize = Self::V3_NUMSAT_OFFSET + 9;

    /// Returns pointer offset to parse this sv
    fn sv_slice_start(v3: bool, sv_index: usize) -> usize {
        let offset = if v3 {
            Self::V3_SV_OFFSET
        } else {
            Self::V1_SV_OFFSET
        };
        offset + sv_index * 3
    }

    /// Returns next [SV]
    fn next_sv(&self) -> Option<SV> {
        let start = Self::sv_slice_start(self.v3, self.sv_ptr);
        let end = (start + 3).min(self.epoch_desc_len);

        // TODO: this might fail on old rinex single constell that ommit the constellation
        if let Ok(sv) = SV::from_str(&self.epoch_descriptor[start..end].trim()) {
            Some(sv)
        } else {
            None
        }
    }

    /// Macro to directly parse numsat from recovered descriptor
    fn epoch_numsat(&self) -> Option<usize> {
        let start = if self.v3 {
            Self::V3_NUMSAT_OFFSET
        } else {
            Self::V1_NUMSAT_OFFSET
        };

        if let Ok(numsat) = &self.epoch_descriptor[start..start + 3].trim().parse::<u8>() {
            Some(*numsat as usize)
        } else {
            None
        }
    }

    /// Builds new CRINEX decompressor.
    /// Inputs
    /// - v3: whether this CRINEX V1 or V3 content will follow
    /// - constellation: [Constellation] as defined in header
    /// - gnss_observables: [Observable]s per [Constellation] as defined in header.
    pub fn new(v3: bool, gnss_observables: HashMap<Constellation, Vec<Observable>>) -> Self {
        Self {
            v3,
            numsat: 0,
            sv_ptr: 0,
            numobs: 0,
            obs_ptr: 0,
            gnss_observables,
            first_epoch: true,
            epoch_desc_len: 0,
            sv: Default::default(),
            state: Default::default(),
            epoch_diff: TextDiff::new(""),
            obs_diff: HashMap::with_capacity(8), // cannot initialize yet
            flags_diff: HashMap::with_capacity(8), // cannot initialize yet
            epoch_descriptor: String::with_capacity(256),
            clock_diff: NumDiff::<M>::new(0, M),
        }
    }

    /// Decompresses following line and pushes recovered content into buffer.
    /// Inputs
    ///  - line: trimed line (no \n termination), which is consistent with
    /// [LinesIterator].
    /// - len: line.len()
    /// - buf: destination buffer
    /// - size: size available in destination buffer
    /// Returns
    ///  - size: produced size (total bytes recovered).
    /// It is possible that, depending on current state, that several input lines
    /// are needed to recover a new line. Recovered content may span several lines as well,
    /// especially when working with a V1 stream.
    pub fn decompress(
        &mut self,
        line: &str,
        len: usize,
        buf: &mut [u8],
        size: usize,
    ) -> Result<usize, Error> {
        if size
            < self
                .state
                .size_to_produce(self.v3, self.numsat, self.numobs)
        {
            return Err(Error::BufferOverflow);
        }

        match self.state {
            State::Epoch => self.run_epoch(line, len),
            State::Clock => self.run_clock(line, len, buf),
            State::ObservationV1 => self.run_observation_v1(line, len, buf, size),
            State::ObservationV3 => self.run_observation_v3(line, len, buf),
        }
    }

    /// Process following line, in [State::Epoch]
    fn run_epoch(&mut self, line: &str, len: usize) -> Result<usize, Error> {
        let min_len = if self.v3 {
            State::MIN_V3_EPOCH_DESCRIPTION_SIZE
        } else {
            State::MIN_V1_EPOCH_DESCRIPTION_SIZE
        };

        if len < min_len {
            return Err(Error::EpochFormat);
        }

        if line.starts_with('&') {
            if self.v3 {
                return Err(Error::BadV3Format);
            }

            self.epoch_diff.force_init(&line[1..]);
            self.epoch_descriptor = line[1..].to_string();
            self.epoch_desc_len = len - 1;
        } else if line.starts_with('>') {
            if !self.v3 {
                return Err(Error::BadV1Format);
            }

            self.epoch_diff.force_init(&line[1..]);
            self.epoch_descriptor = line[1..].to_string();
            self.epoch_desc_len = len - 1;
        } else {
            self.epoch_descriptor = self.epoch_diff.decompress(&line[1..]).to_string();
            self.epoch_desc_len = self.epoch_descriptor.len();
        }

        println!(
            "RECOVERED \"{}\" [{}]",
            self.epoch_descriptor, self.epoch_desc_len
        );

        self.state = State::Clock;
        Ok(0)
    }

    /// Fills user buffer with recovered epoch, following either V1 or V3 standards
    fn format_epoch(&self, clock_data: Option<i64>, buf: &mut [u8]) -> usize {
        if self.v3 {
            self.format_epoch_v3(clock_data, buf)
        } else {
            self.format_epoch_v1(clock_data, buf)
        }
    }

    /// Fills user buffer with recovered epoch, following V3 standards
    fn format_epoch_v3(&self, clock_data: Option<i64>, buf: &mut [u8]) -> usize {
        // V3 format is much simpler
        // all we need to do is extract SV `XXY` to append in each following lines

        let mut produced = 0;
        buf[produced] = b'>'; // special marker
        produced += 1;

        let bytes = self.epoch_descriptor.as_bytes();

        // push timestamp +flag
        buf[produced..produced + 34].copy_from_slice(&bytes[..34]);
        produced += 34;

        // provide clock data, if any
        if let Some(clock_data) = clock_data {
            let value = clock_data as f64 / 1000.0;
            let formatted = format!("       {:.12}", value);
            let fmt_len = formatted.len(); // TODO improve: this is constant
            let bytes = formatted.as_bytes();
            buf[produced..produced + fmt_len].copy_from_slice(&bytes);
            produced += fmt_len; // TODO improve: this is constant
        }

        produced
    }

    /// Fills user buffer with recovered epoch, following V1 standards
    fn format_epoch_v1(&self, clock_data: Option<i64>, buf: &mut [u8]) -> usize {
        let mut produced = 0;

        buf[produced] = b' '; // single whitespace
        produced += 1;

        let bytes = self.epoch_descriptor.as_bytes();

        // push first line (up to 68 bytes)
        let first_len = self.epoch_desc_len.min(67);

        buf[produced..produced + first_len].copy_from_slice(&bytes[..first_len]);

        produced += first_len;

        buf[produced] = b'\n'; // conclude 1st line
        produced += 1;

        // construct all following lines we need,
        // with "standard" padding
        let nb_extra = self.epoch_desc_len / 68;
        for i in 0..nb_extra {
            // extra padding
            buf[produced..produced + 32].copy_from_slice(&[
                b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ',
                b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ',
                b' ', b' ', b' ', b' ',
            ]);

            produced += 32;

            // copy data slice
            let start = (i + 1) * 67;
            let end = (start + 68).min(self.epoch_desc_len);
            let size = end - start;

            buf[produced..produced + size].copy_from_slice(&bytes[start..end]);

            produced += size;

            // terminate this line
            buf[produced] = b'\n';
            produced += 1;
        }

        produced
    }

    /// Process following line, in [State::Clock]
    fn run_clock(&mut self, line: &str, len: usize, buf: &mut [u8]) -> Result<usize, Error> {
        let mut clock_data = Option::<i64>::None;

        // attempts to recover clock data (if it exists)
        if len > 2 {
            if line[1..].starts_with('&') {
                if let Ok(order) = line[..1].parse::<usize>() {
                    if let Ok(val) = line[2..].parse::<i64>() {
                        // valid kernel reset
                        self.clock_diff.force_init(val, order);
                        clock_data = Some(val);
                    }
                }
            }
        }
        if len == 1 {
            // highly compressed clock data
            match line.trim().parse::<i64>() {
                Ok(val) => {
                    let val = self.clock_diff.decompress(val);
                    clock_data = Some(val);
                },
                Err(_) => {},
            }
        }

        // now that we have potentially recovered clock data
        // we can format the complete epoch description
        let produced = self.format_epoch(clock_data, buf);

        // prepare for observation state
        self.obs_ptr = 0;
        self.sv_ptr = 0;
        self.first_epoch = false;

        self.numsat = self.epoch_numsat().expect("bad recovered content (numsat)");

        // grab first sv
        self.sv = self.next_sv().expect("bad recovered content (sv)");

        // cross check recovered content
        // &, at the same time, make sure we are ready to process any new SV
        for i in 0..self.numsat {
            let start = Self::sv_slice_start(self.v3, i);

            // any invalid SV description, will cause us to wait for a new epoch.
            // In other terms, epoch is fully disregarded.
            let sv = SV::from_str(&self.epoch_descriptor[start..start + 3])
                .map_err(|_| Error::SVParsing)?;

            // initialize on first encounter
            if self.flags_diff.get(&sv).is_none() {
                let textdiff = TextDiff::new("");
                self.flags_diff.insert(sv, textdiff);
            }
        }

        let obs = self
            .get_observables(&self.sv.constellation)
            .expect("failed to determine sv definition");

        self.numobs = obs.len();

        if self.v3 {
            self.state = State::ObservationV3
        } else {
            self.state = State::ObservationV1
        }

        Ok(produced)
    }

    /// Process following line, in [State::ObservationV1]
    fn run_observation_v1(
        &mut self,
        line: &str,
        len: usize,
        buf: &mut [u8],
        size: usize,
    ) -> Result<usize, Error> {
        panic!("obs_v1: not yet");
    }

    //         State::ObservationV2 => {
    //             let mut early_termination = false;

    //             if let Some(eol_offset) = ascii.find('\n') {
    //                 if eol_offset < ascii_len -1 {
    //                     // we grabbed part of the following line
    //                     // this happens in case all data flags remain identical (100% compression factor)
    //                     // we must postpone part of this buffer
    //                     ascii_len = eol_offset;
    //                     consumed = eol_offset;
    //                     early_termination = true;
    //                 }
    //             }

    //             // default is BLANK
    //             let mut formatted = "                ".to_string();

    //             // Decoding attempt
    //             if ascii_len > 0 {
    //                 let mut kernel_reset = false;

    //                 if ascii_len > 2 {
    //                     if ascii[1..2].eq("&") {
    //                         kernel_reset = false;

    //                         let order = ascii[0..1]
    //                             .parse::<usize>()
    //                             .expect("bad crinex compression level");

    //                     }
    //                 }

    //                 if !kernel_reset {
    //                     // regular compression
    //                     if let Ok(val) = ascii[..ascii_len].trim().parse::<i64>() {
    //                         let val = if let Some(kernel) =
    //                             self.obs_diff.get_mut(&(self.sv, self.obs_ptr))
    //                         {
    //                             kernel.decompress(val) as f64 / 1.0E3
    //                         } else {
    //                             let kernel = NumDiff::new(val, M);
    //                             self.obs_diff.insert((self.sv, self.obs_ptr), kernel);
    //                             val as f64 / 1.0E3
    //                         };
    //                         formatted = format!("{:14.3}  ", val);
    //                     }
    //                 }
    //             }

    //             // copy to user
    //             let mut ptr = self.obs_ptr * 16;
    //             println!("fmt_len={}", formatted.as_bytes().len());

    //             // push decompressed content
    //             user_buf[user_ptr + ptr..user_ptr + ptr + 16].copy_from_slice(formatted.as_bytes());
    //             ptr += 16;
    //             self.obs_ptr += 1;

    //             // v2 case: need to wrapp into several lines....
    //             if self.obs_ptr % 5 == 0 {
    //                 // line wrapping
    //                 user_buf[user_ptr + ptr] = b'\n';
    //                 ptr += 1;

    //                 // special V2 start of line padding ...
    //                 user_buf[user_ptr + ptr..user_ptr + ptr + 15].copy_from_slice(&[
    //                     b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ',
    //                     b' ', b' ', b' ',
    //                 ]);

    //                 ptr += 15;
    //             }

    //             println!("obs={}/{}", self.obs_ptr, self.numobs);
    //             // End of this observation: we have three cases
    //             //  1. weird case where line is early terminated
    //             //     due to the combination of Blanking (missing signals) and all flags being fully compressed.
    //             //     In this scenario, we enter special case to push the necessary blanking
    //             //     and re-use the past flag description (100% compression)
    //             //  2. this is a regular BLANK, we need to determine whether this terminates
    //             //     the observation serie or not
    //             //  3. regular observation: need to progress to the next one
    //             if early_termination {
    //                 // Early termination case
    //                 next_state = State::EarlyTerminationV2;
    //             } else {
    //                 if ascii_len == 0 {
    //                     // BLANKING case
    //                     if self.obs_ptr == self.numobs {
    //                         // Last blanking
    //                         self.obs_ptr = 0;
    //                         next_state = State::Flags;
    //                     } else {
    //                         // regular progression in case of blanking
    //                         next_state = State::ObservationV2;
    //                     }
    //                 } else {
    //                     // regular progression
    //                     next_state = State::ObservationSeparator;
    //                 }
    //             }
    //         },

    /// Process following line, in [State::ObservationV3]
    fn run_observation_v3(
        &mut self,
        line: &str,
        len: usize,
        buf: &mut [u8],
    ) -> Result<usize, Error> {
        println!("[{}] LINE \"{}\"", self.sv, line);

        let mut produced = 0;
        let mut new_state = self.state;

        // prepend SVNN identity
        let start = Self::sv_slice_start(true, self.sv_ptr);
        let end = (start + 3).min(self.epoch_desc_len);
        let bytes = self.epoch_descriptor.as_bytes();
        buf[..3].copy_from_slice(&bytes[start..end]);

        produced += 3;

        let mut consumed = 0;

        // Retrieving observations is complex.
        // When signal sampling was not feasible: data is omitted (blanked) by a single '_'
        // Which is not particularly clever and means the data flags can only provided at the end of the line.
        // Since data flags are text compressed, it can create some weird situations.
        for ptr in 0..self.numobs {
            // We must output something for each expected data points (whatever happens).
            // So we default to a BLANK, which simplifies the following code:
            // we only implement successful cases.
            let mut formatted = "                ".to_string();

            // Try to locate next '_', which is either
            //  . normal/expected line progression
            //  . or a blanking
            let offset = line[consumed..].find(' ');

            if let Some(offset) = offset {
                if offset > 1 {
                    // observation (made of at least two digits)
                    // Determine whether this is a kernel reset or compression continuation

                    let slice = line[consumed..consumed + offset].trim();
                    println!("slice \"{}\" [{}/{}]", &slice, ptr + 1, self.numobs);

                    if let Some(offset) = slice.find('&') {
                        if offset == 1 {
                            // valid core reset pattern
                            if let Ok(level) = slice[..offset].parse::<usize>() {
                                if let Ok(value) = slice[offset + 1..].parse::<i64>() {
                                    if let Some(kernel) = self.obs_diff.get_mut(&(self.sv, ptr)) {
                                        kernel.force_init(value, level);
                                    } else {
                                        let kernel = NumDiff::<M>::new(value, level);
                                        self.obs_diff.insert((self.sv, ptr), kernel);
                                    }
                                    formatted = format!("{:14.3}  ", value as f64 / 1000.0);
                                }
                            }
                        }
                        // non valid core reset patterns:
                        // we output a BLANK
                    } else {
                        // compressed data case
                        if let Ok(value) = slice.parse::<i64>() {
                            if let Some(kernel) = self.obs_diff.get_mut(&(self.sv, ptr)) {
                                let value = kernel.decompress(value);
                                let value = value as f64 / 1000.0;
                                formatted = format!("{:14.3}  ", value).to_string();
                            }
                        }
                    }

                    consumed += offset + 1; // consume until this point
                } else {
                    // this is either BLANK or highly compressed (=single digit value)
                    let slice = line[consumed..consumed + offset].trim();

                    if slice.len() > 0 {
                        // this is a digit (highly compressed value)
                        println!("slice \"{}\" [{}/{}]", &slice, ptr + 1, self.numobs);
                        if let Ok(value) = slice.parse::<i64>() {
                            if let Some(kernel) = self.obs_diff.get_mut(&(self.sv, ptr)) {
                                let value = kernel.decompress(value);
                                let value = value as f64 / 1000.0;
                                formatted = format!("{:14.3}  ", value).to_string();
                            }
                        }
                        consumed += 2; // consume this byte
                    } else {
                        consumed += 1;
                    }
                }

                let fmt_len = formatted.len(); // TODO: improve, this is constant
                let bytes = formatted.as_bytes();

                // push into user
                buf[produced..produced + fmt_len].copy_from_slice(&bytes);
                produced += fmt_len;

                // this may cause trimed lines to panic on next '_' search
                // so we exist the loop so we do not overflow
                if consumed >= len {
                    break;
                }
            } else {
                // early line termination.
                // This happens when last observations are all missing (possibly more than one)
                // and observation "flags" are all fully compressed (100% compression factor)

                // if we have leftovers, that means we have one last observation
                if len > consumed {
                    let mut formatted = "                ".to_string();

                    // grab slice
                    let slice = line[consumed..].trim();
                    println!("slice \"{}\" [{}/{}]", &slice, ptr + 1, self.numobs);

                    if let Some(offset) = slice.find('&') {
                        if offset == 1 {
                            // valid core reset pattern
                            if let Ok(level) = slice[..offset].parse::<usize>() {
                                if let Ok(value) = slice[offset + 1..].parse::<i64>() {
                                    if let Some(kernel) = self.obs_diff.get_mut(&(self.sv, ptr)) {
                                        kernel.force_init(value, level);
                                    } else {
                                        let kernel = NumDiff::<M>::new(value, level);
                                        self.obs_diff.insert((self.sv, ptr), kernel);
                                    }
                                    formatted = format!("{:14.3}  ", value as f64 / 1000.0);
                                }
                            }
                        }
                        // non valid core reset patterns:
                        // we output a BLANK
                    } else {
                        // compressed data case
                        if let Ok(value) = slice.parse::<i64>() {
                            if let Some(kernel) = self.obs_diff.get_mut(&(self.sv, ptr)) {
                                let value = kernel.decompress(value);
                                let value = value as f64 / 1000.0;
                                formatted = format!("{:14.3}  ", value).to_string();
                            }
                        }
                    }

                    let fmt_len = formatted.len(); // TODO: improve, this is constant
                    let bytes = formatted.as_bytes();

                    // push into user
                    buf[produced..produced + fmt_len].copy_from_slice(&bytes);
                    produced += fmt_len;
                }

                // 1. we need to push all required BLANKING
                // 2. we need to preserve data flags
                // 3. and finally conclude this SV
                let nb_missing = self.numobs - ptr - 1;

                let formatted = "                ".to_string();
                let fmt_len = formatted.len(); // IMPROVE: this is constant
                let bytes = formatted.as_bytes();

                // fill required nb of blanks
                for _ in 0..nb_missing {
                    // push into user
                    buf[produced..produced + fmt_len].copy_from_slice(&bytes);
                    produced += fmt_len;
                }

                // trick to preserve data flags
                let textdiff = self
                    .flags_diff
                    .get_mut(&self.sv)
                    .expect("internal error: bad crinex content?");

                let descriptor = textdiff.decompress("");
                let flags_len = descriptor.len();
                let bytes = descriptor.as_bytes();

                println!("PRESERVED \"{}\"", descriptor);

                // copy all flags to user
                let mut offset = 17;
                for i in 0..self.numobs {
                    let lli_idx = i * 2;
                    if flags_len > lli_idx {
                        if !descriptor[lli_idx..lli_idx + 1].eq(" ") {
                            buf[offset] = bytes[i * 2]; // b'x';
                        }
                    }

                    let snr_idx = lli_idx + 1;
                    if flags_len > snr_idx {
                        if !descriptor[snr_idx..snr_idx + 1].eq(" ") {
                            buf[offset + 1] = bytes[(i * 2) + 1]; // b'y';
                        }
                    }

                    offset += 16;
                }

                // conclude this SV
                self.sv_ptr += 1;

                println!("[{} CONCLUDED {}/{}]", self.sv, self.sv_ptr, self.numsat);

                if self.sv_ptr == self.numsat {
                    // end of epoch
                    println!("[END OF EPOCH]");
                    self.state = State::Epoch;
                } else {
                    self.sv = self.next_sv().expect("failed to determine next sv");

                    let constellation = if self.sv.constellation.is_sbas() {
                        Constellation::SBAS
                    } else {
                        self.sv.constellation
                    };

                    self.numobs = self
                        .get_observables(&constellation)
                        .expect("internal error")
                        .len();

                    self.state = State::ObservationV3;
                }

                return Ok(produced);
            }
        } // for

        // at this point, we should be left with "data flags" in the buffer.
        // That may not be the case. We may have consumed everything when
        // the line is trimed and flags should be preserved

        if consumed < len {
            // proceed to flags recovering
            let flags = &line[consumed..];
            println!("FLAGS \"{}\"", flags);

            let kernel = self.flags_diff.get_mut(&self.sv).expect("internal error");

            let descriptor = kernel.decompress(flags);
            let flags_len = descriptor.len();
            let bytes = descriptor.as_bytes();

            println!("RECOVERED \"{}\"", descriptor);

            // copy all flags to user
            let mut offset = 17;
            for i in 0..self.numobs {
                let lli_idx = i * 2;
                if flags_len > lli_idx {
                    if !descriptor[lli_idx..lli_idx + 1].eq(" ") {
                        buf[offset] = bytes[i * 2]; // b'x';
                    }
                }

                let snr_idx = lli_idx + 1;
                if flags_len > snr_idx {
                    if !descriptor[snr_idx..snr_idx + 1].eq(" ") {
                        buf[offset + 1] = bytes[(i * 2) + 1]; // b'y';
                    }
                }

                offset += 16;
            }
        }

        self.obs_ptr = 0;

        // move on to next
        self.sv_ptr += 1;
        println!("[{} CONCLUDED {}/{}]", self.sv, self.sv_ptr, self.numsat);

        if self.sv_ptr == self.numsat {
            // end of epoch
            println!("[END OF EPOCH]");
            new_state = State::Epoch;
        } else {
            self.sv = self.next_sv().expect("failed to determine next sv");

            let constellation = if self.sv.constellation.is_sbas() {
                Constellation::SBAS
            } else {
                self.sv.constellation
            };

            self.numobs = self
                .get_observables(&constellation)
                .expect("internal error")
                .len();
        }

        self.state = new_state;
        Ok(produced)
    }

    /// Helper to retrieve observable for given system
    fn get_observables(&self, constell: &Constellation) -> Option<&Vec<Observable>> {
        // We use mixed to store a single value for single definitions
        if let Some(mixed) = self.gnss_observables.get(&Constellation::Mixed) {
            Some(mixed)
        } else {
            self.gnss_observables.get(constell)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        hatanaka::decompressor::{Decompressor, State},
        prelude::SV,
    };
    use std::str::FromStr;

    #[test]
    fn epoch_size_to_produce_v1() {
        for (numsat, expected) in [
            (
                9,
                " 17  1  1  3 33 40.0000000  0  9G30G27G11G16G 8G 7G23G 9G 1",
            ),
            (
                10,
                " 17  1  1  0  0  0.0000000  0 10G31G27G 3G32G16G 8G14G23G22G26",
            ),
            (
                11,
                " 17  1  1  0  0  0.0000000  0 11G31G27G 3G32G16G 8G14G23G22G26G27",
            ),
            (
                12,
                " 17  1  1  0  0  0.0000000  0 12G31G27G 3G32G16G 8G14G23G22G26G27G28",
            ),
            (
                13,
                " 21 01 01 00 00 00.0000000  0 13G07G08G10G13G15G16G18G20G21G23G26G27
                G29",
            ),
            (
                14,
                " 21 01 01 00 00 00.0000000  0 14G07G08G10G13G15G16G18G20G21G23G26G27
                G29G30",
            ),
            (
                24,
                " 21 12 21  0  0  0.0000000  0 24G07G08G10G16G18G21G23G26G32R04R05R10
                R12R19R20R21E04E11E12E19E24E25E31E33",
            ),
            (
                25,
                " 21 12 21  0  0  0.0000000  0 25G07G08G10G16G18G21G23G26G32R04R05R10
                R12R19R20R21E04E11E12E19E24E25E31E33
                S23",
            ),
            (
                26,
                " 21 12 21  0  0  0.0000000  0 26G07G08G10G16G18G21G23G26G32R04R05R10
                R12R19R20R21E04E11E12E19E24E25E31E33
                S23S36",
            ),
        ] {
            let size = State::Epoch.size_to_produce(false, numsat, 0);
            assert_eq!(size, 0); // Should wait for Clock data !

            let size = State::Clock.size_to_produce(false, numsat, 0);
            assert_eq!(size, expected.len(), "failed for \"{}\"", expected);
        }
    }

    #[test]
    fn data_size_to_produce_v1() {
        for (numobs, expected) in [
            (1, " 110158976.908 8"),
            (2, " 110158976.908 8  85838153.10248"),
            (3, " 110158976.908 8  85838153.10248  20962551.380  "),
            (
                4,
                " 119147697.073 7  92670417.710 7  22249990.480    22249983.480  ",
            ),
            (
                5,
                "  24017462.340       -3054.209       -2379.903          43.650          41.600  ",
            ),
            (
                6,
                "  24017462.340       -3054.209       -2379.903          43.650          41.600  
                25509828.140  ",
            ),
            (
                9,
                "  24017462.340       -3054.209       -2379.903          43.650          41.600  
                25509828.140        2836.327        2210.128          41.600  ",
            ),
            (
                10,
                "  24017462.340       -3054.209       -2379.903          43.650          41.600  
                25509828.140        2836.327        2210.128          41.600          41.650  ",
            ),
            (
                14,
                "  24017462.340       -3054.209       -2379.903          43.650          41.600  
                25509828.140        2836.327        2210.128          41.600          41.650  
               100106048.706 6  25509827.540        2118.232          39.550  ",
            ),
        ] {
            let size = State::ObservationV1.size_to_produce(false, 0, numobs);
            assert_eq!(size, expected.len(), "failed for \"{}\"", expected);
        }
    }

    #[test]
    fn epoch_size_to_produce_v3() {
        for (numsat, expected) in [
            (18, "> 2022 03 04 00 00  0.0000000  0 18"),
            (22, "> 2022 03 04 00 00  0.0000000  0 22"),
        ] {
            let size = State::Epoch.size_to_produce(false, numsat, 0);
            assert_eq!(size, 0); // Should wait for Clock data !

            let size = State::Clock.size_to_produce(true, numsat, 0);
            assert_eq!(size, expected.len(), "failed for \"{}\"", expected);
        }
    }

    #[test]
    fn data_size_to_produce_v3() {
        for (numobs, expected) in [
            (1, "G01  20243517.560  "),
            (2, "G03  20619020.680   108353702.79708"),
            (4, "R10  22432243.520   119576492.91607      1307.754          43.250  "),
            (8, "R17  20915624.780   111923741.34508      1970.309          49.000    20915629.120    87051816.58507      1532.457          46.500  "),
        ] {
            let size = State::ObservationV3.size_to_produce(true, 0, numobs);
            assert_eq!(size, expected.len(), "failed for \"{}\"", expected);
        }
    }

    #[test]
    fn v1_sv_slice() {
        let recovered = "21 01 01 00 00 00.0000000  0 24G07G08G10G13G15G16G18G20G21G23G26G27G30R01R02R03R08R09R15R16R17R18R19R20";
        for sv_index in 0..24 {
            let start = Decompressor::sv_slice_start(false, sv_index);
            let slice_str = &recovered[start..start + 3];
            if sv_index == 0 {
                assert_eq!(slice_str, "G07");
            } else if sv_index == 1 {
                assert_eq!(slice_str, "G08");
            }
            let _ = SV::from_str(slice_str.trim()).unwrap();
        }
    }

    #[test]
    fn v1_numsat_slice() {
        let recovered = "21 01 01 00 00 00.0000000  0 24G07G08G10G13G15G16G18G20G21G23G26G27G30R01R02R03R08R09R15R16R17R18R19R20";
        let offset = Decompressor::V1_NUMSAT_OFFSET;
        let numsat_str = &recovered[offset..offset + 3];
        assert_eq!(numsat_str, " 24");
        let numsat = numsat_str.trim().parse::<u64>().unwrap();
        assert_eq!(numsat, 24);
    }

    #[test]
    fn v3_sv_slice() {
        let recovered = " 2020 06 25 00 00 00.0000000  0 43      C05C07C10C12C19C20C23C32C34C37E01E03E05E09E13E15E24E31G02G05G07G08G09G13G15G18G21G27G28G30R01R02R08R09R10R11R12R17R18R19S23S25S36";
        for sv_index in 0..43 {
            let start = Decompressor::sv_slice_start(true, sv_index);
            let slice_str = &recovered[start..start + 3];
            if sv_index == 0 {
                assert_eq!(slice_str, "C05");
            } else if sv_index == 1 {
                assert_eq!(slice_str, "C07");
            }
            let _ = SV::from_str(slice_str.trim()).unwrap();
        }
    }

    #[test]
    fn v3_numsat_slice() {
        let recovered = " 2020 06 25 00 00 00.0000000  0 43      C05C07C10C12C19C20C23C32C34C37E01E03E05E09E13E15E24E31G02G05G07G08G09G13G15G18G21G27G28G30R01R02R08R09R10R11R12R17R18R19S23S25S36";
        let offset = Decompressor::V3_NUMSAT_OFFSET;
        let numsat_str = &recovered[offset..offset + 3];
        assert_eq!(numsat_str, " 43");
        let numsat = numsat_str.trim().parse::<u64>().unwrap();
        assert_eq!(numsat, 43);
    }
}
