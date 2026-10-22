#[repr(u8)]enum E{A(u32),B(bool)}const N:u32=unsafe{let mut e=E::A(37);let q=match &mut e{E::A(x)=>x as *mut u32,_=>loop{}};let p=&raw mut e as *mut [u8;8];(*p)[0]=1;(*p)[1]=1;let v=*q;match e{E::B(true)=>v,_=>0}};
fn main(){check(N==37)}fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
