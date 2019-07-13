use std::{default, fmt, io, marker::PhantomData};

use crate::error::{VerificationError, VerificationResult};

pub trait Molecule: fmt::Debug + default::Default {
    fn verify(slice: &[u8]) -> VerificationResult<()>;
}

#[derive(Debug)]
pub struct Reader<'r, M: Molecule>(&'r [u8], PhantomData<M>);

#[derive(Debug, Default)]
pub struct Entity<M: Molecule>(Vec<u8>, PhantomData<M>);

pub trait Builder {
    type Kernel: Molecule;
    fn expected_length(&self) -> usize;
    fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()>;
    fn build(&self) -> io::Result<Entity<Self::Kernel>>;
}

impl<'r, M> Reader<'r, M>
where
    M: Molecule,
{
    pub fn new_unchecked(slice: &'r [u8]) -> Self {
        Reader(slice, PhantomData)
    }
    pub fn from_slice(slice: &'r [u8]) -> VerificationResult<Self> {
        M::verify(slice).map(|_| Reader(slice, PhantomData))
    }
    pub fn as_slice(&self) -> &[u8] {
        self.0
    }
    pub fn to_entity(&self) -> Entity<M> {
        Entity(self.0.to_owned(), PhantomData)
    }
}

impl<M> Entity<M>
where
    M: Molecule,
{
    pub fn new_unchecked(data: Vec<u8>) -> Self {
        Entity(data, PhantomData)
    }
    pub fn from_slice(slice: &[u8]) -> VerificationResult<Self> {
        Reader::from_slice(slice).map(|reader| reader.to_entity())
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.0[..]
    }
    pub fn as_reader(&self) -> Reader<'_, M> {
        Reader(&self.0[..], PhantomData)
    }
}

impl Molecule for u8 {
    fn verify(slice: &[u8]) -> VerificationResult<()> {
        if slice.len() == 1 {
            Ok(())
        } else {
            let err = VerificationError::TotalSizeNotMatch("u8".to_owned(), 1, slice.len());
            Err(err)
        }
    }
}
