use super::{AccessError, Cursor, Error, NUMBER_SIZE};

impl Cursor {
    /// Validate the offset table and retain the selected field in the same pass.
    pub fn table_field_exact(
        &self,
        minimum_fields: usize,
        index: usize,
    ) -> Result<Self, AccessError> {
        if self.size < NUMBER_SIZE {
            return Err(Error::Header(NUMBER_SIZE, self.size).into());
        }
        let size = self.read_number_at(0)?;
        if size != self.size {
            return Err(Error::TotalSize(size, self.size).into());
        }
        if size == NUMBER_SIZE {
            return Err(if minimum_fields == 0 {
                Error::OutOfBound(index, 0)
            } else {
                Error::FieldCount(0)
            }
            .into());
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
        let count = first / NUMBER_SIZE - 1;
        if count < minimum_fields {
            return Err(Error::FieldCount(count).into());
        }

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
                if i == index {
                    field = Some(start..end);
                }
                start = end;
                i += 1;
            }
        }
        if i == index {
            field = Some(start..size);
        }
        let field = field.ok_or(Error::OutOfBound(index, count))?;
        self.slice_exact(field.start, field.len())
    }

    /// Validate a fixed-vector header and total length, then return the selected item.
    pub fn fixvec_item_exact(&self, item_size: usize, index: usize) -> Result<Self, AccessError> {
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
        if index >= count {
            return Err(Error::OutOfBound(index, count).into());
        }
        // `total == self.size` already ruled out overflow for any `index < count`.
        self.slice_exact(NUMBER_SIZE + index * item_size, item_size)
    }
}
