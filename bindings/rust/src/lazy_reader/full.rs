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
        if size == NUMBER_SIZE && minimum_fields == 0 {
            return Err(Error::OutOfBound(index, 0).into());
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
        for i in 0..count {
            let end = if i + 1 < count {
                self.read_number_at(NUMBER_SIZE * (i + 2))?
            } else {
                size
            };
            if end < start || end > size {
                return Err(Error::Offset.into());
            }
            if i == index {
                field = Some(start..end);
            }
            start = end;
        }
        let field = field.ok_or(Error::OutOfBound(index, count))?;
        self.slice_exact(field.start, field.len())
    }
}
