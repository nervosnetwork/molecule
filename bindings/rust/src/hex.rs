#[cfg(feature = "std")]
pub fn hex_string(input: &[u8]) -> String {
    faster_hex::hex_string(input).unwrap()
}

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
pub fn hex_string(input: &[u8]) -> alloc::string::String {
    use core::fmt::Write;
    let mut buf = alloc::string::String::new();
    for b in input {
        write!(buf, "{:02x}", b).expect("write hex byte error");
    }
    buf
}
