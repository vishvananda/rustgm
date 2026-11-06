use std::ffi::CStr;
pub fn good(x:&mut CStr)->&[u8]{let other=&mut *x;std::hint::black_box(other);x.to_bytes_with_nul()}
