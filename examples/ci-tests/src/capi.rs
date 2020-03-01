mod ffi {
    extern "C" {
        pub(super) fn tablea_verify(data: *const u8, data_len: u32) -> u32;
    }
}

pub fn tablea_verify(input: &[u8]) -> bool {
    unsafe { ffi::tablea_verify(input.as_ptr(), input.len() as u32) == 0 }
}
