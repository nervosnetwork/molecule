use std::{clone::Clone, default::Default, fmt, io};

use bytes::Bytes;

use crate::error::VerificationResult;

pub trait Entity: fmt::Debug + Default + Clone {
    type Builder: Builder;
    fn new_unchecked(data: Bytes) -> Self;
    fn as_bytes(&self) -> Bytes;
    fn as_slice(&self) -> &[u8];
    fn from_slice(slice: &[u8]) -> VerificationResult<Self>;
    fn new_builder() -> Self::Builder;
    fn as_builder(self) -> Self::Builder;
}

pub trait Reader<'r>: Sized + fmt::Debug + Clone + Copy {
    type Entity: Entity;
    fn verify(slice: &[u8]) -> VerificationResult<()>;
    fn new_unchecked(slice: &'r [u8]) -> Self;
    fn as_slice(&self) -> &[u8];
    fn from_slice(slice: &'r [u8]) -> VerificationResult<Self> {
        Self::verify(slice).map(|_| Reader::new_unchecked(slice))
    }
    fn to_entity(&self) -> Self::Entity;
}

pub trait Builder: Default {
    type Entity: Entity;
    fn expected_length(&self) -> usize;
    fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()>;
    fn build(&self) -> io::Result<Self::Entity>;
}
