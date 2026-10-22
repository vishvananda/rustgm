#[derive(Copy,Clone)]#[repr(u8)]enum E{A(u32),B(bool)}
const N:(u32,bool)=unsafe{let mut e=E::A(37);let p=match &mut e{E::A(v)=>v as *mut u32,_=>loop{}};let tag=(&raw mut e)as *mut u8;core::ptr::write(tag,1);core::ptr::write(tag.add(1),1);core::ptr::write(p,41);let n=core::ptr::read(p);(n,match e{E::B(v)=>v,_=>false})};
fn main(){if N.0!=41||!N.1{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
