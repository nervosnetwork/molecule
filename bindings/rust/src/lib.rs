#![no_std]

extern crate alloc;

use alloc::string::String;
use core::mem::size_of;

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        extern crate std;

        pub use bytes;
        pub mod io {
            pub use std::io::{Error, Result, Write};
        }
    } else {
        pub mod bytes;
        pub mod io;
    }
}

pub mod error;
pub mod prelude;
mod primitive;

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

pub fn hex_string(input: &[u8]) -> String {
    cfg_if::cfg_if! {
        if #[cfg(feature = "std")] {
            faster_hex::hex_string(input).unwrap()
        } else {
            use core::fmt::Write;
            let mut buf = String::new();
            for b in input {
                let _ = write!(buf, "{:02x}", b);
            }
            buf
        }
    }
}
