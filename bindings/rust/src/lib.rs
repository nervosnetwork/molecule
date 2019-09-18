use std::mem::size_of;

pub mod error;
pub mod prelude;
pub mod primitive;

pub use bytes;
pub use faster_hex;

// Little Endian
pub type Number = u32;
// Size of Number
pub const NUMBER_SIZE: usize = size_of::<Number>();

#[inline]
pub fn unpack_number(slice: &[u8]) -> Number {
    #[allow(clippy::cast_ptr_alignment)]
    let le = slice.as_ptr() as *const Number;
    Number::from_le(unsafe { *le })
}

#[inline]
pub fn pack_number(num: Number) -> [u8; 4] {
    num.to_le_bytes()
}

#[inline]
pub fn unpack_number_vec(slice: &[u8]) -> &[[u8; 4]] {
    #[allow(clippy::cast_ptr_alignment)]
    unsafe {
        &*(slice as *const [u8] as *const [[u8; 4]])
    }
}
