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
pub mod primitive;

// Little Endian
pub type Number = u32;
// Size of Number
pub const NUMBER_SIZE: usize = size_of::<Number>();

#[inline]
pub fn unpack_number(slice: &[u8]) -> Number {
    // the size of slice should be checked before call this function
    let mut b = [0u8; 4];
    b.copy_from_slice(&slice[..4]);
    Number::from_le_bytes(b)
}

#[inline]
pub fn pack_number(num: Number) -> [u8; 4] {
    num.to_le_bytes()
}

pub fn hex_string(input: &[u8]) -> String {
    cfg_if::cfg_if! {
        if #[cfg(feature = "std")] {
            faster_hex::hex_string(input)
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
