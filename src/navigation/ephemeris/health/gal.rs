use bitflags::bitflags;

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags! {
    /// [GalHealth] SV ealth indication
    #[derive(Debug, Default, Copy, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GalHealth : u32 {
        const E1B_DVS = 0x00000001;
        const E1B_HS_BIT0 = 0x00000002;
        const E1B_HS_BIT1 = 0x00000004;
        const E5A_DVS = 0x00000008;
        const E5A_HS_BIT0 = 0x00000010;
        const E5A_HS_BIT1 = 0x00000020;
        const E5B_DVS = 0x00000040;
        const E5B_HS_BIT0 = 0x00000080;
        const E5B_HS_BIT1 = 0x00000100;
    }
}

bitflags! {
    /// [GalDataSource] indication.
    /// All bits cannot be asserted at all times,
    /// because INAV and FNAV contain different information.
    #[derive(Debug, Default, Copy, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GalDataSource : u32 {
        /// [GalDataSouce::INAV_E1B] might be set in conjonction to
        /// [GalDataSource::FNAV_E5B_I] if INAV messages were merged together. 
        const INAV_E1B = 0x00000001;
        /// [GalDataSouce::FNAV_E5A_I] might be set in conjonction to
        /// [GalDataSource::INAV_E1B] if INAV messages were merged together. 
        const FNAV_E5A_I = 0x00000002; 
        const FNAV_E5B_I = 0x00000004; 
        /// Reserved for Galileo internal use
        const GAL_RESERVED1 = 0x00000008;
        /// Reserved for Galileo internal use
        const GAL_RESERVED2 = 0x00000010;
        /// [GalDataSource::TEMPORAL_SISA_E1_E5A] is asserted
        /// if the temporal information (provided ToC) and SISA field addresses
        /// E1 or E5A signal. Both asserted at the same time is illegal.
        const TEMPORAL_SISA_E1_E5A= 0x00000100;
        /// [GalDataSource::TEMPORAL_SISA_E1_E5B] is asserted
        /// if the temporal information (provided ToC) and SISA field addresses
        /// E1 or E5B signal. Both asserted at the same time is illegal.
        const TEMPORAL_SISA_E1_E5A= 0x00000200;
    }
}
