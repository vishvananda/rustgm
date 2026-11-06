use std::ffi::CStr;
pub fn bad(p:*const std::ffi::c_char)->&'static CStr{CStr::from_ptr(p)}
