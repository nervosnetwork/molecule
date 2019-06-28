pub trait Reader<'r>: Sized + Verifiable {
    fn as_slice(&self) -> &[u8];
    // TODO return Result<Self, _>
    fn from_slice<'a: 'r>(slice: &'a [u8]) -> Option<Self>;
}

pub trait Verifiable {
    fn verify(slice: &[u8]) -> bool;
}
