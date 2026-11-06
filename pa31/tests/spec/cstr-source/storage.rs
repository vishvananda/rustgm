use std::ffi::CStr;
const B:&[u8;2]=b"A\0";
const C:&CStr=c"A";
static SB:&[u8;2]=b"A\0";
static SC:&CStr=c"A";
#[unsafe(no_mangle)]pub extern "C" fn byte_const()->*const u8{B.as_ptr()}
#[unsafe(no_mangle)]pub extern "C" fn c_const()->*const std::ffi::c_char{C.as_ptr()}
#[unsafe(no_mangle)]pub extern "C" fn byte_static()->*const u8{SB.as_ptr()}
#[unsafe(no_mangle)]pub extern "C" fn c_static()->*const std::ffi::c_char{SC.as_ptr()}
#[unsafe(no_mangle)]pub extern "C" fn c_view()->*const u8{SC.to_bytes_with_nul().as_ptr()}
#[unsafe(no_mangle)]pub extern "C" fn lengths()->usize{B.len()+C.to_bytes_with_nul().len()+SB.len()+SC.to_bytes_with_nul().len()}
