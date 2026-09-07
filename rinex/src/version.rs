//! `RINEX` revision description
use crate::prelude::ParsingError;

/// Latest `RINEX` major revision supported by this library.
/// Any minor revision of this major revision, or of an older one,
/// is accepted by the parser: see [Version::is_supported].
/// This is also the revision new files are written in by default.
pub const SUPPORTED_VERSION: Version = Version { major: 4, minor: 0 };

/// Version is used to describe RINEX standards revisions.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Version {
    /// Version major number
    pub major: u8,
    /// Version minor number
    pub minor: u8,
}

impl Default for Version {
    /// Builds a default `Version` object
    fn default() -> Self {
        SUPPORTED_VERSION
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl std::ops::Add<u8> for Version {
    type Output = Version;
    fn add(self, major: u8) -> Version {
        Version {
            major: self.major + major,
            minor: self.minor,
        }
    }
}

impl std::ops::AddAssign<u8> for Version {
    fn add_assign(&mut self, major: u8) {
        self.major += major;
    }
}

impl std::ops::Sub<u8> for Version {
    type Output = Version;
    fn sub(self, major: u8) -> Version {
        if major >= self.major {
            // clamp @ V1.X
            Version {
                major: 1,
                minor: self.minor,
            }
        } else {
            Version {
                major: self.major - major,
                minor: self.minor,
            }
        }
    }
}

impl std::ops::SubAssign<u8> for Version {
    fn sub_assign(&mut self, major: u8) {
        if major >= self.major {
            // clamp @ V1.X
            self.major = 1;
        } else {
            self.major -= major;
        }
    }
}

impl From<Version> for (u8, u8) {
    fn from(v: Version) -> Self {
        (v.major, v.minor)
    }
}

impl std::str::FromStr for Version {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut digits = s.split('.');

        match s.contains('.') {
            true => {
                let major = digits.next().ok_or(ParsingError::VersionFormat)?;

                let minor = digits.next().ok_or(ParsingError::VersionFormat)?;

                let major = major.parse::<u8>().or(Err(ParsingError::VersionParsing))?;
                let minor = minor.parse::<u8>().or(Err(ParsingError::VersionParsing))?;

                Ok(Self { major, minor })
            },
            false => {
                let major = digits.next().ok_or(ParsingError::VersionFormat)?;

                let major = major.parse::<u8>().or(Err(ParsingError::VersionParsing))?;

                Ok(Self { major, minor: 0 })
            },
        }
    }
}

impl Version {
    /// Builds a new `Version` object
    pub fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }
    /// Returns true if this revision can be parsed.
    /// Minor revisions only refine a major revision and never
    /// change the overall file structure, so every minor revision
    /// of a supported major revision is accepted, including minor
    /// revisions published after this library. Unknown major
    /// revisions are rejected.
    pub fn is_supported(&self) -> bool {
        self.major <= SUPPORTED_VERSION.major
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn version() {
        let version = Version::default();
        assert_eq!(version.major, SUPPORTED_VERSION.major);
        assert_eq!(version.minor, SUPPORTED_VERSION.minor);

        let version = Version::from_str("1");
        assert!(version.is_ok());
        let version = version.unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);

        let version = Version::from_str("1.2");
        assert!(version.is_ok());
        let version = version.unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);

        let version = Version::from_str("3.02");
        assert!(version.is_ok());
        let version = version.unwrap();
        assert_eq!(version.major, 3);
        assert_eq!(version.minor, 2);

        let version = Version::from_str("a.b");
        assert!(version.is_err());
    }
    #[test]
    fn supported_version() {
        let version = Version::default();
        assert!(version.is_supported());
        let version = SUPPORTED_VERSION;
        assert!(version.is_supported());

        // older revisions
        for version in ["1", "2.11", "3.02", "3.05"] {
            let version = Version::from_str(version).unwrap();
            assert!(version.is_supported(), "{} must be supported", version);
        }

        // minor revisions of the current major revision,
        // including revisions newer than SUPPORTED_VERSION
        for version in ["4.00", "4.01", "4.02", "4.99"] {
            let version = Version::from_str(version).unwrap();
            assert!(version.is_supported(), "{} must be supported", version);
        }
    }
    #[test]
    fn non_supported_version() {
        let version = Version::new(5, 0);
        assert!(!version.is_supported());
        let version = Version::new(5, 2);
        assert!(!version.is_supported());
    }
    #[test]
    fn version_comparison() {
        let v_a = Version::from_str("1.2").unwrap();
        let v_b = Version::from_str("3.02").unwrap();
        assert!(v_b > v_a);
        assert!(v_b != v_a);
    }
    #[test]
    fn version_arithmetics() {
        let version = Version::new(3, 2);
        assert_eq!(version + 1, Version::new(4, 2));
        assert_eq!(version + 2, Version::new(5, 2));
        assert_eq!(version - 2, Version::new(1, 2));
        assert_eq!(version - 3, Version::new(1, 2)); // clamped

        let (maj, min): (u8, u8) = version.into();
        assert_eq!(maj, 3);
        assert_eq!(min, 2);
    }
}
