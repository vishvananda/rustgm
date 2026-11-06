use std::ffi::CStr;
const VALUE:&CStr=c"é";
const BYTES:&[u8]=VALUE.to_bytes_with_nul();
const CHECK:()=assert!(BYTES.len()==3 && BYTES[0]==195 && BYTES[1]==169 && BYTES[2]==0);
pub fn view()->&'static [u8]{BYTES}
