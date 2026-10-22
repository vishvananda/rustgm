enum E{Only(u32)}const N:u32=unsafe{let mut e=E::Only(37);core::ptr::write_bytes(&raw mut e,0,1);match e{E::Only(v)=>v}};fn main(){check(N==0)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
