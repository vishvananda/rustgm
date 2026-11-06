use std::ffi::CStr;
#[unsafe(no_mangle)]pub extern "C" fn rebuild(p:*const std::ffi::c_char)->*const std::ffi::c_char{unsafe{CStr::from_ptr(p)}.as_ptr()}
#[unsafe(no_mangle)]pub extern "C" fn safe_build()->*const std::ffi::c_char{CStr::from_bytes_with_nul(b"ok\0").unwrap().as_ptr()}
