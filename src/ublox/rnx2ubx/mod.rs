//! RINEX to Ublox streaming
use crate::prelude::Rinex;

mod nav;
use nav::Streamer as NavStreamer;

use ublox::PacketRef;

/// RINEX Type dependant record streamer
enum TypeDependentStreamer<'a> {
    /// NAV frames streamer
    Nav(NavStreamer<'a>),
}

impl<'a> TypeDependentStreamer<'a> {
    pub fn new(rinex: &'a Rinex) -> Self {
        // Only one format supported currently
        Self::Nav(NavStreamer::new(rinex))
    }
}

impl<'a> Iterator for TypeDependentStreamer<'a> {
    type Item = PacketRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Nav(streamer) => streamer.next(),
        }
    }
}

/// [RNX2UBX] can serialize a [Rinex] structure into a U-Blox stream
pub struct RNX2UBX<'a> {
    /// RINEX [TypeDependentStreamer]
    streamer: TypeDependentStreamer<'a>,
}

impl<'a> Iterator for RNX2UBX<'a> {
    type Item = PacketRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.streamer.next()
    }
}

impl Rinex {
    /// Obtain a [RNX2UBX] streamer to serialize this [Rinex] into a stream of U-Blox [PacketRef]s.
    /// You can then use the Iterator implementation to iterate each messages.
    /// The stream is RINEX format dependent, and we currently only truly support NAV RINEX.
    pub fn rnx2ubx<'a>(&'a self) -> Option<RNX2UBX<'a>> {
        Some(RNX2UBX {
            streamer: TypeDependentStreamer::new(self),
        })
    }
}
