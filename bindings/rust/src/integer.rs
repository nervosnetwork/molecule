//! Exact scalar access without native-alignment or host-endian assumptions.
use crate::{
    error::VerificationError,
    lazy_reader::{AccessError, Cursor, Error as LazyError},
};
use alloc::vec::Vec;
use core::fmt;

#[derive(Debug)]
#[non_exhaustive]
pub enum IntegerError {
    Size { expected: usize, actual: usize },
    ZeroWidth,
    Overflow,
    Access(AccessError),
    Verification(VerificationError),
}

impl From<AccessError> for IntegerError {
    fn from(error: AccessError) -> Self {
        Self::Access(error)
    }
}

impl From<VerificationError> for IntegerError {
    fn from(error: VerificationError) -> Self {
        Self::Verification(error)
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
            Self::Verification(error) => error.fmt(f),
        }
    }
}

impl core::error::Error for IntegerError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Access(error) => Some(error),
            Self::Verification(error) => Some(error),
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

/// Read an exact-width scalar from a lazy cursor, reporting length mismatches as
/// `IntegerError::Size` consistently with ordinary readers.
pub fn read_wire_bytes<const N: usize>(cursor: &Cursor) -> Result<[u8; N], IntegerError> {
    if cursor.size != N {
        return Err(IntegerError::Size {
            expected: N,
            actual: cursor.size,
        });
    }
    let mut bytes = [0; N];
    cursor.read_exact_at(0, &mut bytes)?;
    Ok(bytes)
}

/// Read an optional exact-width scalar from a lazy cursor (empty => `None`).
pub fn read_optional_wire_bytes<const N: usize>(
    cursor: &Cursor,
) -> Result<Option<[u8; N]>, IntegerError> {
    if cursor.size == 0 {
        Ok(None)
    } else {
        read_wire_bytes(cursor).map(Some)
    }
}

/// Read a fixed-offset scalar field from a fixed-size lazy struct cursor.
pub fn read_fixed_field<const N: usize>(
    cursor: &Cursor,
    total: usize,
    offset: usize,
) -> Result<[u8; N], IntegerError> {
    if cursor.size != total {
        return Err(IntegerError::Size {
            expected: total,
            actual: cursor.size,
        });
    }
    let mut bytes = [0; N];
    cursor.read_exact_at(offset, &mut bytes)?;
    Ok(bytes)
}

/// Read element `index` from a fixed-size lazy array of `N`-byte scalars.
pub fn read_array_item<const N: usize>(
    cursor: &Cursor,
    count: usize,
    index: usize,
) -> Result<[u8; N], IntegerError> {
    let total = count.checked_mul(N).ok_or(IntegerError::Overflow)?;
    if cursor.size != total {
        return Err(IntegerError::Size {
            expected: total,
            actual: cursor.size,
        });
    }
    if index >= count {
        return Err(AccessError::from(LazyError::OutOfBound(index, count)).into());
    }
    let mut bytes = [0; N];
    cursor.read_exact_at(index * N, &mut bytes)?;
    Ok(bytes)
}

/// Read element `index` from a lazy fixed vector of `N`-byte scalars.
pub fn read_fixvec_item<const N: usize>(
    cursor: &Cursor,
    index: usize,
) -> Result<[u8; N], IntegerError> {
    if N == 0 {
        return Err(IntegerError::ZeroWidth);
    }
    read_wire_bytes(&cursor.fixvec_item_exact(N, index)?)
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
    if width == 0 {
        return Err(IntegerError::ZeroWidth);
    }
    count
        .checked_mul(width)
        .and_then(|size| size.checked_add(crate::NUMBER_SIZE))
        .filter(|&size| size <= crate::Number::MAX as usize && size <= isize::MAX as usize)
        .ok_or(IntegerError::Overflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_size_checks_wire_and_address_limits_before_allocation() {
        let limit = (crate::Number::MAX as usize).min(isize::MAX as usize);
        for width in [1, 2, 4, 8, 16] {
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

    #[test]
    fn read_wire_bytes_reports_size_mismatch_consistently() {
        let cursor = Cursor::from([1u8, 2, 3]);
        assert!(matches!(
            read_wire_bytes::<4>(&cursor),
            Err(IntegerError::Size {
                expected: 4,
                actual: 3
            })
        ));
        assert_eq!(read_wire_bytes::<3>(&cursor).unwrap(), [1, 2, 3]);
        assert_eq!(
            read_optional_wire_bytes::<4>(&Cursor::from([])).unwrap(),
            None
        );
        assert!(matches!(
            read_optional_wire_bytes::<4>(&cursor),
            Err(IntegerError::Size {
                expected: 4,
                actual: 3
            })
        ));
        assert!(matches!(
            read_array_item::<2>(&Cursor::from([1u8, 2, 3, 4]), 2, 2),
            Err(IntegerError::Access(AccessError::Read(
                LazyError::OutOfBound(2, 2)
            )))
        ));
        assert!(matches!(
            read_fixvec_item::<0>(&Cursor::from([0u8; 4]), 0),
            Err(IntegerError::ZeroWidth)
        ));
    }
}
