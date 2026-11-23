use super::{Cursor, DataSource, Error, Read, NUMBER_SIZE};
use core::{fmt, ops::Range};

/// Failures of the bounded APIs. The legacy `Error` enum remains unchanged.
#[derive(Debug)]
#[non_exhaustive]
pub enum AccessError {
    Read(Error),
    Bounds {
        offset: usize,
        len: usize,
        size: usize,
    },
    Overflow,
    NoProgress {
        completed: usize,
        required: usize,
    },
    InvalidRead {
        reported: usize,
        capacity: usize,
    },
    BorrowConflict,
}

impl From<Error> for AccessError {
    fn from(error: Error) -> Self {
        Self::Read(error)
    }
}

impl fmt::Display for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(error) => write!(f, "read or verification failed: {error:?}"),
            Self::Bounds { offset, len, size } => {
                write!(
                    f,
                    "range at {offset} of length {len} exceeds cursor size {size}"
                )
            }
            Self::Overflow => f.write_str("cursor range overflows usize"),
            Self::NoProgress {
                completed,
                required,
            } => {
                write!(f, "source stopped after {completed} of {required} bytes")
            }
            Self::InvalidRead { reported, capacity } => {
                write!(
                    f,
                    "source reported {reported} bytes for a {capacity}-byte buffer"
                )
            }
            Self::BorrowConflict => f.write_str("data source is already borrowed"),
        }
    }
}

impl core::error::Error for AccessError {}

fn read_exact(reader: &dyn Read, output: &mut [u8], offset: usize) -> Result<(), AccessError> {
    let mut completed = 0;
    while completed < output.len() {
        let position = offset.checked_add(completed).ok_or(AccessError::Overflow)?;
        let capacity = output.len() - completed;
        let count = reader.read(&mut output[completed..], position)?;
        if count > capacity {
            return Err(AccessError::InvalidRead {
                reported: count,
                capacity,
            });
        }
        if count == 0 {
            return Err(AccessError::NoProgress {
                completed,
                required: output.len(),
            });
        }
        completed += count;
    }
    Ok(())
}

impl DataSource {
    fn window(&mut self, offset: usize, len: usize) -> Result<Range<usize>, AccessError> {
        debug_assert!(len <= self.cache.len());
        let end = offset.checked_add(len).ok_or(AccessError::Overflow)?;
        if self.cache_actual_size <= self.cache.len() && offset >= self.cache_start_point {
            if let Some(cache_end) = self.cache_start_point.checked_add(self.cache_actual_size) {
                if end <= cache_end {
                    let start = offset - self.cache_start_point;
                    return Ok(start..start + len);
                }
                // Extend the cached window for contiguous/overlapping sequential reads
                // without ever reading a byte past `end` or mutating the cached prefix.
                let new_size = end - self.cache_start_point;
                if self.cache_actual_size > 0 && offset <= cache_end && new_size <= self.cache.len()
                {
                    read_exact(
                        self.reader.as_ref(),
                        &mut self.cache[self.cache_actual_size..new_size],
                        cache_end,
                    )?;
                    self.cache_actual_size = new_size;
                    let start = offset - self.cache_start_point;
                    return Ok(start..start + len);
                }
            }
        }

        // A failed read may have overwritten the old cache. Invalidate it first.
        // Never speculatively read past `offset + len`: unread trailing bytes may
        // reside in a region the caller never asked to touch or that faults on read.
        self.cache_actual_size = 0;
        read_exact(self.reader.as_ref(), &mut self.cache[..len], offset)?;
        self.cache_start_point = offset;
        self.cache_actual_size = len;
        Ok(0..len)
    }
}

impl Cursor {
    fn absolute_range(&self, relative_offset: usize, len: usize) -> Result<usize, AccessError> {
        self.offset
            .checked_add(self.size)
            .ok_or(AccessError::Overflow)?;
        let end = relative_offset
            .checked_add(len)
            .ok_or(AccessError::Overflow)?;
        if end > self.size {
            return Err(AccessError::Bounds {
                offset: relative_offset,
                len,
                size: self.size,
            });
        }
        self.offset
            .checked_add(relative_offset)
            .ok_or(AccessError::Overflow)
    }

    /// Fill exactly the requested logical range, retrying short reads.
    ///
    /// The destination may contain a prefix on I/O failure. Bounds and overflow
    /// errors occur before accessing the source or changing the destination.
    /// Like other cached reads, this requires a stable source for a coherent view.
    pub fn read_exact_at(
        &self,
        relative_offset: usize,
        output: &mut [u8],
    ) -> Result<(), AccessError> {
        let offset = self.absolute_range(relative_offset, output.len())?;
        if output.is_empty() {
            return Ok(());
        }
        let mut source = self
            .data_source
            .try_borrow_mut()
            .map_err(|_| AccessError::BorrowConflict)?;
        if output.len() > source.cache.len() {
            read_exact(source.reader.as_ref(), output, offset)
        } else {
            let range = source.window(offset, output.len())?;
            output.copy_from_slice(&source.cache[range]);
            Ok(())
        }
    }

    /// Read a complete fixed-size field; this never accepts a larger prefix.
    pub fn read_array<const N: usize>(&self) -> Result<[u8; N], AccessError> {
        if self.size != N {
            return Err(Error::TotalSize(N, self.size).into());
        }
        let mut bytes = [0; N];
        self.read_exact_at(0, &mut bytes)?;
        Ok(bytes)
    }

    /// Read one little-endian Molecule header word within this cursor.
    pub(super) fn read_number_at(&self, relative_offset: usize) -> Result<usize, AccessError> {
        let mut bytes = [0; NUMBER_SIZE];
        self.read_exact_at(relative_offset, &mut bytes)?;
        Ok(u32::from_le_bytes(bytes) as usize)
    }

    /// Create a bounded subcursor, checking both relative and absolute overflow.
    pub fn slice_exact(&self, relative_offset: usize, size: usize) -> Result<Self, AccessError> {
        let offset = self.absolute_range(relative_offset, size)?;
        Ok(Self {
            offset,
            size,
            data_source: self.data_source.clone(),
        })
    }
}
