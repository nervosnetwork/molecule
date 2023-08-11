extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
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
}
pub trait Read {
    /**
     * try to read `buf.len()` bytes from data source with `offset`, then fill it in `buf`.
     * the return size can be smaller than `buf.len()` which means the remaining data length is
     * smaller than `buf.len()`
     */
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, Error>;
}

pub const MAX_CACHE_SIZE: usize = 2048;
pub const MIN_CACHE_SIZE: usize = 64;
pub const NUM_T_SIZE: usize = 4;

pub struct DataSource {
    reader: Box<dyn Read>,

    total_size: usize,
    cache_start_point: usize,
    // cache size may be smaller than cache.len()
    cache_size: usize,
    cache: Vec<u8>,
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

pub fn read_at(cur: &Cursor, buf: &mut [u8]) -> Result<usize, Error> {
    let read_len = min(cur.size, buf.len());
    let ds = &mut *cur.data_source.borrow_mut();
    if read_len > ds.cache.len() {
        return ds.reader.read(buf, cur.offset);
    }
    if cur.offset < ds.cache_start_point
        || (cur.offset + read_len) > (ds.cache_start_point + ds.cache_size)
    {
        let reader = &ds.reader;
        let size = reader.read(&mut ds.cache[..], cur.offset).unwrap();
        if size < read_len {
            return Err(Error::Read(format!(
                "read_at `if size({}) < read_len({})`",
                size, read_len
            )));
        }
        ds.cache_size = size;
        ds.cache_start_point = cur.offset;

        if ds.cache_size > ds.cache.len() {
            return Err(Error::Read(format!(
                "read_at `if ds.cache_size({}) > ds.cache.len()({})`",
                ds.cache_size,
                ds.cache.len()
            )));
        }
    }
    if cur.offset < ds.cache_start_point || (cur.offset - ds.cache_start_point) > ds.cache.len() {
        return Err(Error::Read(
            "read_at `if cur.offset < ds.start_point || ...`".into(),
        ));
    }
    let read_point = cur.offset - ds.cache_start_point;
    if read_point + read_len > ds.cache_size {
        return Err(Error::Read(
            "read_at `if read_point + read_len > ds.cache_size`".into(),
        ));
    }
    buf.copy_from_slice(&ds.cache[read_point..(read_point + read_len)]);
    Ok(read_len)
}

impl Cursor {
    /**
    cache_size: normally it can be set to MAX_CACHE_SIZE(2K)
    total_size: the size of cursor. If it's set a smaller value,
    `out of bound` will occur when `reader` try to read the data beyond that.
    reader: interface to read underlying data
     */
    pub fn new(total_size: usize, reader: Box<dyn Read>) -> Self {
        let data_source = DataSource {
            reader,
            total_size,
            cache_start_point: 0,
            cache_size: 0, // when created, cache is not filled
            cache: vec![0u8; MAX_CACHE_SIZE],
        };
        Cursor {
            offset: 0,
            size: total_size,
            data_source: Rc::new(RefCell::new(data_source)),
        }
    }

    pub fn add_offset(&mut self, offset: usize) {
        self.offset = self.offset.checked_add(offset).unwrap();
    }

    pub fn sub_size(&mut self, shrink_size: usize) {
        self.size = self.size.checked_sub(shrink_size).unwrap();
    }

    pub fn validate(&self) -> Result<(), Error> {
        if let Some(size) = self.offset.checked_add(self.size) {
            if size > self.data_source.borrow().total_size {
                Err(Error::TotalSize(
                    "validate: size > cur.source.total_size".into(),
                ))
            } else {
                Ok(())
            }
        } else {
            Err(Error::Overflow("validate".into()))
        }
    }

    pub fn unpack_number(&self) -> Result<usize, Error> {
        let mut src = [0u8; 4];
        let size = read_at(self, &mut src[..]).unwrap();
        if size != 4 {
            Err(Error::FieldCount("unpack_number".into()))
        } else {
            let res = u32::from_le_bytes(src);
            Ok(res as usize)
        }
    }

    pub fn verify_fixed_size(&self, total_size: usize) -> Result<(), Error> {
        if self.size == total_size {
            Ok(())
        } else {
            Err(Error::TotalSize(format!(
                "self.size({}) == total_size({})",
                self.size, total_size
            )))
        }
    }

    pub fn fixvec_verify(&self, item_size: usize) -> Result<(), Error> {
        if self.size < NUM_T_SIZE {
            return Err(Error::FieldCount(format!(
                "fixvec_verify, self.size({}) < NUM_T_SIZE",
                self.size
            )));
        }
        let item_count = self.unpack_number()?;
        if item_count == 0 {
            if self.size == NUM_T_SIZE {
                return Ok(());
            } else {
                return Err(Error::Header(format!(
                    "self.size({}) == NUM_T_SIZE",
                    self.size
                )));
            }
        }

        let total_size = calculate_offset(item_size, item_count, NUM_T_SIZE);
        if self.size == total_size {
            Ok(())
        } else {
            Err(Error::TotalSize(format!(
                "self.size({}) == total_size({})",
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
        if self.size == NUM_T_SIZE {
            Ok(0)
        } else {
            let mut cur2 = self.clone();
            cur2.add_offset(NUM_T_SIZE);
            cur2.sub_size(NUM_T_SIZE);
            cur2.validate()?;
            cur2.get_item_count()
        }
    }

    pub fn get_item_count(&self) -> Result<usize, Error> {
        let count = self.unpack_number()? / 4;
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
        cur2.add_offset(offset);
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
                "item_index({}) >= item_count({})",
                item_index, item_count
            )))
        } else {
            let offset = calculate_offset(item_size, item_index, NUM_T_SIZE);
            cur2.add_offset(offset);
            cur2.size = item_size;
            cur2.validate()?;
            Ok(cur2)
        }
    }

    pub fn dynvec_slice_by_index(&self, item_index: usize) -> Result<Cursor, Error> {
        let mut res = self.clone();
        let mut temp = self.clone();
        let total_size = self.unpack_number()?;
        temp.add_offset(NUM_T_SIZE);
        let item_count = temp.get_item_count()?;
        if item_index >= item_count {
            return Err(Error::OutOfBound(format!(
                "item_index({}) >= item_count({})",
                item_index, item_count
            )));
        }
        temp.offset = self.offset;
        let temp_offset = calculate_offset(NUM_T_SIZE, item_index + 1, 0);
        temp.add_offset(temp_offset);
        let item_start = temp.unpack_number()?;
        if (item_index + 1) == item_count {
            res.offset = self.offset;
            res.add_offset(item_start);
            res.size = total_size;
            res.sub_size(item_start)
        } else {
            temp.offset = self.offset;
            let calc_offset = calculate_offset(NUM_T_SIZE, item_index + 2, 0);
            temp.add_offset(calc_offset);

            let item_end = temp.unpack_number()?;
            res.offset = self.offset;
            res.add_offset(item_start);
            res.size = item_end;
            res.sub_size(item_start);
        }
        res.validate()?;
        Ok(res)
    }

    pub fn table_slice_by_index(&self, field_index: usize) -> Result<Cursor, Error> {
        self.dynvec_slice_by_index(field_index)
    }

    pub fn fixvec_slice_raw_bytes(&self) -> Result<Cursor, Error> {
        let mut res = self.clone();
        res.add_offset(NUM_T_SIZE);
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
        cursor.add_offset(NUM_T_SIZE);
        cursor.sub_size(NUM_T_SIZE);
        cursor.validate()?;
        Ok(Union { item_id, cursor })
    }
}

fn calculate_offset(item_size: usize, item_count: usize, offset: usize) -> usize {
    let res = item_size.checked_mul(item_count).unwrap();
    res.checked_add(offset).unwrap()
}

impl TryFrom<Cursor> for u64 {
    type Error = Error;
    fn try_from(cur: Cursor) -> Result<Self, Error> {
        let mut buf = [0u8; 8];
        let size = read_at(&cur, &mut buf[..])?;
        if size != buf.len() {
            Err(Error::FieldCount("convert_to_u64".into()))
        } else {
            Ok(u64::from_le_bytes(buf))
        }
    }
}

impl TryFrom<Cursor> for i64 {
    type Error = Error;
    fn try_from(cur: Cursor) -> Result<Self, Error> {
        let mut buf = [0u8; 8];
        let size = read_at(&cur, &mut buf[..])?;
        if size != buf.len() {
            Err(Error::FieldCount("convert_to_i64".into()))
        } else {
            Ok(i64::from_le_bytes(buf))
        }
    }
}

impl TryFrom<Cursor> for u32 {
    type Error = Error;
    fn try_from(cur: Cursor) -> Result<Self, Error> {
        let mut buf = [0u8; 4];
        let size = read_at(&cur, &mut buf[..])?;
        if size != buf.len() {
            Err(Error::FieldCount("convert_to_u32".into()))
        } else {
            Ok(u32::from_le_bytes(buf))
        }
    }
}

impl TryFrom<Cursor> for i32 {
    type Error = Error;
    fn try_from(cur: Cursor) -> Result<Self, Error> {
        let mut buf = [0u8; 4];
        let size = read_at(&cur, &mut buf[..])?;
        if size != buf.len() {
            Err(Error::FieldCount("convert_to_i32".into()))
        } else {
            Ok(i32::from_le_bytes(buf))
        }
    }
}

impl TryFrom<Cursor> for u16 {
    type Error = Error;
    fn try_from(cur: Cursor) -> Result<Self, Error> {
        let mut buf = [0u8; 2];
        let size = read_at(&cur, &mut buf[..])?;
        if size != buf.len() {
            Err(Error::FieldCount("convert_to_u16".into()))
        } else {
            Ok(u16::from_le_bytes(buf))
        }
    }
}

impl TryFrom<Cursor> for i16 {
    type Error = Error;
    fn try_from(cur: Cursor) -> Result<Self, Error> {
        let mut buf = [0u8; 2];
        let size = read_at(&cur, &mut buf[..])?;
        if size != buf.len() {
            Err(Error::FieldCount("convert_to_i16".into()))
        } else {
            Ok(i16::from_le_bytes(buf))
        }
    }
}

impl TryFrom<Cursor> for u8 {
    type Error = Error;
    fn try_from(cur: Cursor) -> Result<Self, Error> {
        let mut buf = [0u8; 1];
        let size = read_at(&cur, &mut buf[..])?;
        if size != buf.len() {
            Err(Error::FieldCount("convert_to_u8".into()))
        } else {
            Ok(u8::from_le_bytes(buf))
        }
    }
}

impl TryFrom<Cursor> for i8 {
    type Error = Error;
    fn try_from(cur: Cursor) -> Result<Self, Error> {
        let mut buf = [0u8; 1];
        let size = read_at(&cur, &mut buf[..])?;
        if size != buf.len() {
            Err(Error::FieldCount("convert_to_i8".into()))
        } else {
            Ok(i8::from_le_bytes(buf))
        }
    }
}

impl TryFrom<Cursor> for Vec<u8> {
    type Error = Error;
    fn try_from(cur: Cursor) -> Result<Self, Error> {
        let mut buf = Vec::<u8>::new();
        buf.resize(cur.size, 0);

        let size = read_at(&cur, buf.as_mut_slice())?;
        if size != buf.len() {
            return Err(Error::Read(format!(
                "size({}) != buf.len()({})",
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
                "offset({}) >= mem_len({})",
                offset, mem_len
            )));
        }

        let remaining_len = mem_len - offset;
        let min_len = min(remaining_len, buf.len());

        if (offset + min_len) > mem_len {
            return Err(Error::OutOfBound(
                "(offset + min_len) > mem_len".to_string(),
            ));
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
// end of example
