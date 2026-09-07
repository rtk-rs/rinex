//! Y. Hatanaka lossless numerical compression algorithm

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
    /// Builds a [NumDiff] structure dedicated to numerical (de-)compression.
    /// Level must not exceed 6 otherwise this will panic.
    /// ## Inputs
    ///  - data: initial point
    ///  - level: compression level / range.
    pub fn new(data: i64, level: usize) -> Self {
        if level > 6 {
            panic!("M=6 is the compression limit");
        }
        let mut buf = [0; M]; // reset
        buf[0] = data;
        Self { buf, m: 0, level }
    }
    /// [NumDiff] needs to be reinit   when ???
    pub fn force_init(&mut self, data: i64, level: usize) {
        if level > 6 {
            panic!("M=6 is the compression limit");
        }
        self.m = 0;
        self.level = level;
        self.rotate_history(data);
    }

    /// Rotate internal buffer, take new sample into account.
    fn rotate_history(&mut self, data: i64) {
        self.buf.copy_within(0..M - 1, 1);
        self.buf[0] = data;
    }

    /// Decompresses input data point, returns recovered data point.
    pub fn decompress(&mut self, data: i64) -> i64 {
        if self.m < self.level {
            self.m += 1;
        }

        let new: i64 = match self.m {
            1 => data + self.buf[0],
            2 => data + 2 * self.buf[0] - self.buf[1],
            3 => data + 3 * self.buf[0] - 3 * self.buf[1] + self.buf[2],
            4 => data + 4 * self.buf[0] - 6 * self.buf[1] + 4 * self.buf[2] - self.buf[3],
            5 => {
                data + 5 * self.buf[0] - 10 * self.buf[1] + 10 * self.buf[2] - 5 * self.buf[3]
                    + self.buf[4]
            },
            6 => {
                data + 6 * self.buf[0] - 15 * self.buf[1] + 20 * self.buf[2] - 15 * self.buf[3]
                    + 6 * self.buf[4]
                    - self.buf[5]
            },
            _ => panic!("numdiff is limited to M < 7"),
        };

        self.rotate_history(new);
        new
    }

    /// Compresses input data point, returns "compressed" data point.
    ///
    /// The difference is computed against the history *before* the new
    /// sample is pushed, exactly mirroring [Self::decompress]: order `m`
    /// reads `buf[0..m]` in both directions, so a `NumDiff<M>` supports
    /// orders up to `M` for compression and decompression alike.
    pub fn compress(&mut self, data: i64) -> i64 {
        if self.m < self.level {
            self.m += 1;
        }

        let compressed: i64 = match self.m {
            1 => data - self.buf[0],
            2 => data - 2 * self.buf[0] + self.buf[1],
            3 => data - 3 * self.buf[0] + 3 * self.buf[1] - self.buf[2],
            4 => data - 4 * self.buf[0] + 6 * self.buf[1] - 4 * self.buf[2] + self.buf[3],
            5 => {
                data - 5 * self.buf[0] + 10 * self.buf[1] - 10 * self.buf[2] + 5 * self.buf[3]
                    - self.buf[4]
            },
            6 => {
                data - 6 * self.buf[0] + 15 * self.buf[1] - 20 * self.buf[2] + 15 * self.buf[3]
                    - 6 * self.buf[4]
                    + self.buf[5]
            },
            _ => panic!("numdiff is limited to M < 7"),
        };

        self.rotate_history(data);
        compressed
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decompression() {
        let mut diff = NumDiff::<6>::new(126298057858, 3);
        assert_eq!(diff.decompress(-15603288), 126282454570);
        assert_eq!(diff.decompress(521089), 126267372371);
        assert_eq!(diff.decompress(-752), 126252810509);
        assert_eq!(diff.decompress(1575419284), 127814188268);
        assert_eq!(diff.decompress(-3150848707), 127800656941);
        assert_eq!(diff.decompress(1575424909), 127787641437);
        assert_eq!(diff.decompress(-135), 127775141621);

        // test re-init
        diff.force_init(111982965979, 3);
        assert_eq!(diff.decompress(-16266911), 111966699068);
        assert_eq!(diff.decompress(609858), 111951042015);
        assert_eq!(diff.decompress(-213), 111935994607);
        assert_eq!(diff.decompress(1575419307), 113496976151);
        assert_eq!(diff.decompress(-3150848442), 113483138205);
        assert_eq!(diff.decompress(1575425367), 113469906136);
        assert_eq!(diff.decompress(146), 113457280090);
    }

    #[test]
    fn test_compression() {
        let mut diff = NumDiff::<6>::new(126298057858, 3);
        assert_eq!(diff.compress(126282454570), -15603288);
        assert_eq!(diff.compress(126267372371), 521089);
        assert_eq!(diff.compress(126252810509), -752);
        assert_eq!(diff.compress(127814188268), 1575419284);
        assert_eq!(diff.compress(127800656941), -3150848707);
        assert_eq!(diff.compress(127787641437), 1575424909);
        assert_eq!(diff.compress(127775141621), -135);

        diff.force_init(111982965979, 3);
        assert_eq!(diff.compress(111966699068), -16266911);
        assert_eq!(diff.compress(111951042015), 609858);
        assert_eq!(diff.compress(111935994607), -213);
        assert_eq!(diff.compress(113496976151), 1575419307);
        assert_eq!(diff.compress(113483138205), -3150848442);
        assert_eq!(diff.compress(113469906136), 1575425367);
        assert_eq!(diff.compress(113457280090), 146);
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
            recovered.push(diff.decompress(value));
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
            .map(|value| diff.compress(*value))
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
                let compressed = compressor.compress(*value);
                let recovered = decompressor.decompress(compressed);
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
                let compressed = compressor.compress(*value);
                let recovered = decompressor.decompress(compressed);
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
}
