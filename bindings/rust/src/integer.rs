//! Exact scalar access without native-alignment or host-endian assumptions.
use crate::{
    error::VerificationError,
    lazy_reader::{AccessError, Cursor, Error as LazyError},
};
use alloc::vec::Vec;
use core::{cmp::Ordering, fmt};

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

const fn reverse_bytes(bytes: [u8; 32]) -> [u8; 32] {
    let mut out = [0; 32];
    let mut i = 0;
    while i < 32 {
        out[i] = bytes[31 - i];
        i += 1;
    }
    out
}

/// A 256-bit unsigned value, stored as little-endian bytes with numeric ordering.
/// Its Rust memory layout is not a wire-format or FFI contract.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Uint256([u8; 32]);

impl Uint256 {
    pub const ZERO: Self = Self([0; 32]);
    pub const MAX: Self = Self([0xff; 32]);

    pub const fn from_le_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn from_be_bytes(bytes: [u8; 32]) -> Self {
        Self(reverse_bytes(bytes))
    }

    pub const fn to_le_bytes(self) -> [u8; 32] {
        self.0
    }

    pub const fn to_be_bytes(self) -> [u8; 32] {
        reverse_bytes(self.0)
    }

    pub const fn from_u128(value: u128) -> Self {
        let le = value.to_le_bytes();
        let mut bytes = [0; 32];
        let mut i = 0;
        while i < 16 {
            bytes[i] = le[i];
            i += 1;
        }
        Self(bytes)
    }

    pub const fn from_u64(value: u64) -> Self {
        Self::from_u128(value as u128)
    }

    const fn low_bytes<const N: usize>(self) -> Option<[u8; N]> {
        let mut i = N;
        while i < 32 {
            if self.0[i] != 0 {
                return None;
            }
            i += 1;
        }
        let mut bytes = [0; N];
        let mut j = 0;
        while j < N {
            bytes[j] = self.0[j];
            j += 1;
        }
        Some(bytes)
    }

    pub const fn try_to_u128(self) -> Result<u128, IntegerError> {
        match self.low_bytes::<16>() {
            Some(bytes) => Ok(u128::from_le_bytes(bytes)),
            None => Err(IntegerError::Overflow),
        }
    }

    pub const fn try_to_u64(self) -> Result<u64, IntegerError> {
        match self.low_bytes::<8>() {
            Some(bytes) => Ok(u64::from_le_bytes(bytes)),
            None => Err(IntegerError::Overflow),
        }
    }
}

impl From<u64> for Uint256 {
    fn from(value: u64) -> Self {
        Self::from_u64(value)
    }
}

impl From<u128> for Uint256 {
    fn from(value: u128) -> Self {
        Self::from_u128(value)
    }
}

impl TryFrom<Uint256> for u128 {
    type Error = IntegerError;

    fn try_from(value: Uint256) -> Result<Self, Self::Error> {
        value.try_to_u128()
    }
}

impl TryFrom<Uint256> for u64 {
    type Error = IntegerError;

    fn try_from(value: Uint256) -> Result<Self, Self::Error> {
        value.try_to_u64()
    }
}

fn fmt_hex(value: &Uint256, f: &mut fmt::Formatter<'_>, upper: bool) -> fmt::Result {
    const LOWER: &[u8; 16] = b"0123456789abcdef";
    const UPPER: &[u8; 16] = b"0123456789ABCDEF";
    let alphabet = if upper { UPPER } else { LOWER };
    let mut buf = [0u8; 64];
    let mut len = 0;
    for &byte in value.0.iter().rev().skip_while(|&&byte| byte == 0) {
        let hi = (byte >> 4) as usize;
        let lo = (byte & 0x0f) as usize;
        if len > 0 || hi != 0 {
            buf[len] = alphabet[hi];
            len += 1;
        }
        buf[len] = alphabet[lo];
        len += 1;
    }
    if len == 0 {
        buf[0] = b'0';
        len = 1;
    }
    let digits = core::str::from_utf8(&buf[..len]).expect("ASCII hex digits");
    f.pad_integral(true, "0x", digits)
}

impl fmt::LowerHex for Uint256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_hex(self, f, false)
    }
}

impl fmt::UpperHex for Uint256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_hex(self, f, true)
    }
}

impl fmt::Debug for Uint256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Uint256({self:#x})")
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
    use alloc::format;

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

    #[test]
    fn uint256_const_conversions_traits_and_hex_formatting() {
        const FROM_U64: Uint256 = Uint256::from_u64(0x10a);
        const FROM_BE: Uint256 = Uint256::from_be_bytes(FROM_U64.to_be_bytes());
        assert_eq!(FROM_U64, FROM_BE);
        assert_eq!(Uint256::ZERO, Uint256::default());
        assert!(Uint256::ZERO < Uint256::MAX);
        assert!(Uint256::from_u64(0xff) < Uint256::from_u64(0x100));
        assert_eq!(Uint256::from(42u64), Uint256::from_u64(42));
        assert_eq!(Uint256::from(u128::MAX), Uint256::from_u128(u128::MAX));
        assert_eq!(u64::try_from(FROM_U64).unwrap(), 0x10a);
        assert_eq!(
            u64::try_from(Uint256::from_u64(u64::MAX)).unwrap(),
            u64::MAX
        );
        assert!(matches!(
            u64::try_from(Uint256::from_u128(u64::MAX as u128 + 1)),
            Err(IntegerError::Overflow)
        ));
        assert_eq!(u128::try_from(Uint256::from(u128::MAX)).unwrap(), u128::MAX);
        assert!(matches!(
            u128::try_from(Uint256::MAX),
            Err(IntegerError::Overflow)
        ));
        assert_eq!(format!("{:x}", Uint256::ZERO), "0");
        assert_eq!(format!("{:#x}", Uint256::ZERO), "0x0");
        assert_eq!(format!("{:#x}", FROM_U64), "0x10a");
        assert_eq!(format!("{:X}", FROM_U64), "10A");
        assert_eq!(format!("{:#X}", FROM_U64), format!("{:#X}", 0x10au128));
        assert_eq!(format!("{:+x}", FROM_U64), format!("{:+x}", 0x10au128));
        assert_eq!(
            format!("{:+#08x}", FROM_U64),
            format!("{:+#08x}", 0x10au128)
        );
        assert_eq!(format!("{FROM_U64:?}"), "Uint256(0x10a)");
        assert_eq!(
            format!("{:064x}", Uint256::from_u64(1)),
            format!("{:064x}", 1u128)
        );
        assert_eq!(
            format!("{:#066x}", Uint256::from_u64(1)),
            format!("{:#066x}", 1u128)
        );
        assert_eq!(format!("{:064X}", FROM_U64), format!("{:064X}", 0x10au128));
        assert_eq!(
            format!("{:#066X}", FROM_U64),
            format!("{:#066X}", 0x10au128)
        );
        assert_eq!(format!("{:>8x}", FROM_U64), format!("{:>8x}", 0x10au128));
        assert_eq!(format!("{:<#8x}", FROM_U64), format!("{:<#8x}", 0x10au128));
        assert_eq!(format!("{:064x}", Uint256::ZERO), format!("{:064x}", 0u128));
        assert_eq!(format!("{:064x}", Uint256::MAX), "f".repeat(64));
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
