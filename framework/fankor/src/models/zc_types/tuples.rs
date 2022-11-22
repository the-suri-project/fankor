use crate::errors::{FankorErrorCode, FankorResult};
use crate::models::{CopyType, ZeroCopyType};
use solana_program::account_info::AccountInfo;
use std::any::type_name;

impl<'info> ZeroCopyType<'info> for () {
    fn new(
        _info: &'info AccountInfo<'info>,
        _offset: usize,
    ) -> FankorResult<(Self, Option<usize>)> {
        Ok(((), Some(0)))
    }

    fn read_byte_size_from_bytes(_bytes: &[u8]) -> FankorResult<usize> {
        Ok(0)
    }
}

impl<'info> CopyType<'info> for () {
    type ZeroCopyType = ();

    fn byte_size_from_instance(&self) -> usize {
        0
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl<'info, T0: ZeroCopyType<'info>, T1: ZeroCopyType<'info>> ZeroCopyType<'info> for (T0, T1) {
    fn new(
        info: &'info AccountInfo<'info>,
        mut offset: usize,
    ) -> FankorResult<(Self, Option<usize>)> {
        let original_offset = offset;
        let (t0, size) = T0::new(info, offset)?;

        if let Some(size) = size {
            offset += size
        } else {
            let bytes =
                info.try_borrow_data()
                    .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                        type_name: type_name::<Self>(),
                    })?;
            let bytes = &bytes[offset..];
            offset += T0::read_byte_size_from_bytes(bytes)?
        }

        let (t1, size) = T1::new(info, offset)?;

        if let Some(size) = size {
            offset += size
        } else {
            let bytes = info.try_borrow_data().unwrap();
            let bytes = &bytes[offset..];
            offset += T1::read_byte_size_from_bytes(bytes)?
        }

        Ok(((t0, t1), Some(offset - original_offset)))
    }

    fn read_byte_size_from_bytes(bytes: &[u8]) -> FankorResult<usize> {
        let mut size = T0::read_byte_size_from_bytes(bytes)?;
        size += T1::read_byte_size_from_bytes(&bytes[size..])?;

        Ok(size)
    }
}

impl<'info, T0: CopyType<'info>, T1: CopyType<'info>> CopyType<'info> for (T0, T1) {
    type ZeroCopyType = (T0::ZeroCopyType, T1::ZeroCopyType);

    fn byte_size_from_instance(&self) -> usize {
        self.0.byte_size_from_instance() + self.1.byte_size_from_instance()
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

impl<'info, T0: ZeroCopyType<'info>, T1: ZeroCopyType<'info>, T2: ZeroCopyType<'info>>
    ZeroCopyType<'info> for (T0, T1, T2)
{
    fn new(
        info: &'info AccountInfo<'info>,
        mut offset: usize,
    ) -> FankorResult<(Self, Option<usize>)> {
        let original_offset = offset;
        let (t0, size) = T0::new(info, offset)?;

        if let Some(size) = size {
            offset += size
        } else {
            let bytes =
                info.try_borrow_data()
                    .map_err(|_| FankorErrorCode::ZeroCopyPossibleDeadlock {
                        type_name: type_name::<Self>(),
                    })?;
            let bytes = &bytes[offset..];
            offset += T0::read_byte_size_from_bytes(bytes)?
        }

        let (t1, size) = T1::new(info, offset)?;

        if let Some(size) = size {
            offset += size
        } else {
            let bytes = info.try_borrow_data().unwrap();
            let bytes = &bytes[offset..];
            offset += T1::read_byte_size_from_bytes(bytes)?
        }

        let (t2, size) = T2::new(info, offset)?;

        if let Some(size) = size {
            offset += size
        } else {
            let bytes = info.try_borrow_data().unwrap();
            let bytes = &bytes[offset..];
            offset += T2::read_byte_size_from_bytes(bytes)?
        }

        Ok(((t0, t1, t2), Some(offset - original_offset)))
    }

    fn read_byte_size_from_bytes(bytes: &[u8]) -> FankorResult<usize> {
        let mut size = T0::read_byte_size_from_bytes(bytes)?;
        size += T1::read_byte_size_from_bytes(&bytes[size..])?;
        size += T2::read_byte_size_from_bytes(&bytes[size..])?;

        Ok(size)
    }
}

impl<'info, T0: CopyType<'info>, T1: CopyType<'info>, T2: CopyType<'info>> CopyType<'info>
    for (T0, T1, T2)
{
    type ZeroCopyType = (T0::ZeroCopyType, T1::ZeroCopyType, T2::ZeroCopyType);

    fn byte_size_from_instance(&self) -> usize {
        self.0.byte_size_from_instance()
            + self.1.byte_size_from_instance()
            + self.2.byte_size_from_instance()
    }
}
