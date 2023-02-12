use crate::models::types::unsigned::FnkUInt;
use crate::prelude::FnkInt;
use borsh::{BorshDeserialize, BorshSerialize};
use std::io::Write;
use std::ops::RangeInclusive;

/// Custom range impl over two `FnkUInt` points that serializes as point + length.
/// The range is inclusive and the length can be:
/// - 0: when the range is unbounded, e.g. `5..`, this serializes as `5,0`.
/// - positive: when the distance to the starting point is less than the distance
///             to the ending point, e.g. `5..=10`, this serializes as `0,6`.
/// - negative: when the distance to the starting point is greater than the distance
///             to the ending point, e.g. `0..=u64::MAX - 5`, this serializes as `0,-5`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FnkURange {
    from: FnkUInt,
    to: FnkUInt,
}

impl FnkURange {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(from: FnkUInt, to: FnkUInt) -> Self {
        assert!(from.0 <= to.0, "{}: start > end", stringify!(FnkURange));

        Self { from, to }
    }

    pub fn new_unbounded(from: FnkUInt) -> Self {
        Self {
            from,
            to: FnkUInt::from(u64::MAX),
        }
    }

    // GETTERS ----------------------------------------------------------------

    pub fn from(&self) -> FnkUInt {
        self.from
    }

    pub fn to(&self) -> FnkUInt {
        self.to
    }

    pub(crate) fn point_and_length(&self) -> (FnkUInt, FnkInt) {
        let point = self.from;
        let distance_to_end = u64::MAX - self.to.0;

        // Shortcut for unbounded ranges.
        if distance_to_end == 0 {
            let length = FnkInt::from(0);

            return (point, length);
        }

        let distance_to_start = self.to.0 - self.from.0 + 1;

        let length = if distance_to_end <= distance_to_start {
            FnkInt::from(
                i64::try_from(distance_to_end)
                    .map(|v| -v)
                    .unwrap_or(i64::MIN),
            )
        } else {
            FnkInt::from(i64::try_from(distance_to_start).unwrap())
        };

        (point, length)
    }

    // METHODS ----------------------------------------------------------------

    pub fn to_range(&self) -> RangeInclusive<u64> {
        self.from.0..=self.to.0
    }
}

impl BorshSerialize for FnkURange {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let (point, length) = self.point_and_length();

        point.serialize(writer)?;
        length.serialize(writer)?;

        Ok(())
    }
}

impl BorshDeserialize for FnkURange {
    #[inline]
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let point = FnkUInt::deserialize(buf)?;
        let length = FnkInt::deserialize(buf)?;

        let to = if length.0 <= 0 {
            u64::MAX - length.0.unsigned_abs()
        } else {
            point.0 + length.0 as u64 - 1
        };

        Ok(Self {
            from: point,
            to: FnkUInt::from(to),
        })
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// Custom range impl over `FnkInt`. The range is inclusive.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FnkRange {
    from: FnkInt,
    to: FnkInt,
}

impl FnkRange {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(from: FnkInt, to: FnkInt) -> Self {
        assert!(from.0 <= to.0, "{}: start > end", stringify!(FnkRange));

        Self { from, to }
    }

    pub fn new_unbounded(from: FnkInt) -> Self {
        Self {
            from,
            to: FnkInt::from(i64::MAX),
        }
    }

    // GETTERS ----------------------------------------------------------------

    pub fn from(&self) -> FnkInt {
        self.from
    }

    pub fn to(&self) -> FnkInt {
        self.to
    }

    // METHODS ----------------------------------------------------------------

    pub fn to_range(&self) -> RangeInclusive<i64> {
        self.from.0..=self.to.0
    }
}

impl BorshSerialize for FnkRange {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.from.serialize(writer)?;
        self.to.serialize(writer)?;

        Ok(())
    }
}

impl BorshDeserialize for FnkRange {
    #[inline]
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let from = FnkInt::deserialize(buf)?;
        let to = FnkInt::deserialize(buf)?;

        Ok(Self { from, to })
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_serialize_unsigned_range_full() {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let range = FnkURange::new_unbounded(FnkUInt::new(5));
        range.serialize(&mut cursor).expect("Failed to serialize");

        assert_eq!(buffer, vec![5u8, 0]);

        let de_range = FnkURange::deserialize(&mut &buffer[..]).expect("Failed to deserialize");

        assert_eq!(de_range, range);
    }

    #[test]
    fn test_serialize_unsigned_range_positive() {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let range = FnkURange::new(FnkUInt::new(5), FnkUInt::new(10));
        range.serialize(&mut cursor).expect("Failed to serialize");

        assert_eq!(buffer, vec![5u8, 6]);

        let de_range = FnkURange::deserialize(&mut &buffer[..]).expect("Failed to deserialize");

        assert_eq!(de_range, range);
    }

    #[test]
    fn test_serialize_unsigned_range_negative() {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let range = FnkURange::new(FnkUInt::new(0), FnkUInt::new(u64::MAX - 5));
        range.serialize(&mut cursor).expect("Failed to serialize");

        assert_eq!(buffer, vec![0u8, 5 | 0x20]);

        let de_range = FnkURange::deserialize(&mut &buffer[..]).expect("Failed to deserialize");

        assert_eq!(de_range, range);
    }

    #[test]
    fn test_serialize_unsigned_range() {
        for i in [
            0,
            1,
            2,
            u64::MAX / 3,
            u64::MAX / 2 - 1,
            u64::MAX / 2,
            u64::MAX / 2 + 1,
            u64::MAX - 2,
            u64::MAX - 1,
            u64::MAX,
        ] {
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let range = FnkURange::new(FnkUInt::new(0), FnkUInt::new(i));
            range
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", i));

            let de_range = FnkURange::deserialize(&mut &buffer[..])
                .unwrap_or_else(|_| panic!("Failed to deserialize for {}", i));

            assert_eq!(de_range, range);
        }
    }

    #[test]
    fn test_serialize_signed_range() {
        for i in [
            i64::MIN,
            i64::MIN + 1,
            i64::MIN + 2,
            i64::MIN / 2 - 1,
            i64::MIN / 2,
            i64::MIN / 2 + 1,
            i64::MIN / 3,
            -2,
            -1,
            0,
        ] {
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let range = FnkRange::new(FnkInt::new(i), FnkInt::new(0));
            range
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", i));

            let de_range = FnkRange::deserialize(&mut &buffer[..])
                .unwrap_or_else(|_| panic!("Failed to deserialize for {}", i));

            assert_eq!(de_range, range);
        }

        for i in [
            0,
            1,
            2,
            i64::MAX / 3,
            i64::MAX / 2 - 1,
            i64::MAX / 2,
            i64::MAX / 2 + 1,
            i64::MAX - 2,
            i64::MAX - 1,
            i64::MAX,
        ] {
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            let range = FnkRange::new(FnkInt::new(0), FnkInt::new(i));
            range
                .serialize(&mut cursor)
                .unwrap_or_else(|_| panic!("Failed to serialize for {}", i));

            let de_range = FnkRange::deserialize(&mut &buffer[..])
                .unwrap_or_else(|_| panic!("Failed to deserialize for {}", i));

            assert_eq!(de_range, range);
        }
    }

    #[test]
    fn test_serialize_signed_range_full() {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let range = FnkRange::new(FnkInt::new(i64::MIN), FnkInt::new(i64::MAX));
        range.serialize(&mut cursor).expect("Failed to serialize");

        let de_range = FnkRange::deserialize(&mut &buffer[..]).expect("Failed to deserialize");

        assert_eq!(de_range, range);
    }
}
