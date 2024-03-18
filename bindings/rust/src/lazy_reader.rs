extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::cmp::min;
use core::convert::{From, TryFrom};

#[derive(Debug)]
pub enum Error {
    Common(String),
    TotalSize(String),
    Header(String),
    Offset(String),
    UnknownItem(String),
    OutOfBound(String),
    FieldCount(String),
    Data(String),
    Overflow(String),
    Read(String),
    Verify(String),
    Unknow(String),
}
impl From<core::convert::Infallible> for Error {
    fn from(value: core::convert::Infallible) -> Self {
        Self::Unknow(format!("conver failed: {:?}", value))
    }
}

pub trait Read {
    // Pull some bytes from this source into the specified buffer with `offset`, returning how many bytes were read.
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, Error>;
}

pub const MAX_CACHE_SIZE: usize = 2048;
pub const MIN_CACHE_SIZE: usize = 64;
pub const NUMBER_SIZE: usize = 4;

pub struct DataSource {
    reader: Box<dyn Read>,
    total_size: usize,
    cache_start_point: usize,
    // cache actual size may be smaller than cache.len()
    cache_actual_size: usize,
    cache: Vec<u8>,
}

impl DataSource {
    pub fn new(total_size: usize, reader: Box<dyn Read>) -> Self {
        DataSource {
            reader,
            total_size,
            cache_start_point: 0,
            cache_actual_size: 0,
            cache: vec![0u8; MAX_CACHE_SIZE],
        }
    }

    // Pull some bytes from this source into the specified buffer with `offset` and `read_len`, returning how many bytes were read.
    // If the requested range is out of bound, an `Error::Read` will be returned.
    pub fn read_at(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        read_len: usize,
    ) -> Result<usize, Error> {
        // Read directly if the requested length is larger than maximum cache size
        if read_len > self.cache.len() {
            return self.reader.read(buf, offset);
        }
        // Check if the requested data is in cache
        if offset >= self.cache_start_point
            && offset + read_len <= self.cache_start_point + self.cache_actual_size
        {
            let read_point = offset - self.cache_start_point;
            buf.copy_from_slice(&self.cache[read_point..(read_point + read_len)]);
            return Ok(read_len);
        }
        // Cache miss, read from reader and update cache
        let read_actual_size = self.reader.read(&mut self.cache[..], offset)?;
        self.cache_start_point = offset;
        self.cache_actual_size = read_actual_size;
        if read_actual_size < read_len {
            return Err(Error::Read(format!(
                "read_at: read_actual_size({}) < read_len({})",
                read_actual_size, read_len
            )));
        }
        buf[..read_len].copy_from_slice(&self.cache[0..read_len]);
        Ok(read_len)
    }
}

#[derive(Clone)]
pub struct Cursor {
    pub offset: usize,
    pub size: usize,
    pub data_source: Rc<RefCell<DataSource>>,
}

pub struct Union {
    pub item_id: usize,
    pub cursor: Cursor,
}

impl Cursor {
    /**
    total_size: the size of cursor. If it's set a smaller value,
    `out of bound` will occur when `reader` try to read the data beyond that.
    reader: interface to read underlying data
     */
    pub fn new(total_size: usize, reader: Box<dyn Read>) -> Self {
        let data_source = DataSource::new(total_size, reader);
        Cursor {
            offset: 0,
            size: total_size,
            data_source: Rc::new(RefCell::new(data_source)),
        }
    }

    pub fn read_at(&self, buf: &mut [u8]) -> Result<usize, Error> {
        let read_len = min(self.size, buf.len());
        let mut data_source = self.data_source.borrow_mut();
        data_source.read_at(buf, self.offset, read_len)
    }

    pub fn add_offset(&mut self, offset: usize) -> Result<(), Error> {
        self.offset = self.offset.checked_add(offset).ok_or_else(|| {
            Error::Overflow(format!(
                "add_offset: self.offset({}) + offset({})",
                self.offset, offset
            ))
        })?;
        Ok(())
    }

    pub fn sub_size(&mut self, shrink_size: usize) -> Result<(), Error> {
        self.size = self.size.checked_sub(shrink_size).ok_or_else(|| {
            Error::Overflow(format!(
                "sub_size: self.size({}) - shrink_size({})",
                self.size, shrink_size
            ))
        })?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), Error> {
        if let Some(size) = self.offset.checked_add(self.size) {
            if size > self.data_source.borrow().total_size {
                Err(Error::TotalSize(format!(
                    "validate: size({}) > total_size({})",
                    size,
                    self.data_source.borrow().total_size
                )))
            } else {
                Ok(())
            }
        } else {
            Err(Error::Overflow("validate".into()))
        }
    }

    pub fn unpack_number(&self) -> Result<usize, Error> {
        let mut src = [0u8; 4];
        let size = self.read_at(&mut src[..])?;
        if size != 4 {
            Err(Error::FieldCount(format!(
                "unpack_number: size({}) != 4",
                size
            )))
        } else {
            let res = u32::from_le_bytes(src);
            Ok(res as usize)
        }
    }
    pub fn verify_fixed_size(&self, size: usize) -> Result<(), Error> {
        if self.size != size {
            return Err(Error::Header(format!(
                "verify_fixed: self.size({}) != size({})",
                self.size, size
            )));
        }
        Ok(())
    }
    pub fn verify_table(&self, expected_field_count: usize, compatible: bool) -> Result<(), Error> {
        self.verify_dynvec()?;
        let mut cur = self.clone();
        cur.add_offset(NUMBER_SIZE)?;
        if self.size == cur.size {
            // empty table
            return Ok(());
        }
        let first_offset = cur.unpack_number()?;
        let field_count = first_offset / NUMBER_SIZE - 1;
        if field_count < expected_field_count {
            return Err(Error::Verify(format!(
                "field_count({}) < expected_field_count({})",
                field_count, expected_field_count
            )));
        } else if !compatible && field_count > expected_field_count {
            return Err(Error::Verify(format!(
                "field_count({}) > expected_field_count({})",
                field_count, expected_field_count
            )));
        };
        Ok(())
    }

    pub fn verify_dynvec(&self) -> Result<(), Error> {
        let total_size = self.unpack_number()?;
        if self.size != total_size {
            return Err(Error::Verify(format!(
                "verify_dynvec: self.size({}) != total_size({})",
                self.size, total_size
            )));
        }
        if total_size == NUMBER_SIZE {
            return Ok(());
        }
        if total_size < NUMBER_SIZE * 2 {
            return Err(Error::Verify(format!(
                "verify_dynvec: total_size({}) < 8",
                total_size
            )));
        }
        let mut cur = self.clone();
        cur.add_offset(NUMBER_SIZE)?;
        let first_offset = cur.unpack_number()?;
        if first_offset % NUMBER_SIZE != 0 || first_offset < NUMBER_SIZE * 2 {
            return Err(Error::Verify(format!(
                "verify_dynvec: invalid first_offset({})",
                first_offset
            )));
        }
        if total_size < first_offset {
            return Err(Error::Verify(format!(
                "verify_dynvec: invalid first_offset({}), total_size({})",
                first_offset, total_size
            )));
        }
        // offsets are ordered increasingly
        let count = first_offset / 4 - 1;
        let mut last_offset = None;
        for _ in 0..count {
            let offset = cur.unpack_number()?;
            if last_offset.is_some() && last_offset.unwrap() > offset {
                return Err(Error::Verify(format!(
                    "verify_dynvec: invalid offset({} and {})",
                    last_offset.unwrap(),
                    offset
                )));
            }
            last_offset = Some(offset);
            cur.add_offset(NUMBER_SIZE)?;
        }
        Ok(())
    }
    pub fn verify_fixvec(&self, item_size: usize) -> Result<(), Error> {
        if self.size < NUMBER_SIZE {
            return Err(Error::Verify(format!(
                "verify_fixvec: self.size({}) < NUMBER_SIZE(4)",
                self.size
            )));
        }
        let item_count = self.unpack_number()?;
        if item_count == 0 {
            if self.size == NUMBER_SIZE {
                return Ok(());
            } else {
                return Err(Error::Verify(format!(
                    "verify_fixvec: self.size({}) != 4",
                    self.size
                )));
            }
        }

        let total_size = calculate_offset(item_size, item_count, NUMBER_SIZE)?;
        if self.size == total_size {
            Ok(())
        } else {
            Err(Error::Verify(format!(
                "verify_fixvec: self.size({}) != total_size({})",
                self.size, total_size
            )))
        }
    }

    pub fn option_is_none(&self) -> bool {
        self.size == 0
    }
    pub fn fixvec_length(&self) -> Result<usize, Error> {
        self.unpack_number()
    }

    pub fn dynvec_length(&self) -> Result<usize, Error> {
        if self.size == NUMBER_SIZE {
            Ok(0)
        } else {
            let mut cur2 = self.clone();
            cur2.add_offset(NUMBER_SIZE)?;
            cur2.sub_size(NUMBER_SIZE)?;
            cur2.validate()?;
            cur2.get_item_count()
        }
    }

    pub fn get_item_count(&self) -> Result<usize, Error> {
        let len = self.unpack_number()?;
        if len % 4 != 0 {
            return Err(Error::UnknownItem(format!(
                "get_item_count: not aligned, len = {}",
                len
            )));
        }
        let count = len / 4;
        if count == 0 {
            Err(Error::UnknownItem("get_item_count".into()))
        } else {
            Ok(count - 1)
        }
    }

    pub fn table_actual_field_count(&self) -> Result<usize, Error> {
        self.dynvec_length()
    }

    pub fn table_has_extra_fields(&self, field_count: usize) -> Result<bool, Error> {
        let count = self.table_actual_field_count()?;
        Ok(count > field_count)
    }

    pub fn slice_by_offset(&self, offset: usize, size: usize) -> Result<Cursor, Error> {
        let mut cur2 = self.clone();
        cur2.add_offset(offset)?;
        cur2.size = size;
        cur2.validate()?;
        Ok(cur2)
    }

    pub fn fixvec_slice_by_index(
        &self,
        item_size: usize,
        item_index: usize,
    ) -> Result<Cursor, Error> {
        let mut cur2 = self.clone();
        let item_count = self.unpack_number()?;
        if item_index >= item_count {
            Err(Error::OutOfBound(format!(
                "fixvec_slice_by_index: item_index({}) >= item_count({})",
                item_index, item_count
            )))
        } else {
            let offset = calculate_offset(item_size, item_index, NUMBER_SIZE)?;
            cur2.add_offset(offset)?;
            cur2.size = item_size;
            cur2.validate()?;
            Ok(cur2)
        }
    }

    pub fn dynvec_slice_by_index(&self, item_index: usize) -> Result<Cursor, Error> {
        let mut res = self.clone();
        let mut temp = self.clone();
        let total_size = self.unpack_number()?;
        temp.add_offset(NUMBER_SIZE)?;
        let item_count = temp.get_item_count()?;
        if item_index >= item_count {
            return Err(Error::OutOfBound(format!(
                "dynvec_slice_by_index: item_index({}) >= item_count({})",
                item_index, item_count
            )));
        }
        temp.offset = self.offset;
        let temp_offset = calculate_offset(NUMBER_SIZE, item_index + 1, 0)?;
        temp.add_offset(temp_offset)?;
        let item_start = temp.unpack_number()?;
        if (item_index + 1) == item_count {
            res.offset = self.offset;
            res.add_offset(item_start)?;
            res.size = total_size;
            res.sub_size(item_start)?;
        } else {
            temp.offset = self.offset;
            let calc_offset = calculate_offset(NUMBER_SIZE, item_index + 2, 0)?;
            temp.add_offset(calc_offset)?;

            let item_end = temp.unpack_number()?;
            res.offset = self.offset;
            res.add_offset(item_start)?;
            res.size = item_end;
            res.sub_size(item_start)?;
        }
        res.validate()?;
        Ok(res)
    }

    pub fn table_slice_by_index(&self, field_index: usize) -> Result<Cursor, Error> {
        self.dynvec_slice_by_index(field_index)
    }

    pub fn fixvec_slice_raw_bytes(&self) -> Result<Cursor, Error> {
        let mut res = self.clone();
        res.add_offset(NUMBER_SIZE)?;
        res.size = self.unpack_number()?;
        res.validate()?;
        Ok(res)
    }

    pub fn convert_to_array(&self) -> Result<Cursor, Error> {
        Ok(self.clone())
    }

    pub fn convert_to_rawbytes(&self) -> Result<Cursor, Error> {
        self.fixvec_slice_raw_bytes()
    }

    pub fn union_unpack(&self) -> Result<Union, Error> {
        let item_id = self.unpack_number()?;
        let mut cursor = self.clone();
        cursor.add_offset(NUMBER_SIZE)?;
        cursor.sub_size(NUMBER_SIZE)?;
        cursor.validate()?;
        Ok(Union { item_id, cursor })
    }
}

fn calculate_offset(item_size: usize, item_count: usize, offset: usize) -> Result<usize, Error> {
    let res = item_size.checked_mul(item_count).ok_or_else(|| {
        Error::Overflow(format!(
            "calculate_offset: item_size({}) * item_count({}) overflow",
            item_size, item_count
        ))
    })?;
    res.checked_add(offset)
        .ok_or_else(|| Error::Overflow(format!("calculate_offset: offset({}) overflow", offset)))
}

macro_rules! impl_cursor_primitive {
    ($type: ty) => {
        impl TryFrom<Cursor> for $type {
            type Error = Error;
            fn try_from(cur: Cursor) -> Result<Self, Error> {
                let mut buf = [0u8; (<$type>::BITS / 8) as usize];
                let size = cur.read_at(&mut buf[..])?;
                if size != buf.len() {
                    Err(Error::FieldCount(format!(
                        "TryFrom<Cursor>: convert {} bytes to primitive",
                        <$type>::BITS / 8
                    )))
                } else {
                    Ok(<$type>::from_le_bytes(buf))
                }
            }
        }
    };
}

impl_cursor_primitive!(u64);
impl_cursor_primitive!(i64);
impl_cursor_primitive!(u32);
impl_cursor_primitive!(i32);
impl_cursor_primitive!(u16);
impl_cursor_primitive!(i16);
impl_cursor_primitive!(u8);
impl_cursor_primitive!(i8);

impl TryFrom<Cursor> for Vec<u8> {
    type Error = Error;
    fn try_from(cur: Cursor) -> Result<Self, Error> {
        let mut buf = vec![0u8; cur.size];

        let size = cur.read_at(&mut buf[..])?;
        if size != buf.len() {
            return Err(Error::Read(format!(
                "TryFrom<Cursor>: size({}) != buf.len()({})",
                size,
                buf.len()
            )));
        }
        Ok(buf)
    }
}

impl<const N: usize> TryFrom<Cursor> for [u8; N] {
    type Error = Error;
    fn try_from(cur: Cursor) -> Result<Self, Error> {
        let mut buf = [0u8; N];

        let size = cur.read_at(&mut buf[..])?;
        if size != N || size != cur.size {
            return Err(Error::Read(format!(
                "TryFrom<Cursor>: size({}) != buf.len()({})",
                size,
                buf.len()
            )));
        }

        Ok(buf)
    }
}

// it's an example about how to build a data source from memory
impl Read for Vec<u8> {
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, Error> {
        let mem_len = self.len();
        if offset >= mem_len {
            return Err(Error::OutOfBound(format!(
                "read: offset({}) >= mem_len({})",
                offset, mem_len
            )));
        }

        let remaining_len = mem_len - offset;
        let min_len = min(remaining_len, buf.len());

        if (offset + min_len) > mem_len {
            return Err(Error::OutOfBound(format!(
                "read: (offset({}) + min_len({})) > mem_len({})",
                offset, min_len, mem_len
            )));
        }
        buf[0..min_len].copy_from_slice(&self.as_slice()[offset..offset + min_len]);
        Ok(min_len)
    }
}

impl From<Vec<u8>> for Cursor {
    fn from(mem: Vec<u8>) -> Self {
        Cursor::new(mem.len(), Box::new(mem))
    }
}

impl<const N: usize> From<[u8; N]> for Cursor {
    fn from(mem: [u8; N]) -> Self {
        Cursor::new(mem.len(), Box::new(mem.to_vec()))
    }
}
// end of example
