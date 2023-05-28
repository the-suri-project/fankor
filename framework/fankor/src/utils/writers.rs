use std::io::{self, Write};

use solana_program::program_memory::sol_memcpy;

#[derive(Debug)]
pub struct ArrayWriter<'a> {
    inner: &'a mut [u8],
    pos: usize,
}

impl<'a> ArrayWriter<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(inner: &'a mut [u8]) -> Self {
        Self { inner, pos: 0 }
    }
}

impl<'a> Write for ArrayWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.pos >= self.inner.len() {
            return Ok(0);
        }

        let amt = self.inner.len().saturating_sub(self.pos).min(buf.len());
        sol_memcpy(&mut self.inner[self.pos..], buf, amt);
        self.pos += amt;
        Ok(amt)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        if self.write(buf)? == buf.len() {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "failed to write whole buffer",
            ))
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct VecWriter<'a> {
    inner: &'a mut Vec<u8>,
}

impl<'a> VecWriter<'a> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(inner: &'a mut Vec<u8>) -> Self {
        Self { inner }
    }
}

impl<'a> Write for VecWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        if self.write(buf)? == buf.len() {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "failed to write whole buffer",
            ))
        }
    }
}
