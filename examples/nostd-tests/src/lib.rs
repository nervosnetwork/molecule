#![no_std]

#[macro_use]
extern crate alloc;

pub mod types {
    #![allow(clippy::all)]
    pub use molecule::prelude::{Byte, ByteReader};
    include!(concat!(env!("OUT_DIR"), "/", "ci_tests", ".rs"));
}
