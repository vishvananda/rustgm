enum E{Only(u32)}const N:u32=unsafe{let mut e=E::Only(37);core::ptr::write_bytes(&raw mut e,0,1);let p=match &mut e{E::Only(v)=>v as *mut u32};core::ptr::write(p,41);core::ptr::read(p)};fn main(){check(N==41)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
