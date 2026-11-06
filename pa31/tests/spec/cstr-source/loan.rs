use std::ffi::CStr;
pub fn bad(x:&mut CStr)->&[u8]{let shared=x.to_bytes_with_nul();let other=&mut *x;std::hint::black_box(other);shared}
