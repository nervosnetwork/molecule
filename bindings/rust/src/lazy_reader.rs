//! # Molecule lazy reader
//! In the previous implementation, the molecule requires that all data be
//! loaded into memory before deserialization. This is a significant limitation
//! in on-chain scripts as ckb-vm only has 4M memory. This feature
//! implementation aims to resolve this issue by implementing a lazy reader.
//!
//! If we examine the [molecule
//! spec](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0008-serialization/0008-serialization.md),
//! specific data can be retrieved by navigating through "hops". By reading
//! only the header, we can estimate where to navigate and avoid reading the
//! rest of the data. In many scenarios where only certain parts of the data are
//! required, a lazy reader mechanism can be utilized.
//!
//! Here is an example about how to make a lazy reader from transaction:
//!```
//!use blockchain;
//!use alloc::boxed::Box;
//!use ckb_std::{ckb_constants::Source, error::SysError, syscalls};
//!pub use molecule::lazy_reader::{Cursor, Error, Read};
//!fn read_data<F: Fn(&mut [u8], usize) -> Result<usize, SysError>>(
//!    load_func: F,
//!    buf: &mut [u8],
//!    offset: usize,
//!    total_size: usize,
//!) -> Result<usize, Error> {
//!    if offset >= total_size {
//!        return Err(Error::OutOfBound(offset, total_size));
//!    }
//!    let remaining_len = total_size - offset;
//!    let min_len = min(remaining_len, buf.len());
//!    if (offset + min_len) > total_size {
//!        return Err(Error::OutOfBound(offset + min_len, total_size));
//!    }
//!    let actual_len = match load_func(buf, offset) {
//!        Ok(l) => l,
//!        Err(err) => match err {
//!            SysError::LengthNotEnough(l) => l,
//!            _ => return Err(Error::OutOfBound(0, 0)),
//!        },
//!    };
//!    let read_len = min(buf.len(), actual_len);
//!    Ok(read_len)
//!}
//!fn read_size<F: Fn(&mut [u8]) -> Result<usize, SysError>>(load_func: F) -> Result<usize, Error> {
//!    let mut buf = [0u8; 4];
//!    match load_func(&mut buf) {
//!        Ok(l) => Ok(l),
//!        Err(e) => match e {
//!            SysError::LengthNotEnough(l) => Ok(l),
//!            _ => Err(Error::OutOfBound(0, 0)),
//!        },
//!    }
//!}
//!pub struct TransactionReader {
//!    pub total_size: usize,
//!}
//!impl TransactionReader {
//!    pub fn new() -> Self {
//!        let total_size = read_size(|buf| syscalls::load_transaction(buf, 0)).unwrap();
//!        Self { total_size }
//!    }
//!}
//!impl Read for TransactionReader {
//!    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, Error> {
//!        read_data(
//!            |buf, offset| syscalls::load_transaction(buf, offset),
//!            buf,
//!            offset,
//!            self.total_size,
//!        )
//!    }
//!}
//!impl From<TransactionReader> for Cursor {
//!    fn from(data: TransactionReader) -> Self {
//!        Cursor::new(data.total_size, Box::new(data))
//!    }
//!}
//!pub fn new_transaction() -> blockchain::Transaction {
//!    let tx_reader = TransactionReader::new();
//!    let cursor: Cursor = tx_reader.into();
//!    blockchain::Transaction::from(cursor)
//!}
//! ```
//!
extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::cmp::min;
use core::convert::{From, TryFrom};

#[derive(Debug)]
pub enum Error {
    Common,
    TotalSize(usize, usize),
    Header(usize, usize),
    Offset,
    UnknownItem,
    OutOfBound(usize, usize),
    FieldCount(usize),
    Data,
    Overflow,
    Read(usize, usize),
    Verify,
    Unknown,
}

impl From<core::convert::Infallible> for Error {
    fn from(_: core::convert::Infallible) -> Self {
        Self::Unknown
    }
}

///
/// To make a lazy reader cursor from scratch, this trait must be implemented.
/// See `Cursor::new` about how to create a cursor based on this trait.
///
pub trait Read {
    /// Pull some bytes from this source into the specified buffer with
    /// `offset`, returning how many bytes were read.
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, Error>;
}

pub const MAX_CACHE_SIZE: usize = 2048;
pub const MIN_CACHE_SIZE: usize = 64;
pub const NUMBER_SIZE: usize = 4;

///
/// A data source structure keeps internal state: cache, total size and reader.
///
pub struct DataSource {
    reader: Box<dyn Read>,
    total_size: usize,
    cache_start_point: usize,
    // cache actual size may be smaller than cache.len()
    cache_actual_size: usize,
    cache: Vec<u8>,
}

impl DataSource {
    ///
    /// Create a data source from `reader` and corresponding `total_size`
    ///
    pub fn new(total_size: usize, reader: Box<dyn Read>) -> Self {
        DataSource {
            reader,
            total_size,
            cache_start_point: 0,
            cache_actual_size: 0,
            cache: vec![0u8; MAX_CACHE_SIZE],
        }
    }

    /// Pull some bytes from this source into the specified buffer with `offset`
    /// and `read_len`, returning how many bytes were read. If the requested
    /// range is out of bound, an `Error::Read` will be returned.
    pub fn read_at(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        read_len: usize,
    ) -> Result<usize, Error> {
        // Read directly if the requested length is larger than maximum cache size
        if read_len == 0 {
            return Ok(0);
        }
        if read_len > self.cache.len() {
            return self.reader.read(buf, offset);
        }
        // Check if the requested data is in cache
        if offset >= self.cache_start_point
            && offset + read_len <= self.cache_start_point + self.cache_actual_size
        {
            let read_point = offset - self.cache_start_point;
            buf[..read_len].copy_from_slice(&self.cache[read_point..(read_point + read_len)]);
            return Ok(read_len);
        }
        // Cache miss, read from reader and update cache
        let read_actual_size = self.reader.read(&mut self.cache[..], offset)?;
        self.cache_start_point = offset;
        self.cache_actual_size = read_actual_size;
        if read_actual_size < read_len {
            return Err(Error::Read(read_actual_size, read_len));
        }
        buf[..read_len].copy_from_slice(&self.cache[0..read_len]);
        Ok(read_len)
    }
}

///
/// The Cursor represents a slice or view of data without actually being loaded into memory.
/// It is a slice of data source with range `[offset, offset + size)`
///
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
    ///
    /// Create a cursor from `reader` and its corresponding total size
    ///
    pub fn new(total_size: usize, reader: Box<dyn Read>) -> Self {
        let data_source = DataSource::new(total_size, reader);
        Cursor {
            offset: 0,
            size: total_size,
            data_source: Rc::new(RefCell::new(data_source)),
        }
    }
    ///
    /// Read from a cursor into `buf` and returns actually read size.
    ///
    pub fn read_at(&self, buf: &mut [u8]) -> Result<usize, Error> {
        let read_len = min(self.size, buf.len());
        let mut data_source = self.data_source.borrow_mut();
        data_source.read_at(buf, self.offset, read_len)
    }

    ///
    /// Move `offset` forward and shrink a cursor from beginning.
    ///
    pub fn add_offset(&mut self, offset: usize) -> Result<(), Error> {
        self.offset = self.offset.checked_add(offset).ok_or(Error::Overflow)?;
        Ok(())
    }
    ///
    /// Shrink a cursor from end.
    ///
    pub fn sub_size(&mut self, shrink_size: usize) -> Result<(), Error> {
        self.size = self.size.checked_sub(shrink_size).ok_or(Error::Overflow)?;
        Ok(())
    }

    ///
    /// Validate a cursor to ensure that size and offset are not out of bounds.
    ///
    pub fn validate(&self) -> Result<(), Error> {
        if let Some(size) = self.offset.checked_add(self.size) {
            if size > self.data_source.borrow().total_size {
                Err(Error::TotalSize(size, self.data_source.borrow().total_size))
            } else {
                Ok(())
            }
        } else {
            Err(Error::Overflow)
        }
    }

    ///
    /// Read the first 4 bytes and unpack them into a u32 in little endian format.
    ///
    pub fn unpack_number(&self) -> Result<usize, Error> {
        let mut src = [0u8; 4];
        let size = self.read_at(&mut src[..])?;
        if size != 4 {
            Err(Error::FieldCount(size))
        } else {
            let res = u32::from_le_bytes(src);
            Ok(res as usize)
        }
    }
    ///
    /// Verify that a cursor has size bytes.
    ///
    pub fn verify_fixed_size(&self, size: usize) -> Result<(), Error> {
        if self.size != size {
            return Err(Error::Header(self.size, size));
        }
        Ok(())
    }
    ///
    /// Verify that a cursor is a valid molecule `table` with
    /// `expected_field_count` fields. if `compatible` is true, actual fields
    /// count can be larger than `expected_field_count`.
    ///  
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
        if field_count < expected_field_count || !compatible && field_count > expected_field_count {
            return Err(Error::Verify);
        };
        Ok(())
    }

    ///
    /// Verify that a cursor is a valid molecule `dynvec`
    ///
    pub fn verify_dynvec(&self) -> Result<(), Error> {
        let total_size = self.unpack_number()?;
        if self.size != total_size {
            return Err(Error::Verify);
        }
        if total_size == NUMBER_SIZE {
            return Ok(());
        }
        if total_size < NUMBER_SIZE * 2 {
            return Err(Error::Verify);
        }
        let mut cur = self.clone();
        cur.add_offset(NUMBER_SIZE)?;
        let first_offset = cur.unpack_number()?;
        if first_offset % NUMBER_SIZE != 0 || first_offset < NUMBER_SIZE * 2 {
            return Err(Error::Verify);
        }
        if total_size < first_offset {
            return Err(Error::Verify);
        }
        // offsets are ordered increasingly
        let count = first_offset / 4 - 1;
        let mut last_offset = None;
        for _ in 0..count {
            let offset = cur.unpack_number()?;
            if last_offset.is_some() && last_offset.unwrap() > offset {
                return Err(Error::Verify);
            }
            last_offset = Some(offset);
            cur.add_offset(NUMBER_SIZE)?;
        }
        Ok(())
    }
    ///
    /// Verify that a cursor is a valid molecule `fixvec`
    ///
    pub fn verify_fixvec(&self, item_size: usize) -> Result<(), Error> {
        if self.size < NUMBER_SIZE {
            return Err(Error::Verify);
        }
        let item_count = self.unpack_number()?;
        if item_count == 0 {
            if self.size == NUMBER_SIZE {
                return Ok(());
            } else {
                return Err(Error::Verify);
            }
        }

        let total_size = calculate_offset(item_size, item_count, NUMBER_SIZE)?;
        if self.size == total_size {
            Ok(())
        } else {
            Err(Error::Verify)
        }
    }

    ///
    /// Verify that a cursor is with zero size
    ///
    pub fn option_is_none(&self) -> bool {
        self.size == 0
    }
    ///
    /// Assuming a cursor is a fixvec, return the length of the fixvec.
    ///
    pub fn fixvec_length(&self) -> Result<usize, Error> {
        self.unpack_number()
    }
    ///
    /// Assuming a cursor is a dynvec, return the length of the dynvec.
    ///
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
    ///
    /// Assume a cursor is `fixvec`, `dynvec`, or `table`, return the item count.
    ///
    /// See [molecule memory layout](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0008-serialization/0008-serialization.md#memory-layout)
    ///
    pub fn get_item_count(&self) -> Result<usize, Error> {
        let len = self.unpack_number()?;
        if len % 4 != 0 {
            return Err(Error::UnknownItem);
        }
        let count = len / 4;
        if count == 0 {
            Err(Error::UnknownItem)
        } else {
            Ok(count - 1)
        }
    }

    /// Same to `dynvec_length`
    pub fn table_actual_field_count(&self) -> Result<usize, Error> {
        self.dynvec_length()
    }

    ///
    /// Verify a table has extra field larger than `field_count`
    ///
    pub fn table_has_extra_fields(&self, field_count: usize) -> Result<bool, Error> {
        let count = self.table_actual_field_count()?;
        Ok(count > field_count)
    }

    ///
    /// Create a new cursor by adding an `offset` and setting the `size` to that
    /// of the original cursor.    
    ///
    pub fn slice_by_offset(&self, offset: usize, size: usize) -> Result<Cursor, Error> {
        let mut cur2 = self.clone();
        cur2.add_offset(offset)?;
        cur2.size = size;
        cur2.validate()?;
        Ok(cur2)
    }
    ///
    /// Create a new cursor by adding an offset and shrinking the size to that
    /// of the original cursor.
    ///
    pub fn slice_by_start(&self, delta: usize) -> Result<Cursor, Error> {
        let mut cur2 = self.clone();
        cur2.add_offset(delta)?;
        cur2.sub_size(delta)?;
        cur2.validate()?;
        Ok(cur2)
    }

    ///
    /// Assume a cursor is fixvec with item size `item_size`, return an item
    /// with index `item_index`
    ///
    pub fn fixvec_slice_by_index(
        &self,
        item_size: usize,
        item_index: usize,
    ) -> Result<Cursor, Error> {
        let mut cur2 = self.clone();
        let item_count = self.unpack_number()?;
        if item_index >= item_count {
            Err(Error::OutOfBound(item_index, item_count))
        } else {
            let offset = calculate_offset(item_size, item_index, NUMBER_SIZE)?;
            cur2.add_offset(offset)?;
            cur2.size = item_size;
            cur2.validate()?;
            Ok(cur2)
        }
    }
    ///
    /// Assuming a cursor is dynvec, return an item with index `item_index`
    ///
    pub fn dynvec_slice_by_index(&self, item_index: usize) -> Result<Cursor, Error> {
        let mut res = self.clone();
        let mut temp = self.clone();
        let total_size = self.unpack_number()?;
        temp.add_offset(NUMBER_SIZE)?;
        let item_count = temp.get_item_count()?;
        if item_index >= item_count {
            return Err(Error::OutOfBound(item_index, item_count));
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

    /// Assuming a cursor is `table`, return a field with index `field_index`
    pub fn table_slice_by_index(&self, field_index: usize) -> Result<Cursor, Error> {
        self.dynvec_slice_by_index(field_index)
    }

    /// Assuming a cursor is `fixvec`, return raw data without header.
    pub fn fixvec_slice_raw_bytes(&self) -> Result<Cursor, Error> {
        let mut res = self.clone();
        res.add_offset(NUMBER_SIZE)?;
        res.size = self.unpack_number()?;
        res.validate()?;
        Ok(res)
    }

    /// helper function for generated code
    pub fn convert_to_array(&self) -> Result<Cursor, Error> {
        Ok(self.clone())
    }

    /// same to fixvec_slice_raw_bytes
    pub fn convert_to_rawbytes(&self) -> Result<Cursor, Error> {
        self.fixvec_slice_raw_bytes()
    }
    /// Assume a cursor is `union`. Return a `union`.
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
    let res = item_size.checked_mul(item_count).ok_or(Error::Overflow)?;
    res.checked_add(offset).ok_or(Error::Overflow)
}

macro_rules! impl_cursor_primitive {
    ($type: ty) => {
        impl TryFrom<Cursor> for $type {
            type Error = Error;
            fn try_from(cur: Cursor) -> Result<Self, Error> {
                let mut buf = [0u8; (<$type>::BITS / 8) as usize];
                let size = cur.read_at(&mut buf[..])?;
                if size != buf.len() {
                    Err(Error::FieldCount(<$type>::BITS as usize / 8))
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
            return Err(Error::Read(size, buf.len()));
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
            return Err(Error::Read(size, buf.len()));
        }

        Ok(buf)
    }
}

///
/// an example about how to build a cursor from `Vec<u8>`
///
impl Read for Vec<u8> {
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, Error> {
        let mem_len = self.len();
        if offset >= mem_len {
            return Err(Error::OutOfBound(offset, mem_len));
        }

        let remaining_len = mem_len - offset;
        let min_len = min(remaining_len, buf.len());

        if (offset + min_len) > mem_len {
            return Err(Error::OutOfBound(offset + min_len, mem_len));
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
