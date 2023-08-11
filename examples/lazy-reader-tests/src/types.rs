
#![allow(dead_code)]
#![allow(unused_imports)]
extern crate alloc;
use alloc::vec::Vec;
use core::convert::TryInto;
use molecule::lazy_reader::Cursor;
use molecule::lazy_reader::Error;
pub struct Byte2 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte2 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte2 {
    pub fn len(&self) -> usize {
        2
    }
}
impl Byte2 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1 * index, 1)?;
        cur.try_into()
    }
}
pub struct Byte3 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte3 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte3 {
    pub fn len(&self) -> usize {
        3
    }
}
impl Byte3 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1 * index, 1)?;
        cur.try_into()
    }
}
pub struct Byte4 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte4 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte4 {
    pub fn len(&self) -> usize {
        4
    }
}
impl Byte4 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1 * index, 1)?;
        cur.try_into()
    }
}
pub struct Byte5 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte5 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte5 {
    pub fn len(&self) -> usize {
        5
    }
}
impl Byte5 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1 * index, 1)?;
        cur.try_into()
    }
}
pub struct Byte6 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte6 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte6 {
    pub fn len(&self) -> usize {
        6
    }
}
impl Byte6 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1 * index, 1)?;
        cur.try_into()
    }
}
pub struct Byte7 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte7 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte7 {
    pub fn len(&self) -> usize {
        7
    }
}
impl Byte7 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1 * index, 1)?;
        cur.try_into()
    }
}
pub struct Byte8 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte8 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte8 {
    pub fn len(&self) -> usize {
        8
    }
}
impl Byte8 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1 * index, 1)?;
        cur.try_into()
    }
}
pub struct Byte9 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte9 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte9 {
    pub fn len(&self) -> usize {
        9
    }
}
impl Byte9 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1 * index, 1)?;
        cur.try_into()
    }
}
pub struct Byte10 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte10 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte10 {
    pub fn len(&self) -> usize {
        10
    }
}
impl Byte10 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1 * index, 1)?;
        cur.try_into()
    }
}
pub struct Byte11 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte11 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte11 {
    pub fn len(&self) -> usize {
        11
    }
}
impl Byte11 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1 * index, 1)?;
        cur.try_into()
    }
}
pub struct Byte12 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte12 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte12 {
    pub fn len(&self) -> usize {
        12
    }
}
impl Byte12 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1 * index, 1)?;
        cur.try_into()
    }
}
pub struct Byte13 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte13 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte13 {
    pub fn len(&self) -> usize {
        13
    }
}
impl Byte13 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1 * index, 1)?;
        cur.try_into()
    }
}
pub struct Byte14 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte14 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte14 {
    pub fn len(&self) -> usize {
        14
    }
}
impl Byte14 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1 * index, 1)?;
        cur.try_into()
    }
}
pub struct Byte15 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte15 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte15 {
    pub fn len(&self) -> usize {
        15
    }
}
impl Byte15 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1 * index, 1)?;
        cur.try_into()
    }
}
pub struct Byte16 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte16 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte16 {
    pub fn len(&self) -> usize {
        16
    }
}
impl Byte16 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1 * index, 1)?;
        cur.try_into()
    }
}
pub struct Word {
    pub cursor: Cursor,
}
impl From<Cursor> for Word {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Word {
    pub fn len(&self) -> usize {
        2
    }
}
impl Word {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1 * index, 1)?;
        cur.try_into()
    }
}
pub struct Word2 {
    pub cursor: Cursor,
}
impl From<Cursor> for Word2 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Word2 {
    pub fn len(&self) -> usize {
        2
    }
}
impl Word2 {
    pub fn get(&self, index: usize) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(2 * index, 2)?;
        Ok(cur)
    }
}
pub struct Word3 {
    pub cursor: Cursor,
}
impl From<Cursor> for Word3 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Word3 {
    pub fn len(&self) -> usize {
        3
    }
}
impl Word3 {
    pub fn get(&self, index: usize) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(2 * index, 2)?;
        Ok(cur)
    }
}
pub struct Word4 {
    pub cursor: Cursor,
}
impl From<Cursor> for Word4 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Word4 {
    pub fn len(&self) -> usize {
        4
    }
}
impl Word4 {
    pub fn get(&self, index: usize) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(2 * index, 2)?;
        Ok(cur)
    }
}
pub struct Word5 {
    pub cursor: Cursor,
}
impl From<Cursor> for Word5 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Word5 {
    pub fn len(&self) -> usize {
        5
    }
}
impl Word5 {
    pub fn get(&self, index: usize) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(2 * index, 2)?;
        Ok(cur)
    }
}
pub struct Word6 {
    pub cursor: Cursor,
}
impl From<Cursor> for Word6 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Word6 {
    pub fn len(&self) -> usize {
        6
    }
}
impl Word6 {
    pub fn get(&self, index: usize) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(2 * index, 2)?;
        Ok(cur)
    }
}
pub struct Word7 {
    pub cursor: Cursor,
}
impl From<Cursor> for Word7 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Word7 {
    pub fn len(&self) -> usize {
        7
    }
}
impl Word7 {
    pub fn get(&self, index: usize) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(2 * index, 2)?;
        Ok(cur)
    }
}
pub struct Word8 {
    pub cursor: Cursor,
}
impl From<Cursor> for Word8 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Word8 {
    pub fn len(&self) -> usize {
        8
    }
}
impl Word8 {
    pub fn get(&self, index: usize) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(2 * index, 2)?;
        Ok(cur)
    }
}
pub struct Byte3x3 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte3x3 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte3x3 {
    pub fn len(&self) -> usize {
        3
    }
}
impl Byte3x3 {
    pub fn get(&self, index: usize) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(3 * index, 3)?;
        Ok(cur)
    }
}
pub struct Byte5x3 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte5x3 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte5x3 {
    pub fn len(&self) -> usize {
        3
    }
}
impl Byte5x3 {
    pub fn get(&self, index: usize) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(5 * index, 5)?;
        Ok(cur)
    }
}
pub struct Byte7x3 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte7x3 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte7x3 {
    pub fn len(&self) -> usize {
        3
    }
}
impl Byte7x3 {
    pub fn get(&self, index: usize) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(7 * index, 7)?;
        Ok(cur)
    }
}
pub struct Byte9x3 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte9x3 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte9x3 {
    pub fn len(&self) -> usize {
        3
    }
}
impl Byte9x3 {
    pub fn get(&self, index: usize) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(9 * index, 9)?;
        Ok(cur)
    }
}
pub struct StructA {
    pub cursor: Cursor,
}
impl From<Cursor> for StructA {
    fn from(cursor: Cursor) -> Self {
        StructA { cursor }
    }
}
impl StructA {
    pub fn f1(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(0, 1)?;
        cur.try_into()
    }
}

impl StructA {
    pub fn f2(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1, 1)?;
        cur.try_into()
    }
}

impl StructA {
    pub fn f3(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(2, 2)?;
        Ok(cur)
    }
}

impl StructA {
    pub fn f4(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(4, 2)?;
        Ok(cur)
    }
}
pub struct StructB {
    pub cursor: Cursor,
}
impl From<Cursor> for StructB {
    fn from(cursor: Cursor) -> Self {
        StructB { cursor }
    }
}
impl StructB {
    pub fn f1(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(0, 1)?;
        cur.try_into()
    }
}

impl StructB {
    pub fn f2(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1, 1)?;
        cur.try_into()
    }
}

impl StructB {
    pub fn f3(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(2, 2)?;
        Ok(cur)
    }
}

impl StructB {
    pub fn f4(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(4, 3)?;
        Ok(cur)
    }
}
pub struct StructC {
    pub cursor: Cursor,
}
impl From<Cursor> for StructC {
    fn from(cursor: Cursor) -> Self {
        StructC { cursor }
    }
}
impl StructC {
    pub fn f1(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(0, 1)?;
        cur.try_into()
    }
}

impl StructC {
    pub fn f2(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1, 1)?;
        cur.try_into()
    }
}

impl StructC {
    pub fn f3(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(2, 2)?;
        Ok(cur)
    }
}

impl StructC {
    pub fn f4(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(4, 4)?;
        Ok(cur)
    }
}
pub struct StructD {
    pub cursor: Cursor,
}
impl From<Cursor> for StructD {
    fn from(cursor: Cursor) -> Self {
        StructD { cursor }
    }
}
impl StructD {
    pub fn f1(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(0, 1)?;
        cur.try_into()
    }
}

impl StructD {
    pub fn f2(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1, 1)?;
        cur.try_into()
    }
}

impl StructD {
    pub fn f3(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(2, 2)?;
        Ok(cur)
    }
}

impl StructD {
    pub fn f4(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(4, 5)?;
        Ok(cur)
    }
}
pub struct StructE {
    pub cursor: Cursor,
}
impl From<Cursor> for StructE {
    fn from(cursor: Cursor) -> Self {
        StructE { cursor }
    }
}
impl StructE {
    pub fn f1(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(0, 1)?;
        cur.try_into()
    }
}

impl StructE {
    pub fn f2(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(1, 2)?;
        Ok(cur)
    }
}

impl StructE {
    pub fn f3(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(3, 1)?;
        cur.try_into()
    }
}

impl StructE {
    pub fn f4(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(4, 2)?;
        Ok(cur)
    }
}
pub struct StructF {
    pub cursor: Cursor,
}
impl From<Cursor> for StructF {
    fn from(cursor: Cursor) -> Self {
        StructF { cursor }
    }
}
impl StructF {
    pub fn f1(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(0, 1)?;
        cur.try_into()
    }
}

impl StructF {
    pub fn f2(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(1, 3)?;
        Ok(cur)
    }
}

impl StructF {
    pub fn f3(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(4, 1)?;
        cur.try_into()
    }
}
pub struct StructG {
    pub cursor: Cursor,
}
impl From<Cursor> for StructG {
    fn from(cursor: Cursor) -> Self {
        StructG { cursor }
    }
}
impl StructG {
    pub fn f1(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(0, 3)?;
        Ok(cur)
    }
}

impl StructG {
    pub fn f2(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(3, 1)?;
        cur.try_into()
    }
}

impl StructG {
    pub fn f3(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(4, 2)?;
        Ok(cur)
    }
}

impl StructG {
    pub fn f4(&self) -> Result<Word2, Error> {
        let cur = self.cursor.slice_by_offset(6, 4)?;
        Ok(cur.into())
    }
}
pub struct StructH {
    pub cursor: Cursor,
}
impl From<Cursor> for StructH {
    fn from(cursor: Cursor) -> Self {
        StructH { cursor }
    }
}
impl StructH {
    pub fn f1(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(0, 3)?;
        Ok(cur)
    }
}

impl StructH {
    pub fn f2(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(3, 1)?;
        cur.try_into()
    }
}

impl StructH {
    pub fn f3(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(4, 2)?;
        Ok(cur)
    }
}

impl StructH {
    pub fn f4(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(6, 4)?;
        Ok(cur)
    }
}
pub struct StructI {
    pub cursor: Cursor,
}
impl From<Cursor> for StructI {
    fn from(cursor: Cursor) -> Self {
        StructI { cursor }
    }
}
impl StructI {
    pub fn f1(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(0, 3)?;
        Ok(cur)
    }
}

impl StructI {
    pub fn f2(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(3, 1)?;
        cur.try_into()
    }
}
pub struct StructJ {
    pub cursor: Cursor,
}
impl From<Cursor> for StructJ {
    fn from(cursor: Cursor) -> Self {
        StructJ { cursor }
    }
}
impl StructJ {
    pub fn f1(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.slice_by_offset(0, 6)?;
        Ok(cur)
    }
}

impl StructJ {
    pub fn f2(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(6, 1)?;
        cur.try_into()
    }
}
pub struct StructIx3 {
    pub cursor: Cursor,
}
impl From<Cursor> for StructIx3 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl StructIx3 {
    pub fn len(&self) -> usize {
        3
    }
}
impl StructIx3 {
    pub fn get(&self, index: usize) -> Result<StructI, Error> {
        let cur = self.cursor.slice_by_offset(4 * index, 4)?;
        Ok(cur.into())
    }
}
pub struct StructO {
    pub cursor: Cursor,
}
impl From<Cursor> for StructO {
    fn from(cursor: Cursor) -> Self {
        StructO { cursor }
    }
}
impl StructO {
    pub fn f1(&self) -> Result<StructIx3, Error> {
        let cur = self.cursor.slice_by_offset(0, 12)?;
        Ok(cur.into())
    }
}

impl StructO {
    pub fn f2(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(12, 1)?;
        cur.try_into()
    }
}
pub struct StructP {
    pub cursor: Cursor,
}
impl From<Cursor> for StructP {
    fn from(cursor: Cursor) -> Self {
        StructP { cursor }
    }
}
impl StructP {
    pub fn f1(&self) -> Result<StructJ, Error> {
        let cur = self.cursor.slice_by_offset(0, 7)?;
        Ok(cur.into())
    }
}

impl StructP {
    pub fn f2(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(7, 1)?;
        cur.try_into()
    }
}
pub struct Bytes {
    pub cursor: Cursor,
}
impl From<Cursor> for Bytes {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Bytes {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.fixvec_length()
    }
}
impl Bytes {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.fixvec_slice_by_index(1, index)?;
        cur.try_into()
    }
}
pub struct Words {
    pub cursor: Cursor,
}
impl From<Cursor> for Words {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Words {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.fixvec_length()
    }
}
impl Words {
    pub fn get(&self, index: usize) -> Result<Cursor, Error> {
        let cur = self.cursor.fixvec_slice_by_index(2, index)?;
        Ok(cur)
    }
}
pub struct Byte3Vec {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte3Vec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte3Vec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.fixvec_length()
    }
}
impl Byte3Vec {
    pub fn get(&self, index: usize) -> Result<Cursor, Error> {
        let cur = self.cursor.fixvec_slice_by_index(3, index)?;
        Ok(cur)
    }
}
pub struct Byte7Vec {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte7Vec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte7Vec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.fixvec_length()
    }
}
impl Byte7Vec {
    pub fn get(&self, index: usize) -> Result<Cursor, Error> {
        let cur = self.cursor.fixvec_slice_by_index(7, index)?;
        Ok(cur)
    }
}
pub struct StructIVec {
    pub cursor: Cursor,
}
impl From<Cursor> for StructIVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl StructIVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.fixvec_length()
    }
}
impl StructIVec {
    pub fn get(&self, index: usize) -> Result<StructI, Error> {
        let cur = self.cursor.fixvec_slice_by_index(4, index)?;
        Ok(cur.into())
    }
}
pub struct StructJVec {
    pub cursor: Cursor,
}
impl From<Cursor> for StructJVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl StructJVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.fixvec_length()
    }
}
impl StructJVec {
    pub fn get(&self, index: usize) -> Result<StructJ, Error> {
        let cur = self.cursor.fixvec_slice_by_index(7, index)?;
        Ok(cur.into())
    }
}
pub struct StructPVec {
    pub cursor: Cursor,
}
impl From<Cursor> for StructPVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl StructPVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.fixvec_length()
    }
}
impl StructPVec {
    pub fn get(&self, index: usize) -> Result<StructP, Error> {
        let cur = self.cursor.fixvec_slice_by_index(8, index)?;
        Ok(cur.into())
    }
}
pub struct BytesVec {
    pub cursor: Cursor,
}
impl From<Cursor> for BytesVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl BytesVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.dynvec_length()
    }
}
impl BytesVec {
    pub fn get(&self, index: usize) -> Result<Cursor, Error> {
        let cur = self.cursor.dynvec_slice_by_index(index)?;
        cur.convert_to_rawbytes()
    }
}
pub struct WordsVec {
    pub cursor: Cursor,
}
impl From<Cursor> for WordsVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl WordsVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.dynvec_length()
    }
}
impl WordsVec {
    pub fn get(&self, index: usize) -> Result<Words, Error> {
        let cur = self.cursor.dynvec_slice_by_index(index)?;
        Ok(cur.into())
    }
}
pub struct Table0 {
    pub cursor: Cursor,
}
impl From<Cursor> for Table0 {
    fn from(cursor: Cursor) -> Self {
        Table0 { cursor }
    }
}
pub struct Table1 {
    pub cursor: Cursor,
}
impl From<Cursor> for Table1 {
    fn from(cursor: Cursor) -> Self {
        Table1 { cursor }
    }
}
impl Table1 {
    pub fn f1(&self) -> Result<u8, Error> {
        let cur = self.cursor.table_slice_by_index(0)?;
        cur.try_into()
    }
}
pub struct Table2 {
    pub cursor: Cursor,
}
impl From<Cursor> for Table2 {
    fn from(cursor: Cursor) -> Self {
        Table2 { cursor }
    }
}
impl Table2 {
    pub fn f1(&self) -> Result<u8, Error> {
        let cur = self.cursor.table_slice_by_index(0)?;
        cur.try_into()
    }
}

impl Table2 {
    pub fn f2(&self) -> Result<Word2, Error> {
        let cur = self.cursor.table_slice_by_index(1)?;
        Ok(cur.into())
    }
}
pub struct Table3 {
    pub cursor: Cursor,
}
impl From<Cursor> for Table3 {
    fn from(cursor: Cursor) -> Self {
        Table3 { cursor }
    }
}
impl Table3 {
    pub fn f1(&self) -> Result<u8, Error> {
        let cur = self.cursor.table_slice_by_index(0)?;
        cur.try_into()
    }
}

impl Table3 {
    pub fn f2(&self) -> Result<Word2, Error> {
        let cur = self.cursor.table_slice_by_index(1)?;
        Ok(cur.into())
    }
}

impl Table3 {
    pub fn f3(&self) -> Result<StructA, Error> {
        let cur = self.cursor.table_slice_by_index(2)?;
        Ok(cur.into())
    }
}
pub struct Table4 {
    pub cursor: Cursor,
}
impl From<Cursor> for Table4 {
    fn from(cursor: Cursor) -> Self {
        Table4 { cursor }
    }
}
impl Table4 {
    pub fn f1(&self) -> Result<u8, Error> {
        let cur = self.cursor.table_slice_by_index(0)?;
        cur.try_into()
    }
}

impl Table4 {
    pub fn f2(&self) -> Result<Word2, Error> {
        let cur = self.cursor.table_slice_by_index(1)?;
        Ok(cur.into())
    }
}

impl Table4 {
    pub fn f3(&self) -> Result<StructA, Error> {
        let cur = self.cursor.table_slice_by_index(2)?;
        Ok(cur.into())
    }
}

impl Table4 {
    pub fn f4(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(3)?;
        cur.convert_to_rawbytes()
    }
}
pub struct Table5 {
    pub cursor: Cursor,
}
impl From<Cursor> for Table5 {
    fn from(cursor: Cursor) -> Self {
        Table5 { cursor }
    }
}
impl Table5 {
    pub fn f1(&self) -> Result<u8, Error> {
        let cur = self.cursor.table_slice_by_index(0)?;
        cur.try_into()
    }
}

impl Table5 {
    pub fn f2(&self) -> Result<Word2, Error> {
        let cur = self.cursor.table_slice_by_index(1)?;
        Ok(cur.into())
    }
}

impl Table5 {
    pub fn f3(&self) -> Result<StructA, Error> {
        let cur = self.cursor.table_slice_by_index(2)?;
        Ok(cur.into())
    }
}

impl Table5 {
    pub fn f4(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(3)?;
        cur.convert_to_rawbytes()
    }
}

impl Table5 {
    pub fn f5(&self) -> Result<BytesVec, Error> {
        let cur = self.cursor.table_slice_by_index(4)?;
        Ok(cur.into())
    }
}
pub struct Table6 {
    pub cursor: Cursor,
}
impl From<Cursor> for Table6 {
    fn from(cursor: Cursor) -> Self {
        Table6 { cursor }
    }
}
impl Table6 {
    pub fn f1(&self) -> Result<u8, Error> {
        let cur = self.cursor.table_slice_by_index(0)?;
        cur.try_into()
    }
}

impl Table6 {
    pub fn f2(&self) -> Result<Word2, Error> {
        let cur = self.cursor.table_slice_by_index(1)?;
        Ok(cur.into())
    }
}

impl Table6 {
    pub fn f3(&self) -> Result<StructA, Error> {
        let cur = self.cursor.table_slice_by_index(2)?;
        Ok(cur.into())
    }
}

impl Table6 {
    pub fn f4(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(3)?;
        cur.convert_to_rawbytes()
    }
}

impl Table6 {
    pub fn f5(&self) -> Result<BytesVec, Error> {
        let cur = self.cursor.table_slice_by_index(4)?;
        Ok(cur.into())
    }
}

impl Table6 {
    pub fn f6(&self) -> Result<Table5, Error> {
        let cur = self.cursor.table_slice_by_index(5)?;
        Ok(cur.into())
    }
}
pub struct ByteOpt {
    pub cursor: Cursor,
}
impl From<Cursor> for ByteOpt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
pub struct WordOpt {
    pub cursor: Cursor,
}
impl From<Cursor> for WordOpt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
pub struct StructAOpt {
    pub cursor: Cursor,
}
impl From<Cursor> for StructAOpt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
pub struct StructPOpt {
    pub cursor: Cursor,
}
impl From<Cursor> for StructPOpt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
pub struct BytesOpt {
    pub cursor: Cursor,
}
impl From<Cursor> for BytesOpt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
pub struct WordsOpt {
    pub cursor: Cursor,
}
impl From<Cursor> for WordsOpt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
pub struct BytesVecOpt {
    pub cursor: Cursor,
}
impl From<Cursor> for BytesVecOpt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
pub struct WordsVecOpt {
    pub cursor: Cursor,
}
impl From<Cursor> for WordsVecOpt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
pub struct Table0Opt {
    pub cursor: Cursor,
}
impl From<Cursor> for Table0Opt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
pub struct Table6Opt {
    pub cursor: Cursor,
}
impl From<Cursor> for Table6Opt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
pub struct Table6OptOpt {
    pub cursor: Cursor,
}
impl From<Cursor> for Table6OptOpt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
pub struct ByteOptVec {
    pub cursor: Cursor,
}
impl From<Cursor> for ByteOptVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl ByteOptVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.dynvec_length()
    }
}
impl ByteOptVec {
    pub fn get(&self, index: usize) -> Result<Option<u8>, Error> {
        let cur = self.cursor.dynvec_slice_by_index(index)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            Ok(Some(cur.try_into()?))
        }
    }
}
pub struct WordOptVec {
    pub cursor: Cursor,
}
impl From<Cursor> for WordOptVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl WordOptVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.dynvec_length()
    }
}
impl WordOptVec {
    pub fn get(&self, index: usize) -> Result<Option<Cursor>, Error> {
        let cur = self.cursor.dynvec_slice_by_index(index)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            Ok(Some(cur.into()))
        }
    }
}
pub struct WordsOptVec {
    pub cursor: Cursor,
}
impl From<Cursor> for WordsOptVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl WordsOptVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.dynvec_length()
    }
}
impl WordsOptVec {
    pub fn get(&self, index: usize) -> Result<Option<Words>, Error> {
        let cur = self.cursor.dynvec_slice_by_index(index)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            Ok(Some(cur.into()))
        }
    }
}
pub struct BytesOptVec {
    pub cursor: Cursor,
}
impl From<Cursor> for BytesOptVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl BytesOptVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.dynvec_length()
    }
}
impl BytesOptVec {
    pub fn get(&self, index: usize) -> Result<Option<Cursor>, Error> {
        let cur = self.cursor.dynvec_slice_by_index(index)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            let cur = cur.convert_to_rawbytes()?;
            Ok(Some(cur.into()))
        }
    }
}
pub struct UnionA {
    pub cursor: Cursor,
}
impl From<Cursor> for UnionA {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl UnionA {
    pub fn item_id(&self) -> Result<usize, Error> {
        let item = self.cursor.union_unpack()?;
        Ok(item.item_id)
    }
}
impl UnionA {
    pub fn as_byte(&self) -> Result<u8, Error> {
        let item = self.cursor.union_unpack()?;
        let cur = item.cursor.clone();
        cur.try_into()
    }
}
impl UnionA {
    pub fn as_word(&self) -> Result<Cursor, Error> {
        let item = self.cursor.union_unpack()?;
        let cur = item.cursor.clone();
        Ok(cur)
    }
}
impl UnionA {
    pub fn as_structa(&self) -> Result<StructA, Error> {
        let item = self.cursor.union_unpack()?;
        let cur = item.cursor.clone();
        Ok(cur.into())
    }
}
impl UnionA {
    pub fn as_bytes(&self) -> Result<Cursor, Error> {
        let item = self.cursor.union_unpack()?;
        let cur = item.cursor.clone();
        cur.convert_to_rawbytes()
    }
}
impl UnionA {
    pub fn as_words(&self) -> Result<Words, Error> {
        let item = self.cursor.union_unpack()?;
        let cur = item.cursor.clone();
        Ok(cur.into())
    }
}
impl UnionA {
    pub fn as_table0(&self) -> Result<Table0, Error> {
        let item = self.cursor.union_unpack()?;
        let cur = item.cursor.clone();
        Ok(cur.into())
    }
}
impl UnionA {
    pub fn as_table6(&self) -> Result<Table6, Error> {
        let item = self.cursor.union_unpack()?;
        let cur = item.cursor.clone();
        Ok(cur.into())
    }
}
impl UnionA {
    pub fn as_table6opt(&self) -> Result<Option<Table6>, Error> {
        let item = self.cursor.union_unpack()?;
        let cur = item.cursor.clone();
        if cur.option_is_none() {
            Ok(None)
        } else {
            Ok(Some(cur.into()))
        }
    }
}
pub struct UnionB {
    pub cursor: Cursor,
}
impl From<Cursor> for UnionB {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl UnionB {
    pub fn item_id(&self) -> Result<usize, Error> {
        let item = self.cursor.union_unpack()?;
        Ok(item.item_id)
    }
}
impl UnionB {
    pub fn as_byte(&self) -> Result<u8, Error> {
        let item = self.cursor.union_unpack()?;
        let cur = item.cursor.clone();
        cur.try_into()
    }
}
impl UnionB {
    pub fn as_word(&self) -> Result<Cursor, Error> {
        let item = self.cursor.union_unpack()?;
        let cur = item.cursor.clone();
        Ok(cur)
    }
}
pub struct UnionC {
    pub cursor: Cursor,
}
impl From<Cursor> for UnionC {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl UnionC {
    pub fn item_id(&self) -> Result<usize, Error> {
        let item = self.cursor.union_unpack()?;
        Ok(item.item_id)
    }
}
impl UnionC {
    pub fn as_word(&self) -> Result<Cursor, Error> {
        let item = self.cursor.union_unpack()?;
        let cur = item.cursor.clone();
        Ok(cur)
    }
}
impl UnionC {
    pub fn as_byte(&self) -> Result<u8, Error> {
        let item = self.cursor.union_unpack()?;
        let cur = item.cursor.clone();
        cur.try_into()
    }
}
pub struct UnionD {
    pub cursor: Cursor,
}
impl From<Cursor> for UnionD {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl UnionD {
    pub fn item_id(&self) -> Result<usize, Error> {
        let item = self.cursor.union_unpack()?;
        Ok(item.item_id)
    }
}
impl UnionD {
    pub fn as_word(&self) -> Result<Cursor, Error> {
        let item = self.cursor.union_unpack()?;
        let cur = item.cursor.clone();
        Ok(cur)
    }
}
impl UnionD {
    pub fn as_byte(&self) -> Result<u8, Error> {
        let item = self.cursor.union_unpack()?;
        let cur = item.cursor.clone();
        cur.try_into()
    }
}
pub struct TableA {
    pub cursor: Cursor,
}
impl From<Cursor> for TableA {
    fn from(cursor: Cursor) -> Self {
        TableA { cursor }
    }
}
impl TableA {
    pub fn f1(&self) -> Result<Word2, Error> {
        let cur = self.cursor.table_slice_by_index(0)?;
        Ok(cur.into())
    }
}

impl TableA {
    pub fn f2(&self) -> Result<StructA, Error> {
        let cur = self.cursor.table_slice_by_index(1)?;
        Ok(cur.into())
    }
}

impl TableA {
    pub fn f3(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(2)?;
        cur.convert_to_rawbytes()
    }
}

impl TableA {
    pub fn f4(&self) -> Result<BytesVec, Error> {
        let cur = self.cursor.table_slice_by_index(3)?;
        Ok(cur.into())
    }
}

impl TableA {
    pub fn f5(&self) -> Result<Table1, Error> {
        let cur = self.cursor.table_slice_by_index(4)?;
        Ok(cur.into())
    }
}

impl TableA {
    pub fn f6(&self) -> Result<Option<Cursor>, Error> {
        let cur = self.cursor.table_slice_by_index(5)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            let cur = cur.convert_to_rawbytes()?;
            Ok(Some(cur.into()))
        }
    }
}

impl TableA {
    pub fn f7(&self) -> Result<UnionA, Error> {
        let cur = self.cursor.table_slice_by_index(6)?;
        Ok(cur.into())
    }
}

impl TableA {
    pub fn f8(&self) -> Result<u8, Error> {
        let cur = self.cursor.table_slice_by_index(7)?;
        cur.try_into()
    }
}
pub struct AllInOne {
    pub cursor: Cursor,
}
impl From<Cursor> for AllInOne {
    fn from(cursor: Cursor) -> Self {
        AllInOne { cursor }
    }
}
impl AllInOne {
    pub fn f0(&self) -> Result<u8, Error> {
        let cur = self.cursor.table_slice_by_index(0)?;
        cur.try_into()
    }
}

impl AllInOne {
    pub fn f1(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(1)?;
        Ok(cur)
    }
}

impl AllInOne {
    pub fn f2(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(2)?;
        Ok(cur)
    }
}

impl AllInOne {
    pub fn f3(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(3)?;
        Ok(cur)
    }
}

impl AllInOne {
    pub fn f4(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(4)?;
        Ok(cur)
    }
}

impl AllInOne {
    pub fn f5(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(5)?;
        Ok(cur)
    }
}

impl AllInOne {
    pub fn f6(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(6)?;
        Ok(cur)
    }
}

impl AllInOne {
    pub fn f7(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(7)?;
        Ok(cur)
    }
}

impl AllInOne {
    pub fn f8(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(8)?;
        Ok(cur)
    }
}

impl AllInOne {
    pub fn f9(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(9)?;
        Ok(cur)
    }
}

impl AllInOne {
    pub fn f10(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(10)?;
        Ok(cur)
    }
}

impl AllInOne {
    pub fn f11(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(11)?;
        Ok(cur)
    }
}

impl AllInOne {
    pub fn f12(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(12)?;
        Ok(cur)
    }
}

impl AllInOne {
    pub fn f13(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(13)?;
        Ok(cur)
    }
}

impl AllInOne {
    pub fn f14(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(14)?;
        Ok(cur)
    }
}

impl AllInOne {
    pub fn f15(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(15)?;
        Ok(cur)
    }
}

impl AllInOne {
    pub fn f16(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(16)?;
        Ok(cur)
    }
}

impl AllInOne {
    pub fn f17(&self) -> Result<Word2, Error> {
        let cur = self.cursor.table_slice_by_index(17)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f18(&self) -> Result<Word3, Error> {
        let cur = self.cursor.table_slice_by_index(18)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f19(&self) -> Result<Word4, Error> {
        let cur = self.cursor.table_slice_by_index(19)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f20(&self) -> Result<Word5, Error> {
        let cur = self.cursor.table_slice_by_index(20)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f21(&self) -> Result<Word6, Error> {
        let cur = self.cursor.table_slice_by_index(21)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f22(&self) -> Result<Word7, Error> {
        let cur = self.cursor.table_slice_by_index(22)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f23(&self) -> Result<Word8, Error> {
        let cur = self.cursor.table_slice_by_index(23)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f24(&self) -> Result<Byte3x3, Error> {
        let cur = self.cursor.table_slice_by_index(24)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f25(&self) -> Result<Byte5x3, Error> {
        let cur = self.cursor.table_slice_by_index(25)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f26(&self) -> Result<Byte7x3, Error> {
        let cur = self.cursor.table_slice_by_index(26)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f27(&self) -> Result<Byte9x3, Error> {
        let cur = self.cursor.table_slice_by_index(27)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f28(&self) -> Result<StructA, Error> {
        let cur = self.cursor.table_slice_by_index(28)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f29(&self) -> Result<StructB, Error> {
        let cur = self.cursor.table_slice_by_index(29)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f30(&self) -> Result<StructC, Error> {
        let cur = self.cursor.table_slice_by_index(30)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f31(&self) -> Result<StructD, Error> {
        let cur = self.cursor.table_slice_by_index(31)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f32(&self) -> Result<StructE, Error> {
        let cur = self.cursor.table_slice_by_index(32)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f33(&self) -> Result<StructF, Error> {
        let cur = self.cursor.table_slice_by_index(33)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f34(&self) -> Result<StructG, Error> {
        let cur = self.cursor.table_slice_by_index(34)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f35(&self) -> Result<StructH, Error> {
        let cur = self.cursor.table_slice_by_index(35)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f36(&self) -> Result<StructI, Error> {
        let cur = self.cursor.table_slice_by_index(36)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f37(&self) -> Result<StructJ, Error> {
        let cur = self.cursor.table_slice_by_index(37)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f38(&self) -> Result<StructIx3, Error> {
        let cur = self.cursor.table_slice_by_index(38)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f39(&self) -> Result<StructO, Error> {
        let cur = self.cursor.table_slice_by_index(39)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f40(&self) -> Result<StructP, Error> {
        let cur = self.cursor.table_slice_by_index(40)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f41(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(41)?;
        cur.convert_to_rawbytes()
    }
}

impl AllInOne {
    pub fn f42(&self) -> Result<Words, Error> {
        let cur = self.cursor.table_slice_by_index(42)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f43(&self) -> Result<Byte3Vec, Error> {
        let cur = self.cursor.table_slice_by_index(43)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f44(&self) -> Result<Byte7Vec, Error> {
        let cur = self.cursor.table_slice_by_index(44)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f45(&self) -> Result<StructIVec, Error> {
        let cur = self.cursor.table_slice_by_index(45)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f46(&self) -> Result<StructJVec, Error> {
        let cur = self.cursor.table_slice_by_index(46)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f47(&self) -> Result<StructPVec, Error> {
        let cur = self.cursor.table_slice_by_index(47)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f48(&self) -> Result<BytesVec, Error> {
        let cur = self.cursor.table_slice_by_index(48)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f49(&self) -> Result<WordsVec, Error> {
        let cur = self.cursor.table_slice_by_index(49)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f50(&self) -> Result<Table0, Error> {
        let cur = self.cursor.table_slice_by_index(50)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f51(&self) -> Result<Table1, Error> {
        let cur = self.cursor.table_slice_by_index(51)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f52(&self) -> Result<Table2, Error> {
        let cur = self.cursor.table_slice_by_index(52)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f53(&self) -> Result<Table3, Error> {
        let cur = self.cursor.table_slice_by_index(53)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f54(&self) -> Result<Table4, Error> {
        let cur = self.cursor.table_slice_by_index(54)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f55(&self) -> Result<Table5, Error> {
        let cur = self.cursor.table_slice_by_index(55)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f56(&self) -> Result<Table6, Error> {
        let cur = self.cursor.table_slice_by_index(56)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f57(&self) -> Result<Option<u8>, Error> {
        let cur = self.cursor.table_slice_by_index(57)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            Ok(Some(cur.try_into()?))
        }
    }
}

impl AllInOne {
    pub fn f58(&self) -> Result<Option<Cursor>, Error> {
        let cur = self.cursor.table_slice_by_index(58)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            Ok(Some(cur.into()))
        }
    }
}

impl AllInOne {
    pub fn f59(&self) -> Result<Option<StructA>, Error> {
        let cur = self.cursor.table_slice_by_index(59)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            Ok(Some(cur.into()))
        }
    }
}

impl AllInOne {
    pub fn f60(&self) -> Result<Option<StructP>, Error> {
        let cur = self.cursor.table_slice_by_index(60)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            Ok(Some(cur.into()))
        }
    }
}

impl AllInOne {
    pub fn f61(&self) -> Result<Option<Cursor>, Error> {
        let cur = self.cursor.table_slice_by_index(61)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            let cur = cur.convert_to_rawbytes()?;
            Ok(Some(cur.into()))
        }
    }
}

impl AllInOne {
    pub fn f62(&self) -> Result<Option<Words>, Error> {
        let cur = self.cursor.table_slice_by_index(62)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            Ok(Some(cur.into()))
        }
    }
}

impl AllInOne {
    pub fn f63(&self) -> Result<Option<BytesVec>, Error> {
        let cur = self.cursor.table_slice_by_index(63)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            Ok(Some(cur.into()))
        }
    }
}

impl AllInOne {
    pub fn f64(&self) -> Result<Option<WordsVec>, Error> {
        let cur = self.cursor.table_slice_by_index(64)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            Ok(Some(cur.into()))
        }
    }
}

impl AllInOne {
    pub fn f65(&self) -> Result<Option<Table0>, Error> {
        let cur = self.cursor.table_slice_by_index(65)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            Ok(Some(cur.into()))
        }
    }
}

impl AllInOne {
    pub fn f66(&self) -> Result<Option<Table6>, Error> {
        let cur = self.cursor.table_slice_by_index(66)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            Ok(Some(cur.into()))
        }
    }
}

impl AllInOne {
    pub fn f67(&self) -> Result<Option<Option<Table6>>, Error> {
        let cur = self.cursor.table_slice_by_index(67)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            Ok(Some(Some(cur.into())))
        }
    }
}

impl AllInOne {
    pub fn f68(&self) -> Result<ByteOptVec, Error> {
        let cur = self.cursor.table_slice_by_index(68)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f69(&self) -> Result<WordOptVec, Error> {
        let cur = self.cursor.table_slice_by_index(69)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f70(&self) -> Result<WordsOptVec, Error> {
        let cur = self.cursor.table_slice_by_index(70)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f71(&self) -> Result<BytesOptVec, Error> {
        let cur = self.cursor.table_slice_by_index(71)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f72(&self) -> Result<UnionA, Error> {
        let cur = self.cursor.table_slice_by_index(72)?;
        Ok(cur.into())
    }
}

impl AllInOne {
    pub fn f73(&self) -> Result<TableA, Error> {
        let cur = self.cursor.table_slice_by_index(73)?;
        Ok(cur.into())
    }
}
