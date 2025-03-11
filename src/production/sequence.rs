/* File sequence: used to describe batch or day course some files represent. */
use super::Error;

#[cfg(feature = "serde")]
use serde::Serialize;

/// FileSequence is used to describe whether this
/// file is part of a batch of files or
/// which section (time frame) of the day course it represents.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum FileSequence {
    /// This file is integrated in a file batch (# id)
    Batch(u8),
    /// This file represents a specific time frame of a daycourse.
    /// 0 means midnight to midnight +1h.
    /// 10 means midnight past 10h to +11h.
    /// And so forth.
    DayPortion(u8),
    /// This file represents an entire day course
    #[default]
    DayCourse,
}

impl std::str::FromStr for FileSequence {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let chars = content.chars().nth(0).unwrap();

        // "0" means entire day
        if chars == '0' {
            Ok(Self::DayCourse)
        } else if chars.is_ascii_alphabetic() {
            let value = chars as u32 - 97;
            if value < 24 {
                Ok(Self::DayPortion(value as u8))
            } else {
                Err(Error::InvalidFileSequence)
            }
        } else {
            let batch_id = content
                .parse::<u8>()
                .map_err(|_| Error::InvalidFileSequence)?;
            Ok(Self::Batch(batch_id))
        }
    }
}

#[cfg(test)]
mod test {
    use super::FileSequence;
    use std::str::FromStr;
    #[test]
    fn file_sequence_parsing() {
        for (desc, expected) in [
            ("a", FileSequence::DayPortion(0)),
            ("b", FileSequence::DayPortion(1)),
            ("c", FileSequence::DayPortion(2)),
            ("d", FileSequence::DayPortion(3)),
            ("e", FileSequence::DayPortion(4)),
            ("u", FileSequence::DayPortion(20)),
            ("v", FileSequence::DayPortion(21)),
            ("w", FileSequence::DayPortion(22)),
            ("x", FileSequence::DayPortion(23)),
            ("0", FileSequence::DayCourse),
            ("1", FileSequence::Batch(1)),
            ("2", FileSequence::Batch(2)),
            ("10", FileSequence::Batch(10)),
        ] {
            let seq = FileSequence::from_str(desc).unwrap_or_else(|_| {
                panic!("failed to parse \"{}\"", desc);
            });
            assert_eq!(seq, expected);
        }
        assert!(
            FileSequence::from_str("z").is_err(),
            "this file sequence is invalid"
        );
    }
}
