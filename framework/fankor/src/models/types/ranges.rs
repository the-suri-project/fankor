use crate::models::types::unsigned::FnkUInt;
use crate::prelude::FnkInt;
use crate::traits::AccountSize;
use borsh::{BorshDeserialize, BorshSerialize};
use std::io::Write;
use std::ops::RangeInclusive;

/// Custom range impl over `FnkUInt` that serializes as point + length.
/// The range is inclusive.
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

    // GETTERS ----------------------------------------------------------------

    pub fn from(&self) -> FnkUInt {
        self.from
    }

    pub fn to(&self) -> FnkUInt {
        self.to
    }

    // METHODS ----------------------------------------------------------------

    pub fn to_range(&self) -> RangeInclusive<u64> {
        self.from.0..=self.to.0
    }
}

impl BorshSerialize for FnkURange {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let point = FnkUInt::from(self.from.0);
        let length = FnkUInt::from(self.to.0 - self.from.0);

        point.serialize(writer)?;
        length.serialize(writer)?;

        Ok(())
    }
}

impl BorshDeserialize for FnkURange {
    #[inline]
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let point = FnkUInt::deserialize(buf)?;
        let length = FnkUInt::deserialize(buf)?;

        Ok(Self {
            from: point,
            to: FnkUInt::from(point.0 + length.0),
        })
    }
}

impl AccountSize for FnkURange {
    fn min_account_size() -> usize {
        FnkUInt::min_account_size() * 2
    }

    fn actual_account_size(&self) -> usize {
        let point = FnkUInt::from(self.from.0);
        let length = FnkUInt::from(self.to.0 - self.from.0);

        point.actual_account_size() + length.actual_account_size()
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// Custom range impl over `FnkInt` that serializes as point + length.
/// The range is inclusive.
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
        let point = FnkInt::from(self.from.0);
        let length = FnkInt::from(self.to.0 - self.from.0);

        point.serialize(writer)?;
        length.serialize(writer)?;

        Ok(())
    }
}

impl BorshDeserialize for FnkRange {
    #[inline]
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let point = FnkInt::deserialize(buf)?;
        let length = FnkInt::deserialize(buf)?;

        Ok(Self {
            from: point,
            to: FnkInt::from(point.0 + length.0),
        })
    }
}

impl AccountSize for FnkRange {
    fn min_account_size() -> usize {
        FnkInt::min_account_size() * 2
    }

    fn actual_account_size(&self) -> usize {
        let point = FnkInt::from(self.from.0);
        let length = FnkInt::from(self.to.0 - self.from.0);

        point.actual_account_size() + length.actual_account_size()
    }
}
