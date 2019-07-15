use std::{clone::Clone, default::Default, fmt, io};

use bytes::Bytes;

use crate::error::VerificationResult;

pub trait Molecule: fmt::Debug + Default + Clone {
    fn verify(slice: &[u8]) -> VerificationResult<()>;
}

pub trait Entity: fmt::Debug + Default + Clone {
    fn new_unchecked(data: Bytes) -> Self;
    fn as_slice(&self) -> &[u8];
    fn from_slice(slice: &[u8]) -> VerificationResult<Self>;
}

pub trait Reader<'r>: Sized + fmt::Debug {
    type Kernel: Molecule;
    type Entity: Entity;
    fn new_unchecked(slice: &'r [u8]) -> Self;
    fn as_slice(&self) -> &[u8];
    fn from_slice(slice: &'r [u8]) -> VerificationResult<Self> {
        Self::Kernel::verify(slice).map(|_| Reader::new_unchecked(slice))
    }
    fn to_entity(&self) -> Self::Entity;
}

pub trait Builder {
    type Kernel: Molecule;
    type Entity: Entity;
    fn expected_length(&self) -> usize;
    fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()>;
    fn build(&self) -> io::Result<Self::Entity>;
}
