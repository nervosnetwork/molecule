use alloc::{borrow::ToOwned, vec::Vec};
use core::{convert::From, ops::Deref};

#[derive(Debug, Default, Clone)]
pub struct Bytes(Vec<u8>);

impl From<Vec<u8>> for Bytes {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl From<&[u8]> for Bytes {
    fn from(value: &[u8]) -> Self {
        Self(value.to_owned())
    }
}

impl From<Bytes> for Vec<u8> {
    fn from(value: Bytes) -> Self {
        value.0
    }
}

impl Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Bytes {
    pub fn slice(&self, start: usize, end: usize) -> Self {
        Self::from(&self.0[start..end])
    }

    pub fn slice_from(&self, start: usize) -> Self {
        self.slice(start, self.len())
    }

    pub fn slice_to(&self, end: usize) -> Self {
        self.slice(0, end)
    }
}
