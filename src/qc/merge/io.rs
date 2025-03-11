use std::io::{Read, BufReader, Write, BufWriter};

#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;

#[derive(Debug, Copy, Clone)]
enum State {
    InsideHeader,
}

/// [RinexMergeIO] is an efficient structure to merge two streams
/// into one without data interpretation.
pub struct RinexMergeIO<R: Read, W: Write> {
    src_a: BufReader<R>,
    src_b: BufReader<R>,
    dest: BufWriter<W>,
    state: State,
}

impl<R: Read> RinexMergeIO<R> {

    /// Builds a new [RinexMergeIO] to merge [Read]able source into [dest]
    pub fn new(src_a: Read, src_b: Read) -> Self {
        Self {
            src_a: BufReader::new(src_a),
            src_b: BufReader::new(src_b),
            dest: BufReader::new(dest)

        }
    }
    
    /// Builds a new [RinexMergeIO] to merge [Read]able source into [dest]
    #[cfg(feature = "flate2")]
    pub fn new_gzip(src: Read, dest: Read) -> Self {
        let src = GzDecoder::new(src);
        let dest = 
        Self {
            src: BufReader::new(src),
            dest: BufReader::new(dest)

        }
    }
}
