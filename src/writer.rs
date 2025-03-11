//! Generic Buffered Writer, for efficient record production,
//! with integrated optionnal .gz compression
use std::io::{BufWriter, Error, Write}; // Seek, SeekFrom};

#[cfg(feature = "flate2")]
use flate2::{write::GzEncoder, Compression};

/// [BufferedWriter] is an Output abstraction to propose seamless
/// Gzip, Readable and Hatanaka compressed content streaming.
#[derive(Debug)]
pub enum BufferedWriter<W: Write> {
    /// Readable stream
    Plain(BufWriter<W>),
    /// Gzip compressed stream (non readable)
    #[cfg(feature = "flate2")]
    Gz(BufWriter<GzEncoder<W>>),
}

impl<W: Write> BufferedWriter<W> {
    /// Creates new Readable [BufferedWriter]
    pub fn plain(w: W) -> Self {
        Self::Plain(BufWriter::new(w))
    }
    #[cfg(feature = "flate2")]
    /// Creates new [BufferedWriter] to streamed gzip encoded content with
    /// desired compression level. The higher the order, the lower the performance.
    pub fn gzip(w: W, compression_level: u32) -> Self {
        Self::Gz(BufWriter::new(GzEncoder::new(
            w,
            Compression::new(compression_level),
        )))
    }
}

impl<W: Write> Write for BufferedWriter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        match self {
            Self::Plain(ref mut writer) => writer.write(buf),
            #[cfg(feature = "flate2")]
            Self::Gz(ref mut writer) => writer.write(buf),
        }
    }
    fn flush(&mut self) -> Result<(), Error> {
        match self {
            Self::Plain(ref mut writer) => writer.flush(),
            #[cfg(feature = "flate2")]
            Self::Gz(ref mut writer) => writer.flush(),
        }
    }
}
