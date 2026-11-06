use std::ffi::CStr;
pub fn bad()->&'static CStr{let bytes=[65,0];CStr::from_bytes_with_nul(&bytes).unwrap()}
