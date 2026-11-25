use super::{AccessError, Cursor, Error, Union, NUMBER_SIZE};
use core::ops::Range;

impl Cursor {
    /// Validate a dynamic-vector or table header and return the item count.
    pub fn dynvec_len_exact(&self) -> Result<usize, AccessError> {
        if self.size < NUMBER_SIZE {
            return Err(Error::Header(NUMBER_SIZE, self.size).into());
        }
        let size = self.read_number_at(0)?;
        if size != self.size {
            return Err(Error::TotalSize(size, self.size).into());
        }
        if size == NUMBER_SIZE {
            return Ok(0);
        }
        if size < NUMBER_SIZE * 2 {
            return Err(Error::Header(NUMBER_SIZE * 2, size).into());
        }
        let first = self.read_number_at(NUMBER_SIZE)?;
        if first < NUMBER_SIZE * 2 || first % NUMBER_SIZE != 0 {
            return Err(Error::Offset.into());
        }
        if first > size {
            return Err(Error::Header(first, size).into());
        }
        Ok(first / NUMBER_SIZE - 1)
    }

    fn scan_offsets(
        &self,
        count: usize,
        select: Option<usize>,
    ) -> Result<Option<Range<usize>>, AccessError> {
        if count == 0 {
            return Ok(None);
        }
        let size = self.size;
        let first = (count + 1) * NUMBER_SIZE;
        let mut field = None;
        let mut start = first;
        let mut remaining = count - 1;
        let mut pos = NUMBER_SIZE * 2;
        let mut i = 0;
        let mut buf = [0u8; NUMBER_SIZE * 16];
        while remaining > 0 {
            let batch = remaining.min(16);
            let bytes = batch * NUMBER_SIZE;
            self.read_exact_at(pos, &mut buf[..bytes])?;
            pos += bytes;
            remaining -= batch;
            for chunk in buf[..bytes].as_chunks::<NUMBER_SIZE>().0 {
                let end = u32::from_le_bytes(*chunk) as usize;
                if end < start || end > size {
                    return Err(Error::Offset.into());
                }
                if select == Some(i) {
                    field = Some(start..end);
                }
                start = end;
                i += 1;
            }
        }
        if select == Some(i) {
            field = Some(start..size);
        }
        Ok(field)
    }

    /// Validate a dynamic vector's header and full offset table, returning its item count.
    pub fn verify_dynvec_exact(&self) -> Result<usize, AccessError> {
        let count = self.dynvec_len_exact()?;
        self.scan_offsets(count, None)?;
        Ok(count)
    }

    /// Validate a table's header, field count, and full offset table, returning its field count.
    pub fn verify_table_exact(
        &self,
        expected_fields: usize,
        compatible: bool,
    ) -> Result<usize, AccessError> {
        let count = self.verify_dynvec_exact()?;
        if count < expected_fields || (!compatible && count > expected_fields) {
            return Err(Error::FieldCount(count).into());
        }
        Ok(count)
    }

    /// Slice one item of a dynamic vector or table in O(1) using a previously
    /// validated item count (for example from `verify_dynvec_exact` or `verify_table_exact`).
    /// Still checks bounds, offset ordering, and header overlap before slicing.
    pub fn dynvec_item_exact(&self, count: usize, index: usize) -> Result<Self, AccessError> {
        if index >= count {
            return Err(Error::OutOfBound(index, count).into());
        }
        let header_end = count
            .checked_add(1)
            .and_then(|n| n.checked_mul(NUMBER_SIZE))
            .ok_or(AccessError::Overflow)?;
        let start_pos = index
            .checked_add(1)
            .and_then(|n| n.checked_mul(NUMBER_SIZE))
            .ok_or(AccessError::Overflow)?;
        let (start, end) = if index + 1 == count {
            (self.read_number_at(start_pos)?, self.size)
        } else {
            let mut pair = [0; NUMBER_SIZE * 2];
            self.read_exact_at(start_pos, &mut pair)?;
            let words = pair.as_chunks::<NUMBER_SIZE>().0;
            (
                u32::from_le_bytes(words[0]) as usize,
                u32::from_le_bytes(words[1]) as usize,
            )
        };
        let len = end.checked_sub(start).ok_or(Error::Offset)?;
        if start < header_end || end > self.size {
            return Err(Error::Offset.into());
        }
        self.slice_exact(start, len)
    }

    /// Validate a dynamic-vector header and slice item `index` in O(1).
    pub fn dynvec_get_exact(&self, index: usize) -> Result<Self, AccessError> {
        let count = self.dynvec_len_exact()?;
        self.dynvec_item_exact(count, index)
    }

    /// Validate the offset table and retain the selected field in the same pass.
    pub fn table_field_exact(
        &self,
        minimum_fields: usize,
        index: usize,
    ) -> Result<Self, AccessError> {
        let count = self.dynvec_len_exact()?;
        if count < minimum_fields {
            return Err(Error::FieldCount(count).into());
        }
        if count == 0 {
            return Err(Error::OutOfBound(index, 0).into());
        }
        let field = self
            .scan_offsets(count, Some(index))?
            .ok_or(Error::OutOfBound(index, count))?;
        self.slice_exact(field.start, field.len())
    }

    /// Validate a fixed-vector header and total length, returning the item count.
    pub fn fixvec_len_exact(&self, item_size: usize) -> Result<usize, AccessError> {
        if item_size == 0 {
            return Err(Error::Data.into());
        }
        if self.size < NUMBER_SIZE {
            return Err(Error::Header(NUMBER_SIZE, self.size).into());
        }
        let count = self.read_number_at(0)?;
        let total = count
            .checked_mul(item_size)
            .and_then(|bytes| bytes.checked_add(NUMBER_SIZE))
            .ok_or(AccessError::Overflow)?;
        if total != self.size {
            return Err(Error::TotalSize(total, self.size).into());
        }
        Ok(count)
    }

    /// Validate a fixed-vector header and total length, then return the selected item.
    pub fn fixvec_item_exact(&self, item_size: usize, index: usize) -> Result<Self, AccessError> {
        let count = self.fixvec_len_exact(item_size)?;
        if index >= count {
            return Err(Error::OutOfBound(index, count).into());
        }
        // `fixvec_len_exact` already ruled out overflow for any `index < count`.
        self.slice_exact(NUMBER_SIZE + index * item_size, item_size)
    }

    /// Validate a byte fixed-vector header and total length, returning its payload cursor.
    pub fn fixvec_raw_bytes_exact(&self) -> Result<Self, AccessError> {
        let len = self.fixvec_len_exact(1)?;
        self.slice_exact(NUMBER_SIZE, len)
    }

    /// Validate a fixed-array total length and index, then return the selected item.
    pub fn array_item_exact(
        &self,
        count: usize,
        item_size: usize,
        index: usize,
    ) -> Result<Self, AccessError> {
        if item_size == 0 {
            return Err(Error::Data.into());
        }
        let total = count.checked_mul(item_size).ok_or(AccessError::Overflow)?;
        if self.size != total {
            return Err(Error::TotalSize(total, self.size).into());
        }
        if index >= count {
            return Err(Error::OutOfBound(index, count).into());
        }
        self.slice_exact(index * item_size, item_size)
    }

    /// Validate a fixed-layout total length, then return the selected field slice.
    pub fn fixed_field_exact(
        &self,
        total_size: usize,
        offset: usize,
        size: usize,
    ) -> Result<Self, AccessError> {
        if self.size != total_size {
            return Err(Error::TotalSize(total_size, self.size).into());
        }
        self.slice_exact(offset, size)
    }

    /// Validate a union header and return its item id and payload cursor.
    pub fn union_unpack_exact(&self) -> Result<Union, AccessError> {
        if self.size < NUMBER_SIZE {
            return Err(Error::Header(NUMBER_SIZE, self.size).into());
        }
        let item_id = self.read_number_at(0)?;
        let cursor = self.slice_exact(NUMBER_SIZE, self.size - NUMBER_SIZE)?;
        Ok(Union { item_id, cursor })
    }
}
