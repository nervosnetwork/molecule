//! Exact scalar access without native-alignment or host-endian assumptions.
use crate::lazy_reader::AccessError;
use alloc::vec::Vec;
use core::{cmp::Ordering, fmt};

#[derive(Debug)]
pub enum IntegerError {
    Size { expected: usize, actual: usize },
    ZeroWidth,
    Overflow,
    Access(AccessError),
}

impl From<AccessError> for IntegerError {
    fn from(error: AccessError) -> Self {
        Self::Access(error)
    }
}

impl fmt::Display for IntegerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Size { expected, actual } => {
                write!(f, "integer needs {expected} bytes, found {actual}")
            }
            Self::ZeroWidth => f.write_str("fixed-vector elements must have nonzero width"),
            Self::Overflow => f.write_str("integer is outside the requested range"),
            Self::Access(error) => error.fmt(f),
        }
    }
}

impl core::error::Error for IntegerError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Access(error) => Some(error),
            _ => None,
        }
    }
}

pub fn wire_bytes<const N: usize>(bytes: &[u8]) -> Result<&[u8; N], IntegerError> {
    bytes.try_into().map_err(|_| IntegerError::Size {
        expected: N,
        actual: bytes.len(),
    })
}

/// Encode a fixed vector in one buffer, using the byte order selected by the schema.
pub fn pack_fixvec<T, const N: usize>(
    values: &[T],
    mut to_bytes: impl FnMut(&T) -> [u8; N],
) -> Result<crate::bytes::Bytes, IntegerError> {
    if N == 0 {
        return Err(IntegerError::ZeroWidth);
    }
    let count = crate::Number::try_from(values.len()).map_err(|_| IntegerError::Overflow)?;
    let size = fixvec_size(values.len(), N)?;
    let mut wire = Vec::with_capacity(size);
    wire.extend_from_slice(&crate::pack_number(count));
    for value in values {
        wire.extend_from_slice(&to_bytes(value));
    }
    Ok(wire.into())
}

fn fixvec_size(count: usize, width: usize) -> Result<usize, IntegerError> {
    count
        .checked_mul(width)
        .and_then(|size| size.checked_add(crate::NUMBER_SIZE))
        .filter(|&size| size <= crate::Number::MAX as usize && size <= isize::MAX as usize)
        .ok_or(IntegerError::Overflow)
}

/// A 256-bit unsigned value, stored as little-endian bytes with numeric ordering.
/// Its Rust memory layout is not a wire-format or FFI contract.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Uint256([u8; 32]);

impl Uint256 {
    pub const fn from_le_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    pub fn from_be_bytes(mut bytes: [u8; 32]) -> Self {
        bytes.reverse();
        Self(bytes)
    }
    pub const fn to_le_bytes(self) -> [u8; 32] {
        self.0
    }
    pub fn to_be_bytes(self) -> [u8; 32] {
        let mut bytes = self.0;
        bytes.reverse();
        bytes
    }

    pub fn from_u128(value: u128) -> Self {
        let mut bytes = [0; 32];
        bytes[..16].copy_from_slice(&value.to_le_bytes());
        Self(bytes)
    }

    pub fn from_u64(value: u64) -> Self {
        Self::from_u128(value.into())
    }

    pub fn try_to_u128(self) -> Result<u128, IntegerError> {
        if self.0[16..].iter().any(|byte| *byte != 0) {
            return Err(IntegerError::Overflow);
        }
        let mut bytes = [0; 16];
        bytes.copy_from_slice(&self.0[..16]);
        Ok(u128::from_le_bytes(bytes))
    }

    pub fn try_to_u64(self) -> Result<u64, IntegerError> {
        u64::try_from(self.try_to_u128()?).map_err(|_| IntegerError::Overflow)
    }
}

impl Ord for Uint256 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.iter().rev().cmp(other.0.iter().rev())
    }
}
impl PartialOrd for Uint256 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_size_checks_wire_and_address_limits_before_allocation() {
        let limit = (crate::Number::MAX as usize).min(isize::MAX as usize);
        for width in [1, 2, 4, 8, 16, 32] {
            let last = (limit - crate::NUMBER_SIZE) / width;
            assert_eq!(
                fixvec_size(last, width).unwrap(),
                last * width + crate::NUMBER_SIZE
            );
            assert!(matches!(
                fixvec_size(last + 1, width),
                Err(IntegerError::Overflow)
            ));
            assert!(matches!(
                fixvec_size(usize::MAX, width),
                Err(IntegerError::Overflow)
            ));
        }
    }
}
