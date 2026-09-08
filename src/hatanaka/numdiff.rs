//! Y. Hatanaka lossless numerical compression algorithm

use crate::hatanaka::Error;

/// This module implements the Hatanaka scheme for numerical (de)-compression.
/// Values are multiplied by a factor of 10^N (N is the number of decimals in the
/// original data) to turn them into integers.
/// These integers are then put through a higher-order delta-compression scheme.
///
/// There are two tunable parameters:
/// - M: the maximum order accepted by the decompressor. The historical CRX2RNX
///   program supports M=5, but we support M=6.
/// - level (m): the actual order used for compression. It must be <= M. The
///   historical RNX2CRX program uses m=3, which is a good choice for 1-30s
///   sampling rates.
///
/// A higher m does not necessarily mean better compression, since at a high-enough
/// level random noise dominates over any differential trend.
///
/// If you want to produce compatible data, you should respect M = 5.
/// If your remain within our application, you can use higher compression order
/// (m <= M = 6).
///
/// Currently compressor.rs uses level = 3 ([CompressorExpert::format()]).
///
/// Both directions fail with [Error::CompressionOrder] when the order
/// exceeds M or 6, and with [Error::NumericOverflow] when the checked
/// arithmetic leaves the i64 range (corrupt data, or a kernel that was
/// not reset where it should have been).
#[derive(Debug, Clone)]
pub struct NumDiff<const M: usize> {
    /// iteration counter
    m: usize,
    /// compression level, within M maximal range
    level: usize,
    /// internal data history
    buf: [i64; M],
}

impl<const M: usize> NumDiff<M> {
    /// Highest (de)compression order this implementation supports, whatever M.
    pub const MAX_ORDER: usize = 6;

    /// Binomial coefficients (alternating signs) applied to the history,
    /// per order: order m uses the first m entries of row m.
    const COEFFICIENTS: [[i64; 6]; 7] = [
        [0, 0, 0, 0, 0, 0],
        [1, 0, 0, 0, 0, 0],
        [2, -1, 0, 0, 0, 0],
        [3, -3, 1, 0, 0, 0],
        [4, -6, 4, -1, 0, 0],
        [5, -10, 10, -5, 1, 0],
        [6, -15, 20, -15, 6, -1],
    ];

    /// Builds a [NumDiff] structure dedicated to numerical (de-)compression.
    /// ## Inputs
    ///  - data: initial point
    ///  - level: compression level / range. Must not exceed M nor
    /// [Self::MAX_ORDER], otherwise (de)compression will fail with
    /// [Error::CompressionOrder].
    pub fn new(data: i64, level: usize) -> Self {
        let mut buf = [0; M]; // reset
        buf[0] = data;
        Self { buf, m: 0, level }
    }

    /// Reinitializes the kernel with a new reference point and level,
    /// as commanded by a "m&value" kernel reset in the compressed stream.
    pub fn force_init(&mut self, data: i64, level: usize) {
        self.m = 0;
        self.level = level;
        self.rotate_history(data);
    }

    /// Rotate internal buffer, take new sample into account.
    fn rotate_history(&mut self, data: i64) {
        self.buf.copy_within(0..M - 1, 1);
        self.buf[0] = data;
    }

    /// Increments the current order (up to level) and returns the
    /// coefficients to apply to the history, or an error if that
    /// order is not supported by this kernel.
    fn next_order(&mut self) -> Result<&'static [i64], Error> {
        if self.m < self.level {
            self.m += 1;
        }
        if self.m > M || self.m > Self::MAX_ORDER {
            return Err(Error::CompressionOrder);
        }
        Ok(&Self::COEFFICIENTS[self.m][..self.m])
    }

    /// Decompresses input data point, returns recovered data point.
    pub fn decompress(&mut self, data: i64) -> Result<i64, Error> {
        let coefficients = self.next_order()?;

        let mut new = data;
        for (coeff, past) in coefficients.iter().zip(self.buf.iter()) {
            let term = coeff.checked_mul(*past).ok_or(Error::NumericOverflow)?;
            new = new.checked_add(term).ok_or(Error::NumericOverflow)?;
        }

        self.rotate_history(new);
        Ok(new)
    }

    /// Compresses input data point, returns "compressed" data point.
    ///
    /// The difference is computed against the history *before* the new
    /// sample is pushed, exactly mirroring [Self::decompress]: order `m`
    /// reads `buf[0..m]` in both directions, so a `NumDiff<M>` supports
    /// orders up to `M` for compression and decompression alike.
    pub fn compress(&mut self, data: i64) -> Result<i64, Error> {
        let coefficients = self.next_order()?;

        let mut compressed = data;
        for (coeff, past) in coefficients.iter().zip(self.buf.iter()) {
            let term = coeff.checked_mul(*past).ok_or(Error::NumericOverflow)?;
            compressed = compressed.checked_sub(term).ok_or(Error::NumericOverflow)?;
        }

        self.rotate_history(data);
        Ok(compressed)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decompression() {
        let mut diff = NumDiff::<6>::new(126298057858, 3);
        assert_eq!(diff.decompress(-15603288), Ok(126282454570));
        assert_eq!(diff.decompress(521089), Ok(126267372371));
        assert_eq!(diff.decompress(-752), Ok(126252810509));
        assert_eq!(diff.decompress(1575419284), Ok(127814188268));
        assert_eq!(diff.decompress(-3150848707), Ok(127800656941));
        assert_eq!(diff.decompress(1575424909), Ok(127787641437));
        assert_eq!(diff.decompress(-135), Ok(127775141621));

        // test re-init
        diff.force_init(111982965979, 3);
        assert_eq!(diff.decompress(-16266911), Ok(111966699068));
        assert_eq!(diff.decompress(609858), Ok(111951042015));
        assert_eq!(diff.decompress(-213), Ok(111935994607));
        assert_eq!(diff.decompress(1575419307), Ok(113496976151));
        assert_eq!(diff.decompress(-3150848442), Ok(113483138205));
        assert_eq!(diff.decompress(1575425367), Ok(113469906136));
        assert_eq!(diff.decompress(146), Ok(113457280090));
    }

    #[test]
    fn test_compression() {
        let mut diff = NumDiff::<6>::new(126298057858, 3);
        assert_eq!(diff.compress(126282454570), Ok(-15603288));
        assert_eq!(diff.compress(126267372371), Ok(521089));
        assert_eq!(diff.compress(126252810509), Ok(-752));
        assert_eq!(diff.compress(127814188268), Ok(1575419284));
        assert_eq!(diff.compress(127800656941), Ok(-3150848707));
        assert_eq!(diff.compress(127787641437), Ok(1575424909));
        assert_eq!(diff.compress(127775141621), Ok(-135));

        diff.force_init(111982965979, 3);
        assert_eq!(diff.compress(111966699068), Ok(-16266911));
        assert_eq!(diff.compress(111951042015), Ok(609858));
        assert_eq!(diff.compress(111935994607), Ok(-213));
        assert_eq!(diff.compress(113496976151), Ok(1575419307));
        assert_eq!(diff.compress(113483138205), Ok(-3150848442));
        assert_eq!(diff.compress(113469906136), Ok(1575425367));
        assert_eq!(diff.compress(113457280090), Ok(146));
    }

    #[test]
    fn test_history_rotation_full_order() {
        // Regression test for https://github.com/nav-solutions/rinex/issues/426
        //
        // `rotate_history` used to only shift `buf[0..M-2]`, so `buf[M-1]`
        // (the oldest history slot) was frozen at its initial value forever
        // and never took part in the running history. Decompression that
        // reaches the maximal order `M` (as happens for the clock-offset
        // field, which is decompressed with `NumDiff<5>` at order 5) then
        // silently used a stale value instead of the correct one, causing
        // decompression to diverge and eventually overflow further down
        // the file.
        //
        // `compressed` below is the correct order-5 CRINEX encoding of
        // `original`, computed independently of this crate. Decompressing
        // it must reproduce `original` exactly, which requires every slot
        // of the history buffer (including buf[M-1]) to stay up to date.
        let original = [
            126298057858_i64,
            126282454570,
            126267372371,
            126252810509,
            127814188268,
            127800656941,
            127787641437,
            127775141621,
            127762669477,
        ];
        let compressed = [
            -15603288_i64,
            521089,
            -752,
            1575420036,
            -6301688027,
            9452541607,
            -6301698660,
            1574937163,
        ];

        let mut diff = NumDiff::<5>::new(original[0], 5);

        let mut recovered = vec![original[0]];
        for value in compressed {
            recovered.push(diff.decompress(value).unwrap());
        }

        assert_eq!(
            recovered, original,
            "decompressed sequence diverged from the original: \
             history rotation must keep every buffer slot (including buf[M-1]) up to date"
        );
    }

    #[test]
    fn test_compression_full_order() {
        // Regression test for https://github.com/nav-solutions/rinex/issues/429
        //
        // `compress` used to rotate the history *before* differencing, so
        // order `m` read `buf[0..=m]` while `decompress` only reads
        // `buf[0..m]`. At the maximal order `M` this indexed `buf[M]`,
        // one past the end of the buffer, and panicked.
        //
        // Same sequence as `test_history_rotation_full_order`: compressing
        // `original` at order 5 with a `NumDiff<5>` must produce the
        // independently computed `compressed` stream without panicking.
        let original = [
            126298057858_i64,
            126282454570,
            126267372371,
            126252810509,
            127814188268,
            127800656941,
            127787641437,
            127775141621,
            127762669477,
        ];
        let expected = [
            -15603288_i64,
            521089,
            -752,
            1575420036,
            -6301688027,
            9452541607,
            -6301698660,
            1574937163,
        ];

        let mut diff = NumDiff::<5>::new(original[0], 5);

        let compressed: Vec<i64> = original[1..]
            .iter()
            .map(|value| diff.compress(*value).unwrap())
            .collect();

        assert_eq!(compressed, expected);
    }

    #[test]
    fn test_round_trip_full_order() {
        // Compression and decompression must be exact inverses at every
        // supported order, using the same `NumDiff<M>` size on both sides
        // (order M with `NumDiff<M>` included). Exercises both re-init
        // and the steady state past the ramp-up.
        fn round_trip<const M: usize>(level: usize) {
            let original: Vec<i64> = (0..4 * M as i64)
                .map(|i| 126298057858 + i * i * 7919 - i * 15603288)
                .collect();

            let mut compressor = NumDiff::<M>::new(original[0], level);
            let mut decompressor = NumDiff::<M>::new(original[0], level);

            for value in &original[1..] {
                let compressed = compressor.compress(*value).unwrap();
                let recovered = decompressor.decompress(compressed).unwrap();
                assert_eq!(
                    recovered, *value,
                    "round trip mismatch for M={} level={}",
                    M, level
                );
            }

            // reinit both kernels mid-stream and go again
            compressor.force_init(original[0], level);
            decompressor.force_init(original[0], level);

            for value in &original[1..] {
                let compressed = compressor.compress(*value).unwrap();
                let recovered = decompressor.decompress(compressed).unwrap();
                assert_eq!(
                    recovered, *value,
                    "round trip mismatch after force_init for M={} level={}",
                    M, level
                );
            }
        }

        for level in 1..=3 {
            round_trip::<3>(level);
        }
        for level in 1..=5 {
            round_trip::<5>(level);
        }
        for level in 1..=6 {
            round_trip::<6>(level);
        }
    }

    #[test]
    fn test_overflow_is_an_error() {
        // gh-426: corrupt input, or a kernel that was never reset, must
        // surface as an error rather than an arithmetic overflow panic
        // (debug) or a silently wrapped value (release).
        let mut diff = NumDiff::<5>::new(0, 5);
        let mut last = Ok(0);
        for _ in 0..64 {
            last = diff.decompress(i64::MAX / 4);
            if last.is_err() {
                break;
            }
        }
        assert_eq!(last, Err(Error::NumericOverflow));

        let mut diff = NumDiff::<5>::new(i64::MIN, 1);
        assert_eq!(diff.compress(i64::MAX), Err(Error::NumericOverflow));
    }

    #[test]
    fn test_unsupported_order_is_an_error() {
        // level within the buffer: fine
        let mut diff = NumDiff::<5>::new(0, 5);
        for _ in 0..8 {
            assert!(diff.decompress(1).is_ok());
        }

        // level beyond the buffer size M: fails once that order is reached,
        // instead of indexing out of bounds
        let mut diff = NumDiff::<5>::new(0, 6);
        for _ in 0..5 {
            assert!(diff.decompress(1).is_ok());
        }
        assert_eq!(diff.decompress(1), Err(Error::CompressionOrder));

        // same for a reset, and for compression
        let mut diff = NumDiff::<3>::new(0, 3);
        diff.force_init(0, 4);
        assert!(diff.compress(1).is_ok());
        assert!(diff.compress(2).is_ok());
        assert!(diff.compress(3).is_ok());
        assert_eq!(diff.compress(4), Err(Error::CompressionOrder));

        // orders above 6 are never supported, whatever M
        let mut diff = NumDiff::<8>::new(0, 7);
        for _ in 0..6 {
            assert!(diff.decompress(1).is_ok());
        }
        assert_eq!(diff.decompress(1), Err(Error::CompressionOrder));
    }
}
