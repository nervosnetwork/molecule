use alloc::vec::Vec;
use core::{convert, result};

pub type Error = convert::Infallible;
pub type Result<T> = result::Result<T, Error>;

pub trait Write {
    fn write_all(&mut self, buf: &[u8]) -> Result<()>;
}

impl Write for Vec<u8> {
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.extend_from_slice(buf);
        Ok(())
    }
}
