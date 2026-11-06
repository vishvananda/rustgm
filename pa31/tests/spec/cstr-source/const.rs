use std::ffi::CStr;
fn helper()->&'static CStr{c"A"}
pub const fn bad()->&'static CStr{helper()}
