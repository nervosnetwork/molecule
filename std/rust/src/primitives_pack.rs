use crate::bytes::*;
use crate::pack::*;
use crate::prelude::*;

macro_rules! impl_conversion_for_entity_unpack {
    ($original:ty, $entity:ident) => {
        impl Unpack<$original> for $entity {
            fn unpack(&self) -> $original {
                self.as_reader().unpack()
            }
        }
    };
}

impl Pack<Uint64> for u64 {
    fn pack(&self) -> Uint64 {
        Uint64::new_unchecked(Bytes::from(self.to_le_bytes().to_vec()))
    }
}

impl<'r> Unpack<u64> for Uint64Reader<'r> {
    fn unpack(&self) -> u64 {
        let mut b = [0u8; 8];
        b.copy_from_slice(self.as_slice());
        u64::from_le_bytes(b)
    }
}
impl_conversion_for_entity_unpack!(u64, Uint64);

impl Pack<Uint32> for u32 {
    fn pack(&self) -> Uint32 {
        Uint32::new_unchecked(Bytes::from(self.to_le_bytes().to_vec()))
    }
}

impl<'r> Unpack<u32> for Uint32Reader<'r> {
    fn unpack(&self) -> u32 {
        let mut b = [0u8; 4];
        b.copy_from_slice(self.as_slice());
        u32::from_le_bytes(b)
    }
}
impl_conversion_for_entity_unpack!(u32, Uint32);

impl Pack<Uint16> for u16 {
    fn pack(&self) -> Uint16 {
        Uint16::new_unchecked(Bytes::from(self.to_le_bytes().to_vec()))
    }
}

impl<'r> Unpack<u16> for Uint16Reader<'r> {
    fn unpack(&self) -> u16 {
        let mut b = [0u8; 2];
        b.copy_from_slice(self.as_slice());
        u16::from_le_bytes(b)
    }
}
impl_conversion_for_entity_unpack!(u16, Uint16);

impl Pack<Uint8> for u8 {
    fn pack(&self) -> Uint8 {
        Uint8::new_unchecked(Bytes::from(self.to_le_bytes().to_vec()))
    }
}

impl<'r> Unpack<u8> for Uint8Reader<'r> {
    fn unpack(&self) -> u8 {
        let mut b = [0u8; 1];
        b.copy_from_slice(self.as_slice());
        u8::from_le_bytes(b)
    }
}
impl_conversion_for_entity_unpack!(u8, Uint8);

// Signed integers

impl Pack<Int64> for i64 {
    fn pack(&self) -> Int64 {
        Int64::new_unchecked(Bytes::from(self.to_le_bytes().to_vec()))
    }
}

impl<'r> Unpack<i64> for Int64Reader<'r> {
    fn unpack(&self) -> i64 {
        let mut b = [0u8; 8];
        b.copy_from_slice(self.as_slice());
        i64::from_le_bytes(b)
    }
}
impl_conversion_for_entity_unpack!(i64, Int64);

impl Pack<Int32> for i32 {
    fn pack(&self) -> Int32 {
        Int32::new_unchecked(Bytes::from(self.to_le_bytes().to_vec()))
    }
}

impl<'r> Unpack<i32> for Int32Reader<'r> {
    fn unpack(&self) -> i32 {
        let mut b = [0u8; 4];
        b.copy_from_slice(self.as_slice());
        i32::from_le_bytes(b)
    }
}
impl_conversion_for_entity_unpack!(i32, Int32);

impl Pack<Int16> for i16 {
    fn pack(&self) -> Int16 {
        Int16::new_unchecked(Bytes::from(self.to_le_bytes().to_vec()))
    }
}

impl<'r> Unpack<i16> for Int16Reader<'r> {
    fn unpack(&self) -> i16 {
        let mut b = [0u8; 2];
        b.copy_from_slice(self.as_slice());
        i16::from_le_bytes(b)
    }
}
impl_conversion_for_entity_unpack!(i16, Int16);

impl Pack<Int8> for i8 {
    fn pack(&self) -> Int8 {
        Int8::new_unchecked(Bytes::from(self.to_le_bytes().to_vec()))
    }
}

impl<'r> Unpack<i8> for Int8Reader<'r> {
    fn unpack(&self) -> i8 {
        let mut b = [0u8; 1];
        b.copy_from_slice(self.as_slice());
        i8::from_le_bytes(b)
    }
}
impl_conversion_for_entity_unpack!(i8, Int8);

// Bool
impl Pack<Bool> for bool {
    fn pack(&self) -> Bool {
        let b = u8::from(*self);
        Bool::new_unchecked(Bytes::from(vec![b]))
    }
}

impl<'r> Unpack<bool> for BoolReader<'r> {
    fn unpack(&self) -> bool {
        match self.as_slice()[0] {
            0 => false,
            1 => true,
            _ => unreachable!(),
        }
    }
}
impl_conversion_for_entity_unpack!(bool, Bool);
