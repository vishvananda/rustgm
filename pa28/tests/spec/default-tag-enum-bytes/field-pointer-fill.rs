enum E{A(u32),Empty}const N:u32=unsafe{let mut e=E::A(37);let p=match &mut e{E::A(v)=>v as *mut u32,_=>loop{}};core::ptr::write_bytes(p,0,1);match e{E::A(v)=>v,_=>9}};fn main(){check(N==0)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
