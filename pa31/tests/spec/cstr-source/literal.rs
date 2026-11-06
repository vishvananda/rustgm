use std::ffi::CStr;
pub fn ordinary()->&'static CStr{c"A"}
pub fn raw()->&'static CStr{cr#"A\n"#}
pub fn utf8()->&'static CStr{c"é"}
pub fn high()->&'static CStr{c"\xFF"}
pub fn empty()->&'static CStr{c""}
